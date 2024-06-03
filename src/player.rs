use rodio::Decoder;
use rodio::OutputStream;
use rodio::OutputStreamHandle;
use rodio::Sink;
use std::fs::File;
use std::io::BufReader;

use crate::config::Config;

pub struct Player {
    stream_handle: OutputStreamHandle,
    sink: Sink,
    config: Config,
}

impl Player {
    pub fn new(config: Config) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let stream_handle = stream_handle;
        Player {
            config,
            sink,
            stream_handle,
        }
    }

    pub fn play_song_by_id(&self, id: &str) {
        if let Some(filename) = self.config.inputs_to_filenames.get(id.trim()) {
            let path = self.config.sounds_path.as_path().join(filename);
            if !&path.exists() {
                panic!("Sound file {} does not exist.", path.display());
            }
            let file: BufReader<File> = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();
            self.sink.append(source);
        }
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.sink = Sink::try_new(&self.stream_handle).unwrap();
    }
}
