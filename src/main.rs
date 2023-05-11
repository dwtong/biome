use midir::{self, ConnectError, MidiInputConnection, MidiInputPort};
use midir::{Ignore, MidiInput};
use std::fs::File;
use std::io;
use std::io::{stdin, stdout, Write};
use thiserror::Error as ThisError;
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

const MIDI_CC_VOLUME: u8 = 21;

#[derive(Debug, ThisError)]
enum PlaygroundError {
    #[error("failed to open file")]
    OpenFile(#[from] io::Error),
    #[error("failed to decode audio")]
    DecodeAudio(#[from] Box<dyn std::error::Error + Send + Sync>),
}

fn main() -> Result<(), PlaygroundError> {
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

    let midi_callback = |message: &[u8]| {
        let cc_id = message[1];
        let cc_value = message[2] as f32;

        match cc_id {
            MIDI_CC_VOLUME => {
                let level: f32 = 1.0 / 127.0 * cc_value;
                println!("midi change my volume to {}", level);
                // &volume.gain().set_value(level);
            }
            _ => println!("ignored cc with id: {}, value: {}", cc_id, cc_value),
        };
    };

    match connect_midi(midi_callback) {
        Ok(_) => loop {},
        Err(err) => println!("Error: {}", err),
    };

    Ok(())
}

fn connect_midi(
    midi_callback: fn(&[u8]) -> (),
) -> Result<MidiInputConnection<()>, ConnectError<MidiInput>> {
    let mut midi_in = MidiInput::new("midi input").unwrap();
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port: &MidiInputPort = match in_ports.len() {
        0 => return Err("no input port found").unwrap(),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush().unwrap();
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            in_ports
                .get(input.trim().parse::<usize>().unwrap())
                .ok_or("invalid input port selected")
                .unwrap()
        }
    };

    midi_in.connect(
        in_port,
        "midir-read-input",
        move |_, message, _| {
            midi_callback(message);
        },
        (),
    )
}
