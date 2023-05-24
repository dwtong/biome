use std::println;

use monome::{Monome, MonomeDeviceType, MonomeEvent};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to find monome grid device")]
    DeviceNotFound,
    //     #[error("failed to connect to midi input device")]
    // ConnectInput(#[from] ConnectError<MidiInput>),
}

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

    pub fn poll(&mut self) -> Option<MonomeEvent> {
        println!("poll");
        self.device.poll()
    }

    pub fn unlit(&mut self) {
        let mut grid = [false; 128];
        for i in 0..128 {
            grid[i] = (i + 1) % 2 == 0;
        }
        self.device.set_all(&grid);
    }

    pub fn lit(&mut self) {
        let mut grid = [true; 128];
        for i in 0..128 {
            grid[i] = (i + 1) % 2 == 0;
        }
        self.device.set_all(&grid);
    }
}
