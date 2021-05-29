use std::sync::mpsc::Sender;
use std::sync::Mutex;

pub struct AppState {
    pub sender: Mutex<Sender<String>>,
}
