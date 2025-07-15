#![allow(non_camel_case_types)]
#![allow(unsafe_code)]
#![doc = include_str!("../README.md")]

extern crate alloc;
use alloc::rc::Rc;
use core::{
    cell::RefCell,
    ffi::{CStr, c_char},
    fmt,
};
use std::{
    fs::File,
    io,
    os::fd::{AsFd as _, BorrowedFd, OwnedFd},
    path::{Path, PathBuf},
};

use bitflags::bitflags;
use bytemuck::cast_slice;
use facet::Facet;
use facet_enum_repr::FacetEnumRepr;
use linux_raw::KernelVersion;

/// Raw, unsafe, abstraction
pub mod raw;
use raw::{
    media_device_info, media_ioctl_device_info, media_v2_entity, media_v2_interface, media_v2_link,
    media_v2_pad, media_v2_topology,
};

/// Revocable Objects
mod revocable;
pub use revocable::{Revocable, RevocableResult, RevocableValue};

fn chars_to_string(chars: &[c_char], ascii_only: bool) -> String {
    let str = CStr::from_bytes_until_nul(cast_slice(chars))
        .expect("The kernel guarantees the string is null-terminated.")
        .to_str()
        .expect("The kernel guarantees this is an ASCII or UTF-8 string.");

    #[expect(clippy::nonminimal_bool, reason = "This is easier to read that way")]
    {
        assert!(
            !(ascii_only && !str.is_ascii()),
            "The kernel guarantees this is an ASCII string."
        );
    };

    str.to_owned()
}

/// A Device File representation
#[derive(Clone, Debug)]
pub struct DeviceNode {
    major: u32,
    minor: u32,
    path: PathBuf,
}

impl DeviceNode {
    fn new(major: u32, minor: u32) -> io::Result<Self> {
        Ok(Self {
            major,
            minor,
            path: PathBuf::from(&format!("/dev/char/{major}:{minor}")).canonicalize()?,
        })
    }

    /// Device File Major Number
    #[must_use]
    pub fn major(&self) -> u32 {
        self.major
    }

    /// Device File Minor Number
    #[must_use]
    pub fn minor(&self) -> u32 {
        self.minor
    }

    /// Path to the Device File
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Media Controller Entity Function
#[repr(u32)]
#[derive(Clone, Copy, Debug, Facet, FacetEnumRepr, PartialEq)]
pub enum media_entity_function {
    /// Unknown Entity
    MEDIA_ENT_F_UNKNOWN = raw::bindgen::MEDIA_ENT_F_UNKNOWN,
    /// Unknown Sub-Device Entity
    MEDIA_ENT_F_V4L2_SUBDEV_UNKNOWN = raw::bindgen::MEDIA_ENT_F_V4L2_SUBDEV_UNKNOWN,

    // DVB Entity Functions
    /// Digital TV demodulator entity
    MEDIA_ENT_F_DTV_DEMOD = raw::bindgen::MEDIA_ENT_F_DTV_DEMOD,
    /// MPEG Transport stream demux entity
    MEDIA_ENT_F_TS_DEMUX = raw::bindgen::MEDIA_ENT_F_TS_DEMUX,
    /// Digital TV Conditional Access module (CAM) entity
    MEDIA_ENT_F_DTV_CA = raw::bindgen::MEDIA_ENT_F_DTV_CA,
    /// Digital TV network ULE/MLE desencapsulation entity
    MEDIA_ENT_F_DTV_NET_DECAP = raw::bindgen::MEDIA_ENT_F_DTV_NET_DECAP,

    // I/O Entity Functions
    /// Data streaming input and/or output entity
    MEDIA_ENT_F_IO_V4L = raw::bindgen::MEDIA_ENT_F_IO_V4L,
    /// DVB Digital TV streaming input or output entity
    MEDIA_ENT_F_IO_DTV = raw::bindgen::MEDIA_ENT_F_IO_DTV,
    /// V4L VBI streaming input or output entity
    MEDIA_ENT_F_IO_VBI = raw::bindgen::MEDIA_ENT_F_IO_VBI,
    /// V4L Software Digital Radio (SDR) streaming input or output entity
    MEDIA_ENT_F_IO_SWRADIO = raw::bindgen::MEDIA_ENT_F_IO_SWRADIO,

    // Sensor Functions
    /// Camera video sensor entity.
    MEDIA_ENT_F_CAM_SENSOR = raw::bindgen::MEDIA_ENT_F_CAM_SENSOR,
    /// Flash controller entity.
    MEDIA_ENT_F_FLASH = raw::bindgen::MEDIA_ENT_F_FLASH,
    /// Lens controller entity.
    MEDIA_ENT_F_LENS = raw::bindgen::MEDIA_ENT_F_LENS,

    // Digital, Analog TV, radio and/or SDR tuner Functions
    /// Digital TV, analog TV, radio and/or software radio tuner
    MEDIA_ENT_F_TUNER = raw::bindgen::MEDIA_ENT_F_TUNER,

    // Analog TV IF-PLL Decoder Functions
    /// IF-PLL video decoder.
    MEDIA_ENT_F_IF_VID_DECODER = raw::bindgen::MEDIA_ENT_F_IF_VID_DECODER,
    /// IF-PLL sound decoder.
    MEDIA_ENT_F_IF_AUD_DECODER = raw::bindgen::MEDIA_ENT_F_IF_AUD_DECODER,

    // Audio Entity Functions
    /// Audio Capture Function Entity.
    MEDIA_ENT_F_AUDIO_CAPTURE = raw::bindgen::MEDIA_ENT_F_AUDIO_CAPTURE,
    /// Audio Playback Function Entity.
    MEDIA_ENT_F_AUDIO_PLAYBACK = raw::bindgen::MEDIA_ENT_F_AUDIO_PLAYBACK,
    /// Audio Mixer Function Entity.
    MEDIA_ENT_F_AUDIO_MIXER = raw::bindgen::MEDIA_ENT_F_AUDIO_MIXER,

