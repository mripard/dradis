use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

use crate::error::Result;

#[derive(Debug)]
pub struct Device {
    file: File,
}

impl Device {
    pub fn new(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;

        Ok(Device {
            file,
        })
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> i32 {
        self.file.as_raw_fd()
    }
}