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

pub use crate::{
    device::Device,
    error::{Error, Result},
    formats::PixelFormat,
    lowlevel::{
        BufferFlags, CapabilitiesFlags, V4L2_EVENT_CTRL, V4L2_EVENT_EOS, V4L2_EVENT_FRAME_SYNC,
        V4L2_EVENT_MOTION_DET, V4L2_EVENT_PRIVATE_START, V4L2_EVENT_SOURCE_CHANGE,
        V4L2_EVENT_VSYNC, v4l2_buf_type, v4l2_buffer, v4l2_capability, v4l2_dequeue_buffer,
        v4l2_dequeue_event, v4l2_enum_formats, v4l2_enum_framesizes, v4l2_event,
        v4l2_event_frame_sync, v4l2_event_src_change, v4l2_event_subscription, v4l2_fmtdesc,
        v4l2_format, v4l2_frmsizeenum, v4l2_get_dv_timings, v4l2_memory, v4l2_query_buffer,
        v4l2_query_cap, v4l2_query_dv_timings, v4l2_queue_buffer, v4l2_request_buffers,
        v4l2_requestbuffers, v4l2_set_dv_timings, v4l2_set_edid, v4l2_set_format,
        v4l2_start_streaming, v4l2_stop_streaming, v4l2_subscribe_event,
    },
    queue::{FrameFormat, MemoryType, Queue, QueueType},
};
