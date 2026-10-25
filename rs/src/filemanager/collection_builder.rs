use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use regex::Regex;

use super::{
    collection::Collection,
    dto::CollectionLocation,
    model::{Kind, Track},
    utils::generate_track_id,
};

const DEFAULT_CATEGORY: &str = "Music";
const CATEGORY_PREFIX: &str = "_";
const CD_REGEX_STR: &str = r"(?i)(cd|dis(c|k)|digital media|(12 )?vinyl) ?\d";

static CD_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(CD_REGEX_STR).unwrap());

/**
 * What we worked out about a directory: what it is, and the location that led
 * to it. A directory's children need both to work out their own.
 */
struct Dir {
    kind: Kind,
    location: CollectionLocation,
}

pub fn build(files: Vec<PathBuf>) -> Collection {
    let mut collection = Collection::new();
    let mut dirs: HashMap<PathBuf, Dir> = HashMap::new();

    // Directories first, parents before children, so that each one is created
    // inside the container its parent made.
    for path in directories(&files) {
        let dir = match path.parent() {
            // The music dir itself has no parent, so this is a top-level
            // directory.
            Some(parent) if parent.as_os_str().is_empty() => add_top_level(&mut collection, &path),
            // A parent that isn't in `dirs` was deliberately dropped (a
            // non-disc directory under an album, or anything below a disc), so
            // this directory goes with it rather than being promoted.
            Some(parent) => match dirs.get(parent) {
                Some(parent) => add_child(&mut collection, parent, &path),
                None => None,
            },
            None => None,
        };
        if let Some(dir) = dir {
            dirs.insert(path, dir);
        }
    }

    for file in files.iter() {
        match file.parent().and_then(|parent| dirs.get(parent)) {
            Some(parent) => match to_track(file, &parent.location) {
                Ok(track) => {
                    if !collection.add_track(track, &parent.location) {
                        log::error!("No container for {:?}", file);
                    }
                }
                Err(err) => log::error!("{:?}", err),
            },
            None => log::error!(
                "Cannot place {:?}: no parent directory in the collection",
                file
            ),
        }
    }

    collection
}

/**
 * A directory directly inside the music dir. A leading underscore means it's a
 * category; anything else is an artist in the default category, since there is
 * no directory above it to say otherwise.
 */
fn add_top_level(collection: &mut Collection, path: &Path) -> Option<Dir> {
    let name = file_name(path);
    if is_category(&name) {
        let location = location(&name, None, None, None);
        collection.add_folder(Kind::Category, &location, path);
        return Some(Dir {
            kind: Kind::Category,
            location,
        });
    }

    let location = location(DEFAULT_CATEGORY, Some(name), None, None);
    collection.add_folder(Kind::Artist, &location, path);
    Some(Dir {
        kind: Kind::Artist,
        location,
    })
}

/**
 * A directory inside another one, which is what decides what it is. Returns
 * None if there's nowhere for it to go, in which case its files are dropped
 * too.
 */
fn add_child(collection: &mut Collection, parent: &Dir, path: &Path) -> Option<Dir> {
    let name = file_name(path);
    let category = parent.location.category.clone();

    let (kind, location) = match parent.kind {
        Kind::Category => {
            let location = location(&category, Some(name), None, None);
            collection.add_folder(Kind::Artist, &location, path);
            (Kind::Artist, location)
        }
        Kind::Artist => {
            let location = location(&category, parent.location.artist.clone(), Some(name), None);
            collection.add_folder(Kind::Album, &location, path);
            (Kind::Album, location)
        }
        Kind::Album => {
            // Only directories named like a disc are discs. Anything else (an
            // "Artwork" or "Bonus" directory, say) has nowhere to go.
            if !is_disc(&name) {
                return None;
            }
            let location = location(
                &category,
                parent.location.artist.clone(),
                parent.location.album.clone(),
                Some(name),
            );
            collection.add_folder(Kind::Disc, &location, path);
            (Kind::Disc, location)
        }
        // `CollectionLocation` has room for a single disc, and the tree only
        // nests one level, so a directory inside a disc has nowhere to go.
        Kind::Disc => return None,
        Kind::Track => return None,
    };

    Some(Dir { kind, location })
}

