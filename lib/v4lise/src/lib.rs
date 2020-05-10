#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate vmm_sys_util;

mod error;
mod formats;
mod lowlevel;

pub use crate::error::Result;
pub use crate::formats::Format;
pub use crate::lowlevel::v4l2_buf_type;
pub use crate::lowlevel::v4l2_buffer;
pub use crate::lowlevel::v4l2_capability;
pub use crate::lowlevel::v4l2_dequeue_buffer;
pub use crate::lowlevel::v4l2_enum_formats;
pub use crate::lowlevel::v4l2_enum_framesizes;
pub use crate::lowlevel::v4l2_fmtdesc;
pub use crate::lowlevel::v4l2_format;
pub use crate::lowlevel::v4l2_frmsizeenum;
pub use crate::lowlevel::v4l2_memory;
pub use crate::lowlevel::v4l2_query_buffer;
pub use crate::lowlevel::v4l2_query_cap;
pub use crate::lowlevel::v4l2_queue_buffer;
pub use crate::lowlevel::v4l2_request_buffers;
pub use crate::lowlevel::v4l2_requestbuffers;
pub use crate::lowlevel::v4l2_set_format;
pub use crate::lowlevel::v4l2_start_streaming;
pub use crate::lowlevel::BufferFlags;
pub use crate::lowlevel::CapabilitiesFlags;