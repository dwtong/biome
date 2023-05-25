use std::{println, sync::mpsc::Sender, thread};

use monome::{KeyDirection, Monome, MonomeDeviceType, MonomeEvent};

use crate::message::ControlMessage;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to find monome grid device")]
    DeviceNotFound,
    //     #[error("failed to connect to midi input device")]
    // ConnectInput(#[from] ConnectError<MidiInput>),
}

const GRID_X: usize = 16;
const GRID_Y: usize = 8;
const GRID_LENGTH: usize = GRID_X * GRID_Y;

pub struct Grid {
    device: Monome,
}

impl Grid {
    pub fn connect() -> Result<Self, Error> {
        let device = Monome::enumerate_devices()
            .unwrap()
            .into_iter()
            .find(|d| d.device_type() == MonomeDeviceType::Grid)
            .ok_or(Error::DeviceNotFound)?;

        let device = Monome::from_device(&device, "/prefix").unwrap();

        Ok(Grid { device })
    }

    pub fn start(mut self, tx: Sender<ControlMessage>) {
        thread::spawn(move || {
            loop {
                match self.poll() {
                    Some(MonomeEvent::GridKey { x, y, direction }) => match direction {
                        KeyDirection::Down => {
                            println!("Key pressed: {}x{}", x, y);
                            tx.send(ControlMessage::SetChannelSampleFile(1, x as usize))
                                .unwrap();
                            self.lit();
                        }
                        KeyDirection::Up => {
                            println!("Key released: {}x{}", x, y);
                            self.unlit();
                        }
                    },
                    _ => {
                        // break;
                    }
                }
            }
        });
    }

    pub fn poll(&mut self) -> Option<MonomeEvent> {
        self.device.poll()
    }

    pub fn unlit(&mut self) {
        let grid: [bool; GRID_LENGTH] = [false; GRID_LENGTH];
        self.device.set_all(&grid);
    }

    pub fn lit(&mut self) {
        let grid: [bool; GRID_LENGTH] = [true; GRID_LENGTH];
        self.device.set_all(&grid);
    }
}
