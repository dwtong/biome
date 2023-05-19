use crate::{
    audio_graph::{AudioGraph, AudioGraphChannel},
    CHANNEL_COUNT,
};
use midi_control::{Channel, ControlEvent};

enum ControlType {
    Volume,
    FilterFrequency,
    FilterQ,
}

pub struct MessageProcessor;

impl MessageProcessor {
    pub fn process_control_change(
        midi_channel: Channel,
        event: ControlEvent,
        audio_graph: &AudioGraph,
    ) {
        println!("ControlChange: {:?}, event: {:?}", midi_channel, event);

        if midi_channel != Channel::Ch1 {
            return;
        }

        if let Some((channel, control_type)) = parse_control(event.control) {
            if let Some(channel) = audio_graph.get_channel(channel) {
                set_control_value(channel, control_type, event.value);
            }
        };
    }
}

fn parse_control(control: u8) -> Option<(usize, ControlType)> {
    let control = control as usize;
    let control_type = control % 10;
    let channel = control / 10 - 1;

    match control_type {
        0 => Some((channel, ControlType::Volume)),
        1 => Some((channel, ControlType::FilterFrequency)),
        2 => Some((channel, ControlType::FilterQ)),
        _ => None,
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
