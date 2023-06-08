use config::Config;

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    channels: Vec<ChannelSettings>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChannelSettings {
    samples: SampleSettings,
    midi: Vec<MidiSettings>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SampleSettings {
    dir: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct MidiSettings {
    param: String,
    cc_id: u8,
    initial_value: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to parse settings file")]
    ConfigFile(#[from] config::ConfigError),
}

impl Settings {
    pub fn new() -> Result<Self, Error> {
        let settings = Config::builder()
            .add_source(config::File::with_name("settings"))
            .build()?;

        settings
            .try_deserialize::<Settings>()
            .map_err(Error::ConfigFile)
    }

    pub fn sample_dirs(&self) -> Vec<&str> {
        self.channels
            .iter()
            .map(|ch| ch.samples.dir.as_str())
            .collect()
    }

    pub fn midi_initial_values(&self) -> Vec<(u8, u8)> {
        let mut values = Vec::new();
        for channel in &self.channels {
            for param in &channel.midi {
                values.push((param.cc_id, param.initial_value));
            }
        }
        values
    }
}
