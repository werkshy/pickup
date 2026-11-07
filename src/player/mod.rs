use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use rodio;
use rodio::{Decoder, MixerDeviceSink, Source};

use crate::queue::PlaybackQueue;

pub mod client;
pub mod commands;

pub use client::{PlayerClient, PlayerError};

/**
 * How often the Player thread wakes up between commands to check on playback
 * (position tracking, near-end detection, end-of-track cleanup).
 */
pub const POLL_INTERVAL: Duration = Duration::from_millis(250);

/// How long before the end of the current track the near-end trigger fires.
const NEAR_END_THRESHOLD: Duration = Duration::from_secs(2);

/**
 * How long stop() waits for the sink to drain after stopping. The sink
 * clears its sources asynchronously, so without a wait a status query right
 * after stop() would still see the stopped track.
 */
const STOP_DRAIN_TIMEOUT: Duration = Duration::from_secs(1);

pub trait Command: Send {
    fn action(&mut self, player: &mut Player);
}

/// A snapshot of the playback state, as returned by `Player::status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackStatus {
    /// Number of sources queued in the player, including the one playing.
    pub queued: usize,
    /**
     * Whether a track's playback lifecycle is open: it is playing, or it
     * just finished and the Player has not cleaned it up yet. This is not
     * the same as `queued > 0` — the sink empties when the source ends,
     * before tick() advances the queue (up to POLL_INTERVAL later).
     */
    pub active: bool,
    pub paused: bool,
    /// Position in the currently playing track, or None if nothing is queued.
    pub position: Option<Duration>,
    /// Total duration of the currently playing track, if known.
    pub duration: Option<Duration>,
}

/// What happened when the player was asked to play from the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayOutcome {
    /// Playback started.
    Started,
    /// A paused track resumed playing.
    Resumed,
    /// The player was already playing; nothing changed.
    AlreadyPlaying,
    /// The queue has no track to play.
    NothingQueued,
}

/// A decoded, playable source and its estimated duration (if known).
type DecodedTrack = (Decoder<BufReader<File>>, Option<Duration>);

// This is the object that handles playing music
pub struct Player {
    player: rodio::Player,
    sink: MixerDeviceSink,
    /// The playback queue, shared with the API. When a track finishes, the
    /// Player pops the finished track and starts the next one. The queue is
    /// the source of truth for which track is playing (the front).
    queue: Arc<RwLock<PlaybackQueue>>,
    /// The collection root that the queue's track paths are relative to.
    music_dir: PathBuf,
    /**
     * Whether a played track's lifecycle is open: set by play() on success,
     * cleared by stop() and finish(). This is what distinguishes "a track
     * just finished, the queue must advance" from "the queue is idle".
     */
    active: bool,
    /**
     * Duration of the source currently playing, as estimated at decode
     * time, if known.
     */
    duration: Option<Duration>,

    /// Used to track whether we've already run the finish hook when we detect a
    /// track has finished playing and the next tick() is called.
    finishing_fired: bool,
    /**
     * Set by stop() (successful or not) and cleared by the next successful
     * play(). While set, tick() will not advance the queue - the track stays at
     * the front of the queue so playback can resume from it). Also covers a
     * wedged sink draining late after a stop.
     */
    stopped: bool,
}

impl Player {
    pub fn new(queue: Arc<RwLock<PlaybackQueue>>, music_dir: PathBuf) -> Player {
        log::info!("Creating stream and sink");
        // We can't drop `player` or nothing will play, but it doesn't implement Send and can't be
        // shared across threads.
        let mut sink =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        sink.log_on_drop(false);
        let player = rodio::Player::connect_new(sink.mixer());
        Player {
            player,
            sink,
            queue,
            music_dir,
            active: false,
            duration: None,
            finishing_fired: false,
            stopped: false,
        }
    }

    pub fn command(&mut self, mut command: Box<dyn Command>) {
        (*command).action(self)
    }

