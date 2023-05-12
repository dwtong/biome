use midi_control::{Channel, MidiMessage};

mod audio_graph;
mod midi;

use crate::audio_graph::AudioGraph;
use crate::midi::Midi;

const VOLUME_CONTROL: u8 = 21;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to connect midi")]
    Midi(#[from] midi::Error),
    #[error("failed to initialise audiograph")]
    AudioGraph(#[from] audio_graph::Error),
}

fn main() -> Result<(), Error> {
    let (_midi, midi_rx) = Midi::start()?;
    let mut audio_graph = AudioGraph::new()?;

    for midi_msg in midi_rx {
        match midi_msg {
            MidiMessage::ControlChange(ch, ev) => {
                println!("ControlChange: {:?}, ev: {:?}", ch, ev);

                if ch != Channel::Ch8 {
                    continue;
                }

                match ev.control {
                    VOLUME_CONTROL => {
                        let level: f32 = 1.0 / 127.0 * ev.value as f32;
                        audio_graph.set_volume(level);
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }
    Ok(())
}