    // Processing Entity Functions
    /// Video composer (blender).
    MEDIA_ENT_F_PROC_VIDEO_COMPOSER = raw::bindgen::MEDIA_ENT_F_PROC_VIDEO_COMPOSER,
    /// Video pixel formatter.
    MEDIA_ENT_F_PROC_VIDEO_PIXEL_FORMATTER = raw::bindgen::MEDIA_ENT_F_PROC_VIDEO_PIXEL_FORMATTER,
    /// Video pixel encoding converter
    MEDIA_ENT_F_PROC_VIDEO_PIXEL_ENC_CONV = raw::bindgen::MEDIA_ENT_F_PROC_VIDEO_PIXEL_ENC_CONV,
    /// Video look-up table
    MEDIA_ENT_F_PROC_VIDEO_LUT = raw::bindgen::MEDIA_ENT_F_PROC_VIDEO_LUT,
    /// Video scaler
    MEDIA_ENT_F_PROC_VIDEO_SCALER = raw::bindgen::MEDIA_ENT_F_PROC_VIDEO_SCALER,
    /// Video statistics computation (histogram, 3A, ...).
    MEDIA_ENT_F_PROC_VIDEO_STATISTICS = raw::bindgen::MEDIA_ENT_F_PROC_VIDEO_STATISTICS,
    /// Video (`MPEG`, `HEVC`, `VPx`, etc.) encoder.
    MEDIA_ENT_F_PROC_VIDEO_ENCODER = raw::bindgen::MEDIA_ENT_F_PROC_VIDEO_ENCODER,
    /// Video (`MPEG`, `HEVC`, `VPx`, etc.) decoder.
    MEDIA_ENT_F_PROC_VIDEO_DECODER = raw::bindgen::MEDIA_ENT_F_PROC_VIDEO_DECODER,
    /// An Image Signal Processor (ISP) device.
    MEDIA_ENT_F_PROC_VIDEO_ISP = raw::bindgen::MEDIA_ENT_F_PROC_VIDEO_ISP,

    // Switch and Bridge Entity Functions
    /// Video multiplexer.
    MEDIA_ENT_F_VID_MUX = raw::bindgen::MEDIA_ENT_F_VID_MUX,
    /// Video interface bridge.
    MEDIA_ENT_F_VID_IF_BRIDGE = raw::bindgen::MEDIA_ENT_F_VID_IF_BRIDGE,

    // Video Decoder / Encoder Functions
    /// Analog video decoder.
    MEDIA_ENT_F_ATV_DECODER = raw::bindgen::MEDIA_ENT_F_ATV_DECODER,
    /// Digital video decoder.
    MEDIA_ENT_F_DV_DECODER = raw::bindgen::MEDIA_ENT_F_DV_DECODER,
    /// Digital video encoder.
    MEDIA_ENT_F_DV_ENCODER = raw::bindgen::MEDIA_ENT_F_DV_ENCODER,
}

impl fmt::Display for media_entity_function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MEDIA_ENT_F_UNKNOWN => "Unknown Entity",
            Self::MEDIA_ENT_F_IO_V4L => "Data Streaming Entity",
            Self::MEDIA_ENT_F_CAM_SENSOR => "Camera Video Sensor Entity",
            Self::MEDIA_ENT_F_LENS => "Lens Entity",
            Self::MEDIA_ENT_F_PROC_VIDEO_DECODER => "Video Decoder",
            Self::MEDIA_ENT_F_PROC_VIDEO_PIXEL_ENC_CONV => "Video Pixel Encoding Converter Entity",
            Self::MEDIA_ENT_F_PROC_VIDEO_PIXEL_FORMATTER => "Video Pixel Formatter Entity",
            Self::MEDIA_ENT_F_PROC_VIDEO_SCALER => "Scaler Entity",
            Self::MEDIA_ENT_F_VID_IF_BRIDGE => "Video Interface Bridge",
            media_entity_function::MEDIA_ENT_F_V4L2_SUBDEV_UNKNOWN
            | media_entity_function::MEDIA_ENT_F_DTV_DEMOD
            | media_entity_function::MEDIA_ENT_F_TS_DEMUX
            | media_entity_function::MEDIA_ENT_F_DTV_CA
            | media_entity_function::MEDIA_ENT_F_DTV_NET_DECAP
            | media_entity_function::MEDIA_ENT_F_IO_DTV
            | media_entity_function::MEDIA_ENT_F_IO_VBI
            | media_entity_function::MEDIA_ENT_F_IO_SWRADIO
            | media_entity_function::MEDIA_ENT_F_FLASH
            | media_entity_function::MEDIA_ENT_F_TUNER
            | media_entity_function::MEDIA_ENT_F_IF_VID_DECODER
            | media_entity_function::MEDIA_ENT_F_IF_AUD_DECODER
            | media_entity_function::MEDIA_ENT_F_AUDIO_CAPTURE
            | media_entity_function::MEDIA_ENT_F_AUDIO_PLAYBACK
            | media_entity_function::MEDIA_ENT_F_AUDIO_MIXER
            | media_entity_function::MEDIA_ENT_F_PROC_VIDEO_COMPOSER
            | media_entity_function::MEDIA_ENT_F_PROC_VIDEO_LUT
            | media_entity_function::MEDIA_ENT_F_PROC_VIDEO_STATISTICS
            | media_entity_function::MEDIA_ENT_F_PROC_VIDEO_ENCODER
            | media_entity_function::MEDIA_ENT_F_PROC_VIDEO_ISP
            | media_entity_function::MEDIA_ENT_F_VID_MUX
            | media_entity_function::MEDIA_ENT_F_ATV_DECODER
            | media_entity_function::MEDIA_ENT_F_DV_DECODER
            | media_entity_function::MEDIA_ENT_F_DV_ENCODER => {
                return f.write_fmt(format_args!("Unknown {self:#?}"));
            }
        })
    }
}

