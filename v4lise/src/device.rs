use std::{
    fs::{File, OpenOptions},
    os::unix::{io::AsRawFd, prelude::OpenOptionsExt},
    path::Path,
};

use crate::{
    error::Result,
    queue::{Queue, QueueType},
};

#[derive(Debug)]
pub struct Device {
    file: File,
}

impl Device {
    pub fn new(path: &Path, blocking: bool) -> Result<Self> {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);

        if blocking {
            options.custom_flags(libc::O_NONBLOCK);
        }

        let file = options.open(path)?;

        Ok(Device { file })
    }

    pub fn get_queue(&self, queue_type: QueueType) -> Result<Queue<'_>> {
        Queue::new(self, queue_type)
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> i32 {
        self.file.as_raw_fd()
    }
}
