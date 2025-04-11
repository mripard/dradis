use std::{
    cell::RefCell,
    ffi::CStr,
    fmt::Display,
    fs::File,
    io,
    mem::MaybeUninit,
    os::fd::{AsFd, BorrowedFd, OwnedFd},
    path::{Path, PathBuf},
    rc::Rc,
};

use bitflags::bitflags;
use rustix::{
    io::Errno,
    ioctl::{Updater, ioctl, opcode},
};
use tracing::warn;

const MEDIA_IOC_MAGIC: u8 = b'|';
const MEDIA_IOC_DEVICE_INFO: u8 = 0x00;
const MEDIA_IOC_G_TOPOLOGY: u8 = 0x04;

#[derive(Clone, Copy, Debug)]
pub struct KernelVersion {
    major: u16,
    minor: u8,
    patch: u8,
}

impl KernelVersion {
    pub fn major(&self) -> u16 {
        self.major
    }

    pub fn minor(&self) -> u8 {
        self.minor
    }

    pub fn patch(&self) -> u8 {
        self.patch
    }
}

impl Display for KernelVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.major, self.minor, self.patch))
    }
}

impl From<u32> for KernelVersion {
    fn from(value: u32) -> Self {
        let major = ((value >> 16) & Into::<u32>::into(u16::MAX))
            .try_into()
            .expect("I'm terrible at math.");

        let minor = ((value >> 8) & Into::<u32>::into(u8::MAX))
            .try_into()
            .expect("I'm terrible at math.");

        let patch = (value & Into::<u32>::into(u8::MAX))
            .try_into()
            .expect("I'm terrible at math.");

        Self {
            major,
            minor,
            patch,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceNode {
    major: u32,
    minor: u32,
    path: PathBuf,
}

impl DeviceNode {
    fn new(major: u32, minor: u32) -> Self {
        Self {
            major,
            minor,
            path: PathBuf::from(&format!("/dev/char/{}:{}", major, minor))
                .canonicalize()
                .unwrap(),
        }
    }

    pub fn major(&self) -> u32 {
        self.major
    }

    pub fn minor(&self) -> u32 {
        self.minor
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaControllerEntityFunction {
    Unknown,

    // DVB Entity Functions
    DtvDemod,
    TsDemux,
    DtvCa,
    DtvNetDecap,

    // I/O Entity Functions
    IoV4l,

    // Sensor Functions
    CamSensor,

    // Tuners

    // Analog TV IF-PLL decoders

    // Audio Entities

    // Processing Entities
    PixelFormatter,
    VideoDecoder,

    // Switches and Bridges
    VideoInterfaceBridge,
}

const MEDIA_ENT_F_BASE: u32 = 0x00000000;
const MEDIA_ENT_F_OLD_BASE: u32 = 0x00010000;
const MEDIA_ENT_F_OLD_SUBDEV_BASE: u32 = 0x00020000;

const MEDIA_ENT_F_UNKNOWN: u32 = MEDIA_ENT_F_BASE;
const MEDIA_ENT_F_IO_V4L: u32 = MEDIA_ENT_F_OLD_BASE + 1;

const MEDIA_ENT_F_CAM_SENSOR: u32 = MEDIA_ENT_F_OLD_SUBDEV_BASE + 1;

const MEDIA_ENT_F_PROC_VIDEO_PIXEL_FORMATTER: u32 = MEDIA_ENT_F_BASE + 0x4002;
const MEDIA_ENT_F_PROC_VIDEO_DECODER: u32 = MEDIA_ENT_F_BASE + 0x4008;

const MEDIA_ENT_F_VID_IF_BRIDGE: u32 = MEDIA_ENT_F_BASE + 0x5002;

impl MediaControllerEntityFunction {
    fn from_u32(v: u32) -> Self {
        match v {
            MEDIA_ENT_F_UNKNOWN => Self::Unknown,
            MEDIA_ENT_F_IO_V4L => Self::IoV4l,
            MEDIA_ENT_F_CAM_SENSOR => Self::CamSensor,
            MEDIA_ENT_F_PROC_VIDEO_PIXEL_FORMATTER => Self::PixelFormatter,
            MEDIA_ENT_F_PROC_VIDEO_DECODER => Self::VideoDecoder,
            MEDIA_ENT_F_VID_IF_BRIDGE => Self::VideoInterfaceBridge,
            val => {
                warn!("Unknown Entity Function: {val}");
                Self::Unknown
            }
        }
    }
}

impl Display for MediaControllerEntityFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::IoV4l => "Data Streaming Entity",
            Self::CamSensor => "Camera Video Sensor Entity",
            Self::Unknown => "Unknown Entity",
            Self::PixelFormatter => "Video Pixel Formatter Entity",
            Self::VideoDecoder => "Video Decoder",
            Self::VideoInterfaceBridge => "Video Interface Bridge",
            _ => unimplemented!(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MediaControllerInterfaceAlsaKind {
    PcmCapture,
    PcmPlayback,
    Control,
}

#[derive(Clone, Copy, Debug)]
pub enum MediaControllerInterfaceDvbKind {
    Fe,
    Demux,
    Dvr,
    Ca,
    Net,
}

#[derive(Clone, Copy, Debug)]
pub enum MediaControllerInterfaceV4lKind {
    Video,
    Vbi,
    Radio,
    Subdev,
    SwRadio,
    Touch,
}

#[derive(Clone, Copy, Debug)]
pub enum MediaControllerInterfaceKind {
    Alsa(MediaControllerInterfaceAlsaKind),
    DVB(MediaControllerInterfaceDvbKind),
    V4L(MediaControllerInterfaceV4lKind),
}

impl MediaControllerInterfaceKind {
    fn from_u32(v: u32) -> Self {
        let kind = v >> 8;
        let subkind = v & <u8 as Into<u32>>::into(u8::MAX);

        match kind {
            1 => Self::DVB(match subkind {
                0 => MediaControllerInterfaceDvbKind::Fe,
                1 => MediaControllerInterfaceDvbKind::Demux,
                2 => MediaControllerInterfaceDvbKind::Dvr,
                3 => MediaControllerInterfaceDvbKind::Ca,
                4 => MediaControllerInterfaceDvbKind::Net,
                _ => todo!(),
            }),
            2 => Self::V4L(match subkind {
                0 => MediaControllerInterfaceV4lKind::Video,
                1 => MediaControllerInterfaceV4lKind::Vbi,
                2 => MediaControllerInterfaceV4lKind::Radio,
                3 => MediaControllerInterfaceV4lKind::Subdev,
                4 => MediaControllerInterfaceV4lKind::SwRadio,
                5 => MediaControllerInterfaceV4lKind::Touch,
                _ => todo!(),
            }),
            3 => Self::Alsa(match subkind {
                0 => MediaControllerInterfaceAlsaKind::PcmCapture,
                1 => MediaControllerInterfaceAlsaKind::PcmPlayback,
                2 => MediaControllerInterfaceAlsaKind::Control,
                _ => todo!(),
            }),
            _ => todo!(),
        }
    }
}

#[repr(C)]
struct media_device_info {
    driver: [u8; 16],
    model: [u8; 32],
    serial: [u8; 40],
    bus_info: [u8; 32],
    media_version: u32,
    hw_revision: u32,
    driver_version: u32,
    _reserved: [u32; 31],
}

impl Default for media_device_info {
    fn default() -> Self {
        // SAFETY: We can zero all the fields.
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

const MEDIA_IOC_DEVICE_INFO_OPCODE: u32 =
    opcode::read_write::<media_device_info>(MEDIA_IOC_MAGIC, MEDIA_IOC_DEVICE_INFO);

fn media_ioctl_device_info(fd: BorrowedFd<'_>) -> Result<media_device_info, io::Error> {
    let mut info = media_device_info::default();

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl,
    // and to implement it properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe {
        ioctl(
            fd,
            Updater::<MEDIA_IOC_DEVICE_INFO_OPCODE, media_device_info>::new(&mut info),
        )
    }
    .map(|_| info)
    .map_err(<Errno as Into<io::Error>>::into)
}

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
    pub fn bus_info(&self) -> &str {
        &self.bus_info
    }

    pub fn driver(&self) -> &str {
        &self.driver
    }

    pub fn driver_version(&self) -> KernelVersion {
        self.driver_version
    }

    pub fn hardware_revision(&self) -> u32 {
        self.hw_revision
    }

    pub fn media_controller_version(&self) -> KernelVersion {
        self.media_version
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn serial(&self) -> &str {
        &self.serial
    }
}

impl From<media_device_info> for MediaControllerInfo {
    fn from(value: media_device_info) -> Self {
        let driver = CStr::from_bytes_until_nul(&value.driver)
            .expect("The kernel guarantees the string is null-terminated.")
            .to_str()
            .expect("The kernel guarantees this is an ASCII string.")
            .to_string();

        let model = CStr::from_bytes_until_nul(&value.model)
            .expect("The kernel guarantees the string is null-terminated.")
            .to_str()
            .expect("The kernel guarantees this is an UTF-8 string.")
            .to_string();

        let serial = CStr::from_bytes_until_nul(&value.serial)
            .expect("The kernel guarantees the string is null-terminated.")
            .to_str()
            .expect("The kernel guarantees this is an ASCII string.")
            .to_string();

        let bus_info = CStr::from_bytes_until_nul(&value.bus_info)
            .expect("The kernel guarantees the string is null-terminated.")
            .to_str()
            .expect("The kernel guarantees this is an ASCII string.")
            .to_string();

        Self {
            driver,
            model,
            serial,
            bus_info,
            media_version: value.media_version.into(),
            hw_revision: value.hw_revision,
            driver_version: value.driver_version.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct media_v2_entity {
    id: u32,
    name: [u8; 64],
    function: u32,
    flags: u32,
    _reserved: [u32; 5],
}

impl Default for media_v2_entity {
    fn default() -> Self {
        // SAFETY: We can zero all the fields.
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MediaControllerEntityFlags: u32 {
        const DEFAULT = 1 << 0;
        const CONNECTOR = 1 << 1;
    }
}

impl From<u32> for MediaControllerEntityFlags {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}

#[derive(Debug)]
struct MediaControllerEntityInner {
    controller: Rc<RefCell<MediaControllerInner>>,
    id: u32,
    name: String,
    function: MediaControllerEntityFunction,
    flags: MediaControllerEntityFlags,
}

#[derive(Clone, Debug)]
pub struct MediaControllerEntity(Rc<RefCell<MediaControllerEntityInner>>);

impl MediaControllerEntity {
    fn from_raw(controller: &MediaController, value: media_v2_entity) -> Self {
        let name = CStr::from_bytes_until_nul(&value.name)
            .expect("The kernel guarantees the string is null-terminated.")
            .to_str()
            .expect("The kernel guarantees this is an UTF-8 string.")
            .to_string();

        Self(Rc::new(RefCell::new(MediaControllerEntityInner {
            controller: controller.0.clone(),
            id: value.id,
            name,
            function: MediaControllerEntityFunction::from_u32(value.function),
            flags: value.flags.into(),
        })))
    }

    pub fn flag_names(&self) -> impl Iterator<Item = &str> {
        self.0.borrow().flags.iter_names().map(|(n, _)| n)
    }

    pub fn function(&self) -> MediaControllerEntityFunction {
        self.0.borrow().function
    }

    pub fn id(&self) -> u32 {
        self.0.borrow().id
    }

    pub fn interfaces(&self) -> Result<Vec<MediaControllerInterface>, io::Error> {
        let inner = self.0.borrow();
        let controller: MediaController = self.0.borrow().controller.clone().into();
        let interfaces_ids = controller
            .links()?
            .into_iter()
            .filter(|l| l.sink_id() == inner.id)
            .map(|l| l.source_id())
            .collect::<Vec<_>>();

        Ok(controller
            .interfaces()?
            .into_iter()
            .filter(|i| interfaces_ids.contains(&i.id()))
            .collect::<Vec<_>>())
    }

    pub fn is_connector(&self) -> bool {
        self.0
            .borrow()
            .flags
            .contains(MediaControllerEntityFlags::CONNECTOR)
    }

    pub fn is_default(&self) -> bool {
        self.0
            .borrow()
            .flags
            .contains(MediaControllerEntityFlags::DEFAULT)
    }

    pub fn is_v4l2_device(&self) -> io::Result<bool> {
        for itf in self.interfaces()? {
            if matches!(
                itf.kind(),
                MediaControllerInterfaceKind::V4L(MediaControllerInterfaceV4lKind::Video)
            ) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn is_v4l2_sub_device(&self) -> io::Result<bool> {
        for itf in self.interfaces()? {
            if matches!(
                itf.kind(),
                MediaControllerInterfaceKind::V4L(MediaControllerInterfaceV4lKind::Subdev)
            ) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn name(&self) -> String {
        self.0.borrow().name.clone()
    }

    pub fn num_pads(&self) -> Result<usize, io::Error> {
        let inner = self.0.borrow();
        let controller: MediaController = inner.controller.clone().into();

        Ok(controller
            .pads()?
            .iter()
            .filter(|p| p.entity_id() == inner.id)
            .count())
    }

    pub fn pad(&self, idx: u32) -> Result<Option<MediaControllerPad>, io::Error> {
        let inner = self.0.borrow();
        let controller: MediaController = inner.controller.clone().into();

        Ok(controller
            .pads()?
            .into_iter()
            .find(|p| p.entity_id() == inner.id && p.index() == idx))
    }

    pub fn pads(&self) -> Result<Vec<MediaControllerPad>, io::Error> {
        let inner = self.0.borrow();
        let controller: MediaController = inner.controller.clone().into();

        Ok(controller
            .pads()?
            .into_iter()
            .filter(|p| p.entity_id() == inner.id)
            .collect())
    }
}

#[repr(C)]
#[derive(Debug)]
struct media_v2_intf_devnode {
    major: u32,
    minor: u32,
}

impl Default for media_v2_intf_devnode {
    fn default() -> Self {
        // SAFETY: We can zero all the fields.
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[repr(C)]
#[derive(Debug)]
struct media_v2_interface {
    id: u32,
    intf_type: u32,
    _flags: u32,
    _reserved1: [u32; 9],
    devnode: media_v2_intf_devnode,
    _reserved2: [u32; 14],
}

impl Default for media_v2_interface {
    fn default() -> Self {
        // SAFETY: We can zero all the fields.
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[derive(Debug)]
struct MediaControllerInterfaceInner {
    id: u32,
    intf_type: MediaControllerInterfaceKind,
    devnode: Option<DeviceNode>,
}

#[derive(Clone, Debug)]
pub struct MediaControllerInterface(Rc<RefCell<MediaControllerInterfaceInner>>);

impl MediaControllerInterface {
    fn from_raw(_controller: &MediaController, value: media_v2_interface) -> Self {
        Self(Rc::new(RefCell::new(MediaControllerInterfaceInner {
            id: value.id,
            intf_type: MediaControllerInterfaceKind::from_u32(value.intf_type),
            devnode: Some(DeviceNode::new(value.devnode.major, value.devnode.minor)),
        })))
    }

    pub fn id(&self) -> u32 {
        self.0.borrow().id
    }

    pub fn kind(&self) -> MediaControllerInterfaceKind {
        self.0.borrow().intf_type
    }

    pub fn device_node(&self) -> Option<DeviceNode> {
        self.0.borrow().devnode.clone()
    }
}

#[repr(C)]
#[derive(Debug)]
struct media_v2_pad {
    id: u32,
    entity_id: u32,
    flags: u32,
    index: u32,
    _reserved: [u32; 4],
}

impl Default for media_v2_pad {
    fn default() -> Self {
        // SAFETY: We can zero all the fields.
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaControllerPadKind {
    Sink,
    Source,
}

impl MediaControllerPadKind {
    pub fn other(self) -> Self {
        match self {
            Self::Sink => Self::Source,
            Self::Source => Self::Sink,
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MediaControllerPadFlags: u32 {
        const SINK = 1 << 0;
        const SOURCE = 1 << 1;
        const MUST_CONNECT = 1 << 2;
    }
}

impl From<u32> for MediaControllerPadFlags {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}

#[derive(Debug)]
struct MediaControllerPadInner {
    controller: Rc<RefCell<MediaControllerInner>>,
    id: u32,
    entity_id: u32,
    flags: MediaControllerPadFlags,
    index: u32,
}

#[derive(Clone, Debug)]
pub struct MediaControllerPad(Rc<RefCell<MediaControllerPadInner>>);

impl MediaControllerPad {
    fn from_raw(controller: &MediaController, value: media_v2_pad) -> Self {
        Self(Rc::new(RefCell::new(MediaControllerPadInner {
            controller: controller.0.clone(),
            id: value.id,
            entity_id: value.entity_id,
            flags: value.flags.into(),
            index: value.index,
        })))
    }

    pub fn entity(&self) -> Result<MediaControllerEntity, io::Error> {
        let inner = self.0.borrow();
        let controller: MediaController = inner.controller.clone().into();

        Ok(controller
            .entities()?
            .into_iter()
            .find(|e| e.id() == inner.entity_id)
            .unwrap())
    }

    pub fn entity_id(&self) -> u32 {
        self.0.borrow().entity_id
    }

    pub fn flags_name(&self) -> impl Iterator<Item = &str> {
        self.0.borrow().flags.iter_names().map(|(n, _)| n)
    }

    pub fn id(&self) -> u32 {
        self.0.borrow().id
    }

    pub fn index(&self) -> u32 {
        self.0.borrow().index
    }

    pub fn is_sink(&self) -> bool {
        self.0
            .borrow()
            .flags
            .contains(MediaControllerPadFlags::SINK)
    }

    pub fn is_source(&self) -> bool {
        self.0
            .borrow()
            .flags
            .contains(MediaControllerPadFlags::SOURCE)
    }

    pub fn kind(&self) -> MediaControllerPadKind {
        if self.is_sink() {
            MediaControllerPadKind::Sink
        } else if self.is_source() {
            MediaControllerPadKind::Source
        } else {
            unreachable!()
        }
    }

    pub fn must_connect(&self) -> bool {
        self.0
            .borrow()
            .flags
            .contains(MediaControllerPadFlags::MUST_CONNECT)
    }

    pub fn remote_pad(&self) -> Result<Option<Self>, io::Error> {
        let inner = self.0.borrow();
        let controller: MediaController = inner.controller.clone().into();

        let link = if let Some(link) = controller.links()?.into_iter().find(|l| {
            inner.id
                == if self.is_source() {
                    l.source_id()
                } else {
                    l.sink_id()
                }
        }) {
            link
        } else {
            return Ok(None);
        };

        let remote_pad_id = if self.is_source() {
            link.sink_id()
        } else {
            link.source_id()
        };

        Ok(controller
            .pads()?
            .into_iter()
            .find(|p| p.id() == remote_pad_id))
    }
}

#[repr(C)]
#[derive(Debug)]
struct media_v2_link {
    id: u32,
    source_id: u32,
    sink_id: u32,
    flags: u32,
    _reserved: [u32; 6],
}

impl Default for media_v2_link {
    fn default() -> Self {
        // SAFETY: We can zero all the fields.
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MediaControllerLinkFlags: u32 {
        const ENABLED = 1 << 0;
        const IMMUTABLE = 1 << 1;
        const DYNAMIC = 1 << 2;
    }
}

#[derive(Debug, PartialEq)]
pub enum MediaControllerLinkKind {
    Data,
    Interface,
    Ancillary,
}

impl Display for MediaControllerLinkKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MediaControllerLinkKind::Data => "Data Link",
            MediaControllerLinkKind::Interface => "Interface Link",
            MediaControllerLinkKind::Ancillary => "Ancillary Link",
        })
    }
}

#[derive(Debug)]
struct MediaControllerLinkInner {
    id: u32,
    source_id: u32,
    sink_id: u32,
    flags: u32,
}

#[derive(Clone, Debug)]
pub struct MediaControllerLink(Rc<RefCell<MediaControllerLinkInner>>);

impl MediaControllerLink {
    fn from_raw(_controller: &MediaController, value: media_v2_link) -> Self {
        Self(Rc::new(RefCell::new(MediaControllerLinkInner {
            id: value.id,
            source_id: value.source_id,
            sink_id: value.sink_id,
            flags: value.flags,
        })))
    }

    pub fn flags_name(&self) -> impl Iterator<Item = &str> {
        let flags = MediaControllerLinkFlags::from_bits_retain(self.flags_without_kind());

        flags.iter_names().map(|(n, _)| n)
    }

    pub fn id(&self) -> u32 {
        self.0.borrow().id
    }

    fn flags_without_kind(&self) -> u32 {
        self.0.borrow().flags & !(0xf << 28)
    }

    pub fn is_dynamic(&self) -> bool {
        let flags = MediaControllerLinkFlags::from_bits_truncate(self.0.borrow().flags);

        flags.contains(MediaControllerLinkFlags::DYNAMIC)
    }

    pub fn is_enabled(&self) -> bool {
        let flags = MediaControllerLinkFlags::from_bits_truncate(self.0.borrow().flags);

        flags.contains(MediaControllerLinkFlags::ENABLED)
    }

    pub fn is_immutable(&self) -> bool {
        let flags = MediaControllerLinkFlags::from_bits_truncate(self.0.borrow().flags);

        flags.contains(MediaControllerLinkFlags::IMMUTABLE)
    }

    pub fn kind(&self) -> MediaControllerLinkKind {
        let kind = (self.0.borrow().flags >> 28) & 0xf;

        match kind {
            0 => MediaControllerLinkKind::Data,
            1 => MediaControllerLinkKind::Interface,
            2 => MediaControllerLinkKind::Ancillary,
            _ => unimplemented!(),
        }
    }

    pub fn sink_id(&self) -> u32 {
        self.0.borrow().sink_id
    }

    pub fn source_id(&self) -> u32 {
        self.0.borrow().source_id
    }
}

#[repr(C)]
#[derive(Debug)]
struct media_v2_topology {
    version: u64,
    num_entities: u32,
    _reserved1: u32,
    ptr_entities: u64,
    num_interfaces: u32,
    _reserved2: u32,
    ptr_interfaces: u64,
    num_pads: u32,
    _reserved3: u32,
    ptr_pads: u64,
    num_links: u32,
    _reserved4: u32,
    ptr_links: u64,
}

impl Default for media_v2_topology {
    fn default() -> Self {
        // SAFETY: We can zero all the fields.
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}

const MEDIA_IOC_G_TOPOLOGY_OPCODE: u32 =
    opcode::read_write::<media_v2_topology>(MEDIA_IOC_MAGIC, MEDIA_IOC_G_TOPOLOGY);

struct GTopologyArgs<'a> {
    prev: media_v2_topology,
    entities: Option<&'a mut Vec<media_v2_entity>>,
    interfaces: Option<&'a mut Vec<media_v2_interface>>,
    pads: Option<&'a mut Vec<media_v2_pad>>,
    links: Option<&'a mut Vec<media_v2_link>>,
}

fn media_ioctl_g_topology(
    fd: BorrowedFd<'_>,
    mut args: Option<GTopologyArgs>,
) -> Result<media_v2_topology, io::Error> {
    let mut topo = if let Some(args) = &mut args {
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

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl,
    // and to implement it properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe {
        ioctl(
            fd,
            Updater::<MEDIA_IOC_G_TOPOLOGY_OPCODE, media_v2_topology>::new(&mut topo),
        )
    }
    .map_err(<Errno as Into<io::Error>>::into)?;

    if let Some(args) = &mut args {
        if let Some(entities) = &mut args.entities {
            unsafe { entities.set_len(topo.num_entities as usize) };
        }

        if let Some(interfaces) = &mut args.interfaces {
            unsafe { interfaces.set_len(topo.num_interfaces as usize) };
        }
        if let Some(pads) = &mut args.pads {
            unsafe { pads.set_len(topo.num_pads as usize) };
        }

        if let Some(links) = &mut args.links {
            unsafe { links.set_len(topo.num_links as usize) };
        }
    }

    Ok(topo)
}

#[derive(Debug)]
struct MediaControllerInner(OwnedFd);

#[derive(Clone)]
pub struct MediaController(Rc<RefCell<MediaControllerInner>>);

impl MediaController {
    pub fn new(path: &Path) -> Result<Self, io::Error> {
        Ok(Self(Rc::new(RefCell::new(MediaControllerInner(
            File::open(path)?.into(),
        )))))
    }

    pub fn info(&self) -> Result<MediaControllerInfo, io::Error> {
        media_ioctl_device_info(self.0.borrow().0.as_fd()).map(Into::into)
    }

    pub fn entities(&self) -> Result<Vec<MediaControllerEntity>, io::Error> {
        let inner = self.0.borrow();
        let fd = inner.0.as_fd();
        let count = media_ioctl_g_topology(fd, None)?;
        let mut raw_entities = Vec::with_capacity(count.num_entities as usize);

        let _ = media_ioctl_g_topology(
            fd,
            Some(GTopologyArgs {
                prev: count,
                entities: Some(&mut raw_entities),
                interfaces: None,
                pads: None,
                links: None,
            }),
        )?;

        Ok(raw_entities
            .into_iter()
            .map(|e| MediaControllerEntity::from_raw(self, e))
            .collect())
    }

    pub fn interfaces(&self) -> Result<Vec<MediaControllerInterface>, io::Error> {
        let inner = self.0.borrow();
        let fd = inner.0.as_fd();
        let count = media_ioctl_g_topology(fd, None)?;
        let mut raw_interfaces = Vec::with_capacity(count.num_interfaces as usize);

        let _ = media_ioctl_g_topology(
            fd,
            Some(GTopologyArgs {
                prev: count,
                entities: None,
                interfaces: Some(&mut raw_interfaces),
                pads: None,
                links: None,
            }),
        )?;

        Ok(raw_interfaces
            .into_iter()
            .map(|e| MediaControllerInterface::from_raw(self, e))
            .collect())
    }

    pub fn pads(&self) -> Result<Vec<MediaControllerPad>, io::Error> {
        let inner = self.0.borrow();
        let fd = inner.0.as_fd();
        let count = media_ioctl_g_topology(fd, None)?;
        let mut raw_pads = Vec::with_capacity(count.num_pads as usize);

        let _ = media_ioctl_g_topology(
            fd,
            Some(GTopologyArgs {
                prev: count,
                entities: None,
                interfaces: None,
                pads: Some(&mut raw_pads),
                links: None,
            }),
        )?;

        Ok(raw_pads
            .into_iter()
            .map(|e| MediaControllerPad::from_raw(self, e))
            .collect())
    }

    pub fn links(&self) -> Result<Vec<MediaControllerLink>, io::Error> {
        let inner = self.0.borrow();
        let fd = inner.0.as_fd();
        let count = media_ioctl_g_topology(fd, None)?;
        let mut raw_links = Vec::with_capacity(count.num_links as usize);

        let _ = media_ioctl_g_topology(
            fd,
            Some(GTopologyArgs {
                prev: count,
                entities: None,
                interfaces: None,
                pads: None,
                links: Some(&mut raw_links),
            }),
        )?;

        Ok(raw_links
            .into_iter()
            .map(|e| MediaControllerLink::from_raw(self, e))
            .collect())
    }
}

impl From<Rc<RefCell<MediaControllerInner>>> for MediaController {
    fn from(value: Rc<RefCell<MediaControllerInner>>) -> Self {
        Self(value)
    }
}
