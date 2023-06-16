use std::path::PathBuf;
use std::{fs::File, io};
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{
    AudioBufferSourceNode, AudioNode, AudioScheduledSourceNode, BiquadFilterNode, GainNode,
};

use crate::midi::Midi;
use crate::settings::Settings;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to open file")]
    OpenFile(#[from] io::Error),
    #[error("failed to decode audio")]
    DecodeAudio(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct AudioGraphChannel {
    filter: BiquadFilterNode,
    volume: GainNode,
    source: AudioBufferSourceNode,
}

impl AudioGraphChannel {
    fn new(context: &AudioContext, destination: &GainNode) -> Self {
        let volume = context.create_gain();
        volume.gain().set_value(0.0);
        volume.connect(destination);

        let filter = context.create_biquad_filter();
        filter.set_type(web_audio_api::node::BiquadFilterType::Bandpass);
        filter.frequency().set_value(1800.0);
        filter.q().set_value(0.667);
        filter.connect(&volume);

        let source = context.create_buffer_source();
        source.set_loop(true);
        source.connect(&filter);

        Self {
            filter,
            source,
            volume,
        }
    }

    pub fn set_filter_q(&self, value: f32) {
        self.filter.q().set_value(value);
    }

    pub fn set_filter_frequency(&self, value: f32) {
        self.filter.frequency().set_value(value);
    }

    pub fn set_volume(&self, value: f32) {
        self.volume.gain().set_value(value);
    }

    pub fn set_rate(&self, value: f32) {
        // self.source.playback_rate().set_value(value);
        todo!()
    }

    pub fn load(&mut self, context: &AudioContext, path: &PathBuf) -> Result<(), Error> {
        let file = File::open(path)?;
        let buffer = context.decode_audio_data_sync(file)?;
        let source = context.create_buffer_source();
        source.set_loop(true);
        source.connect(&self.filter);
        source.set_buffer(buffer);

        // out with the old and in with the new
        self.source.disconnect();
        self.source = source;

        Ok(())
    }

    pub fn play(&self) {
        self.source.start();
    }
}

pub struct AudioGraph {
    channels: Vec<AudioGraphChannel>,
    volume: GainNode,
    context: AudioContext,
}

impl AudioGraph {
    pub fn new(settings: &Settings) -> Self {
        let context = AudioContext::default();
        let volume = context.create_gain();
        volume.connect(&context.destination());

        let channels: Vec<AudioGraphChannel> = (0..settings.channel_count())
            .map(|_| AudioGraphChannel::new(&context, &volume))
            .collect();

        Self {
            context,
            channels,
            volume,
        }
    }

    pub fn mute_all(&self) {
        let now = self.context.current_time();
        self.volume
            .gain()
            .linear_ramp_to_value_at_time(0., now + 0.2);
    }

    pub fn get_channel(&self, channel_index: usize) -> Option<&AudioGraphChannel> {
        self.channels.get(channel_index)
    }

    pub fn load_and_play_for_channel(&mut self, channel_index: usize, file_path: &PathBuf) {
        let channel = self
            .channels
            .get_mut(channel_index)
            .expect("Channel index in range");
        channel
            .load(&self.context, file_path)
            .expect("Sample file loaded into audio channel");
        channel.play();
    }
}
