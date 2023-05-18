use std::{fs::File, io};
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{
    AudioBufferSourceNode, AudioNode, AudioScheduledSourceNode, BiquadFilterNode, GainNode,
};

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
        volume.gain().set_value(0.5);

        let filter = context.create_biquad_filter();
        filter.set_type(web_audio_api::node::BiquadFilterType::Bandpass);
        filter.frequency().set_value(1800.0);
        filter.q().set_value(0.667);

        let source = context.create_buffer_source();
        source.set_loop(true);

        source.connect(&filter);
        filter.connect(&volume);
        volume.connect(destination);

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

    pub fn load(&self, context: &AudioContext, path: &str) -> Result<(), Error> {
        let file = File::open(path)?;
        let buffer = context.decode_audio_data_sync(file)?;
        self.source.set_buffer(buffer);
        Ok(())
    }

    pub fn play(&self) {
        self.source.start();
    }
}

pub struct AudioGraph {
    channels: Vec<AudioGraphChannel>,
    _volume: GainNode,
    context: AudioContext,
}

impl AudioGraph {
    pub fn new(num_channels: u8) -> Self {
        let context = AudioContext::default();

        let volume = context.create_gain();
        volume.gain().set_value(1.0);
        volume.connect(&context.destination());

        let channels: Vec<AudioGraphChannel> = (0..num_channels)
            .map(|_| AudioGraphChannel::new(&context, &volume))
            .collect();

        Self {
            context,
            channels,
            _volume: volume,
        }
    }

    pub fn get_channel(&self, channel_index: usize) -> Option<&AudioGraphChannel> {
        self.channels.get(channel_index)
    }

    pub fn context(&self) -> &AudioContext {
        &self.context
    }
}
