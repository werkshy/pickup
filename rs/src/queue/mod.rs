use crate::filemanager::model::Track;

pub struct PlaybackQueue {
    pub tracks: Vec<Track>,
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
            tracks: vec![],
            position: 0,
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn add_tracks(&mut self, tracks: Vec<Track>) {
        self.tracks.extend(tracks);
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }

    pub fn pop(&mut self) {
        if !self.tracks.is_empty() {
            self.tracks.remove(0);
        }
    }

    pub fn print_tracks(&self) {
        log::info!("{:?}", self.tracks);
    }
}
