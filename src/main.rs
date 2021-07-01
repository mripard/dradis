#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::unreadable_literal)]

use std::{
    convert::TryFrom,
    hash::Hasher,
    os::unix::io::{AsRawFd, RawFd},
    thread::sleep,
    time::{Duration, Instant},
};

use byteorder::{ByteOrder, LittleEndian};
use clap::{App, Arg};
use dma_heap::{DmaBufHeap, DmaBufHeapType};
use edid::{
    EDIDDescriptor, EDIDDetailedTiming, EDIDDetailedTimingDigitalSync, EDIDDetailedTimingSync,
    EDIDDisplayColorEncoding, EDIDDisplayColorTypeEncoding, EDIDVersion,
    EDIDVideoDigitalColorDepth, EDIDVideoDigitalInterface, EDIDVideoDigitalInterfaceStandard,
    EDIDVideoInput, EDIDWeekYear, EDID,
};
use log::{debug, error, info, warn};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use twox_hash::XxHash32;
use v4lise::{
    v4l2_buf_type, v4l2_buffer, v4l2_dequeue_buffer, v4l2_memory, v4l2_query_buffer,
    v4l2_query_dv_timings, v4l2_queue_buffer, v4l2_set_dv_timings, v4l2_set_edid,
    v4l2_start_streaming, Device, FrameFormat, MemoryType, PixelFormat, QueueType, Result,
};

const BUFFER_TYPE: v4l2_buf_type = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE;
const MEMORY_TYPE: v4l2_memory = v4l2_memory::V4L2_MEMORY_DMABUF;
const NUM_BUFFERS: u32 = 5;

const HEADER_VERSION_MAJOR: u8 = 1;
const HEADER_VERSION_MINOR: u8 = 0;
const HEADER_MAGIC: u32 = u32::from_ne_bytes(*b"CRNO");

const FRAMES_DEQUEUED_TIMEOUT: Duration = Duration::from_secs(10);
const NO_VALID_FRAME_TIMEOUT: Duration = Duration::from_secs(2);
const NO_LINK_TIMEOUT: Duration = Duration::from_secs(10);

fn dequeue_buffer(dev: &Device) -> Result<u32> {
    let mut raw_struct = v4l2_buffer {
        type_: BUFFER_TYPE as u32,
        memory: MEMORY_TYPE as u32,
        ..v4l2_buffer::default()
    };

    raw_struct = v4l2_dequeue_buffer(dev, raw_struct)?;

    Ok(raw_struct.index)
}

fn queue_buffer(dev: &Device, idx: u32, fd: RawFd) -> Result<()> {
    let mut raw_struct = v4l2_buffer {
        index: idx,
        type_: BUFFER_TYPE as u32,
        memory: MEMORY_TYPE as u32,
        ..v4l2_buffer::default()
    };
    raw_struct.m.fd = fd;

    v4l2_queue_buffer(dev, raw_struct)?;

    Ok(())
}

fn set_edid(dev: &impl AsRawFd) -> Result<()> {
    let mut edid = EDID::new(EDIDVersion::V1R4)
        .set_manufacturer_id("CRN")
        .set_week_year(EDIDWeekYear::YearOfManufacture(2021))
        .set_input(EDIDVideoInput::Digital(EDIDVideoDigitalInterface::new(
            EDIDVideoDigitalInterfaceStandard::HDMIa,
            EDIDVideoDigitalColorDepth::Depth8bpc,
        )))
        .set_display_color_type_encoding(EDIDDisplayColorTypeEncoding::ColorEncoding(
            EDIDDisplayColorEncoding::RGB444,
        ))
        .set_preferred_timings_native(true)
        .add_descriptor(EDIDDescriptor::DetailedTiming(
            EDIDDetailedTiming::new()
                .set_front_porch(220, 20)
                .set_display(1280, 720)
                .set_sync_pulse(40, 5)
                .set_blanking(370, 30)
                .set_pixel_clock(74250)
                .set_sync_type(EDIDDetailedTimingSync::Digital(
                    EDIDDetailedTimingDigitalSync::Separate(true, true),
                )),
        ))
        .serialize();

    v4l2_set_edid(dev, &mut edid)?;

    Ok(())
}

fn wait_and_set_dv_timings(dev: &impl AsRawFd, width: usize, height: usize) -> Result<()> {
    let start = Instant::now();

    loop {
        if start.elapsed() > NO_LINK_TIMEOUT {
            return Err(v4lise::Error::Empty);
        }

        let timings = v4l2_query_dv_timings(dev);

        match timings {
            Ok(timings) => {
                let bt = unsafe { timings.__bindgen_anon_1.bt };

                if bt.width as usize == width && bt.height as usize == height {
                    info!("Source started to transmit the proper resolution.");
                    let _ = v4l2_set_dv_timings(dev, timings)?;
                    return Ok(());
                }
            }

            Err(e) => match e {
                v4lise::Error::Io(ref io) => match io.raw_os_error() {
                    Some(libc::ENOLCK) => {
                        debug!("Link detected but unstable.");
                    }
                    Some(libc::ENOLINK) => {
                        debug!("No link detected.")
                    }
                    _ => return Err(e),
                },
                _ => return Err(e),
            },
        }

        sleep(Duration::from_millis(100));
    }
}

