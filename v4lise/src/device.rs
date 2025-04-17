use std::{
    fs::{File, OpenOptions},
    os::{
        fd::{AsFd, BorrowedFd},
        unix::{io::AsRawFd, prelude::OpenOptionsExt},
    },
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
    pub fn new(path: &Path, non_blocking: bool) -> Result<Self> {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);

        if non_blocking {
            options.custom_flags(libc::O_NONBLOCK);
        }

        let file = options.open(path)?;

        Ok(Device { file })
    }

    pub fn get_queue(&self, queue_type: QueueType) -> Result<Queue<'_>> {
        Queue::new(self, queue_type)
    }
}

impl AsFd for Device {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> i32 {
        self.file.as_raw_fd()
    }
}
