use core::ffi::c_int;
use std::{io, os::fd::BorrowedFd};

use rustix::{
    io::Errno,
    ioctl::{Getter, Opcode, Setter, Updater, ioctl, opcode},
};

pub(crate) mod bindgen {
    #![allow(clippy::decimal_literal_representation)]
    #![allow(clippy::multiple_inherent_impl)]
    #![allow(clippy::multiple_unsafe_ops_per_block)]
    #![allow(clippy::pub_underscore_fields)]
    #![allow(clippy::std_instead_of_alloc)]
    #![allow(clippy::std_instead_of_core)]
    #![allow(clippy::type_complexity)]
    #![allow(clippy::undocumented_unsafe_blocks)]
    #![allow(clippy::unreadable_literal)]
    #![allow(dead_code)]
    #![allow(missing_debug_implementations)]
    #![allow(missing_docs)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unsafe_code)]

    use core::fmt;

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    impl fmt::Display for v4l2_colorspace {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                Self::V4L2_COLORSPACE_DEFAULT => "default",
                Self::V4L2_COLORSPACE_SMPTE170M => "SMPTE 170m",
                Self::V4L2_COLORSPACE_SMPTE240M => "SMPTE 240m",
                Self::V4L2_COLORSPACE_REC709 => "Rec. 709",
                Self::V4L2_COLORSPACE_BT878 => {
                    // This is deprecated, must not be used, and no driver will ever return it.
                    unreachable!()
                }
                Self::V4L2_COLORSPACE_470_SYSTEM_M => "NTSC 1953",
                Self::V4L2_COLORSPACE_470_SYSTEM_BG => "EBU Tech. 3213 (PAL/SECAM)",
                Self::V4L2_COLORSPACE_JPEG => "JPEG",
                Self::V4L2_COLORSPACE_SRGB => "sRGB",
                Self::V4L2_COLORSPACE_OPRGB => "opRGB",
                Self::V4L2_COLORSPACE_BT2020 => "Rec. 2020",
                Self::V4L2_COLORSPACE_RAW => "Raw",
                Self::V4L2_COLORSPACE_DCI_P3 => "DCI-P3",
            })
        }
    }

    impl fmt::Display for v4l2_field {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                Self::V4L2_FIELD_ANY => "any",
                Self::V4L2_FIELD_NONE => "none",
                Self::V4L2_FIELD_TOP => "top",
                Self::V4L2_FIELD_BOTTOM => "bottom",
                Self::V4L2_FIELD_INTERLACED => "interlaced",
                Self::V4L2_FIELD_SEQ_TB => "sequential, top then bottom",
                Self::V4L2_FIELD_SEQ_BT => "sequential, bottom then top",
                Self::V4L2_FIELD_ALTERNATE => "alternate",
                Self::V4L2_FIELD_INTERLACED_TB => "interlaced, top then bottom",
                Self::V4L2_FIELD_INTERLACED_BT => "interlaced, bottom then top",
            })
        }
    }

    impl fmt::Display for v4l2_hsv_encoding {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                v4l2_hsv_encoding::V4L2_HSV_ENC_180 => "0-179 Range",
                v4l2_hsv_encoding::V4L2_HSV_ENC_256 => "0-255 Range",
            })
        }
    }

    impl fmt::Display for v4l2_quantization {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                v4l2_quantization::V4L2_QUANTIZATION_DEFAULT => "default",
                v4l2_quantization::V4L2_QUANTIZATION_FULL_RANGE => "full range",
                v4l2_quantization::V4L2_QUANTIZATION_LIM_RANGE => "limited range",
            })
        }
    }

    impl fmt::Display for v4l2_subdev_format_whence {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                Self::V4L2_SUBDEV_FORMAT_TRY => "try",
                Self::V4L2_SUBDEV_FORMAT_ACTIVE => "active",
            })
        }
    }

    impl fmt::Display for v4l2_xfer_func {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                Self::V4L2_XFER_FUNC_DEFAULT => "default",
                Self::V4L2_XFER_FUNC_709 => "Rec. 709",
                Self::V4L2_XFER_FUNC_SRGB => "sRGB",
                Self::V4L2_XFER_FUNC_OPRGB => "opRGB",
                Self::V4L2_XFER_FUNC_SMPTE240M => "SMPTE 240m",
                Self::V4L2_XFER_FUNC_NONE => "none",
                Self::V4L2_XFER_FUNC_DCI_P3 => "DCI-P3",
                Self::V4L2_XFER_FUNC_SMPTE2084 => "SMPTE ST 2084",
            })
        }
    }

    impl fmt::Display for v4l2_ycbcr_encoding {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                Self::V4L2_YCBCR_ENC_DEFAULT => "default",
                Self::V4L2_YCBCR_ENC_601 => "Rec. 601",
                Self::V4L2_YCBCR_ENC_709 => "Rec. 709",
                Self::V4L2_YCBCR_ENC_XV601 => "Rec. 601, Extended Gamut",
                Self::V4L2_YCBCR_ENC_XV709 => "Rec. 709, Extended Gamut",
                Self::V4L2_YCBCR_ENC_SYCC => {
                    // This is deprecated, must not be used, and no driver will ever return it.
                    unreachable!()
                }
                Self::V4L2_YCBCR_ENC_BT2020 => "Rec. 2020",
                Self::V4L2_YCBCR_ENC_BT2020_CONST_LUM => "Rec. 2020, Constant Luminance",
                Self::V4L2_YCBCR_ENC_SMPTE240M => "SMPTE 240m",
            })
        }
    }

    impl fmt::Debug for v4l2_buffer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
            let buf_type = v4l2_buf_type::try_from(self.type_).ok();
            let memory = v4l2_memory::try_from(self.memory).ok();

            let (location_name, location): (&str, &dyn fmt::Debug) = match (buf_type, memory) {
                (
                    Some(
                        v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE
                        | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT,
                    ),
                    Some(v4l2_memory::V4L2_MEMORY_DMABUF),
                ) => ("fd", &unsafe { self.m.fd }),
                (
                    Some(
                        v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE
                        | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT,
                    ),
                    Some(v4l2_memory::V4L2_MEMORY_MMAP),
                ) => ("offset", &unsafe { self.m.offset }),
                (
                    Some(
                        v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE
                        | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT,
                    ),
                    Some(v4l2_memory::V4L2_MEMORY_USERPTR),
                ) => ("userptr", &unsafe { self.m.userptr }),
                _ => unimplemented!(),
            };

            let mut dbg = f.debug_struct("v4l2_buffer");
            dbg.field("index", &self.index)
                .field("type", &self.type_)
                .field("bytesused", &self.bytesused)
                .field("flags", &self.flags)
                .field("field", &self.field)
                .field("timestamp", &self.timestamp)
                .field("timecode", &self.timecode)
                .field("sequence", &self.sequence)
                .field("memory", &self.memory)
                .field(location_name, location)
                .field("length", &self.length);

            let request_fd = unsafe { self.__bindgen_anon_1.request_fd };
            if request_fd != 0 {
                dbg.field("request_fd", &request_fd);
            }

            dbg.finish_non_exhaustive()
        }
    }

    impl fmt::Debug for v4l2_dv_timings {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let kind = self.type_;
            f.debug_struct("v4l2_dv_timings")
                .field("kind", &kind)
                .field(
                    "content",
                    match kind {
                        V4L2_DV_BT_656_1120 => unsafe { &self.__bindgen_anon_1.bt },
                        _ => unreachable!(),
                    },
                )
                .finish()
        }
    }

    impl fmt::Debug for v4l2_pix_format {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("v4l2_pix_format")
                .field("width", &self.width)
                .field("height", &self.height)
                .field("pixelformat", &self.pixelformat)
                .field("field", &self.field)
                .field("bytesperline", &self.bytesperline)
                .field("sizeimage", &self.sizeimage)
                .field("colorspace", &self.colorspace)
                .field("priv_", &self.priv_)
                .field("flags", &self.flags)
                .field("encoding", &unsafe { self.__bindgen_anon_1.ycbcr_enc })
                .field("quantization", &self.quantization)
                .field("xfer_func", &self.xfer_func)
                .finish()
        }
    }

    impl fmt::Debug for v4l2_format {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let kind = unsafe { std::mem::transmute::<u32, v4l2_buf_type>(self.type_) };
            f.debug_struct("v4l2_format")
                .field("type", &kind)
                .field(
                    "content",
                    &match kind {
                        v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE
                        | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT => unsafe { self.fmt.pix },
                        v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OVERLAY
                        | v4l2_buf_type::V4L2_BUF_TYPE_VBI_CAPTURE
                        | v4l2_buf_type::V4L2_BUF_TYPE_VBI_OUTPUT
                        | v4l2_buf_type::V4L2_BUF_TYPE_SLICED_VBI_CAPTURE
                        | v4l2_buf_type::V4L2_BUF_TYPE_SLICED_VBI_OUTPUT
                        | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT_OVERLAY
                        | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE_MPLANE
                        | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT_MPLANE
                        | v4l2_buf_type::V4L2_BUF_TYPE_SDR_CAPTURE
                        | v4l2_buf_type::V4L2_BUF_TYPE_SDR_OUTPUT
                        | v4l2_buf_type::V4L2_BUF_TYPE_META_CAPTURE
                        | v4l2_buf_type::V4L2_BUF_TYPE_META_OUTPUT
                        | v4l2_buf_type::V4L2_BUF_TYPE_PRIVATE => unimplemented!(),
                    },
                )
                .finish()
        }
    }
}

