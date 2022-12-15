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
    fs::File,
    hash::Hasher,
    os::unix::io::{AsRawFd, RawFd},
    thread::sleep,
    time::{Duration, Instant},
};

use clap::{App, Arg};
use dma_buf::{DmaBuf, MappedDmaBuf};
use dma_heap::{Heap, HeapKind};
use edid::{
    EDIDDescriptor, EDIDDetailedTiming, EDIDDetailedTimingDigitalSync, EDIDDetailedTimingSync,
    EDIDDisplayColorEncoding, EDIDDisplayColorTypeEncoding, EDIDVersion,
    EDIDVideoDigitalColorDepth, EDIDVideoDigitalInterface, EDIDVideoDigitalInterfaceStandard,
    EDIDVideoInput, EDIDWeekYear, EDID,
};
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgb, Rgba};
use log::{debug, info, warn};
use rqrr::PreparedImage;
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use strum_macros::Display;
use twox_hash::XxHash64;
use v4lise::{
    v4l2_buf_type, v4l2_buffer, v4l2_dequeue_buffer, v4l2_memory, v4l2_query_buffer,
    v4l2_query_dv_timings, v4l2_queue_buffer, v4l2_set_dv_timings, v4l2_set_edid,
    v4l2_start_streaming, Device, FrameFormat, MemoryType, PixelFormat, Queue, QueueType, Result,
};

const BUFFER_TYPE: v4l2_buf_type = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE;
const MEMORY_TYPE: v4l2_memory = v4l2_memory::V4L2_MEMORY_DMABUF;
const NUM_BUFFERS: u32 = 5;

const HEADER_VERSION_MAJOR: u8 = 2;

const FRAMES_DEQUEUED_TIMEOUT: Duration = Duration::from_secs(10);
const NO_LINK_TIMEOUT: Duration = Duration::from_secs(10);

const fn default_valid_frame_timeout() -> Duration {
    Duration::from_secs(10)
}

#[derive(Debug, Deserialize)]
struct TestEdidDetailedTiming {
    clock: usize,
    hfp: usize,
    hdisplay: usize,
    hbp: usize,
    hsync: usize,
    vfp: usize,
    vdisplay: usize,
    vbp: usize,
    vsync: usize,
}

#[derive(Debug, Deserialize)]
enum TestEdid {
    #[serde(rename = "detailed-timing")]
    DetailedTiming(TestEdidDetailedTiming),
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct TestItem {
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    #[serde(default)]
    duration: Option<Duration>,

    #[serde(rename = "expected-height")]
    expected_height: usize,

    #[serde(rename = "expected-width")]
    expected_width: usize,

    edid: TestEdid,
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct Test {
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_valid_frame_timeout")]
    valid_frame_timeout: Duration,

    tests: Vec<TestItem>,
}

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

fn set_edid(dev: &impl AsRawFd, edid: &TestEdid) -> Result<()> {
    let mut test_edid = EDID::new(EDIDVersion::V1R4)
        .set_manufacturer_id("CRN")
        .set_week_year(EDIDWeekYear::YearOfManufacture(2021))
        .set_input(EDIDVideoInput::Digital(EDIDVideoDigitalInterface::new(
            EDIDVideoDigitalInterfaceStandard::HDMIa,
            EDIDVideoDigitalColorDepth::Depth8bpc,
        )))
        .set_display_color_type_encoding(EDIDDisplayColorTypeEncoding::ColorEncoding(
            EDIDDisplayColorEncoding::RGB444,
        ))
        .set_preferred_timings_native(true);

    test_edid = match edid {
        TestEdid::DetailedTiming(dtd) => {
            let hblanking = dtd.hfp + dtd.hsync + dtd.hbp;
            let vblanking = dtd.vfp + dtd.vsync + dtd.vbp;

            test_edid.add_descriptor(EDIDDescriptor::DetailedTiming(
                EDIDDetailedTiming::new()
                    .set_front_porch(dtd.hfp as u16, dtd.vfp as u16)
                    .set_display(dtd.hdisplay as u16, dtd.vdisplay as u16)
                    .set_sync_pulse(dtd.hsync as u16, dtd.vsync as u16)
                    .set_blanking(hblanking as u16, vblanking as u16)
                    .set_pixel_clock(dtd.clock as u32)
                    .set_sync_type(EDIDDetailedTimingSync::Digital(
                        EDIDDetailedTimingDigitalSync::Separate(true, true),
                    )),
            ))
        }
    };

    v4l2_set_edid(dev, &mut test_edid.serialize())?;

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

#[derive(Display, Debug)]
enum FrameError {
    Invalid,
}

impl std::error::Error for FrameError {}

#[derive(Deserialize)]
struct Metadata {
    version: (u8, u8),
    qrcode_width: usize,
    qrcode_height: usize,
    width: usize,
    height: usize,
    hash: u64,
    index: u64,
}

fn decode_and_check_frame(
    data: &[u8],
    args: Option<(Option<usize>, usize, usize)>,
) -> std::result::Result<usize, Box<dyn std::error::Error>> {
    let args = args.unwrap();
    let last_frame_index = args.0;
    let width = args.1;
    let height = args.2;

    let pixels = data.to_vec();
    let buffer =
        ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width as u32, height as u32, pixels).unwrap();

    let mut image = DynamicImage::ImageRgb8(buffer);
    let luma = image.to_luma8().view(0, 0, 128, 128).to_image();
    let mut prepared = PreparedImage::prepare(luma);

    let grids = prepared.detect_grids();
    if grids.len() != 1 {
        warn!("Didn't find a QR Code");
        return Err(FrameError::Invalid)?;
    }

    let grid = &grids[0];
    let (_, content) = grid.decode().unwrap();

    let metadata: Metadata = serde_json::from_str(&content).unwrap();

    if metadata.version.0 != HEADER_VERSION_MAJOR {
        warn!("Metadata Version Mismatch");
        return Err(FrameError::Invalid)?;
    }

    if let Some(last_index) = last_frame_index {
        let index = metadata.index as usize;

        if index < last_index {
            warn!("Frame Index Mismatch");
            return Err(FrameError::Invalid)?;
        } else if index == last_index {
            debug!("Source cannot keep up?");
        } else if index > last_index + 1 {
            warn!("Dropped Frame!");
        }
    }

    for x in 0..metadata.qrcode_width {
        for y in 0..metadata.qrcode_height {
            image.put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 255]));
        }
    }

    let mut hasher = XxHash64::with_seed(0);
    hasher.write(&image.as_bytes());
    let hash = hasher.finish();

    if hash != metadata.hash {
        warn!(
            "Hash mismatch: {:#x} vs expected {:#x}",
            hash, metadata.hash
        );
        return Err(FrameError::Invalid)?;
    }

    Ok(metadata.index as usize)
}