/// An ALSA Device Node Interface Kind
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaControllerInterfaceAlsaKind {
    /// Device node interface for ALSA PCM Capture
    PcmCapture,

    /// Device node interface for ALSA PCM Playback
    PcmPlayback,

    /// Device node interface for ALSA Control
    Control,
}

/// A DVB Device Node Interface Kind
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaControllerInterfaceDvbKind {
    /// Device node interface for the Digital TV frontend
    Fe,

    /// Device node interface for the Digital TV demux
    Demux,

    /// Device node interface for the Digital TV DVR
    Dvr,

    /// Device node interface for the Digital TV Conditional Access
    Ca,

    /// Device node interface for the Digital TV network control
    Net,
}

/// A V4L2 Device Node Interface Kind
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaControllerInterfaceV4lKind {
    /// Device node interface for video (V4L)
    Video,

    /// Device node interface for VBI (V4L)
    Vbi,

    /// Device node interface for radio (V4L)
    Radio,

    /// Device node interface for a V4L subdevice
    Subdev,

    /// Device node interface for Software Defined Radio (V4L)
    SwRadio,

    /// Device node interface for Touch device (V4L)
    Touch,
}

/// A Device Node Interface Kind
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaControllerInterfaceKind {
    /// ALSA Device Node Interface
    Alsa(MediaControllerInterfaceAlsaKind),

    /// DVB Device Node Interface
    DVB(MediaControllerInterfaceDvbKind),

    /// V4L2 Device Node Interface
    V4L(MediaControllerInterfaceV4lKind),
}

impl TryFrom<u32> for MediaControllerInterfaceKind {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            raw::bindgen::MEDIA_INTF_T_ALSA_CONTROL => {
                Self::Alsa(MediaControllerInterfaceAlsaKind::Control)
            }
            raw::bindgen::MEDIA_INTF_T_ALSA_PCM_CAPTURE => {
                Self::Alsa(MediaControllerInterfaceAlsaKind::PcmCapture)
            }
            raw::bindgen::MEDIA_INTF_T_ALSA_PCM_PLAYBACK => {
                Self::Alsa(MediaControllerInterfaceAlsaKind::PcmPlayback)
            }
            raw::bindgen::MEDIA_INTF_T_DVB_CA => Self::DVB(MediaControllerInterfaceDvbKind::Ca),
            raw::bindgen::MEDIA_INTF_T_DVB_DEMUX => {
                Self::DVB(MediaControllerInterfaceDvbKind::Demux)
            }
            raw::bindgen::MEDIA_INTF_T_DVB_DVR => Self::DVB(MediaControllerInterfaceDvbKind::Dvr),
            raw::bindgen::MEDIA_INTF_T_DVB_FE => Self::DVB(MediaControllerInterfaceDvbKind::Fe),
            raw::bindgen::MEDIA_INTF_T_DVB_NET => Self::DVB(MediaControllerInterfaceDvbKind::Net),
            raw::bindgen::MEDIA_INTF_T_V4L_RADIO => {
                Self::V4L(MediaControllerInterfaceV4lKind::Radio)
            }
            raw::bindgen::MEDIA_INTF_T_V4L_SUBDEV => {
                Self::V4L(MediaControllerInterfaceV4lKind::Subdev)
            }
            raw::bindgen::MEDIA_INTF_T_V4L_SWRADIO => {
                Self::V4L(MediaControllerInterfaceV4lKind::SwRadio)
            }
            raw::bindgen::MEDIA_INTF_T_V4L_TOUCH => {
                Self::V4L(MediaControllerInterfaceV4lKind::Touch)
            }
            raw::bindgen::MEDIA_INTF_T_V4L_VBI => Self::V4L(MediaControllerInterfaceV4lKind::Vbi),
            raw::bindgen::MEDIA_INTF_T_V4L_VIDEO => {
                Self::V4L(MediaControllerInterfaceV4lKind::Video)
            }
            _ => unimplemented!(),
        })
    }
}

/// Media Device Information
#[derive(Debug)]
pub struct MediaControllerInfo {
    driver: String,
    model: String,
    serial: String,
    bus_info: String,
    media_version: KernelVersion,
    hw_revision: u32,
    driver_version: KernelVersion,
}

impl MediaControllerInfo {
    /// Location of the Device in the system. This includes a bus type name, and a bus-specific
    /// identifier.
    #[must_use]
    pub fn bus_info(&self) -> &str {
        &self.bus_info
    }

    /// Name of the driver implementing the Media Controller API
    #[must_use]
    pub fn driver(&self) -> &str {
        &self.driver
    }

    /// Media Device Driver Version. Together with the driver name, this identifies a particular
    /// driver.
    #[must_use]
    pub fn driver_version(&self) -> &KernelVersion {
        &self.driver_version
    }

    /// Hardware Revision. The format is device specific.
    #[must_use]
    pub fn hardware_revision(&self) -> u32 {
        self.hw_revision
    }

    /// Media Controller API Version.
    #[must_use]
    pub fn media_controller_version(&self) -> &KernelVersion {
        &self.media_version
    }

    /// Model Name
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Device Serial Number
    #[must_use]
    pub fn serial(&self) -> &str {
        &self.serial
    }
}

