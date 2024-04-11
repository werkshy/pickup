use std::sync::{mpsc::Sender, Arc, RwLock};

use crate::{filemanager::collection::Collection, player::Command, queue::PlaybackQueue};

// #[derive(Clone)]
pub struct AppState {
    pub player_sender: Sender<Box<dyn Command>>,
    pub collection: Arc<Collection>,
    pub queue: RwLock<PlaybackQueue>,
}