fn test_display_one_mode(dev: &Device, queue: &Queue<'_>, heap: &Heap, suite: &Test, test: &TestItem) {
    set_edid(dev, &test.edid).expect("Couldn't setup the EDID in our bridge");

    wait_and_set_dv_timings(dev, test.expected_width, test.expected_height)
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

        rbuf = v4l2_query_buffer(dev, rbuf).expect("Couldn't query our buffer");

        let len = rbuf.length as usize;
        let buffer = heap
            .allocate(len)
            .expect("Couldn't allocate our dma-buf buffer");

        let buffer: MappedDmaBuf = DmaBuf::from(buffer)
            .memory_map()
            .expect("Couldn't map our dma-buf buffer");

        queue_buffer(&dev, idx, buffer.as_raw_fd()).expect("Couldn't queue our buffer");
        buffers.push(buffer);
    }

    v4l2_start_streaming(dev, BUFFER_TYPE).expect("Couldn't start streaming");

    let start = Instant::now();
    let mut first_frame_valid = None;
    let mut last_frame_valid = None;
    let mut last_frame_index = None;
    loop {
        if last_frame_valid.is_none() && start.elapsed() > suite.valid_frame_timeout {
            panic!(
                "Timeout: no valid frames since {} seconds",
                suite.valid_frame_timeout.as_secs()
            );
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

        let frame_decode_start = Instant::now();

        let buf = &buffers[idx as usize];
        match buf.read(
            decode_and_check_frame,
            Some((last_frame_index, test.expected_width, test.expected_height)),
        ) {
            Ok(frame_index) => {
                debug!("Frame {} Valid", frame_index);
                if first_frame_valid.is_none() {
                    first_frame_valid = Some(Instant::now());
                }

                last_frame_index = Some(frame_index);
                last_frame_valid = Some(Instant::now());
            }
            Err(_) => {
                warn!("Frame Invalid.");
                last_frame_index = None;
            }
        }

        debug!(
            "Took {} ms to process the frame",
            frame_decode_start.elapsed().as_millis()
        );

        queue_buffer(&dev, idx, buf.as_raw_fd()).expect("Couldn't queue our buffer");

        if let Some(duration) = test.duration {
            if let Some(first) = first_frame_valid {
                if first.elapsed() > duration {
                    info!("Test Passed");
                    break;
                }
            }
        }
    }
}

fn main() {
    let matches = App::new("DRADIS DRM/KMS Test Program")
        .arg(
            Arg::with_name("device")
                .long("device")
                .short("D")
                .help("V4L2 Device File")
                .default_value("/dev/video0"),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("Enables debug log level"),
        )
        .arg(
            Arg::with_name("trace")
                .long("trace")
                .short("t")
                .conflicts_with("debug")
                .help("Enables trace log level"),
        )
        .arg(
            Arg::with_name("test")
                .required(true)
                .help("Test Configuration File"),
        )
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

    let test_path = matches
        .value_of("test")
        .expect("Couldn't get test file path.");
    let test_file = File::open(test_path).expect("Coludn't open the test file.");

    let test_config: Test =
        serde_yaml::from_reader(test_file).expect("Couldn't parse the test file.");

    let heap = Heap::new(HeapKind::Cma).expect("Couldn't open the dma-buf Heap");

    let dev_file = matches.value_of("device").unwrap();
    let dev = Device::new(dev_file, true).expect("Couldn't open the V4L2 Device");

    let queue = dev
        .get_queue(QueueType::Capture)
        .expect("Couldn't get the Capture Queue");

    for test in &test_config.tests {
        test_display_one_mode(&dev, &queue, &heap, &test_config, &test);
    }
}
