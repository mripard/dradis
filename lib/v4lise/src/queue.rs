use crate::capabilities::Capability;
use crate::device::Device;
use crate::error::Error;
use crate::error::Result;
use crate::formats::Format;
use crate::lowlevel::v4l2_buf_type;
use crate::lowlevel::v4l2_enum_formats;
use crate::lowlevel::v4l2_fmtdesc;
use crate::lowlevel::v4l2_query_cap;
use crate::lowlevel::CapabilitiesFlags;

#[derive(Clone, Copy, Debug)]
pub enum QueueType {
    Capture,
    Output,
}

impl Into<CapabilitiesFlags> for QueueType {
    fn into(self) -> CapabilitiesFlags {
        match self {
            QueueType::Capture => CapabilitiesFlags::VIDEO_CAPTURE,
            QueueType::Output => CapabilitiesFlags::VIDEO_OUTPUT,
        }
    }
}

impl Into<v4l2_buf_type> for QueueType {
    fn into(self) -> v4l2_buf_type {
        match self {
            QueueType::Capture => v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE,
            QueueType::Output => v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT,
        }
    }
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

        if !caps.device_caps.contains(queue_type.into()) {
            return Err(Error::Invalid);
        }

        Ok(Queue { dev, queue_type })
    }

    pub fn get_formats(&self) -> QueueFormatIter<'_> {
        QueueFormatIter {
            queue: self,
            curr: 0,
        }
    }
}

#[derive(Debug)]
pub struct QueueFormatIter<'a> {
    queue: &'a Queue<'a>,
    curr: usize,
}

impl Iterator for QueueFormatIter<'_> {
    type Item = Format;

    fn next(&mut self) -> Option<Format> {
        let buf_type: v4l2_buf_type = self.queue.queue_type.into();

        let mut raw_desc: v4l2_fmtdesc = Default::default();
        raw_desc.type_ = buf_type as u32;
        raw_desc.index = self.curr as u32;
        let fmt = match v4l2_enum_formats(self.queue.dev, raw_desc) {
            Ok(ret) => {
                let cvt: Format = unsafe { std::mem::transmute(ret.pixelformat as u32) };
                cvt
            }

            Err(_) => return None,
        };

        self.curr = self.curr + 1;
        Some(fmt)
    }
}
