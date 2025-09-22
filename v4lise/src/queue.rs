use std::{io, os::fd::AsFd as _};

use v4l2_raw::{
    format::v4l2_pix_fmt,
    raw::{
        v4l2_ioctl_enum_fmt, v4l2_ioctl_enum_framesizes, v4l2_ioctl_querycap, v4l2_ioctl_reqbufs,
    },
    wrapper::{v4l2_format, v4l2_ioctl_g_fmt},
};

use crate::{
    capabilities::{CapabilitiesFlags, Capability},
    device::Device,
    v4l2_buf_type, v4l2_fmtdesc, v4l2_frmsizeenum, v4l2_memory, v4l2_requestbuffers,
};

#[derive(Clone, Copy, Debug)]
pub enum QueueType {
    Capture,
    Output,
}

impl From<QueueType> for CapabilitiesFlags {
    fn from(val: QueueType) -> Self {
        match val {
            QueueType::Capture => CapabilitiesFlags::VIDEO_CAPTURE,
            QueueType::Output => CapabilitiesFlags::VIDEO_OUTPUT,
        }
    }
}

impl From<QueueType> for v4l2_buf_type {
    fn from(val: QueueType) -> Self {
        match val {
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
    pub fn new(dev: &'a Device, queue_type: QueueType) -> io::Result<Self> {
        let raw_caps = v4l2_ioctl_querycap(dev.as_fd())?;
        let caps = Capability::from(raw_caps);

        if !caps.device_caps.contains(queue_type.into()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Device doesn't support requested capability",
            ));
        }

        Ok(Queue { dev, queue_type })
    }

    pub fn get_pixel_formats(&self) -> QueuePixelFormatIter<'_> {
        QueuePixelFormatIter {
            queue: self,
            curr: 0,
        }
    }

    pub fn get_current_format(&self) -> io::Result<v4l2_format> {
        v4l2_ioctl_g_fmt(self.dev.as_fd(), self.queue_type.into())
    }

    pub fn get_sizes(&self, fmt: v4l2_pix_fmt) -> QueueSizeIter<'_> {
        QueueSizeIter {
            queue: self,
            curr: 0,
            fmt,
        }
    }

    pub fn request_buffers(&self, mem_type: v4l2_memory, num: usize) -> io::Result<()> {
        let buf_type: v4l2_buf_type = self.queue_type.into();
        let rbuf = v4l2_requestbuffers {
            count: num as u32,
            type_: buf_type.into(),
            memory: mem_type.into(),
            ..Default::default()
        };

        v4l2_ioctl_reqbufs(self.dev.as_fd(), rbuf)?;

        Ok(())
    }

    pub fn set_format(&self, fmt: v4l2_format) -> io::Result<v4l2_format> {
        v4l2_raw::wrapper::v4l2_ioctl_s_fmt(self.dev.as_fd(), fmt)
    }
}

#[derive(Debug)]
pub struct QueuePixelFormatIter<'a> {
    queue: &'a Queue<'a>,
    curr: usize,
}

impl Iterator for QueuePixelFormatIter<'_> {
    type Item = v4l2_pix_fmt;

    fn next(&mut self) -> Option<v4l2_pix_fmt> {
        let buf_type: v4l2_buf_type = self.queue.queue_type.into();

        let raw_desc = v4l2_fmtdesc {
            type_: buf_type.into(),
            index: self.curr as u32,
            ..Default::default()
        };

        let fmt = match v4l2_ioctl_enum_fmt(self.queue.dev.as_fd(), raw_desc) {
            Ok(ret) => ret.pixelformat.try_into().ok()?,
            Err(_) => return None,
        };

        self.curr += 1;
        Some(fmt)
    }
}

#[derive(Debug)]
pub struct QueueSizeIter<'a> {
    queue: &'a Queue<'a>,
    fmt: v4l2_pix_fmt,
    curr: usize,
}

impl Iterator for QueueSizeIter<'_> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        let raw_struct = v4l2_frmsizeenum {
            pixel_format: self.fmt.into(),
            index: self.curr as u32,
            ..Default::default()
        };

        let size = match v4l2_ioctl_enum_framesizes(self.queue.dev.as_fd(), raw_struct) {
            Ok(ret) => {
                let height = unsafe { ret.__bindgen_anon_1.discrete.height } as usize;
                let width = unsafe { ret.__bindgen_anon_1.discrete.width } as usize;

                (width, height)
            }
            Err(_) => return None,
        };

        self.curr += 1;
        Some(size)
    }
}
