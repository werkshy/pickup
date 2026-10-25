use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use super::{
    cache,
    collection_builder::build,
    dto::CollectionLocation,
    model::{Folder, Kind, Track},
    options::CollectionOptions,
    utils::{generate_album_id, generate_artist_id, generate_category_id, generate_disc_id},
};

pub struct Collection {
    pub categories: BTreeMap<String, Folder>,
    /// Every folder by ID, so it can be addressed without knowing where it sits.
    folders_by_id: HashMap<String, CollectionLocation>,
    /// Tracks by ID, so a track can be addressed without knowing its folder.
    tracks_by_id: HashMap<String, CollectionLocation>,
}

pub fn init(options: CollectionOptions) -> std::io::Result<Collection> {
    let files = cache::init(options)?;
    let collection = build(files);
    Ok(collection)
}

impl Default for Collection {
    fn default() -> Self {
        Self::new()
    }
}

impl Collection {
    pub fn new() -> Self {
        Self {
            categories: BTreeMap::new(),
            folders_by_id: HashMap::new(),
            tracks_by_id: HashMap::new(),
        }
    }

    // TODO: implement iterator for Collection?
    // https://dev.to/wrongbyte/implementing-iterator-and-intoiterator-in-rust-3nio
    pub fn values(&self) -> impl Iterator<Item = &Folder> {
        self.categories.values()
    }

    /**
     * Get the folder at `location`, creating it (and any missing ancestors) if
     * it doesn't exist yet.
     *
     * The ID is a hash of `path`, so the same directory always gets the same
     * one. Deriving it here, in the same call that puts the folder in the tree
     * and in the index, means there is no way to do one without the others.
     */
    pub fn add_folder(
        &mut self,
        kind: Kind,
        location: &CollectionLocation,
        path: &Path,
    ) -> &mut Folder {
        // Create the ancestors first, so the parent lookup below always
        // succeeds. This also lets callers add an album without first walking
        // up to create its category and artist.
        if let Some(parent) = parent_location(location) {
            let parent_kind = parent_kind(kind, location);
            let parent_path = parent_path(kind, location, path);
            self.add_folder(parent_kind, &parent, &parent_path);
        }

        let key = folder_key(path);
        let id = generate_folder_id(kind, path);
        let exists = match parent_location(location) {
            Some(parent) => self
                .folder_at(&parent)
                .is_some_and(|folder| folder.folders.contains_key(&key)),
            None => self.categories.contains_key(&key),
        };

        if !exists {
            self.folders_by_id.insert(id.clone(), location.clone());
        }

        let folders = match parent_location(location) {
            Some(parent) => {
                &mut self
                    .folder_mut(&parent)
                    .expect("parent folder was just created")
                    .folders
            }
            None => &mut self.categories,
        };
        if !exists {
            folders.insert(
                key.clone(),
                Folder {
                    id,
                    path: path.to_path_buf(),
                    name: folder_name(kind, path),
                    kind,
                    folders: BTreeMap::new(),
                    tracks: Vec::new(),
                },
            );
        }
        folders.get_mut(&key).unwrap()
    }

    /**
     * Add a track to the folder its location points at, and record it in the ID
     * index. Returns false if the location doesn't resolve to a folder, in
     * which case the track is dropped.
     */
    pub fn add_track(&mut self, track: Track, location: &CollectionLocation) -> bool {
        let id = track.id.clone();
        let indexed = CollectionLocation {
            track: Some(track.name.clone()),
            ..location.clone()
        };

        {
            let Some(folder) = self.folder_mut(location) else {
                return false;
            };
            folder.tracks.push(track);
        }

        self.tracks_by_id.insert(id, indexed);
        true
    }

    /**
     * Look a folder up by ID.
     */
    pub fn get_folder(&self, id: &str) -> Option<&Folder> {
        let location = self.folders_by_id.get(id)?;
        self.folder_at(location)
    }

    /**
     * Look a track up by ID.
     */
    pub fn get_track(&self, id: &str) -> Option<&Track> {
        let location = self.tracks_by_id.get(id)?;
        let folder = self.folder_at(location)?;
        folder.tracks.iter().find(|track| track.id == id)
    }

    pub fn all_tracks(&self) -> Vec<&Track> {
        let mut tracks = Vec::new();
        for category in self.values() {
            tracks.extend(category.all_tracks());
        }
        tracks
    }

