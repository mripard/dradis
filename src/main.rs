extern crate glob;
extern crate mmap;
extern crate v4lise;

use std::fs::File;
use std::fs::OpenOptions;
use std::hash::Hasher;
use std::os::unix::io::AsRawFd;
use std::rc::Rc;
use std::slice;

use byteorder::ByteOrder;
use byteorder::LittleEndian;

use mmap::MapOption;
use mmap::MemoryMap;

use v4lise::v4l2_buf_type;
use v4lise::v4l2_buffer;
use v4lise::v4l2_capability;
use v4lise::v4l2_dequeue_buffer;
use v4lise::v4l2_enum_formats;
use v4lise::v4l2_enum_framesizes;
use v4lise::v4l2_fmtdesc;
use v4lise::v4l2_format;
use v4lise::v4l2_frmsizeenum;
use v4lise::v4l2_memory;
use v4lise::v4l2_query_buffer;
use v4lise::v4l2_query_cap;
use v4lise::v4l2_queue_buffer;
use v4lise::v4l2_request_buffers;
use v4lise::v4l2_requestbuffers;
use v4lise::v4l2_set_format;
use v4lise::v4l2_start_streaming;
use v4lise::CapabilitiesFlags;
use v4lise::Result;
use v4lise::Format;

use glob::glob;
use twox_hash::XxHash32;

#[derive(Debug)]
struct V4L2Capability {
	pub driver: String,
	pub card: String,
	pub bus_info: String,
	pub version: u32,
	pub capabilities: CapabilitiesFlags,
	pub device_caps: CapabilitiesFlags,
}

impl From<v4l2_capability> for V4L2Capability {
	fn from(caps: v4l2_capability) -> Self {
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

struct V4L2Buffer<'a> {
	index: u32,
	mmap: MemoryMap,
	slice: &'a [u8],
}

struct V4L2Device<'a> {
	file: File,
	buffers: Vec<Rc<V4L2Buffer<'a>>>,
}

const BUFFER_TYPE: v4l2_buf_type = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE;
const MEMORY_TYPE: v4l2_memory = v4l2_memory::V4L2_MEMORY_MMAP;
const NUM_BUFFERS: usize = 2;

fn dequeue_buffer(dev: &V4L2Device) -> Result<u32> {
	let mut raw_struct: v4l2_buffer = Default::default();
	raw_struct.type_ = BUFFER_TYPE as u32;
	raw_struct.memory = MEMORY_TYPE as u32;

	raw_struct = v4l2_dequeue_buffer(&dev.file, raw_struct)?;

	Ok(raw_struct.index)
}

fn queue_buffer(dev: &V4L2Device, idx: usize) -> Result<()> {
	let mut raw_struct: v4l2_buffer = Default::default();
	raw_struct.index = idx as u32;
	raw_struct.type_ = BUFFER_TYPE as u32;
	raw_struct.memory = MEMORY_TYPE as u32;

	v4l2_queue_buffer(&dev.file, raw_struct)?;

	Ok(())
}

fn main() {
	let file = glob("/dev/video[0-9]*")
		.unwrap()
		.map(|path| {
			OpenOptions::new()
				.read(true)
				.write(true)
				.open(path.unwrap())
				.unwrap()
		})
		.filter(|fd| {
			let raw_caps = v4l2_query_cap(fd).unwrap();
			let caps = V4L2Capability::from(raw_caps);

			caps.device_caps.contains(CapabilitiesFlags::VIDEO_CAPTURE)
		})
		.next()
		.unwrap();

	let mut dev = V4L2Device {
		file,
		buffers: Vec::with_capacity(NUM_BUFFERS),
	};

	let mut fmt: Option<Format> = None;
	let mut fmt_idx = 0;
	loop {
		let mut raw_desc: v4l2_fmtdesc = Default::default();
		raw_desc.type_ = BUFFER_TYPE as u32;
		raw_desc.index = fmt_idx;

		match v4l2_enum_formats(&dev.file, raw_desc) {
			Ok(ret) => {
				let enum_fmt: Format = unsafe { std::mem::transmute(ret.pixelformat as u32) };
				println!("format {:#?}", enum_fmt);

				if enum_fmt == Format::YUYV {
					fmt = Some(enum_fmt);
				}

				fmt_idx += 1;
			}
			Err(_) => break,
		}
	}

	if fmt.is_none() {
		panic!("Couldn't find the YUYV format");
	}

	let mut size_idx = 0;
	loop {
		let mut raw_struct: v4l2_frmsizeenum = Default::default();
		raw_struct.pixel_format = Format::YUYV as u32;
		raw_struct.index = size_idx;

		match v4l2_enum_framesizes(&dev.file, raw_struct) {
			Ok(ret) => {
				println!("size {:#?}", unsafe { ret.__bindgen_anon_1.discrete.width });

				size_idx += 1;
			}
			Err(_) => break,
		}
	}

	let mut raw_fmt: v4l2_format = Default::default();
	raw_fmt.type_ = 1;
	raw_fmt.fmt.pix.width = 320;
	raw_fmt.fmt.pix.height = 240;
	raw_fmt.fmt.pix.pixelformat = Format::YUYV as u32;

	v4l2_set_format(&dev.file, raw_fmt).expect("Couldn't set the target format");

	let mut rbuf: v4l2_requestbuffers = Default::default();
	rbuf.count = NUM_BUFFERS as u32;
	rbuf.type_ = BUFFER_TYPE as u32;
	rbuf.memory = MEMORY_TYPE as u32;

	v4l2_request_buffers(&dev.file, rbuf).expect("Couldn't allocate our buffers");

	for idx in 0..NUM_BUFFERS {
		let mut rbuf: v4l2_buffer = Default::default();
		rbuf.index = idx as u32;
		rbuf.type_ = BUFFER_TYPE as u32;
		rbuf.memory = MEMORY_TYPE as u32;

		rbuf = v4l2_query_buffer(&dev.file, rbuf).expect("Couldn't query our buffer");

		let mmap = MemoryMap::new(
			rbuf.length as usize,
			&[
				MapOption::MapFd(dev.file.as_raw_fd()),
				MapOption::MapOffset(unsafe { rbuf.m.offset as usize }),
				MapOption::MapNonStandardFlags(libc::MAP_SHARED),
				MapOption::MapReadable,
			],
		)
		.expect("Couldn't map our buffer");

		let slice = unsafe { slice::from_raw_parts(mmap.data(), rbuf.length as usize) };

		println!("Buffer {} Address {:#?}", idx, mmap.data());

		let buf = Rc::new(V4L2Buffer {
			index: idx as u32,
			mmap,
			slice,
		});

		dev.buffers.push(buf);
		queue_buffer(&dev, idx).expect("Couldn't queue our buffer");
	}

	v4l2_start_streaming(&dev.file, BUFFER_TYPE).expect("Couldn't start streaming");

	loop {
		let idx = dequeue_buffer(&dev).expect("Couldn't dequeue our buffer");

		let buf = &dev.buffers[idx as usize];
		let ptr = buf.mmap.data();

		println!("Dequeued {} Address {:#?}", idx, ptr);

		let mut hasher = XxHash32::with_seed(0);
		hasher.write(buf.slice);
		println!("Captured buffer: hash {}", hasher.finish());

		queue_buffer(&dev, idx as usize).expect("Couldn't queue our buffer");
	}
}
