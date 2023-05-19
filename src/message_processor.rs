use crate::audio_graph::AudioGraph;
use midi_control::{Channel, ControlEvent};

const VOLUME_CONTROL: u8 = 21;
const FILTER_FREQUENCY: u8 = 22;
const FILTER_Q: u8 = 23;

pub struct MessageProcessor;

impl MessageProcessor {
    pub fn process_control_change(channel: Channel, event: ControlEvent, audio_graph: &AudioGraph) {
        println!("ControlChange: {:?}, event: {:?}", channel, event);

        if channel != Channel::Ch8 {
            return;
        }

        let channel = audio_graph.get_channel(0).unwrap();

        match event.control {
            VOLUME_CONTROL => {
                let level = midi_to_percent(event.value);
                channel.set_volume(level);
            }

            FILTER_FREQUENCY => {
                let freq = midi_to_freq(event.value);
                println!("filter freq: {}", freq);
                channel.set_filter_frequency(freq);
            }

            FILTER_Q => {
                let q = midi_to_percent(event.value);
                println!("filter q: {}", q);
                channel.set_filter_q(q);
            }

            _ => {}
        };
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
