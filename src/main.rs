#[macro_use]
extern crate bitflags;
extern crate glob;
extern crate memmap;

use std::collections::VecDeque;
use std::convert::TryInto;
use std::fs::File;
use std::fs::OpenOptions;
use std::hash::Hasher;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::slice;
use std::rc::Rc;

use glob::glob;
use libc::ioctl;
use twox_hash::XxHash64;

const VIDIOC_REQBUFS: libc::c_ulong = 0xc0145608;
const VIDIOC_QUERYBUF: libc::c_ulong = 0xc0585609;
const VIDIOC_QUERYCAP: libc::c_ulong = 0x80685600;
const VIDIOC_QBUF: libc::c_ulong = 0xc058560f;
const VIDIOC_DQBUF: libc::c_ulong = 0xc0585611;
const VIDIOC_STREAMON: libc::c_ulong = 0x40045612;
const VIDIOC_STREAMOFF: libc::c_ulong = 0x40045613;

const V4L2_MEMORY_MMAP: u32 = 1;

const V4L2_BUF_TYPE_VIDEO_CAPTURE: u32 = 1;

#[repr(C)]
#[derive(Debug)]
struct c_v4l2_capability {
    pub driver: [u8; 16],
    pub card: [u8; 32],
    pub bus_info: [u8; 32],
    pub version: u32,
    pub capabilities: u32,
    pub device_caps: u32,
    pub reserved: [u32; 3],
}

