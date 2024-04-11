use std::collections::BTreeMap;

use super::{
    cache,
    collection_builder::build,
    dto::CollectionLocation,
    model::{Category, Track},
    options::CollectionOptions,
};

pub struct Collection {
    pub categories: BTreeMap<String, Category>,
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
        }
    }

    // TODO: implement iterator for Collection?
    // https://dev.to/wrongbyte/implementing-iterator-and-intoiterator-in-rust-3nio
    pub fn values(&self) -> impl Iterator<Item = &Category> {
        self.categories.values()
    }

    pub fn add_category(&mut self, name: String) -> &mut Category {
        if !(self.categories.contains_key(&name)) {
            let pretty_name = name.strip_prefix('_').unwrap_or(name.as_str()).to_string();
            let category = Category {
                name: pretty_name,
                artists: BTreeMap::new(),
                albums: BTreeMap::new(),
            };
            self.categories.insert(name.clone(), category);
        }
        return self.categories.get_mut(&name).unwrap();
    }

    pub fn all_tracks(&self) -> Vec<&Track> {
        let mut tracks = Vec::new();
        for category in self.values() {
            tracks.extend(category.all_tracks());
        }
        tracks
    }

    /**
     * Return the flat list of tracks under a certain location (idemntified by catrgory, artist, album, and disc).
     * If the location is not found, return None.
     */
    pub fn get_tracks_under(&self, location: &CollectionLocation) -> Option<Vec<&Track>> {
        let maybe_category = self.categories.get(&location.category);
        maybe_category?;
        let category = maybe_category.unwrap();

        let maybe_tracks = match (&location.artist, &location.album, &location.disc) {
            (Some(artist), Some(album), Some(disc)) => category
                .artists
                .get(artist)
                .and_then(|artist| artist.albums.get(album))
                .and_then(|album| album.discs.get(disc))
                .map(|disc| disc.all_tracks()),
            (Some(artist), Some(album), None) => category
                .artists
                .get(artist)
                .and_then(|artist| artist.albums.get(album))
                .map(|album| album.all_tracks()),
            (Some(artist), None, _) => category
                .artists
                .get(artist)
                .map(|artist| artist.all_tracks()),
            (None, Some(album), Some(disc)) => category
                .albums
                .get(album)
                .and_then(|album| album.discs.get(disc))
                .map(|disc| disc.all_tracks()),
            (None, Some(album), None) => category.albums.get(album).map(|album| album.all_tracks()),
            (None, None, _) => Some(category.all_tracks()),
        };

        if location.track.is_none() || maybe_tracks.is_none() {
            return maybe_tracks;
        }
        return maybe_tracks.and_then(|tracks| {
            tracks
                .iter()
                .find(|track| track.name == *location.track.as_ref().unwrap())
                .map(|track| vec![*track])
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filemanager::model::factories::*;

    use factori::create;

    #[test]
    fn test_add_category() {
        let mut collection = Collection::new();
        let category = collection.add_category("Music".to_string());
        assert_eq!(category.name, "Music");

        let category = collection.add_category("Cat2".to_string());
        assert_eq!(category.name, "Cat2");

        let category = collection.add_category("Music".to_string());
        assert_eq!(category.name, "Music");

        assert_eq!(collection.categories.len(), 2);
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

        let maybe_tracks = collection.get_tracks_under(&CollectionLocation {
            category: "Music".to_string(),
            artist: None,
            album: None,
            disc: None,
            track: None,
        });

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

        let maybe_tracks = collection.get_tracks_under(&CollectionLocation {
            category: "Music".to_string(),
            artist: Some("Artist One".to_string()),
            album: None,
            disc: None,
            track: None,
        });

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

        let maybe_tracks = collection.get_tracks_under(&CollectionLocation {
            category: "Music".to_string(),
            artist: Some("Artist Two".to_string()),
            album: Some("Album Two".to_string()),
            disc: None,
            track: None,
        });

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

        let maybe_tracks = collection.get_tracks_under(&CollectionLocation {
            category: "Music".to_string(),
            artist: Some("Artist Two".to_string()),
            album: Some("Album Two".to_string()),
            disc: Some("Disc One".to_string()),
            track: None,
        });

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

        let maybe_tracks = collection.get_tracks_under(&CollectionLocation {
            category: "_Other".to_string(),
            artist: None,
            album: Some("Album Four".to_string()),
            disc: None,
            track: None,
        });

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

        let maybe_tracks = collection.get_tracks_under(&CollectionLocation {
            category: "_Other".to_string(),
            artist: None,
            album: Some("Album Four".to_string()),
            disc: None,
            track: Some("Album Four Track One".to_string()),
        });

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
     */
    fn build_test_collection() -> Collection {
        let mut collection = Collection::new();
        let category = collection.add_category("Music".to_string());
        // 1. First artist in default category
        let artist = category.add_artist("Artist One".to_string());
        let album = artist.add_album("Album One".to_string());
        album.add_track(create!(Track, name: "Album One Track One".to_string()));
        album.add_track(create!(Track, name: "Album One Track Two".to_string()));

        // 2. Second artist in default category
        let artist = category.add_artist("Artist Two".to_string());
        let album = artist.add_album("Album Two".to_string());
        let disc = album.add_disc("Disc One".to_string());
        disc.add_track(create!(Track, name: "Album Two Disc 1 Track One".to_string()));
        let disc = album.add_disc("Disc Two".to_string());
        disc.add_track(create!(Track, name: "Album Two Disc 2 Track One".to_string()));

        // 3. Artist in non-default category
        let category = collection.add_category("_Other".to_string());
        let artist = category.add_artist("Artist Three".to_string());
        let album = artist.add_album("Album Three".to_string());
        album.add_track(create!(Track, name: "Album Three Track One".to_string()));
        album.add_track(create!(Track, name: "Album Three Track Two".to_string()));

        // 4. Bare album in non-default category
        let album = category.add_album("Album Four".to_string());
        album.add_track(create!(Track, name: "Album Four Track One".to_string()));
        album.add_track(create!(Track, name: "Album Four Track Two".to_string()));

        collection
    }
}