impl From<media_device_info> for MediaControllerInfo {
    fn from(value: media_device_info) -> Self {
        Self {
            driver: chars_to_string(&value.driver, true),
            model: chars_to_string(&value.model, false),
            serial: chars_to_string(&value.serial, true),
            bus_info: chars_to_string(&value.bus_info, true),
            media_version: value.media_version.into(),
            hw_revision: value.hw_revision,
            driver_version: value.driver_version.into(),
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct MediaControllerEntityFlags: u32 {
        const DEFAULT = raw::bindgen::MEDIA_ENT_FL_DEFAULT;
        const CONNECTOR = raw::bindgen::MEDIA_ENT_FL_CONNECTOR;
    }
}

impl TryFrom<u32> for MediaControllerEntityFlags {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_bits(value).ok_or(())
    }
}

struct MediaControllerEntityInner {
    controller: Rc<RefCell<MediaControllerInner>>,
    id: u32,
    name: String,
    function: media_entity_function,
    flags: MediaControllerEntityFlags,
}

impl fmt::Debug for MediaControllerEntityInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaControllerEntityInner")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("flags", &self.flags)
            .field("function", &self.function)
            .finish_non_exhaustive()
    }
}

/// A representation of a Media Controller Entity.
#[derive(Clone, Debug)]
pub struct MediaControllerEntity(Rc<RefCell<Revocable<MediaControllerEntityInner>>>);

impl MediaControllerEntity {
    fn flags(&self) -> RevocableValue<MediaControllerEntityFlags> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |e| RevocableValue::Value(e.flags))
    }

    /// Returns an iterator over the flag names set for this entity, if the entity is still valid.
    pub fn flag_names(&self) -> RevocableValue<impl Iterator<Item = &str>> {
        self.flags().map(|f| f.iter_names().map(|(n, _)| n))
    }

    /// Returns this entity function, if the entity is still valid.
    ///
    /// # Panics
    ///
    /// If the function returned by the kernel is unknown
    pub fn function(&self) -> RevocableValue<media_entity_function> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |e| {
                RevocableValue::Value(e.function)
            })
    }

    /// Returns this entity ID, if the entity is still valid.
    pub fn id(&self) -> RevocableValue<u32> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |e| RevocableValue::Value(e.id))
    }

    /// Returns a list of interfaces attached to this entity, if the entity is still valid.
    pub fn interfaces(&self) -> RevocableResult<Vec<MediaControllerInterface>, io::Error> {
        let inner_ref = self.0.borrow();
        let inner = try_option_to_result!(inner_ref.try_access());
        let controller = inner.controller.borrow();

        let mut itf_ids = Vec::new();
        for link in &controller.links {
            let link_ref = link.borrow();
            let link_inner = try_option_to_result!(link_ref.try_access());

            if link_inner.sink_id == inner.id {
                itf_ids.push(link_inner.source_id);
            }
        }

        let mut out_itfs = Vec::new();
        for itf in &controller.interfaces {
            let itf_ref = itf.borrow();
            let itf_inner = try_option_to_result!(itf_ref.try_access());

            if itf_ids.contains(&itf_inner.id) {
                out_itfs.push(MediaControllerInterface(itf.clone()));
            }
        }

        RevocableResult::Ok(out_itfs)
    }

    /// Returns whether this entity is a connector or not, if the entity is still valid.
    pub fn is_connector(&self) -> RevocableValue<bool> {
        self.flags()
            .map(|f| f.contains(MediaControllerEntityFlags::CONNECTOR))
    }

    /// Returns whether this entity is the default entity for its function, if the entity is still
    /// valid.
    pub fn is_default(&self) -> RevocableValue<bool> {
        self.flags()
            .map(|f| f.contains(MediaControllerEntityFlags::DEFAULT))
    }

    /// Returns whether this entity is a v4l2 Device or not, if the entity is still
    /// valid.
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn is_v4l2_device(&self) -> RevocableResult<bool, io::Error> {
        let interfaces = try_result!(self.interfaces());

        for itf in interfaces {
            let itf_kind = try_value!(itf.kind());

            if matches!(
                itf_kind,
                MediaControllerInterfaceKind::V4L(MediaControllerInterfaceV4lKind::Video)
            ) {
                return RevocableResult::Ok(true);
            }
        }

        RevocableResult::Ok(false)
    }

    /// Returns whether this entity is a v4l2 sub-device or not, if the entity is still
    /// valid.
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn is_v4l2_sub_device(&self) -> RevocableResult<bool, io::Error> {
        let interfaces = try_result!(self.interfaces());

        for itf in interfaces {
            let itf_kind = try_value!(itf.kind());

            if matches!(
                itf_kind,
                MediaControllerInterfaceKind::V4L(MediaControllerInterfaceV4lKind::Subdev)
            ) {
                return RevocableResult::Ok(true);
            }
        }

        RevocableResult::Ok(false)
    }

    /// Returns this entity name, if the entity is still valid.
    pub fn name(&self) -> RevocableValue<String> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |e| {
                RevocableValue::Value(e.name.clone())
            })
    }

    /// Returns this entity pads number, if the entity is still valid.
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn num_pads(&self) -> RevocableResult<usize, io::Error> {
        self.pads().map(|f| f.len())
    }

    /// Returns this entity pad located at the given index if it exists, and if the entity is still
    /// valid.
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn pad(&self, idx: u32) -> RevocableResult<Option<MediaControllerPad>, io::Error> {
        for pad in try_result!(self.pads()) {
            if idx == try_value!(pad.index()) {
                return RevocableResult::Ok(Some(pad));
            }
        }

        RevocableResult::Ok(None)
    }

    /// Returns a list of pads attached to this entity, if the entity is still valid.
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn pads(&self) -> RevocableResult<Vec<MediaControllerPad>, io::Error> {
        if let Some(inner) = self.0.borrow().try_access() {
            let controller: MediaController = inner.controller.clone().into();
            RevocableResult::Ok(
                try_result_to_revocable!(controller.pads())
                    .into_iter()
                    .filter(|p| p.entity_id() == RevocableValue::Value(inner.id))
                    .collect(),
            )
        } else {
            RevocableResult::Revoked
        }
    }
}

