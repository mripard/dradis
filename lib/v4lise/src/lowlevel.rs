#![allow(dead_code)]
#![allow(missing_debug_implementations)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::os::unix::io::AsRawFd;

use nix::{ioctl_read, ioctl_readwrite, ioctl_write_ptr};

use crate::error::Result;

const V4L2_IOCTL_BASE: u32 = 'V' as u32;

ioctl_read!(v4l2_ioctl_querycap, V4L2_IOCTL_BASE, 00, v4l2_capability);

ioctl_readwrite!(v4l2_ioctl_enum_fmt, V4L2_IOCTL_BASE, 02, v4l2_fmtdesc);

ioctl_readwrite!(v4l2_ioctl_g_fmt, V4L2_IOCTL_BASE, 04, v4l2_format);

ioctl_readwrite!(v4l2_ioctl_s_fmt, V4L2_IOCTL_BASE, 05, v4l2_format);

ioctl_readwrite!(v4l2_ioctl_reqbufs, V4L2_IOCTL_BASE, 08, v4l2_requestbuffers);

ioctl_readwrite!(v4l2_ioctl_querybuf, V4L2_IOCTL_BASE, 09, v4l2_buffer);

ioctl_readwrite!(v4l2_ioctl_qbuf, V4L2_IOCTL_BASE, 15, v4l2_buffer);

ioctl_readwrite!(v4l2_ioctl_dqbuf, V4L2_IOCTL_BASE, 17, v4l2_buffer);

ioctl_write_ptr!(v4l2_ioctl_streamon, V4L2_IOCTL_BASE, 18, libc::c_int);
ioctl_write_ptr!(v4l2_ioctl_streamoff, V4L2_IOCTL_BASE, 19, libc::c_int);

ioctl_readwrite!(v4l2_ioctl_s_edid, V4L2_IOCTL_BASE, 41, v4l2_edid);

ioctl_readwrite!(
    v4l2_ioctl_enum_framesizes,
    V4L2_IOCTL_BASE,
    74,
    v4l2_frmsizeenum
);

ioctl_readwrite!(
    v4l2_ioctl_s_dv_timings,
    V4L2_IOCTL_BASE,
    87,
    v4l2_dv_timings
);

ioctl_readwrite!(
    v4l2_ioctl_g_dv_timings,
    V4L2_IOCTL_BASE,
    88,
    v4l2_dv_timings
);

ioctl_read!(
    v4l2_ioctl_query_dv_timings,
    V4L2_IOCTL_BASE,
    99,
    v4l2_dv_timings
);

impl std::fmt::Debug for v4l2_dv_timings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.type_ {
            V4L2_DV_BT_656_1120 => unsafe {
                let bt = &self.__bindgen_anon_1.bt;
                let width = bt.width;
                let height = bt.height;

                f.debug_struct("v4l2_dv_timings: v4l2_bt_timings")
                    .field("width", &width)
                    .field("height", &height)
                    .finish()
            },
            _ => Err(std::fmt::Error),
        }
    }
}

bitflags! {
    pub struct BufferFlags: u32 {
       const BUF_FLAG_MAPPED = 0x00000001;
       const BUF_FLAG_QUEUED = 0x00000002;
       const BUF_FLAG_DONE = 0x00000004;
       const BUF_FLAG_KEYFRAME = 0x00000008;
       const BUF_FLAG_PFRAME = 0x00000010;
       const BUF_FLAG_BFRAME = 0x00000020;
       const BUF_FLAG_ERROR = 0x00000040;
       const BUF_FLAG_IN_REQUEST = 0x00000080;
       const BUF_FLAG_TIMECODE = 0x00000100;
       const BUF_FLAG_M2M_CAPTURE_BUF = 0x00000200;
       const BUF_FLAG_NO_CACHE_INVALIDATE = 0x00000800;
       const BUF_FLAG_NO_CACHE_CLEAN = 0x00001000;
       const BUF_FLAG_TIMESTAMP_MONOTONIC = 0x00002000;
       const BUF_FLAG_TIMESTAMP_COPY = 0x00004000;
       const BUF_FLAG_TSTAMP_SRC_SOE = 0x00010000;
       const BUF_FLAG_LAST = 0x00100000;
    }
}

bitflags! {
    pub struct CapabilitiesFlags: u32 {
    const VIDEO_CAPTURE = 0x00000001;
    const VIDEO_OUTPUT = 0x00000002;
    const VIDEO_OVERLAY = 0x00000004;
    const VBI_CAPTURE = 0x00000010;
    const VBI_OUTPUT = 0x00000020;
    const SLICED_VBI_CAPTURE = 0x00000040;
    const SLICED_VBI_OUTPUT = 0x00000080;
    const RDS_CAPTURE = 0x00000100;
    const VIDEO_OUTPUT_OVERLAY = 0x00000200;
    const HW_FREQ_SEEK = 0x00000400;
    const RDS_OUTPUT = 0x00000800;
    const VIDEO_CAPTURE_MPLANE = 0x00001000;
    const VIDEO_OUTPUT_MPLANE = 0x00002000;
    const VIDEO_M2M_MPLANE = 0x00004000;
    const VIDEO_M2M = 0x00008000;
    const TUNER = 0x00010000;
    const AUDIO = 0x00020000;
    const RADIO = 0x00040000;
    const MODULATOR = 0x00080000;
    const SDR_CAPTURE = 0x00100000;
    const EXT_PIX_FORMAT = 0x00200000;
    const SDR_OUTPUT = 0x00400000;
    const META_CAPTURE = 0x00800000;
    const READWRITE = 0x01000000;
    const ASYNCIO = 0x02000000;
    const STREAMING = 0x04000000;
    const META_OUTPUT = 0x08000000;
    const TOUCH = 0x10000000;
    const DEVICE_CAPS = 0x80000000;
    }
}

