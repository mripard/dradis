#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::unreadable_literal)]

mod frame_check;
mod helpers;

use std::{
    fs::File,
    os::unix::io::AsRawFd,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::Context;
use clap::{Arg, Command};
use dma_buf::{DmaBuf, MappedDmaBuf};
use dma_heap::{Heap, HeapKind};
use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use thiserror::Error;
use v4lise::{
    v4l2_buf_type, v4l2_buffer, v4l2_memory, v4l2_query_buffer, v4l2_start_streaming, Device,
    FrameFormat, MemoryType, PixelFormat, Queue, QueueType,
};

use crate::{
    frame_check::decode_and_check_frame,
    helpers::{dequeue_buffer, queue_buffer, set_edid, wait_and_set_dv_timings},
};

const BUFFER_TYPE: v4l2_buf_type = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE;
const MEMORY_TYPE: v4l2_memory = v4l2_memory::V4L2_MEMORY_DMABUF;
const NUM_BUFFERS: u32 = 5;

const FRAMES_DEQUEUED_TIMEOUT: Duration = Duration::from_secs(10);

const fn default_timeout() -> Duration {
    Duration::from_secs(10)
}

#[derive(Debug, Error)]
enum TestError {
    #[error("No Frame Received")]
    NoFrameReceived,

    #[error("Test Setup Failed: {}", .reason)]
    SetupFailed {
        reason: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

fn test_display_one_mode(
    suite: &Dradis<'_>,
    test: &TestItem,
) -> std::result::Result<(), TestError> {
    set_edid(suite.dev, &test.edid).map_err(|e| TestError::SetupFailed {
        reason: String::from("Couldn't set the EDID on the bridge"),
        source: Some(Box::new(e)),
    })?;

    wait_and_set_dv_timings(suite, test.expected_width, test.expected_height).map_err(|e| {
        TestError::SetupFailed {
            reason: String::from("Couldn't set or retrieve the timings detected by the bridge"),
            source: Some(Box::new(e)),
        }
    })?;

    let fmt = suite
        .queue
        .get_pixel_formats()
        .find(|fmt| *fmt == PixelFormat::RGB24)
        .expect("Couldn't find our format");

    suite
        .queue
        .set_format(
            suite
                .queue
                .get_current_format()
                .expect("Couldn't get our queue format")
                .set_pixel_format(fmt),
        )
        .expect("Couldn't change our queue format");

    suite
        .queue
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

        rbuf = v4l2_query_buffer(suite.dev, rbuf).expect("Couldn't query our buffer");

        let len = rbuf.length as usize;
        let buffer = suite
            .heap
            .allocate(len)
            .expect("Couldn't allocate our dma-buf buffer");

        let buffer: MappedDmaBuf = DmaBuf::from(buffer)
            .memory_map()
            .expect("Couldn't map our dma-buf buffer");

        queue_buffer(suite.dev, idx, buffer.as_raw_fd()).expect("Couldn't queue our buffer");
        buffers.push(buffer);
    }

    v4l2_start_streaming(suite.dev, BUFFER_TYPE).expect("Couldn't start streaming");

    let start = Instant::now();
    let mut first_frame_valid = None;
    let mut last_frame_valid = None;
    let mut last_frame_index = None;
    loop {
        if last_frame_valid.is_none() && start.elapsed() > suite.cfg.valid_frame_timeout {
            error!(
                "Timeout: no valid frames since {} seconds",
                suite.cfg.valid_frame_timeout.as_secs()
            );

            return Err(TestError::NoFrameReceived);
        }

        let frame_dequeue_start = Instant::now();

        let idx = loop {
            if frame_dequeue_start.elapsed() > FRAMES_DEQUEUED_TIMEOUT {
                return Err(TestError::NoFrameReceived);
            }

            let buffer_idx = dequeue_buffer(suite.dev);
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

        queue_buffer(suite.dev, idx, buf.as_raw_fd()).expect("Couldn't queue our buffer");

        if let Some(duration) = test.duration {
            if let Some(first) = first_frame_valid {
                if first.elapsed() > duration {
                    info!("Test Passed");
                    break;
                }
            }
        }
    }

    Ok(())
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
    #[serde(default = "default_timeout")]
    valid_frame_timeout: Duration,

    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_timeout")]
    link_timeout: Duration,

    tests: Vec<TestItem>,
}

#[derive(Debug)]
pub(crate) struct Dradis<'a> {
    cfg: Test,
    dev: &'a Device,
    heap: &'a Heap,
    queue: &'a Queue<'a>,
}

fn main() -> anyhow::Result<()> {
    let matches = Command::new("DRADIS DRM/KMS Test Program")
        .arg(
            Arg::new("device")
                .long("device")
                .short('D')
                .help("V4L2 Device File")
                .default_value("/dev/video0"),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .help("Enables debug log level"),
        )
        .arg(
            Arg::new("trace")
                .long("trace")
                .short('t')
                .conflicts_with("debug")
                .help("Enables trace log level"),
        )
        .arg(
            Arg::new("test")
                .required(true)
                .help("Test Configuration File"),
        )
        .get_matches();

    let log_level = if matches.contains_id("trace") {
        LevelFilter::Trace
    } else if matches.contains_id("debug") {
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
    .context("Couldn't initialize our logging configuration")?;

    let test_path = matches
        .get_one::<String>("test")
        .context("Couldn't get the test description path.")?;

    let test_file = File::open(test_path).context("Couldn't open the test description file.")?;

    let test_config: Test =
        serde_yaml::from_reader(test_file).context("Couldn't parse the test description file.")?;

    let heap = Heap::new(HeapKind::Cma).context("Couldn't open the DMA-Buf Heap")?;

    let dev_file = matches
        .get_one::<String>("device")
        .context("Couldn't get the V4L2 Device path.")?;

    let dev = Device::new(dev_file, true).context("Couldn't open the V4L2 Device.")?;

    let queue = dev
        .get_queue(QueueType::Capture)
        .context("Couldn't open the V4L2 Capture Queue")?;

    let dradis = Dradis {
        cfg: test_config,
        dev: &dev,
        heap: &heap,
        queue: &queue,
    };

    for test in &dradis.cfg.tests {
        test_display_one_mode(&dradis, &test)?;
    }

    Ok(())
}
