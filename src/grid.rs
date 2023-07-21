use crate::{message::ControlMessage, settings::Settings};
use monome::{KeyDirection, Monome, MonomeDeviceType, MonomeEvent};
use std::{
    println,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

const SAMPLE_GRID_X: usize = 8;
const SAMPLE_GRID_Y: usize = 6;
const SAMPLE_GRID: usize = SAMPLE_GRID_X * SAMPLE_GRID_Y;

#[derive(Debug)]
pub enum GridMessage {
    Clear,
}

pub struct Grid {
    rx: Receiver<GridMessage>,
    device: Option<Monome>,
    selected_sample_indexes: Vec<usize>,
    selected_channel_index: usize,
}

impl Grid {
    pub fn new(settings: &Settings) -> (Self, Sender<GridMessage>) {
        let device = Monome::enumerate_devices()
            .expect("Monome setup successfully")
            .into_iter()
            .find(|d| d.device_type() == MonomeDeviceType::Grid)
            .and_then(|d| Monome::from_device(&d, "/prefix").ok());
        let selected_sample_indexes = vec![0; settings.channel_count()];
        let (tx, rx) = channel::<GridMessage>();
        (
            Grid {
                rx,
                device,
                selected_sample_indexes,
                selected_channel_index: 0,
            },
            tx,
        )
    }

    pub fn start(mut self, control_tx: Sender<ControlMessage>) {
        self.redraw();

        thread::spawn(move || loop {
            let rx = &self.rx;

            if let Ok(GridMessage::Clear) = rx.try_recv() {
                self.clear_device();
            }
            if let Some(MonomeEvent::GridKey {
                x,
                y,
                direction: KeyDirection::Down,
            }) = self.poll_device()
            {
                self.match_action((x as usize, y as usize))
                    .map(|msg| control_tx.send(msg));
                self.redraw();
            }
        });
    }

    pub fn poll_device(&mut self) -> Option<MonomeEvent> {
        match &mut self.device {
            Some(device) => device.poll(),
            None => None,
        }
    }

    pub fn redraw(&mut self) {
        let channel_offset = 56;
        let mut left_mask = [0; 64];
        let right_mask = [0; 64];

        self.map_sample_selector()
            .into_iter()
            .enumerate()
            .for_each(|(index, value)| left_mask[index] = value);
        self.map_channel_strip()
            .into_iter()
            .enumerate()
            .for_each(|(index, value)| left_mask[index + channel_offset] = value);

        self.redraw_device(&left_mask, &right_mask);
    }

    fn redraw_device(&mut self, left_mask: &[u8; 64], right_mask: &[u8; 64]) {
        if let Some(device) = &mut self.device {
            device.map(0, 0, left_mask);
            device.map(8, 0, right_mask);
        }
    }
    fn clear_device(&mut self) {
        let clear_mask = &[0; 64];
        self.redraw_device(clear_mask, clear_mask);
    }

    fn map_channel_strip(&self) -> [u8; 8] {
        let mut grid_mask = [0; 8];
        (0..self.selected_sample_indexes.len()).for_each(|index| {
            if self.selected_channel_index == index {
                grid_mask[index] = 10;
            } else {
                grid_mask[index] = 5;
            };
        });
        grid_mask
    }

    fn map_sample_selector(&self) -> [u8; SAMPLE_GRID] {
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
            (x, 7) if x < self.selected_sample_indexes.len() => {
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
