use crate::{message::ControlMessage, CHANNEL_COUNT};
use monome::{KeyDirection, Monome, MonomeDeviceType, MonomeEvent};
use std::{
    println,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

const SAMPLE_GRID_X: usize = 8;
const SAMPLE_GRID_Y: usize = 6;
const SAMPLE_GRID: usize = SAMPLE_GRID_X * SAMPLE_GRID_Y;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to find monome grid device")]
    DeviceNotFound,
    #[error("failed to create grid from monome device")]
    FromDevice(String),
}

#[derive(Debug)]
pub enum GridMessage {
    Clear,
}

pub struct Grid {
    receiver: Receiver<GridMessage>,
    device: Monome,
    selected_sample_indexes: [usize; CHANNEL_COUNT],
    selected_channel_index: usize,
}

impl Grid {
    pub fn connect() -> Result<(Self, Sender<GridMessage>), Error> {
        let device = Monome::enumerate_devices()
            .expect("Monome setup successfully")
            .into_iter()
            .find(|d| d.device_type() == MonomeDeviceType::Grid)
            .ok_or(Error::DeviceNotFound)?;
        let device = Monome::from_device(&device, "/prefix").map_err(Error::FromDevice)?;
        let selected_sample_indexes = [0; CHANNEL_COUNT];
        let (sender, receiver) = channel::<GridMessage>();

        Ok((
            Grid {
                receiver,
                device,
                selected_sample_indexes,
                selected_channel_index: 0,
            },
            sender,
        ))
    }

    pub fn start(mut self, sender: Sender<ControlMessage>) {
        self.redraw();

        thread::spawn(move || loop {
            if let Ok(message) = self.receiver.try_recv() {
                match message {
                    GridMessage::Clear => self.clear(),
                }
            }

            if let Some(MonomeEvent::GridKey {
                x,
                y,
                direction: KeyDirection::Down,
            }) = self.device.poll()
            {
                if let Some(control_message) = self.match_action((x as usize, y as usize)) {
                    sender
                        .send(control_message)
                        .expect("Grid control message sent");
                }
                self.redraw();
            }
        });
    }

    pub fn clear(&mut self) {
        self.device.map(0, 0, &[0; 64]);
        self.device.map(8, 0, &[0; 64]);
    }

    pub fn redraw(&mut self) {
        let channel_offset = 56;
        let mut left_mask = [0; 64];

        self.map_sample_selector()
            .into_iter()
            .enumerate()
            .for_each(|(index, value)| left_mask[index] = value);
        self.map_channel_strip()
            .into_iter()
            .enumerate()
            .for_each(|(index, value)| left_mask[index + channel_offset] = value);
        self.device.map(0, 0, &left_mask);
    }

    fn map_channel_strip(&mut self) -> [u8; 8] {
        let mut grid_mask = [0; 8];
        (0..CHANNEL_COUNT).for_each(|index| {
            if self.selected_channel_index == index {
                grid_mask[index] = 10;
            } else {
                grid_mask[index] = 5;
            };
        });
        grid_mask
    }

    fn map_sample_selector(&mut self) -> [u8; SAMPLE_GRID] {
        let mut grid_mask = [0; SAMPLE_GRID];
        // let selected_channel = self.selected_channel().expect("Selected channel exists");
        // let sample_count = selected_channel.sample_count;
        let sample_count = 20;
        let selected_sample = self.selected_sample();

        for (index, button_mask) in grid_mask.iter_mut().enumerate() {
            if index >= sample_count {
                break;
            }
            if index == *selected_sample {
                *button_mask = 10;
                continue;
            }
            *button_mask = 5;
        }
        grid_mask
    }

    pub fn selected_sample(&self) -> &usize {
        self.selected_sample_indexes
            .get(self.selected_channel_index)
            .expect("Selected sample index within bounds")
    }

    pub fn set_selected_sample(&mut self, selected_sample: usize) {
        self.selected_sample_indexes[self.selected_channel_index] = selected_sample;
    }

    pub fn match_action(&mut self, coords: (usize, usize)) -> Option<ControlMessage> {
        match coords {
            (x, 7) if x < CHANNEL_COUNT => {
                self.selected_channel_index = x;
                None
            }
            (x, y) if x < SAMPLE_GRID_X && y < SAMPLE_GRID_Y => {
                let sample_file_index = x + SAMPLE_GRID_X * y;
                let sample_count = 20;

                if sample_file_index >= sample_count {
                    return None;
                }
                self.set_selected_sample(sample_file_index);

                Some(ControlMessage::SetChannelSampleFile(
                    self.selected_channel_index,
                    sample_file_index,
                ))
            }
            (x, y) => {
                println!("Key press ignored: {}x{}", x, y);
                None
            }
        }
    }
}
