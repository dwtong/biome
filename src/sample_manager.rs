use std::ffi::OsStr;
use std::fs;
use std::{io, path::PathBuf};

use crate::settings::Settings;

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
    dirs: Vec<SampleDir>,
}

impl SampleManager {
    pub fn new(settings: &Settings) -> Self {
        let dirs = settings
            .sample_dirs()
            .iter()
            .map(|path| SampleDir::from_path(path))
            .collect();
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
