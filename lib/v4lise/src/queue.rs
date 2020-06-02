use crate::capabilities::Capability;
use crate::device::Device;
use crate::error::Error;
use crate::error::Result;
use crate::lowlevel::v4l2_query_cap;
use crate::lowlevel::CapabilitiesFlags;

#[derive(Debug)]
pub enum QueueType {
    Capture,
    Output,
}

#[derive(Debug)]
pub struct Queue<'a> {
    dev: &'a Device,
    queue_type: QueueType,
}

impl<'a> Queue<'a> {
    pub fn new(dev: &'a Device, queue_type: QueueType) -> Result<Self> {
        let raw_caps = v4l2_query_cap(dev).unwrap();
        let caps = Capability::from(raw_caps);
        
        let flag = match queue_type {
            QueueType::Capture => CapabilitiesFlags::VIDEO_CAPTURE,
            QueueType::Output => CapabilitiesFlags::VIDEO_OUTPUT,
        };

        if !caps.device_caps.contains(flag) {
            return Err(Error::Invalid);
        }

        Ok(Queue {
            dev,
            queue_type,
        })
    }
}