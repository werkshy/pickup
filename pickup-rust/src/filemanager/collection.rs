use super::{model::Track, utils::generate_id};
use crate::filemanager::model::Category;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    path::PathBuf,
};

const DEFAULT_CATEGORY: &str = "Music";
const CATEGORY_PREFIX: &str = "_";
const CD_REGEX_STR: &str = r"(?i)(cd|dis(c|k)) ?\d";

pub type Collection = HashMap<String, Category>;

struct CollectionBuilder {
    collection: Collection,
}

impl CollectionBuilder {
    pub fn new() -> Self {
        Self {
            collection: HashMap::new(),
        }
    }
    fn add_category(&mut self, name: String) -> &mut Category {
        if !(self.collection.contains_key(&name)) {
            let pretty_name = name.strip_prefix('_').unwrap_or(name.as_str()).to_string();
            let category = Category {
                name: pretty_name,
                artists: HashMap::new(),
                albums: HashMap::new(),
            };
            self.collection.insert(name.clone(), category);
        }
        return self.collection.get_mut(&name).unwrap();
    }

    fn add_track(&mut self, track: Track) {
        log::info!("Adding track {:?}", track.path);
        let category = self.add_category(track.category.clone());
        match (&track.artist, &track.album) {
            (Some(artist_name), Some(album_name)) => {
                let artist = category.add_artist(artist_name.clone());
                let album = artist.add_album(album_name.clone());
                match &track.disc {
                    Some(disc_name) => {
                        let disc = album.add_disc(disc_name.clone());
                        disc.tracks.push(track);
                    }
                    None => {
                        album.tracks.push(track);
                    }
                }
            }
            (Some(artist_name), None) => {
                // No album
                let artist = category.add_artist(artist_name.clone());
                artist.tracks.push(track);
            }
            (None, Some(album_name)) => {
                let album = category.add_album(album_name.clone());
                match &track.disc {
                    Some(disc_name) => {
                        let disc = album.add_disc(disc_name.clone());
                        disc.tracks.push(track.clone());
                    }
                    None => {
                        album.tracks.push(track);
                    }
                }
            }
            (None, None) => {
                // No artist, no album? we don't actually handle this yet but we can swallow the error
                log::error!("No artist or album for {:?}", track.path);
            }
        }
    }
}

fn to_track(path: &PathBuf) -> Result<Track, String> {
    let id = generate_id();
    let mut category: String = DEFAULT_CATEGORY.to_string();
    let mut disc: Option<String> = None;

    let mut components: VecDeque<Cow<str>> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect();

    if components.front().is_some() && is_category(components.front()) {
        category = components.pop_front().unwrap().to_string();
    }

    let stem = get_stem(path)?;
    let extension = get_extentsion(path)?;

    // Pop off the name because we got it from the path buf directly
    components.pop_back();

    if is_disc(components.back()) {
        disc = components.pop_back().map(|c| c.to_string());
    }

    let artist = components.pop_front().map(|c| c.to_string());
    let album = components.pop_front().map(|c| c.to_string());
    if !components.is_empty() {
        return Err(format!(
            "Had some extra path components for '{}': {components:?}",
            path.to_string_lossy()
        ));
    }

    Ok(Track {
        id,
        name: stem,
        extension,
        path: path.clone(),
        category,
        artist,
        album,
        disc,
    })
}

fn is_disc(dir: Option<&Cow<str>>) -> bool {
    lazy_static! {
        static ref CD_REGEX: Regex = Regex::new(CD_REGEX_STR).unwrap();
    }
    dir.is_some() && CD_REGEX.is_match(dir.unwrap())
}

fn is_category(dir: Option<&Cow<str>>) -> bool {
    dir.is_some() && dir.unwrap().starts_with(CATEGORY_PREFIX)
}

fn get_extentsion(path: &PathBuf) -> Result<String, String> {
    let maybe_extension = path.extension().map(|s| s.to_string_lossy());
    if maybe_extension.is_none() {
        return Err(format!("Path '{path:?}' has no extension",));
    }
    Ok(maybe_extension.unwrap().to_string())
}

