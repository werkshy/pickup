use std::sync::{Arc, RwLock};

use crate::{
    filemanager::{collection::Collection, options::CollectionOptions},
    player::PlayerClient,
    queue::PlaybackQueue,
    refresh::RefreshService,
    session::SessionStore,
};

/**
 * The collection the app is currently serving, behind a lock so a background
 * refresh can build a new one and swap it in without stopping requests.
 * Readers take a cheap Arc snapshot of the current collection.
 */
#[derive(Clone)]
pub struct CollectionStore {
    current: Arc<RwLock<Arc<Collection>>>,
}

impl CollectionStore {
    pub fn new(collection: Collection) -> Self {
        Self {
            current: Arc::new(RwLock::new(Arc::new(collection))),
        }
    }

    /// The collection to serve for the rest of this request.
    pub fn get(&self) -> Arc<Collection> {
        self.current.read().unwrap().clone()
    }

    /// Replace the collection. Requests that already took a snapshot finish
    /// against the old one; later requests see the new one.
    pub fn replace(&self, collection: Collection) {
        *self.current.write().unwrap() = Arc::new(collection);
    }
}

// #[derive(Clone)]
pub struct AppState {
    pub player: PlayerClient,
    pub collection: CollectionStore,
    /**
     * The playback queue is shared with the Player thread, using an
     * RwLock to synchronize access.
     */
    pub queue: Arc<RwLock<PlaybackQueue>>,
    /**
     * Per-browser session state, keyed by the session cookie. Browser
     * playback queues live here; the jukebox queue above stays global.
     */
    pub sessions: SessionStore,
    /**
     * Rebuilds the collection in the background and swaps it into
     * `collection`, on request or on the nightly schedule.
     */
    pub refresh: RefreshService,
}

impl AppState {
    pub fn new(
        player: PlayerClient,
        collection: Collection,
        queue: Arc<RwLock<PlaybackQueue>>,
        sessions: SessionStore,
        collection_options: CollectionOptions,
    ) -> Self {
        let collection = CollectionStore::new(collection);
        let refresh = RefreshService::new(collection_options, collection.clone());
        Self {
            player,
            collection,
            queue,
            sessions,
            refresh,
        }
    }
}
