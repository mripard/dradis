#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

#[macro_use]
extern crate bitflags;

mod capabilities;
mod device;
mod queue;

use std::{io, os::fd::AsFd};

use v4l2_raw::raw::{
    v4l2_ioctl_dqbuf, v4l2_ioctl_dqevent, v4l2_ioctl_g_fmt, v4l2_ioctl_qbuf,
    v4l2_ioctl_query_dv_timings, v4l2_ioctl_querybuf, v4l2_ioctl_reqbufs, v4l2_ioctl_s_dv_timings,
    v4l2_ioctl_s_edid, v4l2_ioctl_s_fmt, v4l2_ioctl_streamoff, v4l2_ioctl_streamon,
    v4l2_ioctl_subscribe_event,
};
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

pub fn v4l2_set_edid(fd: &impl AsFd, edid: &mut [u8]) -> io::Result<()> {
    let arg = v4l2_edid {
        blocks: (edid.len() / 128) as u32,
        edid: edid.as_mut_ptr(),
        ..Default::default()
    };

    v4l2_ioctl_s_edid(fd.as_fd(), arg).map(|_| ())
}

pub fn v4l2_query_dv_timings(fd: &impl AsFd) -> io::Result<v4l2_dv_timings> {
    let timings = v4l2_dv_timings::default();

    v4l2_ioctl_query_dv_timings(fd.as_fd(), timings)
}

pub fn v4l2_set_dv_timings(fd: &impl AsFd, timings: v4l2_dv_timings) -> io::Result<()> {
    v4l2_ioctl_s_dv_timings(fd.as_fd(), timings).map(|_| ())
}

pub fn v4l2_set_format(fd: &impl AsFd, fmt: v4l2_format) -> io::Result<v4l2_format> {
    v4l2_ioctl_s_fmt(fd.as_fd(), fmt)
}

pub fn v4l2_get_format(fd: &impl AsFd, fmt: v4l2_format) -> io::Result<v4l2_format> {
    v4l2_ioctl_g_fmt(fd.as_fd(), fmt)
}

pub fn v4l2_start_streaming(fd: &impl AsFd, buf_type: v4l2_buf_type) -> io::Result<()> {
    v4l2_ioctl_streamon(fd.as_fd(), buf_type.into())
}

pub fn v4l2_stop_streaming(fd: &impl AsFd, buf_type: v4l2_buf_type) -> io::Result<()> {
    v4l2_ioctl_streamoff(fd.as_fd(), buf_type.into())
}

pub fn v4l2_subscribe_event(fd: &impl AsFd, sub: v4l2_event_subscription) -> io::Result<()> {
    v4l2_ioctl_subscribe_event(fd.as_fd(), sub)
}

pub fn v4l2_queue_buffer(fd: &impl AsFd, buf: v4l2_buffer) -> io::Result<()> {
    v4l2_ioctl_qbuf(fd.as_fd(), buf).map(|_| ())
}

pub fn v4l2_dequeue_buffer(fd: &impl AsFd, buf: v4l2_buffer) -> io::Result<v4l2_buffer> {
    v4l2_ioctl_dqbuf(fd.as_fd(), buf)
}

pub fn v4l2_dequeue_event(fd: &impl AsFd) -> io::Result<v4l2_event> {
    v4l2_ioctl_dqevent(fd.as_fd())
}

pub fn v4l2_request_buffers(
    fd: &impl AsFd,
    rbuf: v4l2_requestbuffers,
) -> io::Result<v4l2_requestbuffers> {
    v4l2_ioctl_reqbufs(fd.as_fd(), rbuf)
}

pub fn v4l2_query_buffer(fd: &impl AsFd, buf: v4l2_buffer) -> io::Result<v4l2_buffer> {
    v4l2_ioctl_querybuf(fd.as_fd(), buf)
}