fn decode_and_check_frame(
    data: &[u8],
    last_idx: Option<usize>,
) -> std::result::Result<usize, dma_buf::Error> {
    let major = data[0];
    let minor = data[1];

    if major != HEADER_VERSION_MAJOR || minor != HEADER_VERSION_MINOR {
        error!(
            "Header Version Mismatch ({}.{} vs {}.{})",
            major, minor, HEADER_VERSION_MAJOR, HEADER_VERSION_MINOR
        );
        return Err(dma_buf::Error::Closure);
    }

    let magic = LittleEndian::read_u32(&data[4..8]);
    if magic != HEADER_MAGIC {
        error!(
            "Header Magic Mismatch ({:#06x} vs {:#06x})",
            magic, HEADER_MAGIC
        );
        return Err(dma_buf::Error::Closure);
    }

    let index = LittleEndian::read_u32(&data[8..12]) as usize;
    if let Some(last) = last_idx {
        if index <= last {
            error!("Frames in invalid order: frame {}, last {}", index, last);
            return Err(dma_buf::Error::Closure);
        }
    }

    let hash = LittleEndian::read_u32(&data[12..16]);

    let mut hasher = XxHash32::with_seed(0);
    hasher.write(&data[16..]);
    let computed_hash = u32::try_from(hasher.finish())
        .expect("Computed Hash was overflowing");

    if hash != computed_hash {
        error!("Frame Corrupted: hash {:#x} vs {:#x}", hash, computed_hash);
        return Err(dma_buf::Error::Closure);
    }

    Ok(index)
}

fn main() {
    let matches = App::new("V4L2 Test Capture Program")
    .arg(Arg::with_name("device")
            .long("device")
            .short("D")
            .help("V4L2 Device File")
            .default_value("/dev/video0"))
    .arg(Arg::with_name("debug")
            .long("debug")
            .short("d")
            .help("Enables debug log level"))
    .arg(Arg::with_name("trace")
            .long("trace")
            .short("t")
            .conflicts_with("debug")
            .help("Enables trace log level"))
    .get_matches();

    let log_level = if matches.is_present("trace") {
        LevelFilter::Trace
    } else if matches.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    TermLogger::init(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Couldn't initialize our logging configuration");

    let heap = DmaBufHeap::new(DmaBufHeapType::Cma)
        .expect("Couldn't open the dma-buf Heap");

    let dev_file = matches.value_of("device").unwrap();
    let dev = Device::new(dev_file, true)
        .expect("Couldn't open the V4L2 Device");

    let queue = dev
        .get_queue(QueueType::Capture)
        .expect("Couldn't get the Capture Queue");

    set_edid(&dev)
        .expect("Couldn't setup the EDID in our bridge");

    wait_and_set_dv_timings(&dev, 1280, 720)
        .expect("Error when retrieving our timings");

    let fmt = queue
        .get_pixel_formats()
        .find(|fmt| *fmt == PixelFormat::RGB24)
        .expect("Couldn't find our format");

    queue
        .set_format(
            queue
                .get_current_format()
                .expect("Couldn't get our queue format")
                .set_pixel_format(fmt),
        )
        .expect("Couldn't change our queue format");

    queue
        .request_buffers(MemoryType::DMABUF, NUM_BUFFERS as usize)
        .expect("Couldn't request our buffers");

    let mut buffers = Vec::with_capacity(NUM_BUFFERS as usize);

    for idx in 0..NUM_BUFFERS {
        let mut rbuf = v4l2_buffer {
            index: idx,
            type_: BUFFER_TYPE as u32,
            memory: MEMORY_TYPE as u32,
            ..v4l2_buffer::default()
        };

        rbuf = v4l2_query_buffer(&dev, rbuf)
            .expect("Couldn't query our buffer");

        let len = rbuf.length as usize;
        let buffer = heap
            .allocate::<dma_buf::DmaBuf>(len)
            .expect("Couldn't allocate our dma-buf buffer")
            .memory_map()
            .expect("Couldn't map our dma-buf buffer");

        queue_buffer(&dev, idx, buffer.as_raw_fd())
            .expect("Couldn't queue our buffer");
        buffers.push(buffer);
    }

    v4l2_start_streaming(&dev, BUFFER_TYPE)
        .expect("Couldn't start streaming");

    let mut last_frame_valid = Instant::now();
    let mut last_frame_index = None;
    loop {
        if last_frame_valid.elapsed() > NO_VALID_FRAME_TIMEOUT {
            panic!("Timeout: no valid frames since {} seconds", NO_VALID_FRAME_TIMEOUT.as_secs());
        }

        let frame_dequeue_start = Instant::now();

        let idx = loop {
            if frame_dequeue_start.elapsed() > FRAMES_DEQUEUED_TIMEOUT {
                break Err(v4lise::Error::Empty);
            }

            let buffer_idx = dequeue_buffer(&dev);
            match buffer_idx {
                Ok(_) => break buffer_idx,
                Err(ref e) => match e {
                    v4lise::Error::Io(io) => match io.raw_os_error() {
                        Some(libc::EAGAIN) => {
                            debug!("No buffer to dequeue.");
                        }
                        _ => break buffer_idx,
                    },
                    _ => break buffer_idx,
                },
            };

            sleep(Duration::from_millis(5));
        }
        .expect("Couldn't dequeue our buffer");

        let buf = &buffers[idx as usize];
        match buf.read(decode_and_check_frame, last_frame_index) {
            Ok(frame_index) => {
                info!("Frame {} Valid", frame_index);
                last_frame_index = Some(frame_index);
                last_frame_valid = Instant::now();
            }
            Err(_) => {
                warn!("Frame Invalid.");
                last_frame_index = None;
            }
        }

        queue_buffer(&dev, idx, buf.as_raw_fd())
            .expect("Couldn't queue our buffer");
    }
}