/**
 * Every directory holding a file, parents before children.
 *
 * The cache holds file paths rather than directories, so the directories are
 * recovered from the files inside them instead of read off the filesystem. A
 * set keeps them sorted and deduplicated, and sorting puts a parent ahead of
 * its children because a parent's path is a strict prefix of theirs.
 */
fn directories(files: &[PathBuf]) -> Vec<PathBuf> {
    let mut directories: BTreeSet<PathBuf> = BTreeSet::new();
    for file in files {
        // Every ancestor of the file, so a directory holding nothing but
        // directories still turns up. The file itself is skipped, and the empty
        // path that ends the walk is the music dir itself, which isn't a
        // category or an artist.
        for directory in file
            .ancestors()
            .skip(1)
            .take_while(|path| !path.as_os_str().is_empty())
        {
            directories.insert(directory.to_path_buf());
        }
    }
    directories.into_iter().collect()
}

/**
 * The track at this path. Which category, artist, album and disc it belongs to
 * comes from the directory it sits in, which we've already classified.
 */
fn to_track(path: &Path, location: &CollectionLocation) -> Result<Track, String> {
    Ok(Track {
        id: generate_track_id(path),
        name: get_stem(path)?,
        extension: get_extension(path)?,
        path: path.to_path_buf(),
        category: location.category.clone(),
        artist: location.artist.clone(),
        album: location.album.clone(),
        disc: location.disc.clone(),
    })
}

fn is_category(name: &str) -> bool {
    name.starts_with(CATEGORY_PREFIX)
}

fn is_disc(name: &str) -> bool {
    CD_REGEX.is_match(name)
}

fn location(
    category: &str,
    artist: Option<String>,
    album: Option<String>,
    disc: Option<String>,
) -> CollectionLocation {
    CollectionLocation {
        category: category.to_string(),
        artist,
        album,
        disc,
        track: None,
    }
}

/**
 * The name a directory is known by in the tree, which is the last component of
 * its path.
 */
fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn get_extension(path: &Path) -> Result<String, String> {
    let maybe_extension = path.extension().map(|s| s.to_string_lossy());
    if maybe_extension.is_none() {
        return Err(format!("Path '{path:?}' has no extension",));
    }
    Ok(maybe_extension.unwrap().to_string())
}

