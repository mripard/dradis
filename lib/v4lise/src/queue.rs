use crate::capabilities::Capability;
use crate::device::Device;
use crate::error::Error;
use crate::error::Result;
use crate::formats::PixelFormat;
use crate::lowlevel::v4l2_buf_type;
use crate::lowlevel::v4l2_enum_formats;
use crate::lowlevel::v4l2_enum_framesizes;
use crate::lowlevel::v4l2_fmtdesc;
use crate::lowlevel::v4l2_format;
use crate::lowlevel::v4l2_frmsizeenum;
use crate::lowlevel::v4l2_get_format;
use crate::lowlevel::v4l2_memory;
use crate::lowlevel::v4l2_query_cap;
use crate::lowlevel::v4l2_request_buffers;
use crate::lowlevel::v4l2_requestbuffers;
use crate::lowlevel::v4l2_set_format;
use crate::lowlevel::CapabilitiesFlags;

#[derive(Clone, Copy, Debug)]
pub enum MemoryType {
    MMAP,
}

impl Into<v4l2_memory> for MemoryType {
    fn into(self) -> v4l2_memory {
        match self {
            MemoryType::MMAP => v4l2_memory::V4L2_MEMORY_MMAP,
        }
    }
}

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

pub trait FrameFormat {
    fn set_frame_size(self, width: usize, height: usize) -> Self
    where
        Self: Sized;
    fn set_pixel_format(self, fmt: PixelFormat) -> Self
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct SinglePlanarCaptureFrameFormat<'a> {
    queue: &'a Queue<'a>,
    width: usize,
    height: usize,
    pixel_format: PixelFormat,
}

impl SinglePlanarCaptureFrameFormat<'_> {
    pub fn set_frame_size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn set_pixel_format(mut self, fmt: PixelFormat) -> Self {
        self.pixel_format = fmt;
        self
    }
}

#[derive(Debug)]
pub enum QueueFrameFormat<'a> {
    SinglePlanarCapture(SinglePlanarCaptureFrameFormat<'a>),
}

impl FrameFormat for QueueFrameFormat<'_> {
    fn set_frame_size(self, width: usize, height: usize) -> Self {
        match self {
            QueueFrameFormat::SinglePlanarCapture(fmt) => {
                let frm = fmt.set_frame_size(width, height);

                QueueFrameFormat::SinglePlanarCapture(frm)
            }
        }
    }

    fn set_pixel_format(self, pixfmt: PixelFormat) -> Self {
        match self {
            QueueFrameFormat::SinglePlanarCapture(fmt) => {
                let frm = fmt.set_pixel_format(pixfmt);

                QueueFrameFormat::SinglePlanarCapture(frm)
            }
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

    pub fn get_current_format(&self) -> Result<QueueFrameFormat<'_>> {
        let buf_type: v4l2_buf_type = self.queue_type.into();

        let mut raw_fmt: v4l2_format = Default::default();
        raw_fmt.type_ = buf_type as u32;

        let raw_fmt = v4l2_get_format(self.dev, raw_fmt)?;

        let buf_type = unsafe { std::mem::transmute(raw_fmt.type_) };
        match buf_type {
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE => {
                let fmt = unsafe { &raw_fmt.fmt.pix };

                let width = fmt.width as usize;
                let height = fmt.height as usize;
                let pixfmt: PixelFormat = unsafe { std::mem::transmute(fmt.pixelformat as u32) };

                Ok(QueueFrameFormat::SinglePlanarCapture(
                    SinglePlanarCaptureFrameFormat {
                        queue: self,
                        width,
                        height,
                        pixel_format: pixfmt,
                    },
                ))
            }

            _ => Err(Error::NotSupported),
        }
    }

    pub fn get_sizes(&self, fmt: PixelFormat) -> QueueSizeIter<'_> {
        QueueSizeIter {
            queue: self,
            curr: 0,
            fmt,
        }
    }

    pub fn request_buffers(self, mem_type: MemoryType, num: usize) -> Result<()> {
        let buf_type: v4l2_buf_type = self.queue_type.into();
        let mem_type: v4l2_memory = mem_type.into();
        let mut rbuf: v4l2_requestbuffers = Default::default();
        rbuf.count = num as u32;
        rbuf.type_ = buf_type as u32;
        rbuf.memory = mem_type as u32;

        v4l2_request_buffers(self.dev, rbuf)?;

        Ok(())
    }

    pub fn set_format(&self, fmt: QueueFrameFormat<'_>) -> Result<()> {
        let buf_type: v4l2_buf_type = self.queue_type.into();
        let mut raw_fmt: v4l2_format = Default::default();

        raw_fmt.type_ = buf_type as u32;
        match fmt {
            QueueFrameFormat::SinglePlanarCapture(inner) => {
                raw_fmt.fmt.pix.width = inner.width as u32;
                raw_fmt.fmt.pix.height = inner.height as u32;
                raw_fmt.fmt.pix.pixelformat = inner.pixel_format as u32;
            }
        }

        v4l2_set_format(self.dev, raw_fmt)?;

        Ok(())
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
