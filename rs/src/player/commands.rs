use std::sync::mpsc::Sender;

use crate::player::{Command, Player};

pub struct PlayCommand {
    pub file: String,
}

impl Command for PlayCommand {
    fn action(&mut self, player: &mut Player) {
        player.play(self.file.clone());
    }
}

pub struct StopCommand {}

impl Command for StopCommand {
    fn action(&mut self, player: &mut Player) {
        player.stop();
    }
}

pub struct VolumeCommand {
    pub volume: u8,
}

impl Command for VolumeCommand {
    fn action(&mut self, player: &mut Player) {
        player.set_volume(self.volume);
    }
}

/**
 * Generic replacement for response-carrying commands. The closure runs on the
 * Player thread and its result is sent back over a short-lived channel.
 */
pub struct QueryCommand<F, T> {
    reply: Sender<T>,
    // Option because a FnOnce closure can't be invoked through &mut self
    query: Option<F>,
}

impl<F, T> QueryCommand<F, T>
where
    F: FnOnce(&mut Player) -> T + Send + 'static,
    T: Send + 'static,
{
    pub fn new(query: F, reply: Sender<T>) -> Self {
        Self {
            reply,
            query: Some(query),
        }
    }
}

impl<F, T> Command for QueryCommand<F, T>
where
    F: FnOnce(&mut Player) -> T + Send + 'static,
    T: Send + 'static,
{
    fn action(&mut self, player: &mut Player) {
        if let Some(query) = self.query.take() {
            let value = query(player);
            let _ = self.reply.send(value);
        }
    }
}
