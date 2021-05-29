use std::fs::File;
use std::io::BufReader;

use rodio::{Decoder, OutputStream, Sink};

// This is the object that handles playing music
pub struct Player {
    stream: OutputStream,
    sink: Sink,
}

pub trait Command: Send {
    fn action(&mut self, player: &mut Player);
}

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

impl Player {
    pub fn new() -> Player {
        log::info!("Creating stream and sink");
        // We can't drop `stream` or nothing will play, but it doesn't implement Send and can't be
        // shared across threads.
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        Player { stream, sink }
    }

    pub fn command(&mut self, mut command: Box<dyn Command>) {
        (*command).action(self)
    }

    pub fn play(&mut self, path: String) {
        log::info!("Playing {}", path);
        self.sink.stop();
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        self.stream = stream;
        self.sink = sink;

        // TODO handle missing file error - don't stop the playing until we have a good file
        let file = BufReader::new(File::open(path).unwrap());
        // Decode that sound file into a source
        // TODO handle error
        let source = Decoder::new(file).unwrap();
        self.sink.append(source);
    }

    pub fn stop(&mut self) {
        log::info!("Stopping playback");
        self.sink.stop();
    }

    pub fn set_volume(&mut self, value: f32) {
        self.sink.set_volume(value);
    }
}