    /**
     * Return the flat list of tracks under a certain location (identified by
     * category, artist, album, and disc). If the location is not found, return
     * None.
     */
    pub fn get_tracks_under(&self, location: &CollectionLocation) -> Option<Vec<&Track>> {
        let folder = self.folder_at(location)?;
        let tracks = folder.all_tracks();

        // No track named, so the whole folder.
        let name = match &location.track {
            Some(name) => name,
            None => return Some(tracks),
        };
        tracks
            .into_iter()
            .find(|track| &track.name == name)
            .map(|track| vec![track])
    }

    /**
     * The folder a location points at, if any. A location with a track names the
     * folder that holds it, so the track is ignored here.
     */
    fn folder_at(&self, location: &CollectionLocation) -> Option<&Folder> {
        let mut folder = self.categories.get(&location.category)?;
        if let Some(artist) = &location.artist {
            folder = folder.folders.get(artist)?;
        }
        if let Some(album) = &location.album {
            folder = folder.folders.get(album)?;
        }
        if let Some(disc) = &location.disc {
            folder = folder.folders.get(disc)?;
        }
        Some(folder)
    }

    fn folder_mut(&mut self, location: &CollectionLocation) -> Option<&mut Folder> {
        let mut folder = self.categories.get_mut(&location.category)?;
        if let Some(artist) = &location.artist {
            folder = folder.folders.get_mut(artist)?;
        }
        if let Some(album) = &location.album {
            folder = folder.folders.get_mut(album)?;
        }
        if let Some(disc) = &location.disc {
            folder = folder.folders.get_mut(disc)?;
        }
        Some(folder)
    }
}

/**
 * The location of the folder that contains this one, or None for a category.
 */
fn parent_location(location: &CollectionLocation) -> Option<CollectionLocation> {
    if location.disc.is_some() {
        Some(CollectionLocation {
            disc: None,
            track: None,
            ..location.clone()
        })
    } else if location.album.is_some() {
        Some(CollectionLocation {
            album: None,
            disc: None,
            track: None,
            ..location.clone()
        })
    } else if location.artist.is_some() {
        Some(CollectionLocation {
            artist: None,
            album: None,
            disc: None,
            track: None,
            ..location.clone()
        })
    } else {
        None
    }
}

/**
 * What kind of folder contains a folder of this kind.
 */
fn parent_kind(kind: Kind, location: &CollectionLocation) -> Kind {
    match kind {
        Kind::Artist => Kind::Category,
        // A bare album hangs off the category, one under an artist off the
        // artist.
        Kind::Album if location.artist.is_some() => Kind::Artist,
        Kind::Album => Kind::Category,
        Kind::Disc => Kind::Album,
        Kind::Category | Kind::Track => {
            unreachable!("{kind:?} folders have no parent")
        }
    }
}

/**
 * The path of the folder that contains this one. A top-level folder's parent is
 * the category directory, which may be implicit (the default category), so fall
 * back to the category's logical path.
 */
fn parent_path(kind: Kind, location: &CollectionLocation, path: &Path) -> PathBuf {
    match kind {
        // A category's directory name is exactly what `location.category`
        // holds, including the implicit default one.
        Kind::Artist => PathBuf::from(&location.category),
        // A bare album hangs off the category; an album under an artist off
        // the directory above it.
        Kind::Album if location.artist.is_none() => PathBuf::from(&location.category),
        Kind::Album | Kind::Disc => path.parent().unwrap_or(path).to_path_buf(),
        Kind::Category | Kind::Track => PathBuf::new(),
    }
}

