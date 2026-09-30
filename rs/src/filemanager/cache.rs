use rkyv::from_bytes;
use rkyv::{rancor::Error, Archive, Deserialize, Serialize};
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::options::CollectionOptions;
use super::utils::generate_track_id;

static DB_ROOT_PATH: &str = ".cache";
const DEFAULT_IGNORES: [&str; 2] = ["Music", "Audio Music Apps"];

/**
 * Bumped whenever the on-disk schema changes. It is mixed into the DB path hash so
 * that a stale cache simply misses in `init` and gets transparently refreshed,
 * rather than reaching the `unwrap` in `deserialize` and panicking.
 */
const CACHE_VERSION: &str = "v2";

#[derive(Serialize, Deserialize, Debug, Archive)]
struct File {
    path: String,
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
    log::info!("Loading music dir cache at {:?}", path);
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
    db_path_for(music_dir, CACHE_VERSION)
}

/**
 * Build the path for the cache DB, hasing the music dir and a version.
 */
fn db_path_for(music_dir: &str, version: &str) -> PathBuf {
    Path::new(DB_ROOT_PATH).join(hash(&format!("{version}:{music_dir}")))
}

/**
 * Collect every cached file, sorted by path.
 *
 * Entries are keyed by track ID, which is a hash and therefore carries no
 * ordering. We previously leaned on an incrementing integer key to get
 * insertion order back, but that was really the raw `readdir` order of the
 * filesystem, which was arbitrary and not stable across machines. Sorting by
 * path makes the order deterministic and, as a bonus, gives alphabetical
 * track ordering within an album.
 */
fn db_to_files(db: sled::Db) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = db
        .iter()
        .map(|kv| deserialize(&kv.unwrap().1))
        .map(|file| PathBuf::from(file.path))
        .collect();
    paths.sort();
    paths
}

fn deserialize(bytes: &[u8]) -> File {
    //let decoded: File = bincode::deserialize(bytes).unwrap();
    let deserialized: File = from_bytes::<File, Error>(bytes).unwrap();
    deserialized
}

fn serialize(path: PathBuf) -> Vec<u8> {
    let file = File {
        path: path.to_str().unwrap().to_string(),
    };
    //bincode::serialize(&file).unwrap()
    rkyv::to_bytes::<Error>(&file).unwrap().into_vec()
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

        // Key by the same ID the collection builder will derive from this path,
        // so the cache is keyed by track identity and dedupes for free.
        let id = generate_track_id(&path);
        let encoded = serialize(path);
        db.insert(id.as_bytes(), encoded).unwrap();
        i += 1;
        if i.is_multiple_of(500) {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        db_path_for, db_to_files, generate_track_id, get_db_path, serialize, DB_ROOT_PATH,
    };

    #[test]
    fn test_db_path_is_rooted_at_cache_dir() {
        let path = get_db_path("../music");

        assert_eq!(DB_ROOT_PATH, path.parent().unwrap().to_str().unwrap());
    }

    #[test]
    fn test_db_path_is_distinct_per_music_dir() {
        assert_ne!(get_db_path("../music"), get_db_path("../other-music"));
    }

    #[test]
    fn test_db_path_differs_across_cache_versions() {
        // A stale cache from a previous schema must not resolve to the current
        // path, otherwise `init` would try to deserialize old bytes and panic.
        assert_ne!(db_path_for("../music", "v1"), db_path_for("../music", "v2"));
    }

    #[test]
    fn test_db_to_files_sorts_by_path() {
        let db = sled::Config::new()
            .path(temp_db_path())
            .temporary(true)
            .open()
            .unwrap();

        let paths = vec![
            PathBuf::from("b/Artist/02.mp3"),
            PathBuf::from("a/Artist/01.mp3"),
            PathBuf::from("a/Artist/02.mp3"),
        ];
        for path in &paths {
            db.insert(generate_track_id(path).as_bytes(), serialize(path.clone()))
                .unwrap();
        }

        let result = db_to_files(db);

        let mut expected = paths;
        expected.sort();
        assert_eq!(expected, result);
    }

    fn temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!("pickup-cache-test-{}", std::process::id()))
    }
}