fn get_stem(path: &PathBuf) -> Result<String, String> {
    let maybe_stem = path.file_stem().map(|s| s.to_string_lossy());
    if maybe_stem.is_none() {
        return Err(format!("Path '{path:?}' has no stem",));
    }
    Ok(maybe_stem.unwrap().to_string())
}

/**
 * TODO: convert this to taking an iterator?
 */
pub fn build(files: Vec<PathBuf>) -> Collection {
    let mut builder = CollectionBuilder::new();
    for file in files.iter() {
        match to_track(file) {
            Ok(track) => builder.add_track(track),
            Err(err) => log::error!("{:?}", err),
        }
    }
    builder.collection
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_is_category() {
        assert_eq!(true, is_category(Some(&Cow::from("_Trance"))));
        assert_eq!(false, is_category(Some(&Cow::from("Smashing Pumpkins"))));
    }

    #[test]
    fn test_is_disc() {
        let disc_strings = vec!["CD 1", "CD2", "cd2", "cd 3", "disc 1", "Disc 2", "Disk 3"];
        let non_disc_strings = vec!["C D1", "thing 1", "An Album", "Album Part 2", "CD II"];

        for test_string in disc_strings {
            assert_eq!(true, is_disc(Some(&Cow::from(test_string))));
        }
        for test_string in non_disc_strings {
            assert_eq!(false, is_disc(Some(&Cow::from(test_string))));
        }
    }

    #[test]
    fn test_to_track_no_disc_no_category() {
        let path = PathBuf::from("Smashing Pumpkins/Gish/01 I Am One.mp3");

        let result = to_track(&path);

        assert_matches!(result, Ok(_));
        assert_track_matches(
            &result.unwrap(),
            Track {
                id: String::from("any"),
                path,
                name: String::from("01 I Am One"),
                extension: String::from("mp3"),
                artist: Some(String::from("Smashing Pumpkins")),
                album: Some(String::from("Gish")),
                category: String::from(DEFAULT_CATEGORY),
                disc: None,
            },
        );
    }

    #[test]
    fn test_to_track_no_disc_with_category() {
        let path = PathBuf::from("_Grunge/Smashing Pumpkins/Gish/01 I Am One.mp3");

        let result = to_track(&path);

        assert_matches!(result, Ok(_));
        assert_track_matches(
            &result.unwrap(),
            Track {
                id: String::from("any"),
                path,
                name: String::from("01 I Am One"),
                extension: String::from("mp3"),
                artist: Some(String::from("Smashing Pumpkins")),
                album: Some(String::from("Gish")),
                category: String::from("_Grunge"),
                disc: None,
            },
        );
    }

    #[test]
    fn test_to_track_with_disc_with_category() {
        let path = PathBuf::from("_Grunge/Smashing Pumpkins/Mellon Collie and the Infinite Sadness/CD 1/01 Mellon Collie And The Infinite Sadness.mp3");

        let result = to_track(&path);

        assert_matches!(result, Ok(_));
        assert_track_matches(
            &result.unwrap(),
            Track {
                id: String::from("any"),
                path,
                name: String::from("01 Mellon Collie And The Infinite Sadness"),
                extension: String::from("mp3"),
                artist: Some(String::from("Smashing Pumpkins")),
                album: Some(String::from("Mellon Collie and the Infinite Sadness")),
                category: String::from("_Grunge"),
                disc: Some(String::from("CD 1")),
            },
        );
    }

    #[test]
    fn test_to_track_no_album_with_category() {
        let path = PathBuf::from("_Grunge/Smashing Pumpkins/01 I Am One.mp3");

        let result = to_track(&path);

        assert_matches!(result, Ok(_));
        assert_track_matches(
            &result.unwrap(),
            Track {
                id: String::from("any"),
                path,
                name: String::from("01 I Am One"),
                extension: String::from("mp3"),
                artist: Some(String::from("Smashing Pumpkins")),
                album: None,
                category: String::from("_Grunge"),
                disc: None,
            },
        );
    }

    // Assert that every field except for ID matches
    fn assert_track_matches(a: &Track, b: Track) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.extension, b.extension);
        assert_eq!(a.path, b.path);
        assert_eq!(a.album, b.album);
        assert_eq!(a.artist, b.artist);
        assert_eq!(a.category, b.category);
        assert_eq!(a.disc, b.disc);
    }
}
