use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use rodio;
use rodio::{Decoder, MixerDeviceSink};

pub mod client;
pub mod commands;

pub use client::{PlayerClient, PlayerError};

pub trait Command: Send {
    fn action(&mut self, player: &mut Player);
}

// This is the object that handles playing music
pub struct Player {
    player: rodio::Player,
    sink: MixerDeviceSink,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Player {
        log::info!("Creating stream and sink");
        // We can't drop `player` or nothing will play, but it doesn't implement Send and can't be
        // shared across threads.
        let sink =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        let player = rodio::Player::connect_new(sink.mixer());
        Player { player, sink }
    }

    pub fn command(&mut self, mut command: Box<dyn Command>) {
        (*command).action(self)
    }

    /**
     * Plays a file. Errors (missing file, undecodable file) are returned to
     * the caller (and logged); this must not panic the Player thread.
     */
    pub fn play(&mut self, path: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        log::info!("Playing {}", path);

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                log::error!("Failed to open {}: {}", path, error);
                return Err(format!("could not open {}: {}", path, error).into());
            }
        };

        // Decode that sound file into a source
        let source = match Decoder::new(BufReader::new(file)) {
            Ok(source) => source,
            Err(error) => {
                log::error!("Failed to decode {}: {}", path, error);
                return Err(format!("could not decode {}: {}", path, error).into());
            }
        };
        self.player.append(source);

        // TODO handle how to trigger the next song in the playlist when the current song is finished.
        Ok(())
    }

    pub fn status(&self) -> usize {
        let len = self.player.len();
        log::info!(
            "Status: {} tracks in the sink queue. paused={}",
            len,
            self.player.is_paused()
        );
        len
    }

    pub fn stop(&mut self) {
        log::info!("Stopping playback");
        self.player.stop();
    }

    pub fn set_volume(&mut self, value: u8) {
        let float_volume = scale_volume_down(value);
        log::info!("Setting volume to {}", float_volume);
        self.player.set_volume(float_volume);
    }

    pub fn get_volume(&mut self) -> u8 {
        let float_volume = self.player.volume();
        log::info!("Current volume is {}", float_volume);
        scale_volume_up(float_volume)
    }

    // This getter really only exists to silence the unused warning about sink.
    pub fn sink(&self) -> &MixerDeviceSink {
        &self.sink
    }
}

// scale down volume from 0-100 to 0-1.0
fn scale_volume_down(volume_percentage: u8) -> f32 {
    let clamped_volume = volume_percentage.clamp(0, 100);
    clamped_volume as f32 / 100.0
}

fn scale_volume_up(vol_fraction: f32) -> u8 {
    let percent_volume = (vol_fraction * 100.0).round();
    let clamped_volume = percent_volume.clamp(0.0, 100.0);
    clamped_volume as u8
}
