#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

#[macro_use]
extern crate bitflags;

mod capabilities;
mod device;
mod error;
mod formats;
mod lowlevel;
mod queue;

pub use crate::device::Device;
pub use crate::error::Error;
pub use crate::error::Result;
pub use crate::formats::PixelFormat;
pub use crate::lowlevel::v4l2_buf_type;
pub use crate::lowlevel::v4l2_buffer;
pub use crate::lowlevel::v4l2_capability;
pub use crate::lowlevel::v4l2_dequeue_buffer;
pub use crate::lowlevel::v4l2_enum_formats;
pub use crate::lowlevel::v4l2_enum_framesizes;
pub use crate::lowlevel::v4l2_fmtdesc;
pub use crate::lowlevel::v4l2_format;
pub use crate::lowlevel::v4l2_frmsizeenum;
pub use crate::lowlevel::v4l2_get_dv_timings;
pub use crate::lowlevel::v4l2_memory;
pub use crate::lowlevel::v4l2_query_buffer;
pub use crate::lowlevel::v4l2_query_cap;
pub use crate::lowlevel::v4l2_query_dv_timings;
pub use crate::lowlevel::v4l2_queue_buffer;
pub use crate::lowlevel::v4l2_request_buffers;
pub use crate::lowlevel::v4l2_requestbuffers;
pub use crate::lowlevel::v4l2_set_dv_timings;
pub use crate::lowlevel::v4l2_set_edid;
pub use crate::lowlevel::v4l2_set_format;
pub use crate::lowlevel::v4l2_start_streaming;
pub use crate::lowlevel::BufferFlags;
pub use crate::lowlevel::CapabilitiesFlags;
pub use crate::queue::FrameFormat;
pub use crate::queue::MemoryType;
pub use crate::queue::Queue;
pub use crate::queue::QueueType;
