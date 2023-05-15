use std::{fs::File, io};
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode, BiquadFilterNode, GainNode};

const SAMPLE_FILE: &str = "samples/rain.wav";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to open file")]
    OpenFile(#[from] io::Error),
    #[error("failed to decode audio")]
    DecodeAudio(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct AudioGraph {
    filter: BiquadFilterNode,
    volume: GainNode,
    _context: AudioContext,
}

impl AudioGraph {
    pub fn new() -> Result<Self, Error> {
        let context = AudioContext::default();

        let file = File::open(SAMPLE_FILE)?;
        let buffer = context.decode_audio_data_sync(file)?;

        let volume = context.create_gain();
        volume.gain().set_value(0.5);

        let filter = context.create_biquad_filter();
        filter.set_type(web_audio_api::node::BiquadFilterType::Bandpass);
        filter.frequency().set_value(1800.0);
        filter.q().set_value(0.667);

        let src = context.create_buffer_source();
        src.set_buffer(buffer);
        src.set_loop(true);
        // TODO: get playback rate working
        // src.playback_rate().set_value(0.1);

        src.connect(&filter);
        filter.connect(&volume);
        volume.connect(&context.destination());

        // play the buffer
        src.start();

        Ok(Self {
            volume,
            filter,
            _context: context,
        })
    }

    pub fn set_filter_q(&mut self, value: f32) {
        self.filter.q().set_value(value);
    }

    pub fn set_filter_frequency(&mut self, value: f32) {
        self.filter.frequency().set_value(value);
    }

    pub fn set_volume(&mut self, value: f32) {
        self.volume.gain().set_value(value);
    }
}
