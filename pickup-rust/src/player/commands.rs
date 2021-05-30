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
    pub volume: f32,
}

impl Command for VolumeCommand {
    fn action(&mut self, player: &mut Player) {
        player.set_volume(self.volume);
    }
}