pub use bindgen::*;

#[doc(hidden)]
pub type v4l2_mbus_framefmt_encoding = v4l2_mbus_framefmt__bindgen_ty_1;

#[doc(hidden)]
pub type v4l2_pix_format_encoding = v4l2_pix_format__bindgen_ty_1;

#[doc(hidden)]
pub type v4l2_format_content = v4l2_format__bindgen_ty_1;

const V4L2_IOC_MAGIC: u8 = b'V';
const V4L2_IOC_QUERYCAP: u8 = 0;
const V4L2_IOC_ENUM_FMT: u8 = 2;
const V4L2_IOC_G_FMT: u8 = 4;
const V4L2_IOC_S_FMT: u8 = 5;
const V4L2_IOC_REQBUFS: u8 = 8;
const V4L2_IOC_QUERYBUF: u8 = 9;
const V4L2_IOC_QBUF: u8 = 15;
const V4L2_IOC_DQBUF: u8 = 17;
const V4L2_IOC_STREAMON: u8 = 18;
const V4L2_IOC_STREAMOFF: u8 = 19;
const V4L2_IOC_S_EDID: u8 = 41;
const V4L2_IOC_TRY_FMT: u8 = 64;
const V4L2_IOC_ENUM_FRAMESIZES: u8 = 74;
const V4L2_IOC_S_DV_TIMINGS: u8 = 87;
const V4L2_IOC_DQEVENT: u8 = 89;
const V4L2_IOC_SUBSCRIBE_EVENT: u8 = 90;
const V4L2_IOC_UNSUBSCRIBE_EVENT: u8 = 91;
const V4L2_IOC_QUERY_DV_TIMINGS: u8 = 99;