fn get_stem(path: &Path) -> Result<String, String> {
    let maybe_stem = path.file_stem().map(|s| s.to_string_lossy());
    if maybe_stem.is_none() {
        return Err(format!("Path '{path:?}' has no stem",));
    }
    Ok(maybe_stem.unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use super::super::utils::{generate_album_id, generate_artist_id, generate_disc_id};
    use super::*;

    #[test]
    fn test_is_category() {
        assert!(is_category("_Trance"));
        assert!(!is_category("Smashing Pumpkins"));
    }

    #[test]
    fn test_is_disc() {
        let disc_strings = vec!["CD 1", "CD2", "cd2", "cd 3", "disc 1", "Disc 2", "Disk 3"];
        let non_disc_strings = vec!["C D1", "thing 1", "An Album", "Album Part 2", "CD II"];

        for test_string in disc_strings {
            assert!(is_disc(test_string), "{test_string} should be a disc");
        }
        for test_string in non_disc_strings {
            assert!(!is_disc(test_string), "{test_string} should not be a disc");
        }
    }

    #[test]
    fn test_directories_are_unique_and_parents_come_first() {
        let files = [
            "_Grunge/Smashing Pumpkins/Mellon Collie/CD 2/02.mp3",
            "_Grunge/Smashing Pumpkins/Gish/01.mp3",
            "_Grunge/Smashing Pumpkins/Mellon Collie/CD 1/01.mp3",
        ]
        .map(PathBuf::from);

        let directories = directories(&files);

        assert_eq!(
            directories,
            vec![
                PathBuf::from("_Grunge"),
                PathBuf::from("_Grunge/Smashing Pumpkins"),
                PathBuf::from("_Grunge/Smashing Pumpkins/Gish"),
                PathBuf::from("_Grunge/Smashing Pumpkins/Mellon Collie"),
                PathBuf::from("_Grunge/Smashing Pumpkins/Mellon Collie/CD 1"),
                PathBuf::from("_Grunge/Smashing Pumpkins/Mellon Collie/CD 2"),
            ]
        );
    }

    #[test]
    fn test_directories_ignores_files_at_the_root() {
        let files = ["01 Loose.mp3", "Album/01.mp3"].map(PathBuf::from);

        assert_eq!(directories(&files), vec![PathBuf::from("Album")]);
    }

    /**
     * The whole hierarchy comes out of the directory names: a category holds
     * artists, an artist holds albums, an album holds discs, and anything
     * without a category directory on top lands in the default one.
     */
    #[test]
    fn test_build_nests_by_directory() {
        let collection = build(
            [
                "Smashing Pumpkins/Gish/01 I Am One.mp3",
                "_Grunge/Smashing Pumpkins/Mellon Collie/CD 1/01 Mellon Collie.mp3",
                "_Grunge/Smashing Pumpkins/Mellon Collie/CD 2/02 Siamese Dream.mp3",
                "Alex-Productions/01 Stand Alone.mp3",
            ]
            .map(PathBuf::from)
            .to_vec(),
        );

        let grunge = &collection.categories["_Grunge"];
        let artist = &grunge.folders["Smashing Pumpkins"];
        assert_eq!(artist.path, PathBuf::from("_Grunge/Smashing Pumpkins"));

        let album = &artist.folders["Mellon Collie"];
        assert_eq!(
            album.path,
            PathBuf::from("_Grunge/Smashing Pumpkins/Mellon Collie")
        );
        assert_eq!(album.folders.len(), 2);
        assert_eq!(
            album.folders["CD 1"].path,
            PathBuf::from("_Grunge/Smashing Pumpkins/Mellon Collie/CD 1")
        );

        // No category directory, so the default category.
        let music = &collection.categories["Music"];
        assert_eq!(music.name, "Music");
        assert_eq!(
            music.folders["Smashing Pumpkins"].folders["Gish"]
                .tracks
                .len(),
            1
        );

        // A track straight under an artist, with no album.
        let alex = &collection.categories["Music"].folders["Alex-Productions"];
        assert!(alex.folders.is_empty());
        assert_eq!(alex.tracks.len(), 1);
    }

    /**
     * Every container is reachable by the ID hashed from its own path, and no
     * two of them share one.
     */
    #[test]
    fn test_build_indexes_every_container_by_its_path() {
        let collection = build(
            [
                "Smashing Pumpkins/Gish/01 I Am One.mp3",
                "_Grunge/Smashing Pumpkins/Mellon Collie/CD 1/01 Mellon Collie.mp3",
            ]
            .map(PathBuf::from)
            .to_vec(),
        );

        let artist = collection
            .get_folder(&generate_artist_id(Path::new("_Grunge/Smashing Pumpkins")))
            .unwrap();
        assert_eq!(artist.name, "Smashing Pumpkins");

        let album = collection
            .get_folder(&generate_album_id(Path::new(
                "_Grunge/Smashing Pumpkins/Mellon Collie",
            )))
            .unwrap();
        assert_eq!(album.name, "Mellon Collie");

        let disc = collection
            .get_folder(&generate_disc_id(Path::new(
                "_Grunge/Smashing Pumpkins/Mellon Collie/CD 1",
            )))
            .unwrap();
        assert_eq!(disc.name, "CD 1");

        let track = collection
            .get_track(&generate_track_id(Path::new(
                "Smashing Pumpkins/Gish/01 I Am One.mp3",
            )))
            .unwrap();
        assert_eq!(track.name, "01 I Am One");
        assert_eq!(track.album.as_deref(), Some("Gish"));
        assert_eq!(track.disc, None);
    }

    /**
     * A file with no directory above it has nowhere to go, and is dropped
     * rather than half-added.
     */
    #[test]
    fn test_build_drops_a_file_at_the_root() {
        let collection = build(["01 Loose.mp3"].map(PathBuf::from).to_vec());

        assert!(collection.categories.is_empty());
        assert!(collection.all_tracks().is_empty());
    }

    /**
     * A directory under an album is only a disc if its name looks like one.
     * Anything else would be a level below an album, where there is no room.
     */
    #[test]
    fn test_build_drops_a_non_disc_directory_under_an_album() {
        let collection = build(
            ["Artist/Album/Artwork/01.mp3", "Artist/Album/01.mp3"]
                .map(PathBuf::from)
                .to_vec(),
        );

        let album = &collection.categories["Music"].folders["Artist"].folders["Album"];
        assert!(album.folders.is_empty());
        assert_eq!(album.tracks.len(), 1);
        assert_eq!(album.tracks[0].name, "01");
        assert_eq!(collection.all_tracks().len(), 1);
    }

    /**
     * A directory dropped under an album takes everything below it with it.
     * Descendants must not be promoted to top-level artists just because their
     * own parent is missing from the tree.
     */
    #[test]
    fn test_build_drops_descendants_of_a_non_disc_directory() {
        let collection = build(
            ["Artist/Album/Artwork/More/01.mp3", "Artist/Album/01.mp3"]
                .map(PathBuf::from)
                .to_vec(),
        );

        let music = &collection.categories["Music"];
        assert_eq!(music.folders.len(), 1);
        let album = &music.folders["Artist"].folders["Album"];
        assert!(album.folders.is_empty());
        assert_eq!(collection.all_tracks().len(), 1);
    }

    /**
     * `CollectionLocation` can only name one disc, so a directory inside a disc
     * has nowhere to go. Its files go with it.
     */
    #[test]
    fn test_build_drops_a_directory_inside_a_disc() {
        let collection = build(
            [
                "Artist/Album/CD 1/Extras/01.mp3",
                "Artist/Album/CD 1/02.mp3",
            ]
            .map(PathBuf::from)
            .to_vec(),
        );

        let album = &collection.categories["Music"].folders["Artist"].folders["Album"];
        assert_eq!(album.folders.len(), 1);
        assert_eq!(album.folders["CD 1"].tracks.len(), 1);
        assert_eq!(album.folders["CD 1"].tracks[0].name, "02");
    }

    #[test]
    fn test_to_track() {
        let location = location(
            "_Grunge",
            Some(String::from("Smashing Pumpkins")),
            Some(String::from("Mellon Collie")),
            Some(String::from("CD 1")),
        );
        let path = Path::new("_Grunge/Smashing Pumpkins/Mellon Collie/CD 1/01 Mellon Collie.mp3");

        let track = to_track(path, &location).unwrap();

        assert_eq!(track.id, generate_track_id(path));
        assert_eq!(track.name, "01 Mellon Collie");
        assert_eq!(track.extension, "mp3");
        assert_eq!(track.path, path);
        assert_eq!(track.category, "_Grunge");
        assert_eq!(track.artist.as_deref(), Some("Smashing Pumpkins"));
        assert_eq!(track.album.as_deref(), Some("Mellon Collie"));
        assert_eq!(track.disc.as_deref(), Some("CD 1"));
    }

    #[test]
    fn test_to_track_without_an_extension_is_an_error() {
        let location = location("Music", Some(String::from("Artist")), None, None);

        assert_matches!(to_track(Path::new("Artist/01 Loose"), &location), Err(_));
    }
}
