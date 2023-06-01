use std::fs;
use std::{io, path::PathBuf};

use crate::CHANNEL_COUNT;

const SAMPLE_DIRS: [&str; CHANNEL_COUNT] = [
    "/Users/dan/dev/dwtong/web-audio-playground/samples/",
    "/Users/dan/dev/dwtong/web-audio-playground/samples/",
    "/Users/dan/dev/dwtong/web-audio-playground/samples/",
    "/Users/dan/dev/dwtong/web-audio-playground/samples/",
];

#[derive(Debug)]
pub struct SampleDir(Vec<PathBuf>);

impl SampleDir {
    fn from_path(path: &str) -> Result<Self, io::Error> {
        let entries = fs::read_dir(path)?
            .map(|dir| dir.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        Ok(SampleDir(entries))
    }
}

#[derive(Debug)]
pub struct SampleManager {
    dirs: [SampleDir; CHANNEL_COUNT],
}

impl SampleManager {
    pub fn new() -> Self {
        let dirs = SAMPLE_DIRS.map(|path| SampleDir::from_path(path).unwrap());
        Self { dirs }
    }

    pub fn get_path_for_sample(
        &self,
        channel_index: usize,
        sample_index: usize,
    ) -> Option<&PathBuf> {
        self.dirs.get(channel_index)?.0.get(sample_index)
    }
}
