use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use rodio;
use rodio::{Decoder, MixerDeviceSink, Source};

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

/// The track the Player is currently playing.
#[derive(Debug, Clone)]
pub struct CurrentTrack {
    pub path: PathBuf,
    /// Total duration of the decoded track, if the decoder could determine it.
    pub duration: Option<Duration>,
}

/// A snapshot of the playback state, as returned by `Player::status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackStatus {
    /// Number of sources queued in the player, including the one playing.
    pub queued: usize,
    pub paused: bool,
    /// Position in the currently playing track, or None if nothing is queued.
    pub position: Option<Duration>,
    /// Total duration of the currently playing track, if known.
    pub duration: Option<Duration>,
}

// This is the object that handles playing music
pub struct Player {
    player: rodio::Player,
    sink: MixerDeviceSink,
    /// The track currently playing, if any.
    current: Option<CurrentTrack>,
    finishing_fired: bool,
    /**
     * Set when stop() (or when replacing a current track in play()) could not
     * drain the sink in time. `current` is kept until the sink actually drains
     * so the state stays consistent; tick() completes the cleanup once it does.
     */
    stop_requested: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Player {
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
            current: None,
            finishing_fired: false,
            stop_requested: false,
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

        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) => {
                log::error!("Failed to open {}: {}", path.display(), error);
                return Err(format!("could not open {}: {}", path.display(), error).into());
            }
        };

        // We want to get the duration of the track in order to grab the next
        // track in the queue just before the end. VBR MP3s make this tricky,
        // but Symphonia can estimate the duration if we provide the byte length
        // of the track. So we decode the file into a Source first.
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
        let total_duration = source.total_duration();

        // If there's a track currently playing, stop it (and let the sink
        // drain) before playing this track. If the track failed to decode
        // above, any currently playing track will be left playing.
        if !self.player.empty() && !self.clear_sink() {
            // Avoid appending to a wedged sink, which would block the Player
            // thread. Better to return an error. If the sink does eventually
            // finish draining, then tick() will recover the state and play
            // requests should succeed.
            self.stop_requested = true;
            let message = format!(
                "could not play {}: the previous track did not stop in time",
                path.display()
            );
            log::error!("{message}");
            return Err(message.into());
        }
        self.player.append(source);

        // Track what is playing for status and end-of-track handling.
        self.current = Some(CurrentTrack {
            path: path.to_path_buf(),
            duration: total_duration,
        });
        self.finishing_fired = false;
        self.stop_requested = false;

        Ok(())
    }

    pub fn status(&self) -> PlaybackStatus {
        let queued = self.player.len();
        let paused = self.player.is_paused();
        let position = self.position();
        let duration = self.current.as_ref().and_then(|track| track.duration);
        log::info!(
            "Status: {} tracks in the sink queue. paused={} position={:?} duration={:?}",
            queued,
            paused,
            position,
            duration
        );
        PlaybackStatus {
            queued,
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
        // Clean up if there was a stop() call where the sink didn't drain
        // withing the timeout. If it has now drained, we recover our state.
        if self.stop_requested {
            if self.player.empty() {
                self.current = None;
                self.stop_requested = false;
            }
            return; // Would just return below anyway, and there's nothing playing.
        }

        if self.player.empty() {
            self.finish(); // back-stop for if the playback finished before we expected.
            return;
        }

        let Some(current) = self.current.as_ref() else {
            return;
        };
        let Some(duration) = current.duration else {
            return; // Duration unknown, so remaining time cannot be estimated.
        };

        let position = self.player.get_pos();
        if near_end(position, duration, NEAR_END_THRESHOLD, self.finishing_fired) {
            let path = current.path.clone();
            let remaining = duration.saturating_sub(position);
            self.track_finishing(&path, Some(position), Some(remaining));
        }
    }

    /**
     * Clean up when the last queued source has finished playing. If the
     * finishing hook has not fired yet (duration estimate too long, or never
     * known), it fires here, at the actual end of the track.
     */
    fn finish(&mut self) {
        // Note: `current.take()` unsets `current` (and its duration with it)
        if let Some(current) = self.current.take() {
            self.track_finishing(&current.path, None, None);
            log::info!("Playback finished: {}", current.path.display());
        }
        self.finishing_fired = false;
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
        // TODO handle pulling the next song in the queue when the current song is close to finishing
    }

    pub fn stop(&mut self) {
        log::info!("Stopping playback");
        self.finishing_fired = false;
        if self.clear_sink() {
            self.current = None;
            self.stop_requested = false;
        } else {
            // The sink is wedged; keep `current` (and this flag) so state
            // stays consistent, and let tick() complete the cleanup if the
            // sink ever drains.
            self.stop_requested = true;
        }
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
        if !drained {
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
        let mut player = Player::new();
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
    fn test_tick_completes_a_pending_stop_without_firing_the_hook() {
        let mut player = Player::new();

        // Simulate a stop() whose sink drain has not completed yet: the
        // track metadata is still set, the flag is up, the hook has not
        // fired (stop() resets it).
        player.current = Some(CurrentTrack {
            path: PathBuf::from("some/track.mp3"),
            duration: Some(Duration::from_secs(10)),
        });
        player.finishing_fired = false;
        player.stop_requested = true;

        player.tick();

        // The (empty) sink has drained, so tick() finishes the cleanup —
        // and since the hook did not run, finishing_fired stays false.
        assert!(player.current.is_none());
        assert!(!player.stop_requested);
        assert!(!player.finishing_fired);
    }

    #[test]
    fn test_tick_holds_state_while_a_pending_stop_has_not_drained() {
        let mut player = Player::new();
        // A long silent source keeps the sink non-empty, like a wedged
        // sink would.
        player.player.append(rodio::buffer::SamplesBuffer::new(
            NonZero::new(1).unwrap(),
            NonZero::new(48_000).unwrap(),
            vec![0.0f32; 480_000],
        ));
        player.current = Some(CurrentTrack {
            path: PathBuf::from("some/track.mp3"),
            duration: Some(Duration::from_secs(10)),
        });
        player.stop_requested = true;

        player.tick();

        // Nothing has drained yet: state must be held until it does.
        assert!(player.current.is_some());
        assert!(player.stop_requested);

        player.stop();
    }
}
