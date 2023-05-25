use grid::Grid;
use message::{process_message, ControlMessage};
use midi::Midi;
use std::sync::mpsc::channel;

mod audio_graph;
mod grid;
mod message;
mod midi;

use crate::audio_graph::AudioGraph;

const CHANNEL_COUNT: usize = 4;
const SAMPLE_FILES: [&str; CHANNEL_COUNT] = [
    "samples/bird.wav",
    "samples/rain.wav",
    "samples/crunch.wav",
    "samples/taps.wav",
];

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to control audio graph")]
    AudioGraph(#[from] audio_graph::Error),
    #[error("failed to connect midi")]
    Midi(#[from] midi::Error),
}

fn main() -> Result<(), Error> {
    let (tx, rx) = channel::<ControlMessage>();
    let _midi = Midi::start(tx.clone())?;
    let grid = Grid::connect().unwrap();
    grid.start(tx);
    let audio_graph = AudioGraph::new(4);

    for channel in 0..CHANNEL_COUNT {
        load_and_play_file_for_channel(&audio_graph, channel)?;
    }

    for control_message in rx {
        process_message(control_message, &audio_graph).unwrap();
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