    /**
     * Plays a file, replacing whatever is currently playing. Errors (missing
     * file, undecodable file) are returned to the caller (and logged); this
     * must not panic the Player thread.
     *
     * Takes a path rather than a String so that names which aren't valid
     * UTF-8 remain playable (they're only rendered lossily in logs).
     */
    pub fn play(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let path = path.as_ref();
        log::info!("Playing {}", path.display());

        let (source, total_duration) = Self::decode_track(path)?;

        // If there's a track currently playing, stop it (and let the sink
        // drain) before playing this track. If the track failed to decode
        // above, any currently playing track will be left playing.
        if !self.player.empty() && !self.clear_sink() {
            // Avoid appending to a wedged sink, which would block the Player
            // thread. Better to return an error. If the sink does eventually
            // finish draining, then tick() will recover the state and play
            // requests should succeed.
            self.stopped = true;
            self.active = false;
            self.duration = None;
            let message = format!(
                "could not play {}: the previous track did not stop in time",
                path.display()
            );
            log::error!("{message}");
            return Err(message.into());
        }
        self.player.append(source);
        // A new track must always actually play: rodio's pause flag survives
        // stop() and clear_sink() (it lives on the sink, independent of the
        // sources), so clear it here or a track queued after a pause would
        // start paused.
        self.player.play();

        // Bookkeeping for the track that is now playing.
        self.active = true;
        self.duration = total_duration;
        self.finishing_fired = false;
        self.stopped = false;

        Ok(())
    }

