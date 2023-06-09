use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::Not,
};

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
    #[error("invalid settings value {0}")]
    InvalidSettings(String),
}

impl Settings {
    pub fn new() -> Result<Self, Error> {
        let settings = Config::builder()
            .add_source(config::File::with_name("settings"))
            .build()?;

        settings
            .try_deserialize::<Settings>()
            .map_err(Error::ConfigFile)?
            .validate()
    }

    pub fn sample_dirs(&self) -> Vec<&str> {
        self.channels
            .iter()
            .map(|channel| channel.samples.dir.as_str())
            .collect()
    }

    pub fn midi_settings(&self) -> Vec<&MidiSettings> {
        self.channels
            .iter()
            .flat_map(|channel| channel.midi.iter())
            .collect()
    }

    pub fn midi_initial_values(&self) -> Vec<(u8, u8)> {
        self.midi_settings()
            .iter()
            .map(|param| (param.cc_id, param.initial_value))
            .collect()
    }

    pub fn validate(self) -> Result<Self, Error> {
        let cc_ids = &self
            .midi_settings()
            .iter()
            .map(|param| param.cc_id)
            .collect::<Vec<u8>>();

        if has_dups(cc_ids) {
            return Err(Error::InvalidSettings("duplicate cc_ids".into()));
        }

        Ok(self)
    }
}

fn has_dups<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(|x| uniq.insert(x)).not()
}
