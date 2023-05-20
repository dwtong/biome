use crate::audio_graph::{AudioGraph, AudioGraphChannel};
use midi_control::{Channel, ControlEvent};

const MIDI_CHANNEL: Channel = Channel::Ch1;

enum ControlType {
    Volume,
    FilterFrequency,
    FilterQ,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("midi message is on incorrect midi channel")]
    IncorrectMidiChannel,
    #[error("midi value is not assigned to an audio channel")]
    MissingAudioChannel,
    #[error("midi value is not assigned to a control type")]
    MissingControlType,
}

pub struct MessageProcessor;

impl MessageProcessor {
    pub fn process_control_change(
        midi_channel: Channel,
        event: ControlEvent,
        audio_graph: &AudioGraph,
    ) -> Result<(), Error> {
        println!("ControlChange: {:?}, event: {:?}", midi_channel, event);

        if midi_channel != MIDI_CHANNEL {
            return Err(Error::IncorrectMidiChannel);
        }

        let (channel_index, control_type) = parse_control(event.control)?;
        let channel = audio_graph
            .get_channel(channel_index)
            .ok_or(Error::MissingAudioChannel)?;

        set_control_value(channel, control_type, event.value);
        Ok(())
    }
}

fn parse_control(control: u8) -> Result<(usize, ControlType), Error> {
    let control = control as usize;
    let control_type = control % 10;
    let channel = control / 10 - 1;

    match control_type {
        0 => Ok((channel, ControlType::Volume)),
        1 => Ok((channel, ControlType::FilterFrequency)),
        2 => Ok((channel, ControlType::FilterQ)),
        _ => Err(Error::MissingControlType),
    }
}

fn set_control_value(channel: &AudioGraphChannel, control_type: ControlType, midi_value: u8) {
    match control_type {
        ControlType::Volume => {
            let level = midi_to_percent(midi_value);
            channel.set_volume(level);
        }

        ControlType::FilterFrequency => {
            let freq = midi_to_freq(midi_value);
            channel.set_filter_frequency(freq);
        }

        ControlType::FilterQ => {
            let q = midi_to_percent(midi_value);
            channel.set_filter_q(q);
        }
    }
}

fn midi_to_percent(midi_value: u8) -> f32 {
    let value = 1.0 / 127.0 * midi_value as f32;

    if value < 0.00001 {
        return 0.00001;
    }

    value
}

fn midi_to_freq(midi_value: u8) -> f32 {
    let value = midi_value as f32;

    if value < 1.0 {
        return 1.0;
    }

    // HACK: produces range from 100 to ~16k for midi values
    value * value
}
