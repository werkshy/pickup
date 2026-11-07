use std::time::Instant;

use crate::filemanager::{
    collection::init,
    model::{Folder, Kind},
    options::CollectionOptions,
};

pub fn list(collection_options: CollectionOptions, depth: Kind) -> std::io::Result<()> {
    let t0 = Instant::now();
    let collection = init(collection_options).unwrap();
    let t1 = Instant::now();
    log::info!("We have got {} categories", collection.categories.len());

    for (category_name, category) in &collection.categories {
        print(category_name, 0, Kind::Category, depth);
        list_folder(category, 2, depth);
    }
    let t2 = Instant::now();
    log::info!(
        "Timings: Load={:?}; List={:?}; Total={:?}",
        t1 - t0,
        t2 - t1,
        t2 - t0
    );

    Ok(())
}

fn list_folder(folder: &Folder, indent: usize, depth: Kind) {
    for child in folder.folders.values() {
        print(&child.name, indent, child.kind, depth);
        list_folder(child, indent + 2, depth);
    }
    for track in &folder.tracks {
        print(&track.name, indent, Kind::Track, depth);
    }
}

fn print(name: &str, indent: usize, kind: Kind, depth: Kind) {
    let space = " ".repeat(indent);
    let prefix = match kind {
        Kind::Category => "[cat]",
        Kind::Artist => "[ar]",
        Kind::Album => "[al]",
        Kind::Disc => "[disc]",
        Kind::Track => "[tr]",
    };
    if kind as u8 <= depth as u8 {
        log::info!("{space}{prefix}{name}");
    }
}
