use std::sync::mpsc::{channel, Sender};

use actix_web::web;

use crate::player::commands::{PlayCommand, QueryCommand, StopCommand, VolumeCommand};
use crate::player::{Command, Player};

/**
 * A clonable handle to the Player thread. Fire-and-forget actions are sent
 * directly; queries run a closure on the Player thread and wait for the
 * result.
 */
#[derive(Clone)]
pub struct PlayerClient {
    player_sender: Sender<Box<dyn Command>>,
}

impl PlayerClient {
    pub fn new(player_sender: Sender<Box<dyn Command>>) -> Self {
        Self { player_sender }
    }

    fn send(&self, command: impl Command + 'static) {
        let _ = self.player_sender.send(Box::new(command));
    }

    pub fn play(&self, file: String) {
        self.send(PlayCommand { file });
    }

    pub fn stop(&self) {
        self.send(StopCommand {});
    }

    pub fn set_volume(&self, volume: u8) {
        self.send(VolumeCommand { volume });
    }

    /**
     * Runs `query` on the Player thread and blocks the calling thread until
     * the result arrives. Returns None if the Player thread has gone away.
     * Prefer request_async from async code.
     */
    pub fn request<T: Send + 'static>(
        &self,
        query: impl FnOnce(&mut Player) -> T + Send + 'static,
    ) -> Option<T> {
        let (reply_tx, reply_rx) = channel();
        self.send(QueryCommand::new(query, reply_tx));
        reply_rx.recv().ok()
    }

    /**
     * Helper to run queries (commands that return a response) from the Actix
     * async runtime by using `web::block()` to run sync code on the Actix
     * blocking pool.
     * Must be awaited inside the actix runtime.
     * Usage:
     *     player
     *       .request_async(|player| player.get_volume())
     *       .await
     *       .unwrap_or(0)
     */
    pub async fn request_async<T: Send + 'static>(
        &self,
        query: impl FnOnce(&mut Player) -> T + Send + 'static,
    ) -> Option<T> {
        let client = self.clone();
        web::block(move || client.request(query))
            .await
            .ok()
            .flatten()
    }
}
