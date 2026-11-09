use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    app_state::CollectionStore,
    filemanager::{collection, options::CollectionOptions},
};

/**
 * Hour (UTC) the nightly refresh runs at, unless overridden with the
 * `REFRESH_HOUR_UTC` environment variable. UTC keeps the schedule
 * predictable without pulling in a timezone database.
 */
pub const DEFAULT_REFRESH_HOUR_UTC: u64 = 6;
const REFRESH_HOUR_ENV: &str = "REFRESH_HOUR_UTC";
const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/**
 * Rebuilds the collection in the background and swaps it into the app state
 * once it is complete. A refresh is single-flight: asking for another one
 * while one is running is a no-op, so the nightly job and a browser click
 * can't race each other.
 */
#[derive(Clone)]
pub struct RefreshService {
    inner: Arc<Inner>,
}

struct Inner {
    options: CollectionOptions,
    collection: CollectionStore,
    running: AtomicBool,
    last: Mutex<LastRefresh>,
}

#[derive(Default)]
struct LastRefresh {
    completed: Option<SystemTime>,
    error: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct RefreshStatus {
    pub running: bool,
    pub last_completed: Option<SystemTime>,
    pub last_error: Option<String>,
}

impl RefreshService {
    pub fn new(options: CollectionOptions, collection: CollectionStore) -> Self {
        Self {
            inner: Arc::new(Inner {
                options,
                collection,
                running: AtomicBool::new(false),
                last: Mutex::new(LastRefresh::default()),
            }),
        }
    }

    /**
     * Start a refresh in the background. Returns false if one is already
     * running, in which case nothing new is started.
     */
    pub fn start(&self) -> bool {
        if self.inner.running.swap(true, Ordering::SeqCst) {
            return false;
        }

        let service = self.clone();
        thread::spawn(move || service.run());
        true
    }

    pub fn status(&self) -> RefreshStatus {
        let last = self.inner.last.lock().unwrap();
        RefreshStatus {
            running: self.inner.running.load(Ordering::SeqCst),
            last_completed: last.completed,
            last_error: last.error.clone(),
        }
    }

    /**
     * Run a refresh every night at the configured UTC hour. Intended to be
     * called once, when the server starts.
     */
    pub fn spawn_nightly(&self) {
        let hour = nightly_hour(std::env::var(REFRESH_HOUR_ENV).ok().as_deref());
        let service = self.clone();
        thread::spawn(move || loop {
            let wait = seconds_until_hour_utc(SystemTime::now(), hour);
            log::info!(
                "Next automatic collection refresh in {} hour(s)",
                wait / 3600
            );
            thread::sleep(Duration::from_secs(wait));
            service.start();
        });
    }

    fn run(&self) {
        // Reset the running flag even if building the collection panics, so
        // one bad refresh does not disable every future one.
        let _running = RunningGuard {
            running: &self.inner.running,
        };
        log::info!("Refreshing the collection in the background");
        match collection::refresh(self.inner.options.clone()) {
            Ok(new_collection) => {
                let track_count = new_collection.all_tracks().len();
                self.inner.collection.replace(new_collection);
                let mut last = self.inner.last.lock().unwrap();
                last.completed = Some(SystemTime::now());
                last.error = None;
                log::info!("Collection refreshed: {track_count} tracks");
            }
            Err(error) => {
                log::error!("Collection refresh failed: {error}");
                self.inner.last.lock().unwrap().error = Some(error.to_string());
            }
        }
    }
}

struct RunningGuard<'a> {
    running: &'a AtomicBool,
}

impl Drop for RunningGuard<'_> {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

/**
 * The configured nightly hour, falling back to the default when the
 * environment variable is missing or not an hour of the day.
 */
fn nightly_hour(value: Option<&str>) -> u64 {
    match value.and_then(|value| value.parse::<u64>().ok()) {
        Some(hour) if hour <= 24 => hour,
        Some(hour) => {
            log::warn!("Ignoring {REFRESH_HOUR_ENV}={hour}: expected an hour between 0 and 24");
            DEFAULT_REFRESH_HOUR_UTC
        }
        None => DEFAULT_REFRESH_HOUR_UTC,
    }
}

/**
 * Seconds from `now` until the next occurrence of `hour` UTC. Used instead
 * of a fixed 24-hour timer so the refresh happens at the same time of day
 * regardless of when the server started.
 */
fn seconds_until_hour_utc(now: SystemTime, hour: u64) -> u64 {
    let now = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let seconds_today = now % SECONDS_PER_DAY;
    let target = hour * 3600;
    if seconds_today < target {
        target - seconds_today
    } else {
        SECONDS_PER_DAY - seconds_today + target
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::filemanager::collection::Collection;

    fn at(hour: u64, minute: u64, second: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(hour * 3600 + minute * 60 + second)
    }

    #[test]
    fn test_seconds_until_hour_utc_later_today() {
        assert_eq!(seconds_until_hour_utc(at(1, 0, 0), 3), 2 * 3600);
    }

    #[test]
    fn test_seconds_until_hour_utc_earlier_today_wraps_to_tomorrow() {
        assert_eq!(
            seconds_until_hour_utc(at(4, 0, 0), 3),
            SECONDS_PER_DAY - 3600
        );
    }

    #[test]
    fn test_seconds_until_hour_utc_is_never_zero() {
        assert_eq!(seconds_until_hour_utc(at(3, 0, 0), 3), SECONDS_PER_DAY);
    }

    #[test]
    fn test_nightly_hour_uses_default_when_unset() {
        assert_eq!(nightly_hour(None), DEFAULT_REFRESH_HOUR_UTC);
    }

    #[test]
    fn test_nightly_hour_uses_a_valid_override() {
        assert_eq!(nightly_hour(Some("22")), 22);
        assert_eq!(nightly_hour(Some("24")), 24);
    }

    #[test]
    fn test_nightly_hour_ignores_an_invalid_override() {
        assert_eq!(nightly_hour(Some("99")), DEFAULT_REFRESH_HOUR_UTC);
        assert_eq!(nightly_hour(Some("soon")), DEFAULT_REFRESH_HOUR_UTC);
    }

    #[test]
    fn test_refresh_swaps_in_the_new_collection() {
        let dir = temp_music_dir("swap");
        let album = dir.join("Artist/Album");
        std::fs::create_dir_all(&album).unwrap();
        std::fs::write(album.join("01 Track.mp3"), "").unwrap();

        let options = CollectionOptions {
            dir: dir.to_string_lossy().to_string(),
            ignores: None,
        };
        let store = CollectionStore::new(Collection::new(options.dir.clone()));
        let service = RefreshService::new(options, store.clone());

        assert!(service.start());
        assert!(!service.start(), "a refresh is already running");

        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while service.status().running {
            assert!(
                std::time::Instant::now() < deadline,
                "refresh did not finish in time"
            );
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(store.get().all_tracks().len(), 1);
        assert_eq!(service.status().last_error, None);
        assert!(service.status().last_completed.is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    fn temp_music_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pickup-refresh-test-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