struct MediaControllerInterfaceInner {
    _controller: Rc<RefCell<MediaControllerInner>>,
    id: u32,
    kind: MediaControllerInterfaceKind,
    device_node: Option<DeviceNode>,
}

impl fmt::Debug for MediaControllerInterfaceInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaControllerInterfaceInner")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("device_node", &self.device_node)
            .finish_non_exhaustive()
    }
}

/// A Representation of a Media Controller Interface
#[derive(Clone, Debug)]
pub struct MediaControllerInterface(Rc<RefCell<Revocable<MediaControllerInterfaceInner>>>);

impl MediaControllerInterface {
    /// Returns this interface id, if the interface is still valid.
    pub fn id(&self) -> RevocableValue<u32> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |i| RevocableValue::Value(i.id))
    }

    /// Returns this interface kind, if the interface is still valid.
    ///
    /// # Panics
    ///
    /// If the interface kind returned by the kernel is unknown
    pub fn kind(&self) -> RevocableValue<MediaControllerInterfaceKind> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |i| RevocableValue::Value(i.kind))
    }

    /// Returns this interface device node if it exists, and if the interface is still
    /// valid.
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn device_node(&self) -> RevocableValue<Option<DeviceNode>> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |i| {
                RevocableValue::Value(i.device_node.clone())
            })
    }
}

/// A representation of the kind of a `MediaControllerPad`
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaControllerPadKind {
    /// Sink Pad
    Sink,

    /// Source Pad
    Source,
}

impl MediaControllerPadKind {
    /// Returns the other `MediaControllerKind` for a given instance
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Self::Sink => Self::Source,
            Self::Source => Self::Sink,
        }
    }
}

impl From<MediaControllerPadFlags> for MediaControllerPadKind {
    fn from(value: MediaControllerPadFlags) -> Self {
        if value.contains(MediaControllerPadFlags::SINK) {
            Self::Sink
        } else if value.contains(MediaControllerPadFlags::SOURCE) {
            Self::Source
        } else {
            unreachable!()
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct MediaControllerPadFlags: u32 {
        const SINK = raw::bindgen::MEDIA_PAD_FL_SINK;
        const SOURCE = raw::bindgen::MEDIA_PAD_FL_SOURCE;
        const MUST_CONNECT = raw::bindgen::MEDIA_PAD_FL_MUST_CONNECT;
    }
}

impl From<u32> for MediaControllerPadFlags {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}

struct MediaControllerPadInner {
    controller: Rc<RefCell<MediaControllerInner>>,
    entity_id: u32,
    id: u32,
    index: u32,
    flags: u32,
}

impl fmt::Debug for MediaControllerPadInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaControllerPadInner")
            .field("entity_id", &self.entity_id)
            .field("id", &self.id)
            .field("index", &self.index)
            .field("flags", &self.flags)
            .finish_non_exhaustive()
    }
}

/// A Representation of a Media Controller Pad
#[derive(Clone, Debug)]
pub struct MediaControllerPad(Rc<RefCell<Revocable<MediaControllerPadInner>>>);

impl MediaControllerPad {
    /// Returns the entity this pad is attached to, if the pad is still valid.
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn entity(&self) -> RevocableResult<MediaControllerEntity, io::Error> {
        let inner_ref = self.0.borrow();
        let inner = try_option_to_result!(inner_ref.try_access());
        let controller = inner.controller.borrow();

        for entity in &controller.entities {
            let entity_ref = entity.borrow();
            let entity_inner = try_option_to_result!(entity_ref.try_access());
            if inner.entity_id == entity_inner.id {
                return RevocableResult::Ok(MediaControllerEntity(entity.clone()));
            }
        }

        unreachable!("A pad is always attached to an entity.");
    }

