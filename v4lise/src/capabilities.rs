use bitflags::bitflags;
use v4l2_raw::{
    raw::{self, v4l2_capability},
    v4l2_buf_type,
};

bitflags! {
    pub struct CapabilitiesFlags: u32 {
    const VIDEO_CAPTURE = raw::V4L2_CAP_VIDEO_CAPTURE;
    const VIDEO_OUTPUT = raw::V4L2_CAP_VIDEO_OUTPUT;
    const VIDEO_OVERLAY = raw::V4L2_CAP_VIDEO_OVERLAY;
    const VBI_CAPTURE = raw::V4L2_CAP_VBI_CAPTURE;
    const VBI_OUTPUT = raw::V4L2_CAP_VBI_OUTPUT;
    const SLICED_VBI_CAPTURE = raw::V4L2_CAP_SLICED_VBI_CAPTURE;
    const SLICED_VBI_OUTPUT = raw::V4L2_CAP_SLICED_VBI_OUTPUT;
    const RDS_CAPTURE = raw::V4L2_CAP_RDS_CAPTURE;
    const VIDEO_OUTPUT_OVERLAY = raw::V4L2_CAP_VIDEO_OUTPUT_OVERLAY;
    const HW_FREQ_SEEK = raw::V4L2_CAP_HW_FREQ_SEEK;
    const RDS_OUTPUT = raw::V4L2_CAP_RDS_OUTPUT;
    const VIDEO_CAPTURE_MPLANE = raw::V4L2_CAP_VIDEO_CAPTURE_MPLANE;
    const VIDEO_OUTPUT_MPLANE = raw::V4L2_CAP_VIDEO_OUTPUT_MPLANE;
    const VIDEO_M2M_MPLANE = raw::V4L2_CAP_VIDEO_M2M_MPLANE;
    const VIDEO_M2M = raw::V4L2_CAP_VIDEO_M2M;
    const TUNER = raw::V4L2_CAP_TUNER;
    const AUDIO = raw::V4L2_CAP_AUDIO;
    const RADIO = raw::V4L2_CAP_RADIO;
    const MODULATOR = raw::V4L2_CAP_MODULATOR;
    const SDR_CAPTURE = raw::V4L2_CAP_SDR_CAPTURE;
    const EXT_PIX_FORMAT = raw::V4L2_CAP_EXT_PIX_FORMAT;
    const SDR_OUTPUT = raw::V4L2_CAP_SDR_OUTPUT;
    const META_CAPTURE = raw::V4L2_CAP_META_CAPTURE;
    const READWRITE = raw::V4L2_CAP_READWRITE;
    const ASYNCIO = raw::V4L2_CAP_ASYNCIO;
    const STREAMING = raw::V4L2_CAP_STREAMING;
    const META_OUTPUT = raw::V4L2_CAP_META_OUTPUT;
    const TOUCH = raw::V4L2_CAP_TOUCH;
    const DEVICE_CAPS = raw::V4L2_CAP_DEVICE_CAPS;
    }
}

impl From<v4l2_buf_type> for CapabilitiesFlags {
    fn from(value: v4l2_buf_type) -> Self {
        match value {
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE => Self::VIDEO_CAPTURE,
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT => Self::VIDEO_OUTPUT,
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OVERLAY => Self::VIDEO_OVERLAY,
            v4l2_buf_type::V4L2_BUF_TYPE_VBI_CAPTURE => Self::VBI_CAPTURE,
            v4l2_buf_type::V4L2_BUF_TYPE_VBI_OUTPUT => Self::VBI_OUTPUT,
            v4l2_buf_type::V4L2_BUF_TYPE_SLICED_VBI_CAPTURE => Self::SLICED_VBI_CAPTURE,
            v4l2_buf_type::V4L2_BUF_TYPE_SLICED_VBI_OUTPUT => Self::SLICED_VBI_OUTPUT,
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT_OVERLAY => Self::VIDEO_OUTPUT_OVERLAY,
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE_MPLANE => Self::VIDEO_CAPTURE_MPLANE,
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT_MPLANE => Self::VIDEO_OUTPUT_MPLANE,
            v4l2_buf_type::V4L2_BUF_TYPE_SDR_CAPTURE => Self::SDR_CAPTURE,
            v4l2_buf_type::V4L2_BUF_TYPE_SDR_OUTPUT => Self::SDR_OUTPUT,
            v4l2_buf_type::V4L2_BUF_TYPE_META_CAPTURE => Self::META_CAPTURE,
            v4l2_buf_type::V4L2_BUF_TYPE_META_OUTPUT => Self::META_OUTPUT,
            v4l2_buf_type::V4L2_BUF_TYPE_PRIVATE => unimplemented!(),
        }
    }
}

#[expect(dead_code)]
pub struct Capability {
    pub driver: String,
    pub card: String,
    pub bus_info: String,
    pub version: u32,
    pub capabilities: CapabilitiesFlags,
    pub device_caps: CapabilitiesFlags,
}

impl From<v4l2_capability> for Capability {
    fn from(caps: v4l2_capability) -> Self {
        Capability {
            driver: String::from_utf8_lossy(&caps.driver).into_owned(),
            card: String::from_utf8_lossy(&caps.card).into_owned(),
            bus_info: String::from_utf8_lossy(&caps.bus_info).into_owned(),
            version: caps.version,
            capabilities: CapabilitiesFlags::from_bits_truncate(caps.capabilities),
            device_caps: CapabilitiesFlags::from_bits_truncate(caps.device_caps),
        }
    }
}
