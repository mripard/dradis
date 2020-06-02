extern crate mmap;
extern crate v4lise;

use std::hash::Hasher;
use std::os::unix::io::AsRawFd;
use std::slice;

use byteorder::ByteOrder;
use byteorder::LittleEndian;

use mmap::MapOption;
use mmap::MemoryMap;

use v4lise::v4l2_buf_type;
use v4lise::v4l2_buffer;
use v4lise::v4l2_dequeue_buffer;
use v4lise::v4l2_enum_formats;
use v4lise::v4l2_enum_framesizes;
use v4lise::v4l2_fmtdesc;
use v4lise::v4l2_format;
use v4lise::v4l2_frmsizeenum;
use v4lise::v4l2_memory;
use v4lise::v4l2_query_buffer;
use v4lise::v4l2_queue_buffer;
use v4lise::v4l2_request_buffers;
use v4lise::v4l2_requestbuffers;
use v4lise::v4l2_set_format;
use v4lise::v4l2_start_streaming;
use v4lise::Device;
use v4lise::Format;
use v4lise::Result;
use v4lise::QueueType;

use twox_hash::XxHash32;

struct V4L2Buffer<'a> {
	index: u32,
	mmap: MemoryMap,
	slice: &'a [u8],
}

const BUFFER_TYPE: v4l2_buf_type = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE;
const MEMORY_TYPE: v4l2_memory = v4l2_memory::V4L2_MEMORY_MMAP;
const NUM_BUFFERS: usize = 2;

fn dequeue_buffer(dev: &Device) -> Result<u32> {
	let mut raw_struct: v4l2_buffer = Default::default();
	raw_struct.type_ = BUFFER_TYPE as u32;
	raw_struct.memory = MEMORY_TYPE as u32;

	raw_struct = v4l2_dequeue_buffer(dev, raw_struct)?;

	Ok(raw_struct.index)
}

fn queue_buffer(dev: &Device, idx: usize) -> Result<()> {
	let mut raw_struct: v4l2_buffer = Default::default();
	raw_struct.index = idx as u32;
	raw_struct.type_ = BUFFER_TYPE as u32;
	raw_struct.memory = MEMORY_TYPE as u32;

	v4l2_queue_buffer(dev, raw_struct)?;

	Ok(())
}

fn main() {
	let mut buffers: Vec<V4L2Buffer> = Vec::with_capacity(NUM_BUFFERS);
	let dev = Device::new("/dev/video0").expect("Couldn't open the v4l2 device");
	let queue = dev.get_queue(QueueType::Capture).expect("Couldn't get our queue");

	let mut fmt: Option<Format> = None;
	let mut fmt_idx = 0;
	loop {
		let mut raw_desc: v4l2_fmtdesc = Default::default();
		raw_desc.type_ = BUFFER_TYPE as u32;
		raw_desc.index = fmt_idx;

		match v4l2_enum_formats(&dev, raw_desc) {
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

		match v4l2_enum_framesizes(&dev, raw_struct) {
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

	v4l2_set_format(&dev, raw_fmt).expect("Couldn't set the target format");

	let mut rbuf: v4l2_requestbuffers = Default::default();
	rbuf.count = NUM_BUFFERS as u32;
	rbuf.type_ = BUFFER_TYPE as u32;
	rbuf.memory = MEMORY_TYPE as u32;

	v4l2_request_buffers(&dev, rbuf).expect("Couldn't allocate our buffers");

	for idx in 0..NUM_BUFFERS {
		let mut rbuf: v4l2_buffer = Default::default();
		rbuf.index = idx as u32;
		rbuf.type_ = BUFFER_TYPE as u32;
		rbuf.memory = MEMORY_TYPE as u32;

		rbuf = v4l2_query_buffer(&dev, rbuf).expect("Couldn't query our buffer");

		let mmap = MemoryMap::new(
			rbuf.length as usize,
			&[
				MapOption::MapFd(dev.as_raw_fd()),
				MapOption::MapOffset(unsafe { rbuf.m.offset as usize }),
				MapOption::MapNonStandardFlags(libc::MAP_SHARED),
				MapOption::MapReadable,
			],
		)
		.expect("Couldn't map our buffer");

		let slice = unsafe { slice::from_raw_parts(mmap.data(), rbuf.length as usize) };

		println!("Buffer {} Address {:#?}", idx, mmap.data());

		let buf = V4L2Buffer {
			index: idx as u32,
			mmap,
			slice,
		};

		buffers.push(buf);
		queue_buffer(&dev, idx).expect("Couldn't queue our buffer");
	}

	v4l2_start_streaming(&dev, BUFFER_TYPE).expect("Couldn't start streaming");

	loop {
		let idx = dequeue_buffer(&dev).expect("Couldn't dequeue our buffer");

		let buf = &buffers[idx as usize];
		let ptr = buf.mmap.data();

		println!("Dequeued {} Address {:#?}", idx, ptr);

		let mut hasher = XxHash32::with_seed(0);
		hasher.write(buf.slice);
		println!("Captured buffer: hash {}", hasher.finish());

		queue_buffer(&dev, idx as usize).expect("Couldn't queue our buffer");
	}
}
