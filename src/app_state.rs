use std::sync::{Arc, RwLock};

use crate::{
    filemanager::collection::Collection, player::PlayerClient, queue::PlaybackQueue,
    session::SessionStore,
};

// #[derive(Clone)]
pub struct AppState {
    pub player: PlayerClient,
    pub collection: Arc<Collection>,
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
}
