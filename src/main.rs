use midi_control::MidiMessage;

mod audio_graph;
mod message_processor;
mod midi;

use crate::audio_graph::AudioGraph;
use crate::message_processor::MessageProcessor;
use crate::midi::Midi;

const CHANNEL_COUNT: usize = 4;
const SAMPLE_FILES: [&str; CHANNEL_COUNT] = [
    "samples/bird.wav",
    "samples/rain.wav",
    "samples/crunch.wav",
    "samples/taps.wav",
];

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

    for channel in 0..CHANNEL_COUNT {
        load_and_play_file_for_channel(&audio_graph, channel)?;
    }

    for midi_msg in midi_rx {
        match midi_msg {
            MidiMessage::ControlChange(channel, event) => {
                MessageProcessor::process_control_change(channel, event, &audio_graph)
                    .unwrap_or_else(|error| eprintln!("{}", error))
            }
            _ => {}
        }
    }
    Ok(())
}

fn load_and_play_file_for_channel(
    audio_graph: &AudioGraph,
    channel_index: usize,
) -> Result<(), Error> {
    let context = audio_graph.context();
    let channel = audio_graph.get_channel(channel_index).unwrap();
    let sample_file = SAMPLE_FILES.get(channel_index).unwrap();

    channel.load(context, sample_file)?;
    channel.play();
    Ok(())
}
