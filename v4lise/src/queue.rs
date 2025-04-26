use crate::{
    capabilities::Capability,
    device::Device,
    error::{Error, Result},
    formats::PixelFormat,
    lowlevel::{
        CapabilitiesFlags, v4l2_buf_type, v4l2_enum_formats, v4l2_enum_framesizes, v4l2_fmtdesc,
        v4l2_format, v4l2_format__bindgen_ty_1, v4l2_frmsizeenum, v4l2_get_format, v4l2_memory,
        v4l2_pix_format, v4l2_query_cap, v4l2_request_buffers, v4l2_requestbuffers,
        v4l2_set_format,
    },
};

#[derive(Clone, Copy, Debug)]
pub enum MemoryType {
    MMAP,
    DMABUF,
}

impl From<MemoryType> for v4l2_memory {
    fn from(val: MemoryType) -> Self {
        match val {
            MemoryType::DMABUF => v4l2_memory::V4L2_MEMORY_DMABUF,
            MemoryType::MMAP => v4l2_memory::V4L2_MEMORY_MMAP,
        }
    }
}

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

pub trait FrameFormat {
    fn set_frame_size(self, width: usize, height: usize) -> Self
    where
        Self: Sized;
    fn set_pixel_format(self, fmt: PixelFormat) -> Self
    where
        Self: Sized;
}

#[expect(dead_code)]
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

        let raw_fmt = v4l2_format {
            type_: buf_type as u32,
            ..Default::default()
        };

        let raw_fmt = v4l2_get_format(self.dev, raw_fmt)?;

        let buf_type = unsafe { std::mem::transmute::<u32, v4l2_buf_type>(raw_fmt.type_) };
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

    pub fn request_buffers(&self, mem_type: MemoryType, num: usize) -> Result<()> {
        let buf_type: v4l2_buf_type = self.queue_type.into();
        let mem_type: v4l2_memory = mem_type.into();
        let rbuf = v4l2_requestbuffers {
            count: num as u32,
            type_: buf_type as u32,
            memory: mem_type as u32,
            ..Default::default()
        };

        v4l2_request_buffers(self.dev, rbuf)?;

        Ok(())
    }

    pub fn set_format(&self, fmt: QueueFrameFormat<'_>) -> Result<()> {
        let buf_type: v4l2_buf_type = self.queue_type.into();
        let raw_fmt = v4l2_format {
            type_: buf_type as u32,
            fmt: match fmt {
                QueueFrameFormat::SinglePlanarCapture(inner) => v4l2_format__bindgen_ty_1 {
                    pix: v4l2_pix_format {
                        width: inner.width as u32,
                        height: inner.height as u32,
                        pixelformat: inner.pixel_format as u32,
                        ..Default::default()
                    },
                },
            },
        };

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

        let raw_desc = v4l2_fmtdesc {
            type_: buf_type as u32,
            index: self.curr as u32,
            ..Default::default()
        };

        let fmt = match v4l2_enum_formats(self.queue.dev, raw_desc) {
            Ok(ret) => {
                let cvt: PixelFormat = unsafe { std::mem::transmute(ret.pixelformat) };
                cvt
            }

            Err(_) => return None,
        };

        self.curr += 1;
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
        let raw_struct = v4l2_frmsizeenum {
            pixel_format: self.fmt as u32,
            index: self.curr as u32,
            ..Default::default()
        };

        let size = match v4l2_enum_framesizes(self.queue.dev, raw_struct) {
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
