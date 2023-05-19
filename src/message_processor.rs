use crate::{audio_graph::AudioGraph, CHANNEL_COUNT};
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

        match parse_control(event.control) {
            Some((channel, _)) if channel >= CHANNEL_COUNT => (),

            Some((channel, ControlType::Volume)) => {
                let channel = audio_graph.get_channel(channel).unwrap();
                let level = midi_to_percent(event.value);
                channel.set_volume(level);
            }

            Some((channel, ControlType::FilterFrequency)) => {
                let channel = audio_graph.get_channel(channel).unwrap();
                let freq = midi_to_freq(event.value);
                println!("filter freq: {}", freq);
                channel.set_filter_frequency(freq);
            }

            Some((channel, ControlType::FilterQ)) => {
                let channel = audio_graph.get_channel(channel).unwrap();
                let q = midi_to_percent(event.value);
                println!("filter q: {}", q);
                channel.set_filter_q(q);
            }

            None => (),
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
