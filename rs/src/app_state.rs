use std::sync::{Arc, RwLock};

use crate::{filemanager::collection::Collection, player::PlayerClient, queue::PlaybackQueue};

// #[derive(Clone)]
pub struct AppState {
    pub player: PlayerClient,
    pub collection: Arc<Collection>,
    pub queue: RwLock<PlaybackQueue>,
}
