use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub extension: String,
    pub path: PathBuf,
    pub category: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub disc: Option<String>,
}

pub struct Album {
    pub name: String,
    pub discs: BTreeMap<String, Box<Album>>,
    pub tracks: Vec<Track>,
}

impl Album {
    pub fn add_disc(&mut self, name: String) -> &mut Album {
        if !(self.discs.contains_key(&name)) {
            let disc = Album {
                name: name.clone(),
                discs: BTreeMap::new(),
                tracks: vec![],
            };
            self.discs.insert(name.clone(), Box::new(disc));
        }
        return self.discs.get_mut(&name).unwrap();
    }
}

pub struct Artist {
    pub name: String,
    pub albums: BTreeMap<String, Album>,
    pub tracks: Vec<Track>,
}

impl Artist {
    pub fn add_album(&mut self, name: String) -> &mut Album {
        if !(self.albums.contains_key(&name)) {
            let album = Album {
                name: name.clone(),
                discs: BTreeMap::new(),
                tracks: vec![],
            };
            self.albums.insert(name.clone(), album);
        }
        return self.albums.get_mut(&name).unwrap();
    }
}

pub struct Category {
    pub name: String,
    pub albums: BTreeMap<String, Album>,
    pub artists: BTreeMap<String, Artist>,
}

impl Category {
    pub fn add_artist(&mut self, name: String) -> &mut Artist {
        if !(self.artists.contains_key(&name)) {
            let artist = Artist {
                name: name.clone(),
                albums: BTreeMap::new(),
                tracks: vec![],
            };
            self.artists.insert(name.clone(), artist);
        }
        return self.artists.get_mut(&name).unwrap();
    }

    pub fn add_album(&mut self, name: String) -> &mut Album {
        if !(self.albums.contains_key(&name)) {
            let album = Album {
                name: name.clone(),
                discs: BTreeMap::new(),
                tracks: vec![],
            };
            self.albums.insert(name.clone(), album);
        }
        return self.albums.get_mut(&name).unwrap();
    }
}
