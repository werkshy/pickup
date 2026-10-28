use rkyv::from_bytes;
use rkyv::util::AlignedVec;
use rkyv::{rancor::Error, Archive, Deserialize, Serialize};
use std::collections::{hash_map::DefaultHasher, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::options::CollectionOptions;

static CACHE_ROOT_PATH: &str = ".cache";
const DEFAULT_IGNORES: [&str; 2] = ["Music", "Audio Music Apps"];

/**
 * File extensions we treat as music. Everything else (cover art, notes, cue
 * sheets, playlists) is left out of the collection.
 */
const MUSIC_EXTENSIONS: [&str; 11] = [
    "mp3", "flac", "ogg", "oga", "opus", "wav", "m4a", "aac", "wma", "aiff", "aif",
];

/**
 * Bumped whenever the on-disk schema changes. It is mixed into the cache path
 * hash so that a stale cache simply misses in `init` and gets transparently
 * refreshed, rather than being decoded with the wrong schema.
 */
const CACHE_VERSION: &str = "v4";

/**
 * The whole cache in one value: every track path, sorted and deduplicated by
 * `refresh`. Stored as a single rkyv blob so a load is one file read and one
 * decode, rather than iterating a key-value store entry by entry.
 */
#[derive(Serialize, Deserialize, Debug, Archive)]
struct Snapshot {
    paths: Vec<String>,
}

pub fn init(options: CollectionOptions) -> std::io::Result<Vec<PathBuf>> {
    let cache_path = get_cache_path(&options.dir);
    if cache_path.exists() {
        match load(&cache_path) {
            Ok(paths) => return Ok(paths),
            Err(error) => {
                log::warn!(
                    "Failed to load cache at {:?}: {error}: refreshing",
                    cache_path
                )
            }
        }
    } else {
        log::info!("Cache does not exist '{:?}': refreshing", cache_path);
    }

    refresh(options)
}

// TODO shouldn't this be async?!
// Assumes that the music dir exists
pub fn refresh(options: CollectionOptions) -> std::io::Result<Vec<PathBuf>> {
    log::info!("Refreshing music dir at {}", options.dir);

    let ignores: Vec<String> = options
        .ignores
        .unwrap_or_else(|| DEFAULT_IGNORES.iter().map(|i| i.to_string()).collect());
    let ignore_set: HashSet<String> = HashSet::from_iter(ignores);

    let mut paths = walk_dirs(Path::new(&options.dir), ignore_set);

    // Sorted paths make the order deterministic and give alphabetical track
    // ordering within an album. Dedup guards against a repeated path, which a
    // single walk cannot produce but which the key-value cache used to dedupe
    // for free.
    paths.sort();
    paths.dedup();

    write_snapshot(&get_cache_path(&options.dir), &paths)?;

    // The walked paths are the snapshot, so there is nothing to read back.
    Ok(paths.into_iter().map(PathBuf::from).collect())
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
 * and using CACHE_ROOT_PATH as the root
 */
fn get_cache_path(music_dir: &str) -> PathBuf {
    cache_path_for(music_dir, CACHE_VERSION)
}

/**
 * Build the path for the cache file, hashing the music dir and a version.
 */
fn cache_path_for(music_dir: &str, version: &str) -> PathBuf {
    Path::new(CACHE_ROOT_PATH).join(hash(&format!("{version}:{music_dir}")))
}

/**
 * Decode the snapshot at `cache_path`.
 *
 * rkyv can't read a value from an unaligned buffer, and the bytes from a file
 * read have no alignment guarantee, so they are copied into an aligned buffer
 * before decoding.
 */
fn load(cache_path: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    log::info!("Loading music dir cache at {:?}", cache_path);
    let bytes = fs::read(cache_path)?;

    let mut buffer = AlignedVec::<16>::new();
    buffer.extend_from_slice(&bytes);

    let snapshot: Snapshot = from_bytes::<Snapshot, Error>(&buffer[..])
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;

    // The snapshot is written sorted, so it comes back sorted.
    Ok(snapshot.paths.into_iter().map(PathBuf::from).collect())
}

/**
 * Write the snapshot to a temp file and rename it into place, so a crash
 * mid-write can never leave a truncated snapshot at the cache path.
 */
fn write_snapshot(cache_path: &Path, paths: &[String]) -> std::io::Result<()> {
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let snapshot = Snapshot {
        paths: paths.to_vec(),
    };
    let bytes = rkyv::to_bytes::<Error>(&snapshot).unwrap().into_vec();

    // A pid suffix keeps concurrent refreshes from writing through each other's
    // temp file; whichever rename lands last leaves a valid snapshot behind.
    let mut temp_path = cache_path.as_os_str().to_os_string();
    temp_path.push(format!(".tmp.{}", std::process::id()));
    fs::write(&temp_path, bytes)?;
    fs::rename(&temp_path, cache_path)?;

    Ok(())
}

/**
 * Walk the music dir, collecting the relative path of every file we can
 * represent.
 *
 * Files we can't handle are skipped with a warning rather than unwrapped (e.g.
 * a filename that isn't valid UTF-8.)
 */
fn walk_dirs(dir: &Path, ignores: HashSet<String>) -> Vec<String> {
    log::info!("Walking {}", dir.display());
    let mut i: u64 = 0;
    let mut skipped: u64 = 0;
    let mut paths: Vec<String> = Vec::new();
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| is_music_file(e.path()))
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

        match relative.to_str() {
            Some(path) => {
                paths.push(path.to_string());
                i += 1;
            }
            None => {
                log::warn!("Skipping {:?}: path is not valid UTF-8", full_path);
                skipped += 1;
                continue;
            }
        }
        if i.is_multiple_of(500) {
            log::info!("{}", i);
        }
    }
    log::info!("Found {} entries", i);
    if skipped > 0 {
        log::warn!("Skipped {} file(s)", skipped);
    }
    paths
}

