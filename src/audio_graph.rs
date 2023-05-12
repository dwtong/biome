use std::{fs::File, io};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};
use web_audio_api::{
    context::{AudioContext, BaseAudioContext},
    node::GainNode,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to open file")]
    OpenFile(#[from] io::Error),
    #[error("failed to decode audio")]
    DecodeAudio(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct AudioGraph {
    volume: GainNode,
    _context: AudioContext,
}

impl AudioGraph {
    pub fn new() -> Result<Self, Error> {
        // set up the audio context with optimized settings for your hardware
        let context = AudioContext::default();

        // for background music, read from local file
        let file = File::open("samples/major-scale.ogg")?;
        let buffer = context.decode_audio_data_sync(file)?;

        // create gain control
        let volume = context.create_gain();
        volume.gain().set_value(0.5);

        // create low pass filter
        let lp_filter = context.create_biquad_filter();
        lp_filter.set_type(web_audio_api::node::BiquadFilterType::Lowpass);
        lp_filter
            .frequency()
            .set_value_at_time(1900.0, context.current_time());

        // create high pass filter
        let hp_filter = context.create_biquad_filter();
        hp_filter.set_type(web_audio_api::node::BiquadFilterType::Highpass);
        hp_filter
            .frequency()
            .set_value_at_time(1800.0, context.current_time());

        // setup an AudioBufferSourceNode
        let src = context.create_buffer_source();
        src.set_buffer(buffer);
        src.set_loop(true);
        // TODO: get playback rate working
        // src.playback_rate().set_value(0.1);

        // pipe it all together
        src.connect(&lp_filter);
        lp_filter.connect(&hp_filter);
        hp_filter.connect(&volume);
        volume.connect(&context.destination());

        // play the buffer
        src.start();

        Ok(Self {
            volume,
            _context: context,
        })
    }

    pub fn set_volume(&mut self, value: f32) {
        self.volume.gain().set_value(value);
    }
}
