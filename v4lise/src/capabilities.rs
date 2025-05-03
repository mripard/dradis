use crate::v4l2_capability;

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
