use crate::lowlevel::CapabilitiesFlags;
use crate::lowlevel::v4l2_capability;

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
