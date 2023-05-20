use midi_control::MidiMessage;
use midir::{self, ConnectError, MidiInput, MidiInputConnection};
use std::sync::mpsc::{channel, Receiver, Sender};

// https://github.com/mmckegg/rust-loop-drop/blob/master/src/midi_connection.rs

/// String to look for when enumerating the MIDI devices
// const DEVICE: &str = "Launch Control";
const DEVICE: &str = "Faderfox EC4";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to find midi input device")]
    DeviceNotFound,
    #[error("failed to connect to midi input device")]
    ConnectInput(#[from] ConnectError<MidiInput>),
}

pub struct Midi {
    _input: MidiInputConnection<Sender<MidiMessage>>,
}

impl Midi {
    pub fn start() -> Result<(Self, Receiver<MidiMessage>), Error> {
        let midi_input = midir::MidiInput::new("MIDITest").unwrap();
        let device_port = find_port(&midi_input).ok_or(Error::DeviceNotFound)?;

        let (tx, rx) = channel::<MidiMessage>();

        println!("Port: {:?}", midi_input.port_name(&device_port));

        let connect_input = midi_input
            .connect(
                &device_port,
                DEVICE,
                move |timestamp, data, tx| {
                    let msg = MidiMessage::from(data);
                    println!("{}: received {:?} => {:?}", timestamp, data, msg);
                    tx.send(msg).expect("message transmitted on mpsc channel");
                },
                tx,
            )
            .map_err(Error::ConnectInput)?;

        Ok((
            Self {
                _input: connect_input,
            },
            rx,
        ))
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