    /// Returns the entity ID this pad is connected to, if the pad is still valid.
    pub fn entity_id(&self) -> RevocableValue<u32> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |p| {
                RevocableValue::Value(p.entity_id)
            })
    }

    fn flags(&self) -> RevocableValue<MediaControllerPadFlags> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |p| {
                RevocableValue::Value(p.flags.into())
            })
    }

    /// Returns an iterator over the flag names set for this pad, if the pad is still valid.
    pub fn flag_names(&self) -> RevocableValue<impl Iterator<Item = &str>> {
        self.flags().map(|f| f.iter_names().map(|(n, _)| n))
    }

    /// Returns this pad ID, if the pad is still valid.
    pub fn id(&self) -> RevocableValue<u32> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |p| RevocableValue::Value(p.id))
    }

    /// Returns the pad index, if the pad is still valid.
    pub fn index(&self) -> RevocableValue<u32> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |p| RevocableValue::Value(p.index))
    }

    /// Returns whether this pad is a sink or not, if the pad is still valid.
    pub fn is_sink(&self) -> RevocableValue<bool> {
        self.flags()
            .map(|f| f.contains(MediaControllerPadFlags::SINK))
    }

    /// Returns whether this pad is a source or not, if the pad is still valid.
    pub fn is_source(&self) -> RevocableValue<bool> {
        self.flags()
            .map(|f| f.contains(MediaControllerPadFlags::SOURCE))
    }

    /// Returns this pad kind, if the pad is still valid.
    pub fn kind(&self) -> RevocableValue<MediaControllerPadKind> {
        self.flags().map(Into::into)
    }

    /// Returns the links to this pad, if the pad is still valid.
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn links(&self) -> RevocableResult<Vec<MediaControllerLink>, io::Error> {
        let pad_kind = try_value!(self.kind());

        let inner_ref = self.0.borrow();
        let inner = try_option_to_result!(inner_ref.try_access());
        let controller = inner.controller.borrow();

        let mut out_links = Vec::new();
        for link in &controller.links {
            let link_ref = link.borrow();
            let link_inner = try_option_to_result!(link_ref.try_access());
            let link_pad_id = match pad_kind {
                MediaControllerPadKind::Sink => link_inner.sink_id,
                MediaControllerPadKind::Source => link_inner.source_id,
            };

            if link_pad_id == inner.id {
                out_links.push(MediaControllerLink(link.clone()));
            }
        }

        RevocableResult::Ok(out_links)
    }

    /// Returns whether this pad must be connected or not, if the pad is still valid.
    pub fn must_connect(&self) -> RevocableValue<bool> {
        self.flags()
            .map(|f| f.contains(MediaControllerPadFlags::MUST_CONNECT))
    }

    /// Returns the other pad this pad is connected to if it exists, and if the pad is still valid.
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn remote_pad(&self) -> RevocableResult<Option<Self>, io::Error> {
        let pad_kind = try_value!(self.kind());

        let inner_ref = self.0.borrow();
        let inner = try_option_to_result!(inner_ref.try_access());
        let controller = inner.controller.borrow();

        let mut link: Option<_> = None;
        for l in &controller.links {
            let link_ref = l.borrow();
            let link_inner = try_option_to_result!(link_ref.try_access());

            let link_pad_id = match pad_kind {
                MediaControllerPadKind::Sink => link_inner.sink_id,
                MediaControllerPadKind::Source => link_inner.source_id,
            };

            if inner.id == link_pad_id {
                link = Some(l);
                break;
            }
        }

        let link = try_option_to_result!(link);
        let link_ref = link.borrow();
        let link_inner = try_option_to_result!(link_ref.try_access());
        let remote_pad_id = match pad_kind {
            MediaControllerPadKind::Sink => link_inner.source_id,
            MediaControllerPadKind::Source => link_inner.sink_id,
        };

        for pad in &controller.pads {
            let pad_ref = pad.borrow();
            let pad_inner = try_option_to_result!(pad_ref.try_access());

            if pad_inner.id == remote_pad_id {
                return RevocableResult::Ok(Some(MediaControllerPad(pad.clone())));
            }
        }

        RevocableResult::Ok(None)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct MediaControllerLinkFlags: u32 {
        const ENABLED = raw::bindgen::MEDIA_LNK_FL_ENABLED;
        const IMMUTABLE = raw::bindgen::MEDIA_LNK_FL_IMMUTABLE;
        const DYNAMIC = raw::bindgen::MEDIA_LNK_FL_DYNAMIC;
    }
}

/// A Representation of a Media Controller Link type
#[derive(Debug, PartialEq)]
pub enum MediaControllerLinkKind {
    /// Data Connection between two pads
    Data,

    /// Association between an interface and its entity
    Interface,

    /// Physical Relationship between two entities.
    Ancillary,
}

impl fmt::Display for MediaControllerLinkKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MediaControllerLinkKind::Data => "Data Link",
            MediaControllerLinkKind::Interface => "Interface Link",
            MediaControllerLinkKind::Ancillary => "Ancillary Link",
        })
    }
}

struct MediaControllerLinkInner {
    _controller: Rc<RefCell<MediaControllerInner>>,
    id: u32,
    source_id: u32,
    sink_id: u32,
    flags: u32,
}

impl fmt::Debug for MediaControllerLinkInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaControllerLinkInner")
            .field("id", &self.id)
            .field("source_id", &self.source_id)
            .field("sink_id", &self.sink_id)
            .field("flags", &self.flags)
            .finish_non_exhaustive()
    }
}

/// A Representation of a Media Controller Link
#[derive(Clone, Debug)]
pub struct MediaControllerLink(Rc<RefCell<Revocable<MediaControllerLinkInner>>>);

impl MediaControllerLink {
    /// Returns an iterator over the flag names set for this links, if the link is still valid.
    pub fn flag_names(&self) -> RevocableValue<impl Iterator<Item = &str>> {
        self.flags_without_kind().map(|f| {
            MediaControllerLinkFlags::from_bits_retain(f)
                .iter_names()
                .map(|(n, _)| n)
        })
    }

