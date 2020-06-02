use crate::error::Result;
use crate::queue::Queue;
use crate::queue::QueueType;
use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

#[derive(Debug)]
pub struct Device {
    file: File,
}

impl Device {
    pub fn new(path: &str) -> Result<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;

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
