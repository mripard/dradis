use std::{
    io,
    os::{
        fd::{AsFd, BorrowedFd, OwnedFd},
        unix::io::AsRawFd,
    },
    path::Path,
};

use rustix::fs::{Mode, OFlags, open};

use crate::queue::{Queue, QueueType};

#[derive(Debug)]
pub struct Device {
    file: OwnedFd,
}

impl Device {
    pub fn new(path: &Path, non_blocking: bool) -> io::Result<Self> {
        let flags = OFlags::union(
            OFlags::RDWR,
            if non_blocking {
                OFlags::NONBLOCK
            } else {
                OFlags::empty()
            },
        );

        Ok(Device {
            file: open(path, flags, Mode::empty())?,
        })
    }

    pub fn get_queue(&self, queue_type: QueueType) -> io::Result<Queue<'_>> {
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
