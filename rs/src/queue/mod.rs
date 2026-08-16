use crate::filemanager::model::Track;
use std::collections::VecDeque;

pub struct PlaybackQueue {
    pub tracks: VecDeque<Track>,
    pub position: usize,
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
            position: 0,
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push_back(track);
    }

    pub fn add_tracks(&mut self, tracks: Vec<Track>) {
        self.tracks.extend(tracks);
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }

    pub fn pop(&mut self) {
        self.tracks.pop_front();
    }

    pub fn print_tracks(&self) {
        log::info!("{:?}", self.tracks);
    }
}