fn generate_folder_id(kind: Kind, path: &Path) -> String {
    match kind {
        Kind::Category => generate_category_id(path),
        Kind::Artist => generate_artist_id(path),
        Kind::Album => generate_album_id(path),
        Kind::Disc => generate_disc_id(path),
        Kind::Track => unreachable!("tracks are not folders"),
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

/// The key a folder is stored under: its directory name.
fn folder_key(path: &Path) -> String {
    file_name(path)
}

/// The display name of a folder. A category's leading underscore is stripped.
fn folder_name(kind: Kind, path: &Path) -> String {
    let name = file_name(path);
    match kind {
        Kind::Category => name.strip_prefix('_').unwrap_or(name.as_str()).to_string(),
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::filemanager::model::factories::*;

    #[test]
    fn test_add_category() {
        let mut collection = Collection::new();
        let category = collection.add_folder(
            Kind::Category,
            &collection_location("Music", None, None, None),
            Path::new("Music"),
        );
        assert_eq!(category.name, "Music");

        let category = collection.add_folder(
            Kind::Category,
            &collection_location("Cat2", None, None, None),
            Path::new("Cat2"),
        );
        assert_eq!(category.name, "Cat2");

        let category = collection.add_folder(
            Kind::Category,
            &collection_location("Music", None, None, None),
            Path::new("Music"),
        );
        assert_eq!(category.name, "Music");

        assert_eq!(collection.categories.len(), 2);
    }

    #[test]
    fn test_get_artist_by_id() {
        let collection = build_test_collection();

        let id = collection.categories["Music"].folders["Artist One"]
            .id
            .clone();
        let artist = collection.get_folder(&id).unwrap();

        assert_eq!(artist.id, id);
        assert_eq!(artist.name, "Artist One");
        assert_eq!(artist.kind, Kind::Artist);
        assert_eq!(artist.folders.len(), 1);
    }

    #[test]
    fn test_get_album_by_id() {
        let collection = build_test_collection();

        let id = collection.categories["Music"].folders["Artist One"].folders["Album One"]
            .id
            .clone();
        let album = collection.get_folder(&id).unwrap();

        assert_eq!(album.id, id);
        assert_eq!(album.name, "Album One");
        assert_eq!(album.kind, Kind::Album);
        assert_eq!(album.tracks.len(), 2);
    }

    /**
     * A disc is an album nested under an album, and is addressed the same way.
     */
    #[test]
    fn test_get_disc_by_id() {
        let collection = build_test_collection();

        let id = collection.categories["Music"].folders["Artist Two"].folders["Album Two"].folders
            ["Disc One"]
            .id
            .clone();
        let disc = collection.get_folder(&id).unwrap();

        assert_eq!(disc.id, id);
        assert_eq!(disc.name, "Disc One");
        assert_eq!(disc.kind, Kind::Disc);
        assert_eq!(disc.tracks.len(), 1);
    }

    #[test]
    fn test_get_track_by_id() {
        let collection = build_test_collection();

        let id = collection.categories["_Other"].folders["Artist Three"].folders["Album Three"]
            .tracks[0]
            .id
            .clone();
        let track = collection.get_track(&id).unwrap();

        assert_eq!(track.id, id);
        assert_eq!(track.name, "Album Three Track One");
    }

    #[test]
    fn test_get_track_by_id_on_a_disc() {
        let collection = build_test_collection();

        let id = collection.categories["Music"].folders["Artist Two"].folders["Album Two"].folders
            ["Disc Two"]
            .tracks[0]
            .id
            .clone();

        assert_eq!(
            collection.get_track(&id).unwrap().name,
            "Album Two Disc 2 Track One"
        );
    }

    /**
     * A track can sit directly under an artist with no album, so the lookup has
     * to find it there too.
     */
    #[test]
    fn test_get_track_by_id_for_bare_artist_track() {
        let mut collection = Collection::new();
        let location = collection_location("Music", Some("Solo"), None, None);
        collection.add_folder(Kind::Artist, &location, Path::new("Solo"));
        let track = test_track(&location, "Bare");
        let id = track.id.clone();
        assert!(collection.add_track(track, &location));

        assert_eq!(collection.get_track(&id).unwrap().name, "Bare");
    }

    #[test]
    fn test_get_by_unknown_id_returns_none() {
        let collection = build_test_collection();

        assert!(collection.get_folder("cat_nope").is_none());
        assert!(collection.get_track("trk_nope").is_none());
    }

    /**
     * Every folder in the tree has to be reachable by its ID, otherwise
     * something in the tree can't be addressed.
     */
    #[test]
    fn test_every_folder_is_reachable_by_id() {
        let collection = build_test_collection();

        let mut ids = Vec::new();
        collect_ids(collection.categories.values(), &mut ids);

        // Two categories, three artists, four albums, two discs.
        assert_eq!(ids.len(), 11);
        for id in ids {
            assert_eq!(collection.get_folder(&id).unwrap().id, id);
        }
    }

    /**
     * The same artist name in two categories is two separate entries in the
     * tree, so it has to be two separate IDs too.
     */
    #[test]
    fn test_same_name_in_different_categories_has_distinct_ids() {
        let mut collection = Collection::new();
        add_album_tracks(
            &mut collection,
            "Music",
            Some("Shared"),
            "Shared/Album",
            &[],
        );
        add_album_tracks(
            &mut collection,
            "_Other",
            Some("Shared"),
            "_Other/Shared/Album",
            &[],
        );

        let music_id = collection.categories["Music"].folders["Shared"].id.clone();
        let other_id = collection.categories["_Other"].folders["Shared"].id.clone();
        assert_ne!(music_id, other_id);

        assert_eq!(collection.get_folder(&music_id).unwrap().id, music_id);
        assert_eq!(collection.get_folder(&other_id).unwrap().id, other_id);
    }

    /**
     * The tree is keyed by name, so a second path with the same name in the
     * same place reuses the existing folder. Its own ID must not be indexed,
     * because no folder with that ID exists.
     */
    #[test]
    fn test_a_duplicate_folder_name_does_not_index_its_other_ids() {
        let mut collection = Collection::new();
        let location = collection_location("Music", Some("Shared"), None, None);
        let first = collection
            .add_folder(Kind::Artist, &location, Path::new("One/Shared"))
            .id
            .clone();
        let second_id = generate_artist_id(Path::new("Two/Shared"));

        collection.add_folder(Kind::Artist, &location, Path::new("Two/Shared"));

        assert_eq!(collection.categories["Music"].folders.len(), 1);
        assert_eq!(collection.get_folder(&first).unwrap().id, first);
        assert!(collection.get_folder(&second_id).is_none());
    }

    /**
     * A track with neither an artist nor an album has nowhere to live, so it is
     * dropped rather than indexed: the index must not point at a track that
     * isn't in the tree.
     */
    #[test]
    fn test_unplaceable_track_is_dropped_rather_than_indexed() {
        let mut collection = Collection::new();
        // A location with no artist and no album: nowhere to put a track.
        let location = collection_location("Music", None, None, None);
        let track = test_track(&location, "Loose");
        let id = track.id.clone();

        assert!(!collection.add_track(track, &location));

        assert!(collection.get_track(&id).is_none());
        assert_eq!(collection.all_tracks().len(), 0);
    }

    #[test]
    fn test_all_tracks() {
        let collection = build_test_collection();

        let tracks = collection.all_tracks();

        assert_eq!(tracks.len(), 8);
        let track_names = tracks
            .into_iter()
            .map(|track| track.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(
            track_names,
            vec![
                "Album One Track One",
                "Album One Track Two",
                "Album Two Disc 1 Track One",
                "Album Two Disc 2 Track One",
                "Album Four Track One", // Bare album comes before artist albums
                "Album Four Track Two",
                "Album Three Track One",
                "Album Three Track Two",
            ]
        );
    }

    #[test]
    fn test_get_tracks_under_category() {
        let collection = build_test_collection();

        let maybe_tracks =
            collection.get_tracks_under(&collection_location("Music", None, None, None));

        assert!(maybe_tracks.is_some());

        let track_names = maybe_tracks
            .unwrap()
            .into_iter()
            .map(|track| track.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(
            track_names,
            vec![
                "Album One Track One",
                "Album One Track Two",
                "Album Two Disc 1 Track One",
                "Album Two Disc 2 Track One",
            ]
        );
    }

    #[test]
    fn test_get_tracks_under_artist() {
        let collection = build_test_collection();

        let maybe_tracks = collection.get_tracks_under(&collection_location(
            "Music",
            Some("Artist One"),
            None,
            None,
        ));

        assert!(maybe_tracks.is_some());

        let track_names = maybe_tracks
            .unwrap()
            .into_iter()
            .map(|track| track.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(
            track_names,
            vec!["Album One Track One", "Album One Track Two",]
        );
    }

    #[test]
    fn test_get_tracks_under_artist_album() {
        let collection = build_test_collection();

        let maybe_tracks = collection.get_tracks_under(&collection_location(
            "Music",
            Some("Artist Two"),
            Some("Album Two".to_string()),
            None,
        ));

        assert!(maybe_tracks.is_some());

        let track_names = maybe_tracks
            .unwrap()
            .into_iter()
            .map(|track| track.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(
            track_names,
            vec!["Album Two Disc 1 Track One", "Album Two Disc 2 Track One",]
        );
    }

    #[test]
    fn test_get_tracks_under_disc() {
        let collection = build_test_collection();

        let maybe_tracks = collection.get_tracks_under(&collection_location(
            "Music",
            Some("Artist Two"),
            Some("Album Two".to_string()),
            Some("Disc One".to_string()),
        ));

        assert!(maybe_tracks.is_some());

        let track_names = maybe_tracks
            .unwrap()
            .into_iter()
            .map(|track| track.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(track_names, vec!["Album Two Disc 1 Track One",]);
    }

    #[test]
    fn test_get_tracks_under_bare_album() {
        let collection = build_test_collection();

        let maybe_tracks = collection.get_tracks_under(&collection_location(
            "_Other",
            None,
            Some("Album Four".to_string()),
            None,
        ));

        assert!(maybe_tracks.is_some());

        let track_names = maybe_tracks
            .unwrap()
            .into_iter()
            .map(|track| track.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(
            track_names,
            vec!["Album Four Track One", "Album Four Track Two",]
        );
    }

    #[test]
    fn test_get_tracks_under_track() {
        let collection = build_test_collection();

        let mut location =
            collection_location("_Other", None, Some("Album Four".to_string()), None);
        location.track = Some("Album Four Track One".to_string());

        let maybe_tracks = collection.get_tracks_under(&location);

        assert!(maybe_tracks.is_some());

        let track_names = maybe_tracks
            .unwrap()
            .into_iter()
            .map(|track| track.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(track_names, vec!["Album Four Track One",]);
    }

    /**
     * Build a test collection with the following types of data:
     * 1. The default "Music" category containing two artists
     *    a) Each artist has one album.
     *    b) One album has two discs.
     * 2. A category with a name starting with an underscore:
     *    a) One artist with an album
     *    b) One bare album (no artist)
     *
     * Built through `add_folder` rather than by discovery, because discovery
     * reads a category's first directory as an artist and so can't produce a
     * bare album. The paths are the ones each folder would really sit at.
     */
    fn build_test_collection() -> Collection {
        let mut collection = Collection::new();

        // 1. First artist in default category
        add_album_tracks(
            &mut collection,
            "Music",
            Some("Artist One"),
            "Artist One/Album One",
            &["Album One Track One", "Album One Track Two"],
        );

        // 2. Second artist in default category, one album with two discs
        add_disc_tracks(
            &mut collection,
            "Music",
            Some("Artist Two"),
            "Artist Two/Album Two/Disc One",
            &["Album Two Disc 1 Track One"],
        );
        add_disc_tracks(
            &mut collection,
            "Music",
            Some("Artist Two"),
            "Artist Two/Album Two/Disc Two",
            &["Album Two Disc 2 Track One"],
        );

        // 3. Artist in non-default category
        add_album_tracks(
            &mut collection,
            "_Other",
            Some("Artist Three"),
            "_Other/Artist Three/Album Three",
            &["Album Three Track One", "Album Three Track Two"],
        );

        // 4. Bare album in non-default category
        add_album_tracks(
            &mut collection,
            "_Other",
            None,
            "_Other/Album Four",
            &["Album Four Track One", "Album Four Track Two"],
        );

        collection
    }

    fn add_album_tracks(
        collection: &mut Collection,
        category: &str,
        artist: Option<&str>,
        path: &str,
        tracks: &[&str],
    ) {
        let location = album_location(category, artist, path);
        collection.add_folder(Kind::Album, &location, Path::new(path));
        for name in tracks {
            collection.add_track(test_track(&location, name), &location);
        }
    }

    fn add_disc_tracks(
        collection: &mut Collection,
        category: &str,
        artist: Option<&str>,
        path: &str,
        tracks: &[&str],
    ) {
        let location = disc_location(category, artist, path);
        collection.add_folder(Kind::Disc, &location, Path::new(path));
        for name in tracks {
            collection.add_track(test_track(&location, name), &location);
        }
    }

    /// The location of the album directory at `path`.
    fn album_location(category: &str, artist: Option<&str>, path: &str) -> CollectionLocation {
        collection_location(category, artist, Some(name_of(Path::new(path))), None)
    }

    /// The location of the disc directory at `path`, whose parent is the album.
    fn disc_location(category: &str, artist: Option<&str>, path: &str) -> CollectionLocation {
        let path = Path::new(path);
        let album = path.parent().map(name_of).unwrap_or_default();
        collection_location(category, artist, Some(album), Some(name_of(path)))
    }

    fn collection_location(
        category: &str,
        artist: Option<&str>,
        album: Option<String>,
        disc: Option<String>,
    ) -> CollectionLocation {
        CollectionLocation {
            category: category.to_string(),
            artist: artist.map(String::from),
            album,
            disc,
            track: None,
        }
    }

    fn name_of(path: &Path) -> String {
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    fn test_track(location: &CollectionLocation, name: &str) -> Track {
        Track {
            id: random_string(),
            name: name.to_string(),
            extension: String::from("mp3"),
            path: PathBuf::from(name),
            category: location.category.clone(),
            artist: location.artist.clone(),
            album: location.album.clone(),
            disc: location.disc.clone(),
        }
    }

    fn collect_ids<'a>(folders: impl Iterator<Item = &'a Folder>, ids: &mut Vec<String>) {
        for folder in folders {
            ids.push(folder.id.clone());
            collect_ids(folder.folders.values(), ids);
        }
    }
}
