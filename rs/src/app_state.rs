use std::sync::{Arc, RwLock};

use crate::{filemanager::collection::Collection, player::PlayerClient, queue::PlaybackQueue};

// #[derive(Clone)]
pub struct AppState {
    pub player: PlayerClient,
    pub collection: Arc<Collection>,
    /**
     * The playback queue is shared with the Player thread, using an
     * RwLock to synchronize access.
     */
    pub queue: Arc<RwLock<PlaybackQueue>>,
}
