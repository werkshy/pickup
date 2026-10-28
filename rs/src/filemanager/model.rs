use std::{collections::BTreeMap, path::PathBuf};

/**
 * What a directory in the music tree turned out to be. Only the top level needs
 * its own name inspected; everything below it is decided by its parent, since a
 * category holds artists, an artist holds albums and an album holds discs.
 *
 * `Track` is not a folder kind: it is only used when a listing asks how deep to
 * print, so tracks can be shown alongside folders.
 */
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Kind {
    Category,
    Artist,
    Album,
    Disc,
    Track,
}

/**
 * A single playable file. The ID is a hash of `path`, so it is stable for as
 * long as the file sits at that path relative to the music dir.
 */
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

/**
 * Where a folder or track sits in the collection: the category, and the
 * artist, album and disc it descends from, each by name. The builder derives
 * it from the directory structure, and `Collection` uses it to place folders
 * and to resolve what an ID points at.
 */
#[derive(Clone, Debug)]
pub struct CollectionLocation {
    pub category: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub disc: Option<String>,
}

/**
 * A directory in the collection, whatever its kind. It holds the directories
 * below it and the tracks directly inside it.
 *
 * Built via `Collection::add_folder`, which derives the ID and records the
 * folder in the collection's ID index. There is deliberately no way to create
 * one directly, so nothing can end up in the tree without also being
 * addressable by ID.
 */
#[derive(Debug)]
pub struct Folder {
    pub id: String,
    pub path: PathBuf,
    /// Display name. A category's leading underscore is stripped here, while the
    /// map key keeps it.
    pub name: String,
    pub kind: Kind,
    pub folders: BTreeMap<String, Folder>,
    pub tracks: Vec<Track>,
}

impl Folder {
    /**
     * Return all the tracks under this folder in a flat list.
     */
    pub fn all_tracks(&self) -> Vec<&Track> {
        let mut tracks = Vec::new();
        for folder in self.folders.values() {
            tracks.extend(folder.all_tracks());
        }
        tracks.extend(self.tracks.iter());
        tracks
    }
}

#[cfg(test)]
pub mod factories {
    use super::Track;
    use factori::factori;
    use rand::distr::Alphanumeric;
    use rand::{rng, RngExt};
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
        rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect()
    }
}