    /// Returns this link ID, if the link is still valid.
    pub fn id(&self) -> RevocableValue<u32> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |l| RevocableValue::Value(l.id))
    }

    fn flags(&self) -> RevocableValue<u32> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |l| RevocableValue::Value(l.flags))
    }

    fn flags_without_kind(&self) -> RevocableValue<u32> {
        self.flags()
            .map(|f| f & !raw::bindgen::MEDIA_LNK_FL_LINK_TYPE)
    }

    /// Returns whether this link is dynamic or not, if the link is still valid.
    pub fn is_dynamic(&self) -> RevocableValue<bool> {
        self.flags_without_kind().map(|f| {
            MediaControllerLinkFlags::from_bits_truncate(f)
                .contains(MediaControllerLinkFlags::DYNAMIC)
        })
    }

    /// Returns whether this link is enabled or not, if the link is still valid.
    pub fn is_enabled(&self) -> RevocableValue<bool> {
        self.flags_without_kind().map(|f| {
            MediaControllerLinkFlags::from_bits_truncate(f)
                .contains(MediaControllerLinkFlags::ENABLED)
        })
    }

    /// Returns whether this link is immutable or not, if the link is still valid.
    pub fn is_immutable(&self) -> RevocableValue<bool> {
        self.flags_without_kind().map(|f| {
            MediaControllerLinkFlags::from_bits_truncate(f)
                .contains(MediaControllerLinkFlags::IMMUTABLE)
        })
    }

    /// Returns whether this link kind, if the link is still valid.
    pub fn kind(&self) -> RevocableValue<MediaControllerLinkKind> {
        self.flags().map(|f| {
            let kind = f & raw::bindgen::MEDIA_LNK_FL_LINK_TYPE;

            match kind {
                raw::bindgen::MEDIA_LNK_FL_DATA_LINK => MediaControllerLinkKind::Data,
                raw::bindgen::MEDIA_LNK_FL_INTERFACE_LINK => MediaControllerLinkKind::Interface,
                raw::bindgen::MEDIA_LNK_FL_ANCILLARY_LINK => MediaControllerLinkKind::Ancillary,
                _ => unimplemented!(),
            }
        })
    }

    /// Returns the ID of the sink, if the link is still valid.
    pub fn sink_id(&self) -> RevocableValue<u32> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |l| {
                RevocableValue::Value(l.sink_id)
            })
    }

    /// Returns the ID of the source, if the link is still valid.
    pub fn source_id(&self) -> RevocableValue<u32> {
        self.0
            .borrow()
            .try_access()
            .map_or(RevocableValue::Revoked, |l| {
                RevocableValue::Value(l.source_id)
            })
    }
}

struct GTopologyArgs<'a> {
    prev: media_v2_topology,
    entities: Option<&'a mut Vec<media_v2_entity>>,
    interfaces: Option<&'a mut Vec<media_v2_interface>>,
    pads: Option<&'a mut Vec<media_v2_pad>>,
    links: Option<&'a mut Vec<media_v2_link>>,
}

fn media_ioctl_g_topology(
    fd: BorrowedFd<'_>,
    mut args: Option<GTopologyArgs<'_>>,
) -> Result<media_v2_topology, io::Error> {
    let topo = if let Some(args) = &mut args {
        let mut topo = media_v2_topology::default();

        if let Some(entities) = &mut args.entities {
            entities.clear();
            entities.reserve(args.prev.num_entities as usize);

            topo.num_entities = args.prev.num_entities;
            topo.ptr_entities = entities.as_mut_ptr() as u64;
        }

        if let Some(interfaces) = &mut args.interfaces {
            interfaces.clear();
            interfaces.reserve(args.prev.num_interfaces as usize);

            topo.num_interfaces = args.prev.num_interfaces;
            topo.ptr_interfaces = interfaces.as_mut_ptr() as u64;
        }

        if let Some(pads) = &mut args.pads {
            pads.clear();
            pads.reserve(args.prev.num_pads as usize);

            topo.num_pads = args.prev.num_pads;
            topo.ptr_pads = pads.as_mut_ptr() as u64;
        }

        if let Some(links) = &mut args.links {
            links.clear();
            links.reserve(args.prev.num_pads as usize);

            topo.num_links = args.prev.num_links;
            topo.ptr_links = links.as_mut_ptr() as u64;
        }

        topo
    } else {
        media_v2_topology::default()
    };

    let topo = raw::media_ioctl_g_topology(fd, topo)?;

    if let Some(args) = &mut args {
        if let Some(entities) = &mut args.entities {
            // SAFETY: The kernel has filled the buffer with num_entities entries.
            unsafe { entities.set_len(topo.num_entities as usize) };
        }

        if let Some(interfaces) = &mut args.interfaces {
            // SAFETY: The kernel has filled the buffer with num_interfaces entries.
            unsafe { interfaces.set_len(topo.num_interfaces as usize) };
        }
        if let Some(pads) = &mut args.pads {
            // SAFETY: The kernel has filled the buffer with num_pads entries.
            unsafe { pads.set_len(topo.num_pads as usize) };
        }

        if let Some(links) = &mut args.links {
            // SAFETY: The kernel has filled the buffer with num_links entries.
            unsafe { links.set_len(topo.num_links as usize) };
        }
    }

    Ok(topo)
}

#[derive(Debug)]
struct MediaControllerInner {
    fd: OwnedFd,
    last_topology_version: Option<u64>,
    entities: Vec<Rc<RefCell<Revocable<MediaControllerEntityInner>>>>,
    interfaces: Vec<Rc<RefCell<Revocable<MediaControllerInterfaceInner>>>>,
    links: Vec<Rc<RefCell<Revocable<MediaControllerLinkInner>>>>,
    pads: Vec<Rc<RefCell<Revocable<MediaControllerPadInner>>>>,
}

