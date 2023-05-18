use midi_control::{Channel, MidiMessage};

mod audio_graph;
mod midi;

use crate::audio_graph::AudioGraph;
use crate::midi::Midi;

const VOLUME_CONTROL: u8 = 21;
const FILTER_FREQUENCY: u8 = 22;
const FILTER_Q: u8 = 23;
const SAMPLE_FILE: &str = "samples/rain.wav";

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to connect midi")]
    Midi(#[from] midi::Error),
    #[error("failed to control audio graph")]
    AudioGraph(#[from] audio_graph::Error),
}

fn main() -> Result<(), Error> {
    let (_midi, midi_rx) = Midi::start()?;
    let audio_graph = AudioGraph::new(4);
    {
        let context = audio_graph.context();
        let channel = audio_graph.get_channel(0).unwrap();
        channel.load(context, SAMPLE_FILE)?;
        channel.play();
    }

    for midi_msg in midi_rx {
        match midi_msg {
            MidiMessage::ControlChange(ch, ev) => {
                println!("ControlChange: {:?}, ev: {:?}", ch, ev);

                let channel = audio_graph.get_channel(0).unwrap();

                if ch != Channel::Ch8 {
                    continue;
                }

                match ev.control {
                    VOLUME_CONTROL => {
                        let level = midi_to_percent(ev.value);
                        channel.set_volume(level);
                    }

                    FILTER_FREQUENCY => {
                        let freq = midi_to_freq(ev.value);
                        println!("filter freq: {}", freq);
                        channel.set_filter_frequency(freq);
                    }

                    FILTER_Q => {
                        let q = midi_to_percent(ev.value);
                        println!("filter q: {}", q);
                        channel.set_filter_q(q);
                    }

                    _ => {}
                };
            }
            _ => {}
        }
    }
    Ok(())
}

fn midi_to_percent(midi_value: u8) -> f32 {
    let value = 1.0 / 127.0 * midi_value as f32;

    if value < 0.00001 {
        return 0.00001;
    }

    value
}

// TODO: this is wrong, for testing only
fn midi_to_freq(midi_value: u8) -> f32 {
    let value: i32 = midi_value as i32;

    if value < 1 {
        return 1.0;
    }

    value.pow(2) as f32
}
