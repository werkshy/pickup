use serde::{Deserialize, Serialize};
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::options::CollectionOptions;

static DB_ROOT_PATH: &str = ".cache";
const DEFAULT_IGNORES: [&str; 2] = ["Music", "Audio Music Apps"];

#[derive(Serialize, Deserialize, Debug)]
struct File {
    path: String,
    id: String,
}

pub fn init(options: CollectionOptions) -> std::io::Result<Vec<PathBuf>> {
    let db_path = get_db_path(&options.dir);
    if !db_path.exists() {
        log::info!("DB does not exist '{:?}': refreshing", db_path);
        return refresh(options);
    }

    load(&db_path)
}

// Assumes that the music dir exists
// TODO we could wrap the DB in a RAAI type struct?
fn load(path: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    log::info!("Loading music dir at {:?}", path);
    let db: sled::Db = sled::open(path.as_path())?;

    Ok(db_to_files(db))
}

// TODO shouldn't this be async?!
pub fn refresh(options: CollectionOptions) -> std::io::Result<Vec<PathBuf>> {
    log::info!("Refreshing music dir at {}", options.dir);

    let ignores: Vec<String> = options
        .ignores
        .unwrap_or_else(|| DEFAULT_IGNORES.iter().map(|i| i.to_string()).collect());
    let ignore_set: HashSet<String> = HashSet::from_iter(ignores);

    let db_path = get_db_path(&options.dir);
    if db_path.exists() {
        log::info!("Deleting existing DB at {:?}", db_path);
        fs::remove_dir_all(db_path.as_path())?;
    }
    let mut db: sled::Db = sled::open(db_path).unwrap();
    walk_dirs(options.dir, &mut db, ignore_set);
    db.flush().unwrap();

    // Minor performance hit but reading from the DB is a lot quicker than the refresh itself.
    Ok(db_to_files(db))
}

/**
 * Hash a string and return a string of the hex digest.
 */
fn hash(s: &str) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/**
 * Get a unique path for a cache of the current music_dir by hashing music_dir
 * and using DB_ROOT_PATH as the root
 */
fn get_db_path(music_dir: &str) -> PathBuf {
    Path::new(DB_ROOT_PATH).join(hash(music_dir))
}

fn db_to_files(db: sled::Db) -> Vec<PathBuf> {
    let first_key = db.first().unwrap().unwrap().0;
    db.range(first_key..)
        .map(|kv| deserialize(&kv.unwrap().1))
        .map(|file| PathBuf::from(file.path))
        .collect()
}

fn deserialize(bytes: &[u8]) -> File {
    let decoded: File = bincode::deserialize(bytes).unwrap();
    decoded
}

fn serialize(path: PathBuf, id: String) -> Vec<u8> {
    let file = File {
        path: path.to_str().unwrap().to_string(),
        id,
    };
    bincode::serialize(&file).unwrap()
}

fn walk_dirs(dir: String, db: &mut sled::Db, ignores: HashSet<String>) {
    log::info!("Walking {}", dir);
    let mut i: u64 = 0;
    for entry in WalkDir::new(dir.as_str())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
    {
        let full_path = entry.into_path();
        let path = full_path.strip_prefix(&dir).unwrap().to_path_buf();
        let root = get_root(path.as_path());
        if ignores.contains(&root) {
            continue;
        }

        let id = hash(path.to_str().unwrap());
        let encoded = serialize(path, id);
        db.insert(i.to_be_bytes(), encoded).unwrap();
        i += 1;
        if i % 500 == 0 {
            log::info!("{}", i);
        }
    }
    log::info!("Found {} entries", i);
}

fn get_root(path: &Path) -> String {
    path.components()
        .next() // First Component
        .map(|path| path.as_os_str().to_str().unwrap().to_string())
        .unwrap() // We are guaranteed to have at least one component in the Path
}
