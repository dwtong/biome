use grid::Grid;
use message::ControlMessage;
use midi::Midi;
use sample_manager::SampleManager;
use std::{process, sync::mpsc::channel, time::Duration};

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
    let (control_sender, control_receiver) = channel::<ControlMessage>();
    let _midi = Midi::start(control_sender.clone())?;
    let (grid, grid_sender) = Grid::connect()?;
    grid.start(control_sender);
    let mut audio_graph = AudioGraph::new(CHANNEL_COUNT);
    let sample_manager = SampleManager::new();

    ctrlc::set_handler(move || {
        grid_sender.send(grid::GridMessage::Clear).unwrap();
        // wait for grid to clear
        std::thread::sleep(Duration::from_millis(100));
        process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");

    for channel_index in 0..CHANNEL_COUNT {
        let sample_file = sample_manager
            .get_path_for_sample(channel_index, 0)
            .expect("Found default file in sample directory for channel");
        audio_graph.load_and_play_for_channel(channel_index, &sample_file);
    }

    for control_message in control_receiver {
        message::process_message(control_message, &mut audio_graph, &sample_manager)?;
    }

    Ok(())
}
