use std::iter::Map;
use std::path::PathBuf;

pub struct Track {
    id: String,
    name: String,
    path: PathBuf,
    artist: Option<String>,
    album: Option<String>,
    disc: Option<String>,
}

pub struct Album {
    pub discs: Map<String, Box<Album>>,
    pub tracks: Vec<Track>,
}

pub struct Artist {
    pub albums: Map<String, Album>,
    pub tracks: Vec<Track>,
}

pub struct Category {
    pub albums: Map<String, Album>,
    pub artists: Map<String, Artist>,
}
