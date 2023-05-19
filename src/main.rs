use midi_control::MidiMessage;

mod audio_graph;
mod message_processor;
mod midi;

use crate::audio_graph::AudioGraph;
use crate::message_processor::MessageProcessor;
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
            MidiMessage::ControlChange(channel, event) => {
                MessageProcessor::process_control_change(channel, event, &audio_graph)
            }
            _ => {}
        }
    }
    Ok(())
}

}