impl ::std::default::Default for c_v4l2_capability {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

bitflags! {
    struct CapabilitiesFlags: u32 {
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

#[derive(Debug)]
struct V4L2Capability {
    pub driver: String,
    pub card: String,
    pub bus_info: String,
    pub version: u32,
    pub capabilities: CapabilitiesFlags,
    pub device_caps: CapabilitiesFlags,
}

impl From<c_v4l2_capability> for V4L2Capability {
    fn from(caps: c_v4l2_capability) -> Self {
	V4L2Capability {
	    driver: String::from_utf8_lossy(&caps.driver).into_owned(),
	    card: String::from_utf8_lossy(&caps.card).into_owned(),
	    bus_info: String::from_utf8_lossy(&caps.bus_info).into_owned(),
	    version: caps.version,
	    capabilities: CapabilitiesFlags::from_bits_truncate(caps.capabilities),
	    device_caps: CapabilitiesFlags::from_bits_truncate(caps.device_caps),
	}
    }
}

fn get_caps(device: &File) -> Result<V4L2Capability, ()> {
    let mut caps: c_v4l2_capability = Default::default();

    let result = unsafe { ioctl(device.as_raw_fd(), VIDIOC_QUERYCAP, &mut caps) };
    if result < 0 {
	panic!("AAAAAAH");
    }

    Ok(V4L2Capability::from(caps))
}

#[repr(C)]
#[derive(Debug)]
struct c_v4l2_requestbuffers {
    pub count: u32,
    pub r#type: u32,
    pub memory: u32,
    pub capabilities: u32,
    pub reserved: u32,
}

impl ::std::default::Default for c_v4l2_requestbuffers {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[repr(C)]
union c_v4l2_buffer_m {
    pub offset: u32,
    pub userptr: libc::c_ulong,
    pub planes: usize,
    pub fd: i32,
}

impl ::std::fmt::Debug for c_v4l2_buffer_m {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
	unsafe {
	    f.write_fmt(format_args!("{{ offset: {} }}", self.offset))
	}
    }
}

#[repr(C)]
#[derive(Debug)]
struct c_timeval {
    pub tv_sec: libc::c_long,
    pub tv_nsec: libc::c_long,
}

#[repr(C)]
#[derive(Debug)]
struct c_v4l2_timecode {
    pub r#type: u32,
    pub flags: u32,
    pub frames: u8,
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub userbits: [u8; 4],
}

#[repr(C)]
#[derive(Debug)]
struct c_v4l2_buffer {
    pub index: u32,
    pub r#type: u32,
    pub bytesused: u32,
    pub flags: u32,
    pub field: u32,
    pub timestamp: c_timeval,
    pub timecode: c_v4l2_timecode,
    pub sequence: u32,
    pub memory: u32,
    pub m: c_v4l2_buffer_m,
    pub length: u32,
    pub reserved2: u32,
    pub request_fd: i32,
}

impl ::std::default::Default for c_v4l2_buffer {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

bitflags! {
    struct BufferFlags: u32 {
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

struct V4L2Buffer<'a> {
    index: u32,
    ptr: &'a [u8],
}

fn alloc_buffers(device: &mut V4L2Device) -> Result<(), ()> {
    let mut rbufs: c_v4l2_requestbuffers = Default::default();

    rbufs.count = 16;
    rbufs.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    rbufs.memory = V4L2_MEMORY_MMAP;

    let result = unsafe { ioctl(device.file.as_raw_fd(), VIDIOC_REQBUFS, &mut rbufs) };
    if result < 0 {
	panic!("AAAAAAH");
    }

    for index in 0..16 {
	let mut c_buf: c_v4l2_buffer = Default::default();

	c_buf.index = index;
	c_buf.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
	c_buf.memory = V4L2_MEMORY_MMAP;

	let result = unsafe { ioctl(device.file.as_raw_fd(), VIDIOC_QUERYBUF, &mut c_buf) };
	if result < 0 {
	    panic!("AAAAAAAH");
	}

	println!("{:?}", c_buf);
	println!("{:?}", BufferFlags::from_bits_truncate(c_buf.flags));

	let ptr = unsafe {
	    let mmap = libc::mmap(ptr::null_mut(), c_buf.length as usize,
				  libc::PROT_READ, libc::MAP_SHARED,
				  device.file.as_raw_fd(),
				  c_buf.m.offset as i64) as *const u8;
	    slice::from_raw_parts(mmap, c_buf.length as usize)
	};

	let buf = Rc::new(V4L2Buffer {
	    index: index,
	    ptr: ptr as &[u8],
	});

	device.buffers.push(buf.clone());
	device.pending.push_back(buf.clone());
    }

    Ok(())
}

struct V4L2Device<'a> {
    file: File,
    buffers: Vec<Rc<V4L2Buffer<'a>>>,
    queued: VecDeque<Rc<V4L2Buffer<'a>>>,
    pending: VecDeque<Rc<V4L2Buffer<'a>>>,
}

fn queue_buffer(dev: &mut V4L2Device) {
    let mut buf = dev.pending.pop_front().unwrap();
    let mut c_buf: c_v4l2_buffer = Default::default();

    c_buf.index = buf.index;
    c_buf.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    c_buf.memory = V4L2_MEMORY_MMAP;

    let result = unsafe { ioctl(dev.file.as_raw_fd(), VIDIOC_QBUF, &c_buf) };
    if result < 0 {
	panic!("AAAAAAH");
    }

    dev.queued.push_back(buf);
}

fn dequeue_buffer(dev: &mut V4L2Device) {
    let mut c_buf: c_v4l2_buffer = Default::default();

    c_buf.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    c_buf.memory = V4L2_MEMORY_MMAP;

    let result = unsafe { ioctl(dev.file.as_raw_fd(), VIDIOC_DQBUF, &c_buf) };
    if result < 0 {
	panic!("AAAAAAH");
    }

    let buf = &dev.buffers[c_buf.index as usize];
    let mut hasher = XxHash64::with_seed(42);

    hasher.write(buf.ptr);

    println!("Captured buffer, hash {}", hasher.finish());

    dev.pending.push_back(buf.clone());
}

fn start_streaming(dev: &mut V4L2Device) {
    let arg: u32 = V4L2_BUF_TYPE_VIDEO_CAPTURE;

    let result = unsafe { ioctl(dev.file.as_raw_fd(), VIDIOC_STREAMON, &arg) };
    if result < 0 {
	panic!("AAAAAAH");
    }
}

fn main() {
    let file = glob("/dev/video[0-9]*").unwrap()
	.map(|path| {
	    OpenOptions::new()
		.read(true)
		.write(true)
		.open(path.unwrap())
		.unwrap()
	})
	.filter(|fd| {
	    let caps = get_caps(&fd).unwrap();

	    caps.device_caps.contains(CapabilitiesFlags::VIDEO_CAPTURE)
	})
	.next()
	.unwrap();

    let mut dev = V4L2Device {
	file: file,
	pending: VecDeque::new(),
	queued: VecDeque::new(),
	buffers: Vec::new(),
    };

    let _ = alloc_buffers(&mut dev);

    for _ in 0..16 {
	queue_buffer(&mut dev);
    }

    start_streaming(&mut dev);

    loop {
	dequeue_buffer(&mut dev);
	queue_buffer(&mut dev);
    }
}
