use crate::lowlevel::v4l2_frmsizeenum;
use crate::lowlevel::v4l2_enum_framesizes;
use crate::capabilities::Capability;
use crate::device::Device;
use crate::error::Error;
use crate::error::Result;
use crate::formats::PixelFormat;
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
        let raw_caps = v4l2_query_cap(dev)?;
        let caps = Capability::from(raw_caps);

        if !caps.device_caps.contains(queue_type.into()) {
            return Err(Error::Invalid);
        }

        Ok(Queue { dev, queue_type })
    }

    pub fn get_pixel_formats(&self) -> QueuePixelFormatIter<'_> {
        QueuePixelFormatIter {
            queue: self,
            curr: 0,
        }
    }

    pub fn get_sizes(&self, fmt: PixelFormat) -> QueueSizeIter<'_> {
        QueueSizeIter {
            queue: self,
            curr: 0,
            fmt,
        }
    }
}

#[derive(Debug)]
pub struct QueuePixelFormatIter<'a> {
    queue: &'a Queue<'a>,
    curr: usize,
}

impl Iterator for QueuePixelFormatIter<'_> {
    type Item = PixelFormat;

    fn next(&mut self) -> Option<PixelFormat> {
        let buf_type: v4l2_buf_type = self.queue.queue_type.into();

        let mut raw_desc: v4l2_fmtdesc = Default::default();
        raw_desc.type_ = buf_type as u32;
        raw_desc.index = self.curr as u32;
        let fmt = match v4l2_enum_formats(self.queue.dev, raw_desc) {
            Ok(ret) => {
                let cvt: PixelFormat = unsafe { std::mem::transmute(ret.pixelformat as u32) };
                cvt
            }

            Err(_) => return None,
        };

        self.curr = self.curr + 1;
        Some(fmt)
    }
}

#[derive(Debug)]
pub struct QueueSizeIter<'a> {
    queue: &'a Queue<'a>,
    fmt: PixelFormat,
    curr: usize,
}

impl Iterator for QueueSizeIter<'_> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        let mut raw_struct: v4l2_frmsizeenum = Default::default();
		raw_struct.pixel_format = self.fmt as u32;
		raw_struct.index = self.curr as u32;

		let size = match v4l2_enum_framesizes(self.queue.dev, raw_struct) {
			Ok(ret) => {
                let height = unsafe { ret.__bindgen_anon_1.discrete.height } as usize;
                let width = unsafe { ret.__bindgen_anon_1.discrete.width } as usize;

                (width, height)
			}
			Err(_) => return None,
		};

        self.curr = self.curr + 1;
        Some(size)
    }
}