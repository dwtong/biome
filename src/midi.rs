use midi_control::{Channel, ControlEvent, MidiMessage, MidiMessageSend};
use midir::{
    self, ConnectError, InitError, MidiInput, MidiInputConnection, MidiOutput,
    MidiOutputConnection, SendError,
};
use std::{eprintln, sync::mpsc::Sender};

use crate::{message::ControlMessage, settings::Settings};

// https://github.com/mmckegg/rust-loop-drop/blob/master/src/midi_connection.rs

/// String to look for when enumerating the MIDI devices
// const DEVICE: &str = "Launch Control";
const DEVICE: &str = "Faderfox EC4";
const CLIENT_NAME: &str = "WAP";

const MIDI_CHANNEL: Channel = Channel::Ch1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to find midi input device")]
    InputDeviceNotFound,
    #[error("failed to find midi output device")]
    OutputDeviceNotFound,
    #[error("failed to connect to midi input device")]
    ConnectInput(#[from] ConnectError<MidiInput>),
    #[error("failed to connect to midi output device")]
    ConnectOutput(#[from] ConnectError<MidiOutput>),
    #[error("failed to initialise midi input device")]
    DeviceInit(#[from] InitError),
    #[error("failed to echo midi value to output device")]
    EchoValue(#[from] SendError),
    #[error("midi message is on incorrect midi channel")]
    IncorrectMidiChannel,
    #[error("midi value is not assigned to a control type")]
    MissingControlType,
}

pub struct Midi {
    _input: MidiInputConnection<Sender<ControlMessage>>,
    output: MidiOutputConnection,
}

impl Midi {
    pub fn start(tx: Sender<ControlMessage>) -> Result<Self, Error> {
        let midi_output = MidiOutput::new(CLIENT_NAME)?;
        let midi_input = MidiInput::new(CLIENT_NAME)?;
        let in_port = find_port(&midi_input).ok_or(Error::InputDeviceNotFound)?;
        let out_port = find_port(&midi_output).ok_or(Error::OutputDeviceNotFound)?;

        println!("midi in: {:?}", midi_input.port_name(&in_port));
        println!("midi out: {:?}", midi_output.port_name(&out_port));

        let connect_output = midi_output
            .connect(&out_port, DEVICE)
            .map_err(Error::ConnectOutput)?;

        let connect_input = midi_input
            .connect(
                &in_port,
                DEVICE,
                move |timestamp, data, tx| {
                    let midi_msg = MidiMessage::from(data);
                    println!("{}: received {:?} => {:?}", timestamp, data, tx);
                    if let MidiMessage::ControlChange(channel, event) = midi_msg {
                        match process_control_change(channel, event) {
                            Ok(ctrl_msg) => tx
                                .send(ctrl_msg)
                                .expect("message transmitted on mpsc channel"),
                            Err(error) => eprintln!("Couldn't process control change: {}", error),
                        }
                    }
                },
                tx,
            )
            .map_err(Error::ConnectInput)?;

        Ok(Self {
            _input: connect_input,
            output: connect_output,
        })
    }

    pub fn init_values(&mut self, settings: &Settings) -> Result<(), Error> {
        for (cc_id, value) in settings.midi_initial_values() {
            let msg = midi_control::control_change(MIDI_CHANNEL, cc_id, value);
            self.output.send_message(msg).map_err(Error::EchoValue)?;
        }
        Ok(())
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
