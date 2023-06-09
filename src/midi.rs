use midi_control::{ControlEvent, MidiMessage, MidiMessageSend};
use midir::{
    self, ConnectError, InitError, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection,
};
use std::sync::mpsc;

use crate::{
    message::ControlMessage,
    settings::{ControlParam, Settings},
};

// https://github.com/mmckegg/rust-loop-drop/blob/master/src/midi_connection.rs

/// String to look for when enumerating the MIDI devices
const CLIENT_NAME: &str = "biome";

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
    EchoValue(#[from] midir::SendError),
    #[error("midi value is not assigned to a control type")]
    MissingControlType,
    #[error("failed to transmit control message")]
    TransmitControlMessage(#[from] mpsc::SendError<ControlMessage>),
}

pub struct Midi {
    _input: MidiInputConnection<(mpsc::Sender<ControlMessage>, Settings)>,
    output: MidiOutputConnection,
    tx: mpsc::Sender<ControlMessage>,
}

impl Midi {
    pub fn start(tx: mpsc::Sender<ControlMessage>, settings: Settings) -> Result<Self, Error> {
        let midi_output = MidiOutput::new(CLIENT_NAME)?;
        let midi_input = MidiInput::new(CLIENT_NAME)?;
        let in_port = find_port(&midi_input, &settings).ok_or(Error::InputDeviceNotFound)?;
        let out_port = find_port(&midi_output, &settings).ok_or(Error::OutputDeviceNotFound)?;

        println!("midi in: {:?}", midi_input.port_name(&in_port));
        println!("midi out: {:?}", midi_output.port_name(&out_port));

        let connect_output = midi_output
            .connect(&out_port, settings.midi_device())
            .map_err(Error::ConnectOutput)?;

        let connect_input = midi_input
            .connect(
                &in_port,
                settings.midi_device(),
                move |timestamp, data, (tx, settings)| {
                    let midi_msg = MidiMessage::from(data);
                    println!("{}: received {:?} => {:?}", timestamp, data, tx);
                    match midi_msg {
                        MidiMessage::ControlChange(channel, event) => {
                            if channel != settings.midi_channel() {
                                eprintln!(
                                    "ignored control message on incorrect midi channel {:?}",
                                    channel
                                );
                            }
                            match parse_control_event(event, settings) {
                                Ok(ctrl_msg) => {
                                    tx.send(ctrl_msg).expect("Transmitted control message");
                                }
                                Err(error) => {
                                    eprintln!("couldn't process control message {:?}", error);
                                }
                            }
                        }
                        message => {
                            eprintln!("unsupported midi message {:?}", message);
                        }
                    }
                },
                (tx.clone(), settings.clone()),
            )
            .map_err(Error::ConnectInput)?;

        Ok(Self {
            _input: connect_input,
            output: connect_output,
            tx,
        })
    }

    pub fn init_values(&mut self, settings: &Settings) -> Result<(), Error> {
        for (control, value) in settings.midi_initial_values() {
            let msg = midi_control::control_change(settings.midi_channel(), control, value);
            let event = ControlEvent { control, value };

            // echo midi values to midi device
            self.output.send_message(msg).map_err(Error::EchoValue)?;

            // transmit initial values to audiograph
            parse_control_event(event, settings).and_then(|ctrl_msg| {
                self.tx
                    .send(ctrl_msg)
                    .map_err(Error::TransmitControlMessage)
            })?;
        }
        Ok(())
    }
}

fn find_port<T>(midi_io: &T, settings: &Settings) -> Option<T::Port>
where
    T: midir::MidiIO,
{
    let mut device_port: Option<T::Port> = None;
    for port in midi_io.ports() {
        if let Ok(port_name) = midi_io.port_name(&port) {
            if port_name.contains(settings.midi_device()) {
                device_port = Some(port);
                break;
            }
        }
    }
    device_port
}

fn parse_control_event(event: ControlEvent, settings: &Settings) -> Result<ControlMessage, Error> {
    println!("ControlChange event: {:?}", event);

    let (audio_channel, control_type) = settings
        .channel_and_param_from_midi_event(event.control)
        .ok_or(Error::MissingControlType)?;

    match control_type {
        ControlParam::FilterFrequency => Ok(ControlMessage::SetChannelFilterFrequency(
            audio_channel,
            midi_to_freq(event.value),
        )),
        ControlParam::FilterQ => Ok(ControlMessage::SetChannelFilterQ(
            audio_channel,
            midi_to_percent(event.value),
        )),
        ControlParam::Rate => Ok(ControlMessage::SetChannelRate(
            audio_channel,
            midi_to_percent(event.value),
        )),
        ControlParam::Volume => Ok(ControlMessage::SetChannelVolume(
            audio_channel,
            midi_to_percent(event.value),
        )),
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
