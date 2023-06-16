use grid::Grid;
use message::ControlMessage;
use midi::Midi;
use sample_manager::SampleManager;
use settings::Settings;
use std::{process, sync::mpsc::channel, time::Duration};

mod audio_graph;
mod grid;
mod message;
mod midi;
mod sample_manager;
mod settings;

use crate::audio_graph::AudioGraph;

pub const MAX_CHANNEL_COUNT: usize = 4;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to control audio graph")]
    AudioGraph(#[from] audio_graph::Error),
    #[error("failed to process control message")]
    ControlMessage(#[from] message::Error),
    #[error("failed to connect midi")]
    Midi(#[from] midi::Error),
    #[error("failed to parse settings")]
    Settings(#[from] settings::Error),
}

fn main() -> Result<(), Error> {
    let settings = Settings::new()?;
    let (control_tx, control_rx) = channel::<ControlMessage>();
    let (grid, grid_tx) = Grid::new(&settings);
    let sample_manager = SampleManager::new(&settings);
    let mut midi = Midi::start(control_tx.clone(), settings.clone())?;
    let mut audio_graph = AudioGraph::new(&settings);

    grid.start(control_tx.clone());
    midi.init_values(&settings)?;

    ctrlc::set_handler(move || {
        grid_tx.send(grid::GridMessage::Clear).unwrap();
        control_tx.send(ControlMessage::MuteAll).unwrap();
        // wait for grid to clear and audio graph to fade
        std::thread::sleep(Duration::from_millis(250));
        process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");

    for channel_index in 0..settings.channel_count() {
        let sample_file = sample_manager
            .get_path_for_sample(channel_index, 0)
            .expect("Found default file in sample directory for channel");
        audio_graph.load_and_play_for_channel(channel_index, &sample_file);
    }

    for control_message in control_rx {
        message::process_message(control_message, &mut audio_graph, &sample_manager)?;
    }

    Ok(())
}
