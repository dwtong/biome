use crate::{message::ControlMessage, CHANNEL_COUNT, SAMPLE_FILES};
use monome::{KeyDirection, Monome, MonomeDeviceType, MonomeEvent};
use std::{println, sync::mpsc::Sender, thread};

const SAMPLE_GRID_X: usize = 8;
const SAMPLE_GRID_Y: usize = 6;
const MAX_SAMPLES: usize = SAMPLE_GRID_X * SAMPLE_GRID_Y;

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
pub struct GridChannel {
    selected_sample: usize,
    sample_count: usize,
}

impl GridChannel {
    fn new() -> Self {
        GridChannel {
            selected_sample: 0,
            sample_count: SAMPLE_FILES.len(),
        }
    }
}

pub struct Grid {
    device: Monome,
    channels: [GridChannel; CHANNEL_COUNT],
    selected_channel_index: usize,
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
            selected_channel_index: 0,
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
        self.redraw_channel_strip();
        self.redraw_sample_selector();
    }

    pub fn selected_channel(&self) -> Option<&GridChannel> {
        self.channels.get(self.selected_channel_index)
    }

    fn redraw_channel_strip(&mut self) {
        for (index, _) in self.channels.iter().enumerate() {
            let x = index as i32;
            let y = 7;
            let brightness = if self.selected_channel_index == index {
                10
            } else {
                5
            };
            self.device.set(x, y, brightness);
        }
    }

    fn redraw_sample_selector(&mut self) {
        let selected_channel = self.selected_channel().unwrap();
        let sample_count = selected_channel.sample_count;
        let selected_sample = selected_channel.selected_sample;

        for index in 0..MAX_SAMPLES {
            let pos = index as i32;
            let x = pos % SAMPLE_GRID_X as i32;
            let y = pos / SAMPLE_GRID_X as i32;

            if index > sample_count {
                break;
            }
            if index == selected_sample {
                self.device.set(x, y, 10);
                continue;
            }
            self.device.set(x, y, 5);
        }
    }

    pub fn match_action(&mut self, coords: (i32, i32)) -> Option<ControlMessage> {
        match coords {
            (x, 7) if x < CHANNEL_COUNT as i32 => {
                self.selected_channel_index = x as usize;
                None
            }

            (x, y) if x < (SAMPLE_GRID_X as i32) && y < (SAMPLE_GRID_Y as i32) => {
                let sample_file = x + (SAMPLE_GRID_X as i32) * y;

                Some(ControlMessage::SetChannelSampleFile(
                    self.selected_channel_index,
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
