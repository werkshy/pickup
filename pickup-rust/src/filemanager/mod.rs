use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// I compared using walkdir to just using fs to visit every dir and it is
// significantly faster on the network drive.

static DB_ROOT_PATH: &str = ".cache";

pub type MusicDb = Vec<PathBuf>;

pub fn load(dir: String) -> std::io::Result<MusicDb> {
    log::info!("Loading music dir at {}", dir);
    // TODO check that the music dir exists
    let db_path = get_db_path(&dir);
    if !db_path.exists() {
        log::info!("DB does not exist '{:?}': refreshing", db_path);
        return refresh(dir);
    }
    let db: sled::Db = sled::open(db_path.as_path()).unwrap();

    Ok(db_to_files(db))
}

pub fn refresh(dir: String) -> std::io::Result<MusicDb> {
    log::info!("Refreshing music dir at {}", dir);
    // TODO check that the music dir exists
    let db_path = get_db_path(&dir);
    if db_path.exists() {
        log::info!("Deleting existing DB at {:?}", db_path);
        fs::remove_dir_all(db_path.as_path())?;
    }
    let mut db: sled::Db = sled::open(db_path).unwrap();
    walk_dirs(dir, &mut db);
    db.flush().unwrap();
    Ok(db_to_files(db))
}

/**
 * Get a unique path for a cache of the current music_dir by hashing music_dir
 * and using DB_ROOT_PATH as the root
 */
fn get_db_path(music_dir: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    let _hash = music_dir.hash(&mut hasher);
    let s = format!("{:x}", hasher.finish());

    Path::new(DB_ROOT_PATH).join(s)
}

/**
 * Once we've go a DB with the saved files in it, we can turn that into the in-memory structure
 * we want at runtime. For now, this is just a Vec<PathBuf>.
 */
fn db_to_files(db: sled::Db) -> MusicDb {
    let first_key = db.first().unwrap().unwrap().0;
    db.range(first_key..)
        .map(|kv| deserialize(&kv.unwrap().1))
        .collect()
}

fn deserialize(bytes: &sled::IVec) -> PathBuf {
    let decoded: Option<String> = bincode::deserialize(&bytes[..]).unwrap();
    PathBuf::from(decoded.unwrap())
}

fn walk_dirs(dir: String, db: &mut sled::Db) {
    log::info!("Walking {}", dir);
    let mut i: u64 = 0;
    for entry in WalkDir::new(dir.as_str())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
    {
        let encoded = bincode::serialize(&entry.into_path().to_str()).unwrap();
        db.insert(i.to_be_bytes(), encoded).unwrap();
        i += 1;
        if i % 500 == 0 {
            log::info!("{}", i);
        }
    }
    log::info!("Found {} entries", i);
}
