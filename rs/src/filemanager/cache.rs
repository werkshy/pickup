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
    walk_dirs(Path::new(&options.dir), &mut db, ignore_set);
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

/**
 * Encode a path for the cache.
 *
 * Returns None if the path isn't valid UTF-8: the on-disk schema stores paths as
 * a String, so such a file simply cannot be cached and the caller skips it.
 */
fn serialize(path: &Path) -> Option<Vec<u8>> {
    let file = File {
        path: path.to_str()?.to_string(),
    };
    //bincode::serialize(&file).unwrap()
    Some(rkyv::to_bytes::<Error>(&file).unwrap().into_vec())
}

/**
 * Walk the music dir, writing every file we can represent into the cache.
 *
 * Files we can't handle are skipped with a warning rather than unwrapped (e.g.
 * a filename that isn't valid UTF-8.)
 */
fn walk_dirs(dir: &Path, db: &mut sled::Db, ignores: HashSet<String>) {
    log::info!("Walking {}", dir.display());
    let mut i: u64 = 0;
    let mut skipped: u64 = 0;
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
    {
        let full_path = entry.into_path();
        let relative = match full_path.strip_prefix(dir) {
            Ok(relative) => relative,
            Err(error) => {
                log::warn!("Skipping {:?}: {error}", full_path);
                skipped += 1;
                continue;
            }
        };
        let root = match get_root(relative) {
            Some(root) => root,
            None => {
                log::warn!("Skipping {:?}: no usable root directory", full_path);
                skipped += 1;
                continue;
            }
        };
        if ignores.contains(&root) {
            continue;
        }

        // Key by the same ID the collection builder will derive from this path,
        // so the cache is keyed by track identity and dedupes for free.
        let id = generate_track_id(relative);
        let encoded = match serialize(relative) {
            Some(encoded) => encoded,
            None => {
                log::warn!("Skipping {:?}: path is not valid UTF-8", full_path);
                skipped += 1;
                continue;
            }
        };
        db.insert(id.as_bytes(), encoded).unwrap();
        i += 1;
        if i.is_multiple_of(500) {
            log::info!("{}", i);
        }
    }
    log::info!("Found {} entries", i);
    if skipped > 0 {
        log::warn!("Skipped {} file(s)", skipped);
    }
}

/**
 * The name of the top-level directory a path sits under, which is what the
 * ignore list is matched against.
 *
 * None if the path has no components, or if its first component isn't valid
 * UTF-8 and so can't be compared against the ignore list.
 */
fn get_root(path: &Path) -> Option<String> {
    path.components()
        .next() // First Component
        .and_then(|component| component.as_os_str().to_str())
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    #[cfg(unix)]
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    use super::{
        db_path_for, db_to_files, generate_track_id, get_db_path, get_root, serialize, walk_dirs,
        DB_ROOT_PATH,
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
        let db = open_temp_db("sorts-by-path");

        let paths = vec![
            PathBuf::from("b/Artist/02.mp3"),
            PathBuf::from("a/Artist/01.mp3"),
            PathBuf::from("a/Artist/02.mp3"),
        ];
        for path in &paths {
            db.insert(
                generate_track_id(path).as_bytes(),
                serialize(path).expect("these test paths are valid UTF-8"),
            )
            .unwrap();
        }

        let result = db_to_files(db);

        let mut expected = paths;
        expected.sort();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_get_root_returns_first_component() {
        assert_eq!(
            Some(String::from("Artist")),
            get_root(Path::new("Artist/Album/01.mp3"))
        );
    }

    #[test]
    fn test_get_root_is_none_for_an_empty_path() {
        // A Path is not guaranteed to have any components, so this must not panic.
        assert_eq!(None, get_root(Path::new("")));
    }

    /// A path we can't represent in the cache is reported as such, not unwrapped.
    #[cfg(unix)]
    #[test]
    fn test_serialize_is_none_for_a_non_utf8_path() {
        let path = PathBuf::from("Artist/Album").join(non_utf8_name());

        assert!(serialize(&path).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn test_walk_dirs_skips_non_utf8_file_names() {
        let dir = temp_music_dir("skips-file-names");
        let album = dir.join("Artist/Album");
        std::fs::create_dir_all(&album).unwrap();
        std::fs::write(album.join("01 Track.mp3"), "").unwrap();
        std::fs::write(album.join(non_utf8_name()), "").unwrap();

        let mut db = open_temp_db("skips-file-names");
        walk_dirs(&dir, &mut db, HashSet::new());

        // The bad file is skipped; the good one is still cached.
        assert_eq!(
            vec![PathBuf::from("Artist/Album/01 Track.mp3")],
            db_to_files(db)
        );
    }

    /// Same, but the bad name is a directory: the files under it are skipped too
    /// rather than cached under a mangled (U+FFFD) path that no longer resolves.
    #[cfg(unix)]
    #[test]
    fn test_walk_dirs_skips_non_utf8_dir_names() {
        let dir = temp_music_dir("skips-dir-names");
        let album = dir.join("Artist").join(non_utf8_name());
        std::fs::create_dir_all(&album).unwrap();
        std::fs::write(album.join("01 Track.mp3"), "").unwrap();

        let mut db = open_temp_db("skips-dir-names");
        walk_dirs(&dir, &mut db, HashSet::new());

        assert!(db_to_files(db).is_empty());
    }

    /// A file name that is not valid UTF-8: the 0xFF byte is never valid UTF-8.
    #[cfg(unix)]
    fn non_utf8_name() -> OsString {
        OsString::from_vec(vec![b'b', b'a', b'd', 0xFF, b'.', b'm', b'p', b'3'])
    }

    /// A temporary DB per test, so the tests can run in parallel (sled locks
    /// its directory, so they each need their own).
    fn open_temp_db(label: &str) -> sled::Db {
        sled::Config::new()
            .path(temp_path(&format!("db-{label}")))
            .temporary(true)
            .open()
            .unwrap()
    }

    /// A temporary music dir per test, emptied first so a leftover from an
    /// earlier run can't change what the walk finds.
    fn temp_music_dir(label: &str) -> PathBuf {
        let dir = temp_path(&format!("music-{label}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn temp_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("pickup-cache-test-{}-{label}", std::process::id()))
    }
}
