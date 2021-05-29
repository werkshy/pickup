use std::sync::mpsc::Sender;

use crate::player::Command;

pub struct AppState {
    pub sender: Sender<Box<dyn Command>>,
}
