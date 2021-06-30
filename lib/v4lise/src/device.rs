use crate::error::Result;
use crate::queue::Queue;
use crate::queue::QueueType;
use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::os::unix::prelude::OpenOptionsExt;

#[derive(Debug)]
pub struct Device {
    file: File,
}

impl Device {
    pub fn new(path: &str, blocking: bool) -> Result<Self> {
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
