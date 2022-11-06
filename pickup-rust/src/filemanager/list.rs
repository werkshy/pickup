use crate::filemanager::cache::{self, CacheOptions};

use super::model::{Album, Artist, Category};

pub fn list(cache_options: CacheOptions) -> std::io::Result<()> {
    let collection = cache::init(cache_options).unwrap();
    log::info!("We have got {} categories", collection.len());

    for (category_name, category) in collection {
        log::info!("[cat]{category_name}");
        list_category(&category);
    }

    Ok(())
}

fn list_category(category: &Category) {
    for (artist_name, artist) in &category.artists {
        log::info!("  [ar]{artist_name}");
        list_artist(artist, 4);
    }
}

fn list_artist(artist: &Artist, indent: usize) {
    let space = " ".repeat(indent);
    for album in artist.albums.values() {
        list_album(album, indent + 2);
    }
    if !artist.tracks.is_empty() {
        log::info!("{space}[bare tracks]");
        for track in &artist.tracks {
            log::info!("{space}  [tr]{}", track.name);
        }
    }
}

fn list_album(album: &Album, indent: usize) {
    let space = " ".repeat(indent);
    log::info!("{space}[al]{}", album.name);

    for track in &album.tracks {
        log::info!("{space}  [tr]{}", track.name);
    }
}