pub fn v4l2_dequeue_buffer(file: &impl AsRawFd, mut buf: v4l2_buffer) -> Result<v4l2_buffer> {
    let _ = unsafe { v4l2_ioctl_dqbuf(file.as_raw_fd(), &mut buf) }?;

    Ok(buf)
}

pub fn v4l2_enum_formats(file: &impl AsRawFd, mut desc: v4l2_fmtdesc) -> Result<v4l2_fmtdesc> {
    let _ = unsafe { v4l2_ioctl_enum_fmt(file.as_raw_fd(), &mut desc) }?;

    Ok(desc)
}

pub fn v4l2_enum_framesizes(
    file: &impl AsRawFd,
    mut desc: v4l2_frmsizeenum,
) -> Result<v4l2_frmsizeenum> {
    let _ = unsafe { v4l2_ioctl_enum_framesizes(file.as_raw_fd(), &mut desc) }?;

    Ok(desc)
}

pub fn v4l2_query_buffer(file: &impl AsRawFd, mut buf: v4l2_buffer) -> Result<v4l2_buffer> {
    let _ = unsafe { v4l2_ioctl_querybuf(file.as_raw_fd(), &mut buf) }?;

    Ok(buf)
}

pub fn v4l2_query_dv_timings(file: &impl AsRawFd) -> Result<v4l2_dv_timings> {
    let mut timings = v4l2_dv_timings::default();

    let _ = unsafe { v4l2_ioctl_query_dv_timings(file.as_raw_fd(), &mut timings) }?;

    Ok(timings)
}

pub fn v4l2_get_dv_timings(file: &impl AsRawFd) -> Result<v4l2_dv_timings> {
    let mut timings = v4l2_dv_timings::default();

    let _ = unsafe { v4l2_ioctl_g_dv_timings(file.as_raw_fd(), &mut timings) }?;

    Ok(timings)
}

pub fn v4l2_set_dv_timings(
    file: &impl AsRawFd,
    mut timings: v4l2_dv_timings,
) -> Result<v4l2_dv_timings> {
    let _ = unsafe { v4l2_ioctl_s_dv_timings(file.as_raw_fd(), &mut timings) }?;

    Ok(timings)
}

pub fn v4l2_query_cap(file: &impl AsRawFd) -> Result<v4l2_capability> {
    let mut caps: v4l2_capability = Default::default();

    let _ = unsafe { v4l2_ioctl_querycap(file.as_raw_fd(), &mut caps) }?;

    Ok(caps)
}

pub fn v4l2_queue_buffer(file: &impl AsRawFd, mut buf: v4l2_buffer) -> Result<()> {
    let _ = unsafe { v4l2_ioctl_qbuf(file.as_raw_fd(), &mut buf) }?;

    Ok(())
}

pub fn v4l2_request_buffers(
    file: &impl AsRawFd,
    mut rbuf: v4l2_requestbuffers,
) -> Result<v4l2_requestbuffers> {
    let _ = unsafe { v4l2_ioctl_reqbufs(file.as_raw_fd(), &mut rbuf) }?;

    Ok(rbuf)
}

pub fn v4l2_get_format(file: &impl AsRawFd, mut fmt: v4l2_format) -> Result<v4l2_format> {
    let _ = unsafe { v4l2_ioctl_g_fmt(file.as_raw_fd(), &mut fmt) }?;

    Ok(fmt)
}

pub fn v4l2_set_format(file: &impl AsRawFd, mut fmt: v4l2_format) -> Result<v4l2_format> {
    let _ = unsafe { v4l2_ioctl_s_fmt(file.as_raw_fd(), &mut fmt) }?;

    Ok(fmt)
}

pub fn v4l2_start_streaming(file: &impl AsRawFd, buf_type: v4l2_buf_type) -> Result<()> {
    let arg = buf_type as libc::c_int;

    let _ = unsafe { v4l2_ioctl_streamon(file.as_raw_fd(), &arg as *const i32) }?;

    Ok(())
}

pub fn v4l2_stop_streaming(file: &impl AsRawFd, buf_type: v4l2_buf_type) -> Result<()> {
    let arg = buf_type as libc::c_int;

    let _ = unsafe { v4l2_ioctl_streamoff(file.as_raw_fd(), &arg as *const i32) }?;

    Ok(())
}

pub fn v4l2_set_edid(file: &impl AsRawFd, edid: &mut [u8]) -> Result<()> {
    let mut arg = v4l2_edid::default();
    arg.blocks = (edid.len() / 128) as u32;
    arg.edid = edid.as_mut_ptr();

    let _ = unsafe { v4l2_ioctl_s_edid(file.as_raw_fd(), &mut arg) }?;

    Ok(())
}
