use std::fs::File;
use std::io::BufReader;

use rodio::{Decoder, OutputStream, Sink};

// This is the actor that handles playing music
pub struct Player {
    stream: OutputStream,
    sink: Sink,
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

    // TODO change data to be a better type with an enum command and array of args?
    pub fn command(&mut self, data: String) {
        log::info!("Received command '{}'", data);
        let mp3_file = "../../Music/New Order/Substance CD 1/04 - Blue Monday.mp3";
        match data.as_str() {
            "stop" => self.stop(),
            "play" => self.play(mp3_file.to_string()),
            _ => log::error!("Unknown command: {}", data),
        }
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
        log::info!("Done playing");
    }

    pub fn stop(&mut self) {
        log::info!("Stopping playback");
        self.sink.stop();
    }
}
