use grid::Grid;
use message::ControlMessage;
use midi::Midi;
use std::sync::mpsc::channel;

mod audio_graph;
mod grid;
mod message;
mod midi;

use crate::audio_graph::AudioGraph;

pub const CHANNEL_COUNT: usize = 4;

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
    let mut audio_graph = AudioGraph::new(CHANNEL_COUNT);

    for (channel_index, _) in SAMPLE_FILES.iter().enumerate().take(CHANNEL_COUNT) {
        audio_graph.load_and_play_for_channel(channel_index, SAMPLE_FILES[channel_index]);
    }

    for control_message in rx {
        message::process_message(control_message, &mut audio_graph).unwrap();
    }

    Ok(())
}