    /**
     * Open and decode a track into a playable source, with the duration
     * estimated at decode time. Has no side effects: used by play(), and by
     * check_playable() to validate a track before it replaces the queue.
     */
    fn decode_track(path: &Path) -> Result<DecodedTrack, Box<dyn Error + Send + Sync>> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) => {
                log::error!("Failed to open {}: {}", path.display(), error);
                return Err(format!("could not open {}: {}", path.display(), error).into());
            }
        };

        // We want to get the duration of the track in order to know when it
        // is nearing its end. VBR MP3s make this tricky, but Symphonia can
        // estimate the duration if we provide the byte length of the track.
        // So we decode the file into a Source first.
        let mut builder = Decoder::builder().with_data(BufReader::new(file));
        if let Ok(file_metadata) = path.metadata() {
            builder = builder.with_byte_len(file_metadata.len());
        }
        let source = match builder.build() {
            Ok(source) => source,
            Err(error) => {
                log::error!("Failed to decode {}: {}", path.display(), error);
                return Err(format!("could not decode {}: {}", path.display(), error).into());
            }
        };
        let duration = source.total_duration();
        Ok((source, duration))
    }

    /**
     * Check that a track can be decoded, without touching playback or the
     * queue. The path is relative to the music root, like queue track paths.
     */
    pub fn check_playable(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Self::decode_track(&self.music_dir.join(path)).map(|_| ())
    }

    pub fn status(&self) -> PlaybackStatus {
        let queued = self.player.len();
        let paused = self.player.is_paused();
        let position = self.position();
        let duration = if self.player.empty() {
            None
        } else {
            self.duration
        };
        log::info!(
            "Status: {} tracks in the sink queue. paused={} position={:?} duration={:?}",
            queued,
            paused,
            position,
            duration
        );
        PlaybackStatus {
            queued,
            active: self.active,
            paused,
            position,
            duration,
        }
    }

    /// Position in the currently playing track, or None if nothing is queued.
    pub fn position(&self) -> Option<Duration> {
        if self.player.empty() {
            None
        } else {
            Some(self.player.get_pos())
        }
    }

    /**
     * Poll the playback state. Called by the Player thread between commands
     * so that time-based handling (position tracking, the near-end trigger,
     * end-of-track cleanup) can happen even when no commands arrive.
     */
    pub fn tick(&mut self) {
        if self.stopped {
            // The sink was emptied by an explicit stop() (or is a wedged sink
            // that was asked to stop): hold state — no queue advance, no
            // near-end checks — until the next play() clears this. The
            // stopped track stays at the front of the queue so playback can
            // resume from it.
            return;
        }

        if self.player.empty() {
            self.finish(); // back-stop for if the playback finished before we expected.
            return;
        }

        let Some(duration) = self.duration else {
            return; // Duration unknown, so remaining time cannot be estimated.
        };
        // The playing track is the front of the queue; near-end handling is
        // only meaningful when the sink is playing a queued track.
        let Some(path) = self
            .queue
            .read()
            .unwrap()
            .current_or_next()
            .map(|track| track.path.clone())
        else {
            return;
        };

        let position = self.player.get_pos();
        // TODO if this is reliable, we can use it for gapless playback by
        // advancing the queue and appending the next track just before the
        // current one has finished.
        if near_end(position, duration, NEAR_END_THRESHOLD, self.finishing_fired) {
            let remaining = duration.saturating_sub(position);
            self.track_finishing(&path, Some(position), Some(remaining));
        }
    }

    /**
     * Clean up when the last queued source has finished playing. If the
     * finishing hook has not fired yet (duration estimate too long, or never
     * known), it fires here, at the actual end of the track.
     *
     * `active` is the lifecycle guard: it is set by play() and cleared here
     * and in stop(), so a set flag means the sink just emptied naturally
     * and the front track finished. Without this guard, tick() on an
     * idle-but-populated queue (e.g. after /queue/add without play_now)
     * would spuriously start it.
     */
    fn finish(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        self.duration = None;

        let Some(path) = self
            .queue
            .read()
            .unwrap()
            .current_or_next()
            .map(|track| track.path.clone())
        else {
            return;
        };
        self.track_finishing(&path, None, None);
        log::info!("Playback finished: {}", path.display());
        self.finishing_fired = false;
        self.start_next_from_queue();
    }

    /**
     * Advance the queue past the track that just finished and start the next
     * one, if the queue has one. Tracks that fail to play are skipped (and
     * logged).
     */
    fn start_next_from_queue(&mut self) {
        loop {
            let next = self.queue.write().unwrap().advance();
            let Some(track) = next else {
                return;
            };
            match self.play_queue() {
                Ok(()) => return,
                Err(error) => {
                    log::error!("Skipping track {}: {}", track.path.display(), error);
                }
            }
        }
    }

    /**
     * Skip to the next track in the queue: the current track is discarded
     * (without firing the finishing hook — skipping is a user action, not
     * a natural end) and the next one starts immediately. No-op when no
     * track is playing (idle or stopped). If the queue has no next track,
     * playback stops.
     */
    pub fn skip(&mut self) {
        if !self.active {
            log::info!("Nothing to skip: no track is playing");
            return;
        }
        log::info!("Skipping the current track");
        self.active = false;
        self.duration = None;
        self.finishing_fired = false;
        self.start_next_from_queue();

        // start_next_from_queue() starts the next track when there is one;
        // if the queue ran out, the skipped source is still in the sink and
        // must be stopped.
        if !self.player.empty() && !self.active {
            self.stop();
        }
    }

    /**
     * Pause the currently playing track: the sink keeps the track and its
     * position, tick() holds state while paused (a paused source neither
     * advances nor ends). No-op when nothing is playing or already paused.
     */
    pub fn pause(&mut self) {
        // We don't want to pause a player that is draining the sink after being
        // stopped, because the pause state is sticky.
        if !self.active || self.player.empty() {
            log::info!("Nothing to pause: no track is playing");
            return;
        }
        log::info!("Pausing playback");
        self.player.pause();
    }

    /**
     * Resume playback of a paused track, at the position it was paused at.
     * No-op when playback is not paused.
     */
    pub fn unpause(&mut self) {
        if !self.player.is_paused() {
            log::info!("Nothing to resume: playback is not paused");
            return;
        }
        log::info!("Resuming playback");
        self.player.play();
    }

    /**
     * Start playing the track at the front of the queue, replacing whatever
     * is currently playing. The queue is the source of truth: callers mutate
     * it first, then ask the player to play from it.
     *
     * Fails if the queue is empty or the front track cannot be played.
     */
    pub fn play_queue(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let Some(track) = self.queue.read().unwrap().current_or_next().cloned() else {
            return Err("the queue is empty".into());
        };
        self.play(self.music_dir.join(&track.path))
    }

    /**
     * Make the queue play: unpause a paused track, or start the front of the
     * queue when the player is idle. Neither a resume nor a play_now request
     * may interrupt playback that is already running.
     */
    pub fn play_queue_if_idle(&mut self) -> Result<PlayOutcome, Box<dyn Error + Send + Sync>> {
        if !self.player.empty() {
            if self.player.is_paused() {
                self.unpause();
                return Ok(PlayOutcome::Resumed);
            }
            return Ok(PlayOutcome::AlreadyPlaying);
        }
        let queued = self.queue.read().unwrap().current_or_next().is_some();
        if !queued {
            return Ok(PlayOutcome::NothingQueued);
        }
        self.play_queue()?;
        Ok(PlayOutcome::Started)
    }

    /**
     * The finishing hook: fires at most once per track, normally about 2s
     * before the end. If the duration estimate was too long (e.g. a VBR MP3
     * estimated as CBR) or was never known, the source runs out first and
     * the hook fires at the actual end instead (position/remaining None).
     * This is where pulling the next track from the queue will hook in.
     */
    fn track_finishing(
        &mut self,
        path: &Path,
        position: Option<Duration>,
        remaining: Option<Duration>,
    ) {
        if self.finishing_fired {
            return;
        }
        self.finishing_fired = true;

        match (position, remaining) {
            (Some(position), Some(remaining)) => log::info!(
                "Track nearing end: {} (position {:?}, remaining {:?})",
                path.display(),
                position,
                remaining
            ),
            _ => log::info!(
                "Track ended before the near-end trigger: {} (duration estimate missing or too long)",
                path.display()
            ),
        }
        // TODO pre-append the next track from the queue here for gapless
        // playback; for now the next track starts when this one finishes
        // (see finish()).
    }

    pub fn stop(&mut self) {
        log::info!("Stopping playback");
        self.finishing_fired = false;
        self.active = false;
        self.duration = None;
        // We set `stopped` regardless of whether draining succeeds: the stopped
        // track stays at the front of the queue (the resume anchor) and must
        // not be advanced by tick(), no matter when the sink drains.
        self.stopped = true;
        self.clear_sink();
    }

    /**
     * Stop playback and wait (bounded) for the sink to actually drain. The
     * sink clears its sources asynchronously, so without a wait, state
     * queries right after stopping would still see the stopped track.
     * Returns whether the sink drained within STOP_DRAIN_TIMEOUT.
     */
    fn clear_sink(&mut self) -> bool {
        self.player.stop();
        let deadline = Instant::now() + STOP_DRAIN_TIMEOUT;
        while !self.player.empty() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        let drained = self.player.empty();
        if drained {
            // rodio's pause flag is independent of the sources and survives
            // stop(); clear it so the sink is fully idle once drained -
            // status must not report paused for a stopped player, and a
            // later append must not inherit the pause.
            self.player.play();
        } else {
            // Wedged player - we don't expect this to happen but there's not much we can do.
            log::warn!("Sink did not drain within {STOP_DRAIN_TIMEOUT:?} after stop");
        }
        drained
    }

    pub fn set_volume(&mut self, value: u8) {
        let float_volume = scale_volume_down(value);
        log::info!("Setting volume to {}", float_volume);
        self.player.set_volume(float_volume);
    }

    pub fn get_volume(&mut self) -> u8 {
        let float_volume = self.player.volume();
        log::info!("Current volume is {}", float_volume);
        scale_volume_up(float_volume)
    }

    // This getter really only exists to silence the unused warning about sink.
    pub fn sink(&self) -> &MixerDeviceSink {
        &self.sink
    }
}