const V4L2_IOC_SUBDEV_S_FMT: u8 = V4L2_IOC_S_FMT;
const V4L2_IOC_SUBDEV_S_EDID: u8 = V4L2_IOC_S_EDID;
const V4L2_IOC_SUBDEV_S_DV_TIMINGS: u8 = V4L2_IOC_S_DV_TIMINGS;
const V4L2_IOC_SUBDEV_QUERY_DV_TIMINGS: u8 = V4L2_IOC_QUERY_DV_TIMINGS;

const V4L2_IOC_QUERYCAP_OPCODE: u32 =
    opcode::read::<v4l2_capability>(V4L2_IOC_MAGIC, V4L2_IOC_QUERYCAP);

/// Queries Device Capabilities
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
pub fn v4l2_ioctl_querycap(fd: BorrowedFd<'_>) -> io::Result<v4l2_capability> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Getter::<V4L2_IOC_QUERYCAP_OPCODE, v4l2_capability>::new() };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }.map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_ENUM_FMT_OPCODE: u32 =
    opcode::read_write::<v4l2_fmtdesc>(V4L2_IOC_MAGIC, V4L2_IOC_ENUM_FMT);

/// Enumerates image formats
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
pub fn v4l2_ioctl_enum_fmt(fd: BorrowedFd<'_>, mut desc: v4l2_fmtdesc) -> io::Result<v4l2_fmtdesc> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Updater::<V4L2_IOC_ENUM_FMT_OPCODE, v4l2_fmtdesc>::new(&mut desc) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| desc)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_G_FMT_OPCODE: u32 =
    opcode::read_write::<v4l2_format>(V4L2_IOC_MAGIC, V4L2_IOC_G_FMT);

/// Gets the current data format.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
pub fn v4l2_ioctl_g_fmt(fd: BorrowedFd<'_>, mut fmt: v4l2_format) -> io::Result<v4l2_format> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Updater::<V4L2_IOC_G_FMT_OPCODE, v4l2_format>::new(&mut fmt) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| fmt)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_S_FMT_OPCODE: u32 =
    opcode::read_write::<v4l2_format>(V4L2_IOC_MAGIC, V4L2_IOC_S_FMT);

/// Sets the data format on a v4l2 device.
///
/// The driver can adjust the format to accommodate hardware limitations. The actual format used by
/// the driver will be returned.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
pub fn v4l2_ioctl_s_fmt(fd: BorrowedFd<'_>, mut fmt: v4l2_format) -> io::Result<v4l2_format> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Updater::<V4L2_IOC_S_FMT_OPCODE, v4l2_format>::new(&mut fmt) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| fmt)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_SUBDEV_S_FMT_OPCODE: u32 =
    opcode::read_write::<v4l2_subdev_format>(V4L2_IOC_MAGIC, V4L2_IOC_SUBDEV_S_FMT);

/// Sets the data format on a v4l2 sub-device.
///
/// The driver can adjust the format to accommodate hardware limitations. The actual format used by
/// the driver will be returned.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
pub fn v4l2_ioctl_subdev_s_fmt(
    fd: BorrowedFd<'_>,
    mut fmt: v4l2_subdev_format,
) -> io::Result<v4l2_subdev_format> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj =
        unsafe { Updater::<V4L2_IOC_SUBDEV_S_FMT_OPCODE, v4l2_subdev_format>::new(&mut fmt) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| fmt)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_REQBUFS_OPCODE: u32 =
    opcode::read_write::<v4l2_requestbuffers>(V4L2_IOC_MAGIC, V4L2_IOC_REQBUFS);

/// Allocates Device Buffers.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
pub fn v4l2_ioctl_reqbufs(
    fd: BorrowedFd<'_>,
    mut bufs: v4l2_requestbuffers,
) -> io::Result<v4l2_requestbuffers> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj =
        unsafe { Updater::<V4L2_IOC_REQBUFS_OPCODE, v4l2_requestbuffers>::new(&mut bufs) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| bufs)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_QUERYBUF_OPCODE: u32 =
    opcode::read_write::<v4l2_buffer>(V4L2_IOC_MAGIC, V4L2_IOC_QUERYBUF);

/// Queries the status of a buffer.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
pub fn v4l2_ioctl_querybuf(fd: BorrowedFd<'_>, mut buf: v4l2_buffer) -> io::Result<v4l2_buffer> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Updater::<V4L2_IOC_QUERYBUF_OPCODE, v4l2_buffer>::new(&mut buf) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| buf)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_QBUF_OPCODE: u32 = opcode::read_write::<v4l2_buffer>(V4L2_IOC_MAGIC, V4L2_IOC_QBUF);

/// Queue a buffer in the driver's incoming queue.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
pub fn v4l2_ioctl_qbuf(fd: BorrowedFd<'_>, mut buf: v4l2_buffer) -> io::Result<v4l2_buffer> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Updater::<V4L2_IOC_QBUF_OPCODE, v4l2_buffer>::new(&mut buf) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| buf)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_DQBUF_OPCODE: u32 =
    opcode::read_write::<v4l2_buffer>(V4L2_IOC_MAGIC, V4L2_IOC_DQBUF);

/// Dequeue a buffer from the driver's outgoing queue.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
pub fn v4l2_ioctl_dqbuf(fd: BorrowedFd<'_>, mut buf: v4l2_buffer) -> io::Result<v4l2_buffer> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Updater::<V4L2_IOC_DQBUF_OPCODE, v4l2_buffer>::new(&mut buf) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| buf)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_STREAMON_OPCODE: u32 = opcode::write::<c_int>(V4L2_IOC_MAGIC, V4L2_IOC_STREAMON);

/// Starts Streaming I/O
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
///
/// # Panics
///
/// If the buffer type can't be converted to a signed integer
pub fn v4l2_ioctl_streamon(fd: BorrowedFd<'_>, buf_kind: u32) -> io::Result<()> {
    let val = c_int::try_from(buf_kind).expect("v4l2_buf_type fits on both a u32 and an i32");

    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Setter::<V4L2_IOC_STREAMON_OPCODE, c_int>::new(val) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }.map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_STREAMOFF_OPCODE: u32 = opcode::write::<c_int>(V4L2_IOC_MAGIC, V4L2_IOC_STREAMOFF);

/// Stops Streaming I/O
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
///
/// # Panics
///
/// If the buffer type can't be converted to a signed integer
pub fn v4l2_ioctl_streamoff(fd: BorrowedFd<'_>, buf_kind: u32) -> io::Result<()> {
    let val = c_int::try_from(buf_kind).expect("v4l2_buf_type fits on both a u32 and an i32");

    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Setter::<V4L2_IOC_STREAMOFF_OPCODE, c_int>::new(val) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }.map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_S_EDID_OPCODE: u32 =
    opcode::read_write::<v4l2_edid>(V4L2_IOC_MAGIC, V4L2_IOC_S_EDID);
const V4L2_IOC_SUBDEV_S_EDID_OPCODE: u32 =
    opcode::read_write::<v4l2_edid>(V4L2_IOC_MAGIC, V4L2_IOC_SUBDEV_S_EDID);

fn v4l2_ioctl_s_edid_inner<const OPCODE: Opcode>(
    fd: BorrowedFd<'_>,
    mut edid: v4l2_edid,
) -> io::Result<v4l2_edid> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Updater::<OPCODE, v4l2_edid>::new(&mut edid) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| edid)
        .map_err(<Errno as Into<io::Error>>::into)
}

/// Sets the EDID of a v4l2 device
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn v4l2_ioctl_s_edid(fd: BorrowedFd<'_>, edid: v4l2_edid) -> io::Result<v4l2_edid> {
    v4l2_ioctl_s_edid_inner::<V4L2_IOC_S_EDID_OPCODE>(fd, edid)
}

/// Sets the EDID of a v4l2 sub-device
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn v4l2_ioctl_subdev_s_edid(fd: BorrowedFd<'_>, edid: v4l2_edid) -> io::Result<v4l2_edid> {
    v4l2_ioctl_s_edid_inner::<V4L2_IOC_SUBDEV_S_EDID_OPCODE>(fd, edid)
}

const V4L2_IOC_TRY_FMT_OPCODE: u32 =
    opcode::read_write::<v4l2_format>(V4L2_IOC_MAGIC, V4L2_IOC_TRY_FMT);

/// Tries to the data format.
///
/// The driver can adjust the format to accommodate hardware limitations. The format the driver
/// would use will be returned.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor.
pub fn v4l2_ioctl_try_fmt(fd: BorrowedFd<'_>, mut fmt: v4l2_format) -> io::Result<v4l2_format> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Updater::<V4L2_IOC_TRY_FMT_OPCODE, v4l2_format>::new(&mut fmt) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| fmt)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_ENUM_FRAMESIZES_OPCODE: u32 =
    opcode::read_write::<v4l2_frmsizeenum>(V4L2_IOC_MAGIC, V4L2_IOC_ENUM_FRAMESIZES);

/// Enumerates Frame Sizes.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor.
pub fn v4l2_ioctl_enum_framesizes(
    fd: BorrowedFd<'_>,
    mut size: v4l2_frmsizeenum,
) -> io::Result<v4l2_frmsizeenum> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj =
        unsafe { Updater::<V4L2_IOC_ENUM_FRAMESIZES_OPCODE, v4l2_frmsizeenum>::new(&mut size) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| size)
        .map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_S_DV_TIMINGS_OPCODE: u32 =
    opcode::read_write::<v4l2_dv_timings>(V4L2_IOC_MAGIC, V4L2_IOC_S_DV_TIMINGS);

const V4L2_IOC_SUBDEV_S_DV_TIMINGS_OPCODE: u32 =
    opcode::read_write::<v4l2_dv_timings>(V4L2_IOC_MAGIC, V4L2_IOC_SUBDEV_S_DV_TIMINGS);

fn v4l2_ioctl_update_dv_timings<const OPCODE: Opcode>(
    fd: BorrowedFd<'_>,
    mut timings: v4l2_dv_timings,
) -> io::Result<v4l2_dv_timings> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Updater::<OPCODE, v4l2_dv_timings>::new(&mut timings) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| timings)
        .map_err(<Errno as Into<io::Error>>::into)
}

/// Sets the DV Timings on a v4l2 device.
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn v4l2_ioctl_s_dv_timings(
    fd: BorrowedFd<'_>,
    timings: v4l2_dv_timings,
) -> io::Result<v4l2_dv_timings> {
    v4l2_ioctl_update_dv_timings::<V4L2_IOC_S_DV_TIMINGS_OPCODE>(fd, timings)
}

/// Sets the DV Timings on a v4l2 sub-device.
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn v4l2_ioctl_subdev_s_dv_timings(
    fd: BorrowedFd<'_>,
    timings: v4l2_dv_timings,
) -> io::Result<v4l2_dv_timings> {
    v4l2_ioctl_update_dv_timings::<V4L2_IOC_SUBDEV_S_DV_TIMINGS_OPCODE>(fd, timings)
}

const V4L2_IOC_QUERY_DV_TIMINGS_OPCODE: u32 =
    opcode::read::<v4l2_dv_timings>(V4L2_IOC_MAGIC, V4L2_IOC_QUERY_DV_TIMINGS);

const V4L2_IOC_SUBDEV_QUERY_DV_TIMINGS_OPCODE: u32 =
    opcode::read::<v4l2_dv_timings>(V4L2_IOC_MAGIC, V4L2_IOC_SUBDEV_QUERY_DV_TIMINGS);

/// Senses the DV Timings on a v4l2 device
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn v4l2_ioctl_query_dv_timings(
    fd: BorrowedFd<'_>,
    timings: v4l2_dv_timings,
) -> io::Result<v4l2_dv_timings> {
    v4l2_ioctl_update_dv_timings::<V4L2_IOC_QUERY_DV_TIMINGS_OPCODE>(fd, timings)
}

const V4L2_IOC_DQEVENT_OPCODE: u32 = opcode::read::<v4l2_event>(V4L2_IOC_MAGIC, V4L2_IOC_DQEVENT);

/// Dequeue event
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn v4l2_ioctl_dqevent(fd: BorrowedFd<'_>) -> io::Result<v4l2_event> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj = unsafe { Getter::<V4L2_IOC_DQEVENT_OPCODE, v4l2_event>::new() };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }.map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_SUBSCRIBE_EVENT_OPCODE: u32 =
    opcode::write::<v4l2_event_subscription>(V4L2_IOC_MAGIC, V4L2_IOC_SUBSCRIBE_EVENT);

/// Subscribes to events
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn v4l2_ioctl_subscribe_event(
    fd: BorrowedFd<'_>,
    sub: v4l2_event_subscription,
) -> io::Result<()> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj =
        unsafe { Setter::<V4L2_IOC_SUBSCRIBE_EVENT_OPCODE, v4l2_event_subscription>::new(sub) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }.map_err(<Errno as Into<io::Error>>::into)
}

const V4L2_IOC_UNSUBSCRIBE_EVENT_OPCODE: u32 =
    opcode::write::<v4l2_event_subscription>(V4L2_IOC_MAGIC, V4L2_IOC_UNSUBSCRIBE_EVENT);

/// Unsubscribes from events
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn v4l2_ioctl_unsubscribe_event(
    fd: BorrowedFd<'_>,
    sub: v4l2_event_subscription,
) -> io::Result<()> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj =
        unsafe { Setter::<V4L2_IOC_UNSUBSCRIBE_EVENT_OPCODE, v4l2_event_subscription>::new(sub) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }.map_err(<Errno as Into<io::Error>>::into)
}

/// Senses the DV Timings on a v4l2 sub-device
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn v4l2_ioctl_subdev_query_dv_timings(
    fd: BorrowedFd<'_>,
    timings: v4l2_dv_timings,
) -> io::Result<v4l2_dv_timings> {
    v4l2_ioctl_update_dv_timings::<V4L2_IOC_SUBDEV_QUERY_DV_TIMINGS_OPCODE>(fd, timings)
}
