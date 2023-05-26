use crate::{message::ControlMessage, CHANNEL_COUNT};
use monome::{KeyDirection, Monome, MonomeDeviceType, MonomeEvent};
use std::{println, sync::mpsc::Sender, thread};

const SAMPLE_GRID_X: i32 = 8;
const SAMPLE_GRID_Y: i32 = 6;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to find monome grid device")]
    DeviceNotFound,
    //     #[error("failed to connect to midi input device")]
    // ConnectInput(#[from] ConnectError<MidiInput>),
    #[error("failed to create grid from monome device")]
    FromDevice(String),
}

// const GRID_X: usize = 16;
// const GRID_Y: usize = 8;
// const GRID_LENGTH: usize = GRID_X * GRID_Y;

#[derive(Clone, Copy)]
pub struct GridChannel {}

impl GridChannel {
    fn new() -> Self {
        GridChannel {}
    }
}

pub struct Grid {
    device: Monome,
    channels: [GridChannel; CHANNEL_COUNT],
    selected_channel: i32,
}

impl Grid {
    pub fn connect() -> Result<Self, Error> {
        let device = Monome::enumerate_devices()
            .unwrap()
            .into_iter()
            .find(|d| d.device_type() == MonomeDeviceType::Grid)
            .ok_or(Error::DeviceNotFound)?;

        let device =
            Monome::from_device(&device, "/prefix").map_err(|string| Error::FromDevice(string))?;

        let channels = [GridChannel::new(); CHANNEL_COUNT];

        Ok(Grid {
            device,
            channels,
            selected_channel: 0,
        })
    }

    pub fn start(mut self, tx: Sender<ControlMessage>) {
        self.redraw();

        thread::spawn(move || loop {
            if let Some(MonomeEvent::GridKey {
                x,
                y,
                direction: KeyDirection::Down,
            }) = self.device.poll()
            {
                if let Some(control_message) = self.match_action((x, y)) {
                    tx.send(control_message).unwrap();
                }
                self.redraw();
            }
        });
    }

    pub fn redraw(&mut self) {
        for (index, _) in self.channels.iter().enumerate() {
            let x = index as i32;
            let y = 7;
            let brightness = if self.selected_channel == x { 10 } else { 5 };
            self.device.set(x, y, brightness);
        }
    }

    pub fn match_action(&mut self, coords: (i32, i32)) -> Option<ControlMessage> {
        match coords {
            (x, 7) if x < CHANNEL_COUNT as i32 => {
                self.selected_channel = x;
                None
            }

            (x, y) if x < SAMPLE_GRID_X && y < SAMPLE_GRID_Y => {
                let sample_file = x + SAMPLE_GRID_X * y;

                Some(ControlMessage::SetChannelSampleFile(
                    self.selected_channel as usize,
                    sample_file as usize,
                ))
            }

            (x, y) => {
                println!("Key press ignored: {}x{}", x, y);
                None
            }
        }
    }
}
