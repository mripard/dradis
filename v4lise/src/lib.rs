#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

#[macro_use]
extern crate bitflags;

mod capabilities;
mod device;
mod queue;

pub use v4l2_raw::{
    format::v4l2_pix_fmt,
    raw::{
        V4L2_EVENT_CTRL, V4L2_EVENT_EOS, V4L2_EVENT_FRAME_SYNC, V4L2_EVENT_MOTION_DET,
        V4L2_EVENT_PRIVATE_START, V4L2_EVENT_SOURCE_CHANGE, V4L2_EVENT_VSYNC, v4l2_buf_type,
        v4l2_buffer, v4l2_capability, v4l2_dv_timings, v4l2_edid, v4l2_event,
        v4l2_event_frame_sync, v4l2_event_src_change, v4l2_event_subscription, v4l2_fmtdesc,
        v4l2_format, v4l2_format__bindgen_ty_1, v4l2_frmsizeenum, v4l2_memory, v4l2_pix_format,
        v4l2_requestbuffers,
    },
};

pub use crate::{device::Device, queue::Queue};