#[cfg(test)]
pub(crate) fn test_player() -> Player {
    Player::new(
        Arc::new(RwLock::new(PlaybackQueue::new())),
        PathBuf::from("."),
    )
}

// scale down volume from 0-100 to 0-1.0
fn scale_volume_down(volume_percentage: u8) -> f32 {
    let clamped_volume = volume_percentage.clamp(0, 100);
    clamped_volume as f32 / 100.0
}

fn scale_volume_up(vol_fraction: f32) -> u8 {
    let percent_volume = (vol_fraction * 100.0).round();
    let clamped_volume = percent_volume.clamp(0.0, 100.0);
    clamped_volume as u8
}

/// Decide whether the near-end trigger should fire now: it fires exactly
/// once per track, when the remaining playback time has dropped to
/// `threshold` or below.
fn near_end(
    position: Duration,
    duration: Duration,
    threshold: Duration,
    already_fired: bool,
) -> bool {
    !already_fired && duration.saturating_sub(position) <= threshold
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use super::*;

    #[test]
    fn test_near_end_fires_once_inside_threshold() {
        let duration = Duration::from_secs(180);

        assert!(!near_end(
            Duration::from_secs(100),
            duration,
            NEAR_END_THRESHOLD,
            false
        ));
        assert!(near_end(
            Duration::from_secs(178),
            duration,
            NEAR_END_THRESHOLD,
            false
        ));
        // Once fired, it must not fire again for the same track.
        assert!(!near_end(
            Duration::from_secs(179),
            duration,
            NEAR_END_THRESHOLD,
            true
        ));
    }

    #[test]
    fn test_near_end_handles_position_past_duration() {
        // A position past the end (e.g. an inaccurate duration estimate)
        // is still "near the end", not a negative remaining time.
        let duration = Duration::from_secs(10);

        assert!(near_end(
            Duration::from_secs(15),
            duration,
            NEAR_END_THRESHOLD,
            false
        ));
    }

    #[test]
    fn test_near_end_fires_immediately_for_very_short_tracks() {
        // A track shorter than the threshold is near its end from the start.
        let duration = Duration::from_secs(1);

        assert!(near_end(
            Duration::ZERO,
            duration,
            NEAR_END_THRESHOLD,
            false
        ));
    }

    #[test]
    fn test_track_finishing_fires_once_per_track() {
        let mut player = test_player();
        let path = Path::new("some/track.mp3");

        player.track_finishing(
            path,
            Some(Duration::from_secs(198)),
            Some(Duration::from_secs(2)),
        );
        assert!(player.finishing_fired);

        // A second call (e.g. at the actual end after firing early) must
        // not re-trigger the hook.
        player.track_finishing(path, None, None);
        assert!(player.finishing_fired);
    }

    #[test]
    fn test_tick_does_nothing_while_stopped() {
        let mut player = test_player();

        // Whatever the sink state (empty or still draining after a stop),
        // tick() must hold state while the stopped flag is up: no queue
        // advance, no near-end checks.
        player.stopped = true;

        player.tick();

        assert!(player.stopped);
        assert!(player.player.empty());
    }

    #[test]
    fn test_tick_holds_state_while_a_stopped_sink_has_not_drained() {
        let mut player = test_player();
        // A long silent source keeps the sink non-empty, like a wedged
        // sink would.
        player.player.append(rodio::buffer::SamplesBuffer::new(
            NonZero::new(1).unwrap(),
            NonZero::new(48_000).unwrap(),
            vec![0.0f32; 480_000],
        ));
        player.stopped = true;

        player.tick();

        // Nothing has drained yet: state must be held until it does.
        assert!(player.stopped);
        assert!(!player.player.empty());

        player.stop();
    }

    // ---- queue-driven playback ----

    /// A 16-bit mono PCM WAV file, so tests have real decodable tracks
    /// without depending on the music collection.
    fn write_wav(path: &Path, samples: usize) {
        use std::io::Write;

        const SAMPLE_RATE: u32 = 48_000;
        let data_len = (samples * 2) as u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
        bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        bytes.extend_from_slice(&1u16.to_le_bytes()); // channels
        bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
        bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
        bytes.extend_from_slice(&2u16.to_le_bytes()); // block align
        bytes.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for i in 0..samples {
            bytes
                .write_all(&(((i % 64) * 64) as i16).to_le_bytes())
                .unwrap();
        }
        std::fs::write(path, bytes).unwrap();
    }

    /// A private temp dir for one test's WAV files.
    fn test_audio_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pickup-player-tests-{}-{}",
            std::process::id(),
            name
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_track(path: PathBuf) -> crate::filemanager::model::Track {
        crate::filemanager::model::Track {
            id: String::from("trk_test"),
            name: String::from("test track"),
            extension: String::from("wav"),
            path,
            category: String::from("test"),
            artist: None,
            album: None,
            disc: None,
        }
    }

    #[test]
    fn test_finish_advances_the_queue_and_starts_the_next_track() {
        let dir = test_audio_dir("advance");
        let path_a = dir.join("a.wav");
        let path_b = dir.join("b.wav");
        write_wav(&path_a, 2400); // 50ms
        write_wav(&path_b, 48_000); // 1s

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue
            .write()
            .unwrap()
            .add_tracks(vec![make_track(path_a.clone()), make_track(path_b.clone())]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        // Track A has just finished: the sink is already empty (as tick()
        // observes it right after a source ends) and the player still holds
        // the duration of the track it was playing.
        player.active = true;
        player.duration = Some(Duration::from_millis(50));
        player.tick();

        // Track A was popped; track B is the front and is playing.
        let queue = queue.read().unwrap();
        assert_eq!(queue.tracks.len(), 1);
        assert_eq!(queue.tracks.front().unwrap().path, path_b);
        drop(queue);
        assert!(
            player.duration.is_some(),
            "track B's duration should be set"
        );
        assert!(!player.player.empty());

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_finish_leaves_the_queue_exhausted_at_the_end() {
        let dir = test_audio_dir("exhausted");
        let path_a = dir.join("a.wav");
        write_wav(&path_a, 2400);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue
            .write()
            .unwrap()
            .add_tracks(vec![make_track(path_a.clone())]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));
        player.active = true;
        player.duration = Some(Duration::from_millis(50));

        player.tick();

        // The finished track was popped; the queue is now empty.
        assert!(queue.read().unwrap().tracks.is_empty());
        assert!(player.duration.is_none());
        assert!(player.player.empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_finish_skips_tracks_that_fail_to_play() {
        let dir = test_audio_dir("skip");
        let path_a = dir.join("a.wav");
        let path_b = dir.join("b.wav");
        write_wav(&path_a, 2400);
        write_wav(&path_b, 48_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue.write().unwrap().add_tracks(vec![
            make_track(path_a.clone()),
            make_track(dir.join("missing.wav")),
            make_track(path_b.clone()),
        ]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));
        player.active = true;
        player.duration = Some(Duration::from_millis(50));

        player.tick();

        // The finished and undecodable tracks were popped; only B remains
        // and it is playing.
        let queue = queue.read().unwrap();
        assert_eq!(queue.tracks.len(), 1);
        assert_eq!(queue.tracks.front().unwrap().path, path_b);
        drop(queue);
        assert!(
            player.duration.is_some(),
            "track B's duration should be set"
        );

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_stop_does_not_advance_the_queue() {
        let dir = test_audio_dir("stop");
        let path_a = dir.join("a.wav");
        write_wav(&path_a, 2400);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue.write().unwrap().add_tracks(vec![
            make_track(path_a.clone()),
            make_track(dir.join("b.wav")),
        ]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));
        player.active = true;
        player.duration = Some(Duration::from_millis(50));

        player.stop();
        player.tick();

        // The stopped track stays at the front (nothing was popped) so
        // playback can resume from it; nothing was started.
        let queue = queue.read().unwrap();
        assert_eq!(queue.tracks.len(), 2);
        assert_eq!(queue.tracks.front().unwrap().path, path_a);
        drop(queue);
        assert!(!player.active);
        assert!(player.duration.is_none());
        assert!(player.stopped);
        assert!(player.player.empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_tick_does_not_start_an_idle_queue() {
        let dir = test_audio_dir("idle");
        let path_a = dir.join("a.wav");
        let path_b = dir.join("b.wav");
        write_wav(&path_a, 2400);
        write_wav(&path_b, 48_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue
            .write()
            .unwrap()
            .add_tracks(vec![make_track(path_a.clone()), make_track(path_b.clone())]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        // The queue was populated (e.g. /queue/add without play_now) while
        // the player was idle: tick() must not start it on its own.
        player.tick();
        player.tick();

        let queue = queue.read().unwrap();
        assert_eq!(queue.tracks.len(), 2);
        assert_eq!(queue.tracks.front().unwrap().path, path_a);
        drop(queue);
        assert!(player.duration.is_none());
        assert!(player.player.empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_play_queue_plays_the_front_track() {
        let dir = test_audio_dir("play-queue");
        let path_a = dir.join("a.wav");
        write_wav(&path_a, 48_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue
            .write()
            .unwrap()
            .add_tracks(vec![make_track(path_a.clone())]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        player.play_queue().expect("the queue has a track");

        assert!(!player.player.empty());
        assert!(player.duration.is_some());
        assert_eq!(queue.read().unwrap().tracks.front().unwrap().path, path_a);

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_play_queue_fails_on_an_empty_queue() {
        let mut player = test_player();

        assert!(player.play_queue().is_err());
        assert!(player.player.empty());
    }

    #[test]
    fn test_play_queue_if_idle_reports_what_happened() {
        let dir = test_audio_dir("play-queue-if-idle");
        let path_a = dir.join("a.wav");
        write_wav(&path_a, 48_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        // Nothing queued yet.
        assert_eq!(
            player.play_queue_if_idle().unwrap(),
            PlayOutcome::NothingQueued
        );

        queue.write().unwrap().add_tracks(vec![make_track(path_a)]);
        assert_eq!(player.play_queue_if_idle().unwrap(), PlayOutcome::Started);
        assert!(!player.player.empty());

        // Already playing: a second request must not interrupt or restart.
        assert_eq!(
            player.play_queue_if_idle().unwrap(),
            PlayOutcome::AlreadyPlaying
        );

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pause_and_unpause_hold_the_track() {
        let dir = test_audio_dir("pause");
        let path_a = dir.join("a.wav");
        write_wav(&path_a, 480_000); // 10s, so it does not end while paused

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue.write().unwrap().add_tracks(vec![make_track(path_a)]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        player.play_queue().unwrap();
        player.pause();

        // The sink keeps the paused source, and the queue keeps the track at
        // the front: a resume continues the same track.
        let status = player.status();
        assert!(status.paused);
        assert!(!player.player.empty());
        assert!(status.active);

        player.unpause();

        assert!(!player.status().paused);
        assert!(!player.player.empty());

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pause_is_a_noop_when_idle() {
        let mut player = test_player();

        player.pause();
        assert!(!player.status().paused);

        // Unpausing a player that is not paused is a no-op too.
        player.unpause();
        assert!(!player.status().paused);
    }

    #[test]
    fn test_play_queue_if_idle_resumes_a_paused_track() {
        let dir = test_audio_dir("resume");
        let path_a = dir.join("a.wav");
        write_wav(&path_a, 480_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue.write().unwrap().add_tracks(vec![make_track(path_a)]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        player.play_queue().unwrap();
        player.pause();
        assert!(player.status().paused);

        // The resume request must unpause, not restart the track.
        assert_eq!(player.play_queue_if_idle().unwrap(), PlayOutcome::Resumed);
        assert!(!player.status().paused);
        assert!(!player.player.empty());

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_a_new_track_does_not_inherit_the_pause() {
        let dir = test_audio_dir("paused-replace");
        let path_a = dir.join("a.wav");
        let path_b = dir.join("b.wav");
        write_wav(&path_a, 480_000);
        write_wav(&path_b, 48_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue
            .write()
            .unwrap()
            .add_tracks(vec![make_track(path_a), make_track(path_b.clone())]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        player.play_queue().unwrap();
        player.pause();

        // Skipping the paused track starts the next one - which must
        // actually play, not sit paused behind the old pause flag.
        player.skip();

        let status = player.status();
        assert!(!status.paused, "the next track must not start paused");
        assert!(!player.player.empty());
        assert_eq!(queue.read().unwrap().tracks.front().unwrap().path, path_b);

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_stop_after_a_pause_resets_the_pause() {
        let dir = test_audio_dir("stop-paused");
        let path_a = dir.join("a.wav");
        write_wav(&path_a, 480_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue.write().unwrap().add_tracks(vec![make_track(path_a)]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        player.play_queue().unwrap();
        player.pause();
        player.stop();

        // A stopped player is idle, not paused: status must say so, and the
        // next play must not be held back by the stale pause flag.
        let status = player.status();
        assert!(!status.paused);
        assert!(player.player.empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_finish_advances_the_queue_even_without_a_known_duration() {
        let dir = test_audio_dir("unknown-duration");
        let path_a = dir.join("a.wav");
        let path_b = dir.join("b.wav");
        write_wav(&path_a, 2400);
        write_wav(&path_b, 48_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue
            .write()
            .unwrap()
            .add_tracks(vec![make_track(path_a), make_track(path_b.clone())]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        // Track A finished, but its duration was never known (the decoder
        // could not estimate it). That must not stall the queue.
        player.active = true;
        player.duration = None;
        player.tick();

        let queue = queue.read().unwrap();
        assert_eq!(queue.tracks.len(), 1);
        assert_eq!(queue.tracks.front().unwrap().path, path_b);
        drop(queue);
        assert!(player.active, "track B should be playing");
        assert!(!player.player.empty());

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_status_active_while_a_finished_track_awaits_cleanup() {
        let mut player = test_player();

        // The sink empties as soon as the source ends, but the track's
        // lifecycle stays open until tick() runs finish(). During that
        // window the player still counts as "playing" for the API.
        player.active = true;
        player.duration = Some(Duration::from_millis(50));

        let status = player.status();
        assert_eq!(status.queued, 0);
        assert!(status.active);

        player.stop();
        assert!(!player.status().active);
    }

    #[test]
    fn test_check_playable_does_not_touch_playback() {
        let dir = test_audio_dir("check-playable");
        let path_a = dir.join("a.wav");
        write_wav(&path_a, 2400);

        let mut player = test_player();
        assert!(player.check_playable(&path_a).is_ok());
        assert!(player.check_playable("definitely/missing.wav").is_err());
        assert!(player.player.empty());

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_skip_starts_the_next_track() {
        let dir = test_audio_dir("skip-next");
        let path_a = dir.join("a.wav");
        let path_b = dir.join("b.wav");
        write_wav(&path_a, 48_000);
        write_wav(&path_b, 48_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue
            .write()
            .unwrap()
            .add_tracks(vec![make_track(path_a.clone()), make_track(path_b.clone())]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        player.play_queue().unwrap();
        assert_eq!(queue.read().unwrap().tracks.front().unwrap().path, path_a);

        player.skip();

        // A was popped, B is the front and is playing.
        let queue = queue.read().unwrap();
        assert_eq!(queue.tracks.len(), 1);
        assert_eq!(queue.tracks.front().unwrap().path, path_b);
        drop(queue);
        assert!(player.active);
        assert!(!player.player.empty());

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_skip_on_the_last_track_stops_playback() {
        let dir = test_audio_dir("skip-last");
        let path_a = dir.join("a.wav");
        write_wav(&path_a, 48_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue.write().unwrap().add_tracks(vec![make_track(path_a)]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        player.play_queue().unwrap();
        player.skip();

        assert!(queue.read().unwrap().tracks.is_empty());
        assert!(!player.active);
        assert!(player.player.empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_skip_is_a_noop_when_idle() {
        let mut player = test_player();

        player.skip();

        assert!(!player.active);
        assert!(player.player.empty());
    }

    #[test]
    fn test_skip_skips_unplayable_tracks() {
        let dir = test_audio_dir("skip-broken");
        let path_a = dir.join("a.wav");
        let path_b = dir.join("b.wav");
        write_wav(&path_a, 48_000);
        write_wav(&path_b, 48_000);

        let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
        queue.write().unwrap().add_tracks(vec![
            make_track(path_a),
            make_track(dir.join("missing.wav")),
            make_track(path_b.clone()),
        ]);
        let mut player = Player::new(queue.clone(), PathBuf::from(""));

        player.play_queue().unwrap();
        player.skip();

        // The broken track was skipped too; B is playing.
        let queue = queue.read().unwrap();
        assert_eq!(queue.tracks.len(), 1);
        assert_eq!(queue.tracks.front().unwrap().path, path_b);
        drop(queue);
        assert!(player.active);

        player.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
