use std::sync::{mpsc::Sender, Arc};

use crate::{filemanager::collection::Collection, player::Command};

#[derive(Clone)]
pub struct AppState {
    pub sender: Sender<Box<dyn Command>>,
    pub collection: Arc<Collection>,
}
