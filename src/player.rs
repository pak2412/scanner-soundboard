use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStreamHandle, Sink};

use crate::config::Config;

pub struct Player {
    sink: Sink,
    config: Config,
}

impl Player {
    pub fn new(config: Config, stream_handle: OutputStreamHandle) -> Self {
        let sink = Sink::try_new(&stream_handle).unwrap();
        Self { sink, config }
    }

    pub fn play_by_id(&self, id: &str) -> anyhow::Result<()> {
        let sound_source = self.create_sound_decoder(id)?;
        self.sink.stop();
        self.sink.append(sound_source);
        Ok(())
    }

    pub fn set_volume(&self, volume: u8) {
        let volume = volume as f32 / 100.0;
        self.sink.set_volume(volume);
    }

    fn create_sound_decoder(&self, id: &str) -> anyhow::Result<Decoder<BufReader<File>>> {
        if let Some(filename) = self.config.inputs_to_filenames.get(id.trim()) {
            let path = self.config.sounds_path.join(filename);
            if !&path.exists() {
                return Err(anyhow::anyhow!(
                    "Sound file {} does not exist.",
                    path.display()
                ));
            }
            let file = BufReader::new(File::open(path)?);
            let source = Decoder::new(file)?;
            Ok(source)
        } else {
            Err(anyhow::anyhow!(
                "ID {} not mapped to a sound file in config.toml.",
                id
            ))
        }
    }
}
