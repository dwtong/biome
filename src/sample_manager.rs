use std::ffi::OsStr;
use std::fs;
use std::{io, path::PathBuf};

use crate::CHANNEL_COUNT;

const SAMPLE_DIRS: [&str; CHANNEL_COUNT] = [
    "/Users/dan/Sync/audio/collections/field/rain/glass/",
    "/Users/dan/Sync/audio/collections/field/wind/general/",
    "/Users/dan/Sync/audio/collections/field/wood/misc/",
    "/Users/dan/Sync/audio/collections/field/water/bubbles/",
];

#[derive(Debug)]
pub struct SampleDir {
    path: PathBuf,
}

impl SampleDir {
    fn from_path(path: &str) -> Self {
        Self { path: path.into() }
    }

    fn entries(&self) -> Result<Vec<PathBuf>, io::Error> {
        let entries = fs::read_dir(&self.path)?
            .map(|dir| dir.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        let entries: Vec<PathBuf> = entries
            .into_iter()
            .filter(|entry| entry.extension() == Some(OsStr::new("wav")))
            .collect();

        Ok(entries)
    }
}

#[derive(Debug)]
pub struct SampleManager {
    dirs: [SampleDir; CHANNEL_COUNT],
}

impl SampleManager {
    pub fn new() -> Self {
        let dirs = SAMPLE_DIRS.map(SampleDir::from_path);
        Self { dirs }
    }

    pub fn get_path_for_sample(
        &self,
        channel_index: usize,
        sample_index: usize,
    ) -> Option<PathBuf> {
        let entries = self.dirs.get(channel_index)?.entries().ok()?;
        let path = entries.get(sample_index)?;
        Some(path.clone())
    }
}
