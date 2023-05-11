use midi_control::{Channel, MidiMessage};
use std::fs::File;
use std::io;
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

mod midi;

use crate::midi::Midi;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to open file")]
    OpenFile(#[from] io::Error),
    #[error("failed to decode audio")]
    DecodeAudio(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("failed to connect midi")]
    Midi(#[from] midi::Error),
}

fn main() -> Result<(), Error> {
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
    // src.playback_rate()
    //     .set_value_at_time(2.0, context.current_time());

    // pipe it all together
    src.connect(&lp_filter);
    lp_filter.connect(&hp_filter);
    hp_filter.connect(&volume);
    volume.connect(&context.destination());

    // play the buffer
    src.start();

    let (_midi, midi_rx) = Midi::start()?;

    for midi_msg in midi_rx {
        match midi_msg {
            MidiMessage::ControlChange(ch, ev) => {
                println!("ControlChange: {:?}, ev: {:?}", ch, ev);

                match ch {
                    Channel::Ch1 => {
                        let level: f32 = 1.0 / 127.0 * ev.control as f32;
                        println!("midi change my volume to {}", level);
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }

    loop {}
}
