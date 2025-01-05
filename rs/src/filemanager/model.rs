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
        self.discs.get_mut(&name).unwrap()
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
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
        self.albums.get_mut(&name).unwrap()
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
        self.artists.get_mut(&name).unwrap()
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
        self.albums.get_mut(&name).unwrap()
    }

    pub fn all_tracks(&self) -> Vec<&Track> {
        let mut tracks = Vec::new();
        for album in self.albums.values() {
            tracks.extend(album.all_tracks());
        }
        for artist in self.artists.values() {
            tracks.extend(artist.all_tracks());
        }
        tracks
    }
}

impl Artist {
    /**
     * Return all the tracks under this artist in a flat list.
     */
    pub fn all_tracks(&self) -> Vec<&Track> {
        let mut tracks = Vec::new();
        for album in self.albums.values() {
            tracks.extend(album.all_tracks());
        }
        tracks.extend(self.tracks.iter());
        tracks
    }
}

impl Album {
    /**
     * Return all the tracks under this album in a flat list.
     */
    pub fn all_tracks(&self) -> Vec<&Track> {
        let mut tracks = Vec::new();
        for disc in self.discs.values() {
            tracks.extend(disc.all_tracks());
        }
        tracks.extend(self.tracks.iter());
        tracks
    }
}

#[cfg(test)]
pub mod factories {
    use super::Track;
    use factori::factori;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::path::PathBuf;

    factori!(Track, {
        default {
            id = random_string(),
            name = random_string(),
            extension = "mp3".to_string(),
            path = PathBuf::from(random_string()),
            category = random_string(),
            artist = Some(random_string()),
            album = Some(random_string()),
            disc = None,
        }
    });

    pub fn random_string() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect()
    }
}
