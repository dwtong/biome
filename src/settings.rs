use std::{collections::HashSet, hash::Hash, ops::Not};

use config::Config;

use crate::MAX_CHANNEL_COUNT;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Settings {
    midi_channel: u8,
    midi_device: String,
    channels: Vec<ChannelSettings>,
    midi: Vec<MidiSettings>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ChannelSettings {
    sample_dir: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct MidiSettings {
    param: ControlParam,
    cc_id: u8,
    channel: u8,
    initial_value: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to parse settings file")]
    ConfigFile(#[from] config::ConfigError),
    #[error("invalid settings value {0}")]
    InvalidSettings(String),
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlParam {
    FilterFrequency,
    FilterQ,
    Rate,
    Volume,
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
            .map(|channel| channel.sample_dir.as_str())
            .collect()
    }

    pub fn midi_channel(&self) -> midi_control::Channel {
        let channel_index = self.midi_channel - 1;
        channel_index.into()
    }

    pub fn midi_device(&self) -> &str {
        &self.midi_device
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn midi_settings(&self) -> Vec<&MidiSettings> {
        self.midi.iter().collect()
    }

    pub fn midi_initial_values(&self) -> Vec<(u8, u8)> {
        self.midi_settings()
            .iter()
            .map(|param| (param.cc_id, param.initial_value))
            .collect()
    }

    pub fn channel_and_param_from_midi_event(&self, cc_id: u8) -> Option<(usize, &ControlParam)> {
        let setting = self.midi.iter().find(|setting| setting.cc_id == cc_id)?;
        Some((setting.channel.into(), &setting.param))
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

        if self.channel_count() > MAX_CHANNEL_COUNT {
            return Err(Error::InvalidSettings("too many channels".into()));
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