/**
 * Whether a path's extension is one we can play.
 */
fn is_music_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .is_some_and(|extension| MUSIC_EXTENSIONS.contains(&extension.as_str()))
}

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
        cache_path_for, get_cache_path, get_root, is_music_file, load, walk_dirs, write_snapshot,
        CACHE_ROOT_PATH,
    };

    #[test]
    fn test_cache_path_is_rooted_at_cache_dir() {
        let path = get_cache_path("../music");

        assert_eq!(CACHE_ROOT_PATH, path.parent().unwrap().to_str().unwrap());
    }

    #[test]
    fn test_cache_path_is_distinct_per_music_dir() {
        assert_ne!(get_cache_path("../music"), get_cache_path("../other-music"));
    }

    #[test]
    fn test_cache_path_differs_across_cache_versions() {
        // A stale cache from a previous schema must not resolve to the current
        // path, so `init` misses it and refreshes rather than decoding bytes
        // written by another schema.
        assert_ne!(
            cache_path_for("../music", "v3"),
            cache_path_for("../music", "v4")
        );
    }

    #[test]
    fn test_write_and_load_round_trips_paths() {
        let paths = vec![
            String::from("a/Artist/01.mp3"),
            String::from("a/Artist/02.mp3"),
            String::from("b/Artist/02.mp3"),
        ];

        let cache_path = temp_path("round-trip");
        write_snapshot(&cache_path, &paths).unwrap();

        let loaded = load(&cache_path).unwrap();

        assert_eq!(
            paths.iter().map(PathBuf::from).collect::<Vec<PathBuf>>(),
            loaded
        );
        let _ = std::fs::remove_file(&cache_path);
    }

    /**
     * A cache that isn't a valid snapshot (a truncated write, say) is an
     * error, which `init` answers with a refresh.
     */
    #[test]
    fn test_load_rejects_a_corrupt_cache() {
        let cache_path = temp_path("corrupt");
        std::fs::write(&cache_path, b"not a snapshot").unwrap();

        assert!(load(&cache_path).is_err());

        let _ = std::fs::remove_file(&cache_path);
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

    #[test]
    fn test_is_music_file() {
        for name in ["01 Track.mp3", "02 Track.FLAC", "03.ogg", "04.m4a"] {
            assert!(is_music_file(Path::new(name)), "{name} should be music");
        }
        for name in ["cover.jpg", "notes.txt", "playlist.m3u", "no-extension"] {
            assert!(
                !is_music_file(Path::new(name)),
                "{name} should not be music"
            );
        }
    }

    #[test]
    fn test_walk_dirs_skips_non_music_files() {
        let dir = temp_music_dir("skips-non-music");
        let album = dir.join("Artist/Album");
        std::fs::create_dir_all(&album).unwrap();
        std::fs::write(album.join("01 Track.mp3"), "").unwrap();
        std::fs::write(album.join("02 Track.FLAC"), "").unwrap();
        std::fs::write(album.join("cover.jpg"), "").unwrap();
        std::fs::write(album.join("notes.txt"), "").unwrap();

        let mut found = walk_dirs(&dir, HashSet::new());
        found.sort();

        assert_eq!(
            vec![
                String::from("Artist/Album/01 Track.mp3"),
                String::from("Artist/Album/02 Track.FLAC"),
            ],
            found
        );
    }

    /// A path we can't represent in the cache is reported as such, not unwrapped.
    #[cfg(unix)]
    #[test]
    fn test_walk_dirs_skips_non_utf8_file_names() {
        let dir = temp_music_dir("skips-file-names");
        let album = dir.join("Artist/Album");
        std::fs::create_dir_all(&album).unwrap();
        std::fs::write(album.join("01 Track.mp3"), "").unwrap();
        std::fs::write(album.join(non_utf8_name()), "").unwrap();

        let found = walk_dirs(&dir, HashSet::new());

        // The bad file is skipped; the good one is still cached.
        assert_eq!(vec![String::from("Artist/Album/01 Track.mp3")], found);
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

        assert!(walk_dirs(&dir, HashSet::new()).is_empty());
    }

    /// A file name that is not valid UTF-8: the 0xFF byte is never valid UTF-8.
    #[cfg(unix)]
    fn non_utf8_name() -> OsString {
        OsString::from_vec(vec![b'b', b'a', b'd', 0xFF, b'.', b'm', b'p', b'3'])
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