fn update_topology(
    mc: &Rc<RefCell<MediaControllerInner>>,
    count: Option<media_v2_topology>,
) -> io::Result<()> {
    let mut inner = mc.borrow_mut();

    let count = if let Some(count) = count {
        count
    } else {
        media_ioctl_g_topology(inner.fd.as_fd(), None)?
    };

    let mut raw_entities = Vec::with_capacity(count.num_entities as usize);
    let mut raw_interfaces = Vec::with_capacity(count.num_interfaces as usize);
    let mut raw_links = Vec::with_capacity(count.num_links as usize);
    let mut raw_pads = Vec::with_capacity(count.num_pads as usize);

    let topo = media_ioctl_g_topology(
        inner.fd.as_fd(),
        Some(GTopologyArgs {
            prev: count,
            entities: Some(&mut raw_entities),
            interfaces: Some(&mut raw_interfaces),
            pads: Some(&mut raw_pads),
            links: Some(&mut raw_links),
        }),
    )?;

    inner.last_topology_version = Some(topo.topology_version);

    let entities = raw_entities
        .into_iter()
        .map(|e| {
            let function = e.function;

            Ok(Rc::new(RefCell::new(Revocable::new(
                MediaControllerEntityInner {
                    controller: mc.clone(),
                    id: e.id,
                    name: chars_to_string(&e.name, false),
                    function: function.try_into().map_err(|_e| {
                        io::Error::new(io::ErrorKind::InvalidData, "Unexpected entity function")
                    })?,
                    flags: e.flags.try_into().map_err(|_e| {
                        io::Error::new(io::ErrorKind::InvalidData, "Unexpected entity flag")
                    })?,
                },
            ))))
        })
        .collect::<io::Result<Vec<_>>>()?;

    inner.entities = entities;

    let interfaces = raw_interfaces
        .into_iter()
        .map(|e| {
            let intf_type = e.intf_type;
            Ok(Rc::new(RefCell::new(Revocable::new(
                MediaControllerInterfaceInner {
                    _controller: mc.clone(),
                    id: e.id,
                    kind: intf_type.try_into().map_err(|_e| {
                        io::Error::new(io::ErrorKind::InvalidData, "Unexpected interface type")
                    })?,
                    device_node: {
                        // SAFETY: All known interface types are device node interfaces.
                        let devnode = unsafe { e.__bindgen_anon_1.devnode };

                        DeviceNode::new(devnode.major, devnode.minor).ok()
                    },
                },
            ))))
        })
        .collect::<io::Result<Vec<_>>>()?;

    inner.interfaces = interfaces;

    let pads = raw_pads
        .into_iter()
        .map(|p| {
            Rc::new(RefCell::new(Revocable::new(MediaControllerPadInner {
                controller: mc.clone(),
                entity_id: p.entity_id,
                id: p.id,
                index: p.index,
                flags: p.flags,
            })))
        })
        .collect();

    inner.pads = pads;

    let links = raw_links
        .into_iter()
        .map(|l| {
            Rc::new(RefCell::new(Revocable::new(MediaControllerLinkInner {
                _controller: mc.clone(),
                id: l.id,
                source_id: l.source_id,
                sink_id: l.sink_id,
                flags: l.flags,
            })))
        })
        .collect();

    inner.links = links;

    Ok(())
}

/// A Representation of a Media Controller
#[derive(Clone, Debug)]
pub struct MediaController(Rc<RefCell<MediaControllerInner>>);

impl MediaController {
    /// Creates a new `MediaController` from its media device file
    ///
    /// # Errors
    ///
    /// If the file access fails.
    pub fn new(path: &Path) -> Result<Self, io::Error> {
        let file = File::open(path)?;

        let mc = Rc::new(RefCell::new(MediaControllerInner {
            fd: file.into(),
            last_topology_version: None,
            entities: Vec::new(),
            interfaces: Vec::new(),
            links: Vec::new(),
            pads: Vec::new(),
        }));

        update_topology(&mc, None)?;

        Ok(MediaController(mc))
    }

    /// Returns Media Controller Information
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn info(&self) -> Result<MediaControllerInfo, io::Error> {
        media_ioctl_device_info(self.0.borrow().fd.as_fd()).map(Into::into)
    }

    #[expect(
        clippy::unwrap_in_result,
        reason = "The expect condition can never be true, so there's no point in returning an error."
    )]
    fn check_topology_version(&self) -> io::Result<()> {
        let inner = self.0.borrow();
        let current_version = inner.last_topology_version.expect(
            "After the initial construction in new(), the topology version will always be set",
        );

        let topo = raw::media_ioctl_g_topology(inner.fd.as_fd(), media_v2_topology::default())?;
        if topo.topology_version > current_version {
            update_topology(&self.0, Some(topo))?;
        }

        Ok(())
    }

    /// Returns the current topology version
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    #[expect(
        clippy::missing_panics_doc,
        clippy::unwrap_in_result,
        reason = "This can't happen, so using expect is fine."
    )]
    pub fn topology_version(&self) -> Result<u64, io::Error> {
        self.check_topology_version()?;

        Ok(self
            .0
            .borrow()
            .last_topology_version
            .expect("We just set the version by updating the topology, so it will never be none."))
    }

    /// Return a list of entities
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn entities(&self) -> Result<Vec<MediaControllerEntity>, io::Error> {
        self.check_topology_version()?;

        let inner = self.0.borrow();
        Ok(inner
            .entities
            .iter()
            .map(|e| MediaControllerEntity(e.clone()))
            .collect())
    }

    /// Return a list of interfaces
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn interfaces(&self) -> Result<Vec<MediaControllerInterface>, io::Error> {
        self.check_topology_version()?;

        let inner = self.0.borrow();
        Ok(inner
            .interfaces
            .iter()
            .map(|e| MediaControllerInterface(e.clone()))
            .collect())
    }

    /// Return a list of pads
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn pads(&self) -> Result<Vec<MediaControllerPad>, io::Error> {
        self.check_topology_version()?;

        let inner = self.0.borrow();
        Ok(inner
            .pads
            .iter()
            .map(|e| MediaControllerPad(e.clone()))
            .collect())
    }

    /// Return a list of links
    ///
    /// # Errors
    ///
    /// If the Media Controller device file access fails.
    pub fn links(&self) -> Result<Vec<MediaControllerLink>, io::Error> {
        self.check_topology_version()?;

        let inner = self.0.borrow();
        Ok(inner
            .links
            .iter()
            .map(|e| MediaControllerLink(e.clone()))
            .collect())
    }
}

impl From<Rc<RefCell<MediaControllerInner>>> for MediaController {
    fn from(value: Rc<RefCell<MediaControllerInner>>) -> Self {
        Self(value)
    }
}
