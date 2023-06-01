use grid::Grid;
use message::ControlMessage;
use midi::Midi;
use sample_manager::SampleManager;
use std::sync::mpsc::channel;

mod audio_graph;
mod grid;
mod message;
mod midi;
mod sample_manager;

use crate::audio_graph::AudioGraph;

pub const CHANNEL_COUNT: usize = 4;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to control audio graph")]
    AudioGraph(#[from] audio_graph::Error),
    #[error("failed to connect grid")]
    Grid(#[from] grid::Error),
    #[error("failed to process control message")]
    ControlMessage(#[from] message::Error),
    #[error("failed to connect midi")]
    Midi(#[from] midi::Error),
}

fn main() -> Result<(), Error> {
    let (tx, rx) = channel::<ControlMessage>();
    let _midi = Midi::start(tx.clone())?;
    let grid = Grid::connect()?;
    grid.start(tx);
    let mut audio_graph = AudioGraph::new(CHANNEL_COUNT);
    let sample_manager = SampleManager::new();
    println!("sample manager: {:?}", sample_manager);

    // for (channel_index, _) in SAMPLE_FILES.iter().enumerate().take(CHANNEL_COUNT) {
    //     audio_graph.load_and_play_for_channel(channel_index, SAMPLE_FILES[channel_index]);
    // }

    for control_message in rx {
        message::process_message(control_message, &mut audio_graph)?;
    }

    Ok(())
}
