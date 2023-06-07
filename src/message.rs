use std::println;

use crate::{audio_graph::AudioGraph, sample_manager::SampleManager};

type AudioChannel = usize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("audio channel does not exist")]
    MissingAudioChannel,
}

#[derive(Copy, Clone, Debug)]
pub enum ControlMessage {
    SetChannelVolume(AudioChannel, f32),
    SetChannelFilterFrequency(AudioChannel, f32),
    SetChannelFilterQ(AudioChannel, f32),
    SetChannelSampleFile(AudioChannel, usize),
}

pub fn process_message(
    msg: ControlMessage,
    audio_graph: &mut AudioGraph,
    sample_manager: &SampleManager,
) -> Result<(), Error> {
    println!("Message: {:?}", msg);

    match msg {
        ControlMessage::SetChannelVolume(channel_index, level) => {
            let channel = audio_graph
                .get_channel(channel_index)
                .ok_or(Error::MissingAudioChannel)?;
            channel.set_volume(level);
        }

        ControlMessage::SetChannelFilterFrequency(channel_index, freq) => {
            let channel = audio_graph
                .get_channel(channel_index)
                .ok_or(Error::MissingAudioChannel)?;
            channel.set_filter_frequency(freq);
        }

        ControlMessage::SetChannelFilterQ(channel_index, q) => {
            let channel = audio_graph
                .get_channel(channel_index)
                .ok_or(Error::MissingAudioChannel)?;
            channel.set_filter_q(q);
        }

        ControlMessage::SetChannelSampleFile(channel_index, sample_index) => {
            let sample_file = sample_manager.get_path_for_sample(channel_index, sample_index);

            if let Some(sample_file) = sample_file {
                audio_graph.load_and_play_for_channel(channel_index, &sample_file);
            };
        }
    }

    Ok(())
}
