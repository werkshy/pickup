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
 * Current pattern for a command that sends a response: pass a channel Sender
 * over in the Command. This has a fair amount of boilerplate in the API call
 * site.
 */
pub struct GetVolumeCommand {
    pub reply: Sender<u8>,
}

impl Command for GetVolumeCommand {
    fn action(&mut self, player: &mut Player) {
        let volume = player.get_volume();
        let _ = self.reply.send(volume);
    }
}
