use crate::filemanager::model::Track;
use std::collections::VecDeque;

/**
 * The playback queue. The track at the front of `tracks` is the one playing
 * right now; it stays at the front until it has finished playing, and the
 * Player pops it (and starts the next one) only when it is done. When
 * playback is idle, the front is the next track to play.
 */
pub struct PlaybackQueue {
    pub tracks: VecDeque<Track>,
}

impl Default for PlaybackQueue {
    fn default() -> Self {
        Self::new()
    }
}

/**
 * PlaybackQueue is a struct that holds a list of tracks to be played on the server (in jukebox mode).
 */
impl PlaybackQueue {
    pub fn new() -> PlaybackQueue {
        PlaybackQueue {
            tracks: VecDeque::new(),
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push_back(track);
    }

    pub fn add_tracks(&mut self, tracks: Vec<Track>) {
        self.tracks.extend(tracks);
    }

    /**
     * Insert tracks directly after the track at the front (the one playing
     * right now), so they play next. On an empty queue there is nothing to
     * go behind, so they become the front.
     */
    pub fn insert_next(&mut self, tracks: Vec<Track>) {
        let at = self.tracks.len().min(1);
        for (offset, track) in tracks.into_iter().enumerate() {
            self.tracks.insert(at + offset, track);
        }
    }

    /**
     * Insert tracks at the front of the queue, so they play before anything
     * currently queued. Only useful while the player is idle, if there is a
     * track currently playing, insert_next() is the better option.
     */
    pub fn insert_front(&mut self, tracks: Vec<Track>) {
        for (offset, track) in tracks.into_iter().enumerate() {
            self.tracks.insert(offset, track);
        }
    }

    /**
     * Replace the whole queue with these tracks; the front of the new queue
     * is the track to play.
     */
    pub fn replace_with(&mut self, tracks: Vec<Track>) {
        self.tracks.clear();
        self.tracks.extend(tracks);
    }

    /**
     * Pop the track that just finished playing and return the next track to
     * play, if the queue has one.
     */
    pub fn advance(&mut self) -> Option<Track> {
        self.tracks.pop_front();
        self.tracks.front().cloned()
    }

    /// The track at the front of the queue: the one playing, or the next
    /// one to play when idle. None once the queue is empty.
    pub fn current_or_next(&self) -> Option<&Track> {
        self.tracks.front()
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }

    pub fn print_tracks(&self) {
        log::info!("{:?}", self.tracks);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::filemanager::model::Track;

    fn track(name: &str) -> Track {
        Track {
            id: format!("trk_{name}"),
            name: String::from(name),
            extension: String::from("wav"),
            path: PathBuf::from(format!("{name}.wav")),
            category: String::from("test"),
            artist: None,
            album: None,
            disc: None,
        }
    }

    fn names(queue: &PlaybackQueue) -> Vec<String> {
        queue.tracks.iter().map(|t| t.name.clone()).collect()
    }

    #[test]
    fn test_insert_next_goes_behind_the_playing_track() {
        let mut queue = PlaybackQueue::new();
        queue.add_track(track("a"));

        queue.insert_next(vec![track("b"), track("c")]);

        assert_eq!(names(&queue), ["a", "b", "c"]);
    }

    #[test]
    fn test_insert_next_on_an_empty_queue_inserts_at_the_front() {
        // Nothing to go behind: inserting at index 1 would panic.
        let mut queue = PlaybackQueue::new();

        queue.insert_next(vec![track("a"), track("b")]);

        assert_eq!(names(&queue), ["a", "b"]);
    }

    #[test]
    fn test_insert_front_goes_before_everything() {
        let mut queue = PlaybackQueue::new();
        queue.add_track(track("a"));

        queue.insert_front(vec![track("b"), track("c")]);

        assert_eq!(names(&queue), ["b", "c", "a"]);
    }

    #[test]
    fn test_advance_pops_the_finished_track() {
        let mut queue = PlaybackQueue::new();
        queue.add_tracks(vec![track("a"), track("b")]);

        // "a" finished: it is popped, "b" is next.
        let next = queue.advance().expect("queue should have a next track");
        assert_eq!(next.name, "b");
        assert_eq!(names(&queue), ["b"]);

        // "b" finished: the queue is played out.
        assert!(queue.advance().is_none());
        assert!(queue.tracks.is_empty());

        // Advancing an empty queue is a no-op.
        assert!(queue.advance().is_none());
    }

    #[test]
    fn test_current_or_next_is_the_front() {
        let mut queue = PlaybackQueue::new();
        assert!(queue.current_or_next().is_none());

        queue.add_track(track("a"));
        assert_eq!(queue.current_or_next().unwrap().name, "a");

        queue.advance();
        assert!(queue.current_or_next().is_none());
    }

    #[test]
    fn test_replace_with_resets_the_queue() {
        let mut queue = PlaybackQueue::new();
        queue.add_track(track("a"));

        queue.replace_with(vec![track("b"), track("c")]);

        assert_eq!(names(&queue), ["b", "c"]);
        assert_eq!(queue.current_or_next().unwrap().name, "b");
    }
}
