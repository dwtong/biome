use midi_control::{Channel, ControlEvent, MidiMessage};
use midir::{self, ConnectError, MidiInput, MidiInputConnection};
use std::sync::mpsc::Sender;

use crate::message::ControlMessage;

// https://github.com/mmckegg/rust-loop-drop/blob/master/src/midi_connection.rs

/// String to look for when enumerating the MIDI devices
// const DEVICE: &str = "Launch Control";
const DEVICE: &str = "Faderfox EC4";

const MIDI_CHANNEL: Channel = Channel::Ch1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to find midi input device")]
    DeviceNotFound,
    #[error("failed to connect to midi input device")]
    ConnectInput(#[from] ConnectError<MidiInput>),
    #[error("midi message is on incorrect midi channel")]
    IncorrectMidiChannel,
    #[error("midi value is not assigned to a control type")]
    MissingControlType,
}

pub struct Midi {
    _input: MidiInputConnection<Sender<ControlMessage>>,
}

impl Midi {
    pub fn start(tx: Sender<ControlMessage>) -> Result<Self, Error> {
        let midi_input = midir::MidiInput::new("MIDITest").unwrap();
        let device_port = find_port(&midi_input).ok_or(Error::DeviceNotFound)?;

        println!("Port: {:?}", midi_input.port_name(&device_port));

        let connect_input = midi_input
            .connect(
                &device_port,
                DEVICE,
                move |timestamp, data, tx| {
                    let midi_msg = MidiMessage::from(data);
                    println!("{}: received {:?} => {:?}", timestamp, data, tx);
                    match midi_msg {
                        MidiMessage::ControlChange(channel, event) => {
                            let ctrl_msg = process_control_change(channel, event).unwrap();
                            // .unwrap_or_else(|error| eprintln!("{}", error));
                            tx.send(ctrl_msg)
                                .expect("message transmitted on mpsc channel");
                        }
                        _ => {}
                    }
                },
                tx,
            )
            .map_err(Error::ConnectInput)?;

        Ok(Self {
            _input: connect_input,
        })
    }
}

fn find_port<T>(midi_io: &T) -> Option<T::Port>
where
    T: midir::MidiIO,
{
    let mut device_port: Option<T::Port> = None;
    for port in midi_io.ports() {
        if let Ok(port_name) = midi_io.port_name(&port) {
            if port_name.contains(DEVICE) {
                device_port = Some(port);
                break;
            }
        }
    }
    device_port
}

fn process_control_change(
    midi_channel: Channel,
    event: ControlEvent,
) -> Result<ControlMessage, Error> {
    println!("ControlChange: {:?}, event: {:?}", midi_channel, event);

    if midi_channel != MIDI_CHANNEL {
        return Err(Error::IncorrectMidiChannel);
    }

    let (channel, control_type) = parse_control_value(event.control);

    match control_type {
        0 => Ok(ControlMessage::SetChannelVolume(
            channel,
            midi_to_percent(event.value),
        )),
        1 => Ok(ControlMessage::SetChannelFilterFrequency(
            channel,
            midi_to_freq(event.value),
        )),
        2 => Ok(ControlMessage::SetChannelFilterQ(
            channel,
            midi_to_percent(event.value),
        )),
        _ => Err(Error::MissingControlType),
    }
}

fn parse_control_value(control: u8) -> (usize, usize) {
    let control = control as usize;
    let control_type = control % 10;
    let channel = control / 10 - 1;
    (channel, control_type)
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
