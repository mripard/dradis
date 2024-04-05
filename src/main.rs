#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::pedantic)]
#![warn(clippy::multiple_crate_versions)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::unreadable_literal)]

mod frame_check;
mod helpers;

use core::fmt;
use std::{
    fs::File,
    os::unix::io::AsRawFd,
    path::PathBuf,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::Context;
use clap::Parser;
use dma_buf::{DmaBuf, MappedDmaBuf};
use dma_heap::{Heap, HeapKind};
use log::{debug, error, info, warn};
use redid::EdidTypeConversionError;
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use thiserror::Error;
use v4lise::{
    v4l2_buf_type, v4l2_buffer, v4l2_memory, v4l2_query_buffer, Device, FrameFormat, MemoryType,
    PixelFormat, Queue, QueueType, V4L2_EVENT_SOURCE_CHANGE,
};

use crate::{
    frame_check::decode_and_check_frame,
    helpers::{
        dequeue_buffer, dequeue_event, queue_buffer, set_edid, start_streaming, subscribe_event,
        wait_and_set_dv_timings, EventKind,
    },
};

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

const BUFFER_TYPE: v4l2_buf_type = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE;
const MEMORY_TYPE: v4l2_memory = v4l2_memory::V4L2_MEMORY_DMABUF;
const NUM_BUFFERS: u32 = 5;

const FRAMES_DEQUEUED_TIMEOUT: Duration = Duration::from_secs(10);

const fn default_timeout() -> Duration {
    Duration::from_secs(10)
}

#[derive(Debug, Error)]
enum TestError {
    #[error("Test Needs to be Started Again")]
    Retry,

    #[error("No Frame Received")]
    NoFrameReceived,

    #[error("Couldn't convert our value")]
    ValueError { reason: String },

    #[error("Test Setup Failed: {}", .reason)]
    SetupFailed {
        reason: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl<T> From<EdidTypeConversionError<T>> for TestError
where
    T: fmt::Display,
{
    fn from(value: EdidTypeConversionError<T>) -> Self {
        Self::ValueError {
            reason: value.to_string(),
        }
    }
}

impl From<v4lise::Error> for TestError {
    fn from(value: v4lise::Error) -> Self {
        Self::SetupFailed {
            reason: String::from("Unknown Error"),
            source: Some(Box::new(value)),
        }
    }
}

fn test_prepare_queue(suite: &Dradis<'_>, test: &TestItem) -> std::result::Result<(), TestError> {
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

    Ok(())
}

fn test_run(suite: &Dradis<'_>, test: &TestItem) -> std::result::Result<(), TestError> {
    test_prepare_queue(suite, test)?;

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

    let _stream = start_streaming(suite.dev, BUFFER_TYPE).expect("Couldn't start streaming");

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

            let evt = dequeue_event(suite.dev);
            if let Ok(e) = evt {
                if let EventKind::SourceChange(v) = e.kind {
                    debug! {"Source Changed: seq: {}, timestamp: {}, rem: {}, flags {:#?}", e.sequence, e.timestamp, e.pending, v};
                    return Err(TestError::Retry);
                }
            } else {
                debug!("No Event to Dequeue.");
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
        if let Ok(frame_index) = buf.read(
            decode_and_check_frame,
            Some((last_frame_index, test.expected_width, test.expected_height)),
        ) {
            debug!("Frame {} Valid", frame_index);
            if first_frame_valid.is_none() {
                first_frame_valid = Some(Instant::now());
                info!("Source started to transmit a valid frame");
            }

            last_frame_index = Some(frame_index);
            last_frame_valid = Some(Instant::now());
        } else {
            debug!("Frame Invalid.");
            last_frame_index = None;
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

fn test_display_one_mode(
    suite: &Dradis<'_>,
    test: &TestItem,
) -> std::result::Result<(), TestError> {
    set_edid(suite.dev, &test.edid).map_err(|e| TestError::SetupFailed {
        reason: String::from("Couldn't set the EDID on the bridge"),
        source: Some(Box::new(e)),
    })?;

    loop {
        match test_run(suite, test) {
            Ok(()) => break,
            Err(e) => match e {
                TestError::Retry => {
                    warn!("Test needs to be restarted.");
                }
                _ => {
                    return Err(e);
                }
            },
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct TestEdidDetailedTiming {
    clock_khz: u32,
    hfp: u16,
    hdisplay: u16,
    hbp: u16,
    hsync: u16,
    vfp: u8,
    vdisplay: u16,
    vbp: u8,
    vsync: u8,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "timings")]
enum TestEdid {
    #[serde(rename = "detailed")]
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

#[derive(Parser)]
#[command(version, about = "DRADIS DRM/KMS Test Program")]
struct Cli {
    #[arg(default_value = "/dev/video0", help = "V4L2 Device File", long, short)]
    device: PathBuf,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(help = "Test Configuration File")]
    test: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    info!(
        "Running {} {}",
        built_info::PKG_NAME,
        if let Some(version) = built_info::GIT_VERSION {
            version
        } else {
            built_info::PKG_VERSION
        }
    );

    TermLogger::init(
        match cli.verbose {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        },
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("Couldn't initialize our logging configuration")?;

    let test_file = File::open(cli.test).context("Couldn't open the test description file.")?;

    let test_config: Test =
        serde_yaml::from_reader(test_file).context("Couldn't parse the test description file.")?;

    let heap = Heap::new(HeapKind::Cma).context("Couldn't open the DMA-Buf Heap")?;

    let dev = Device::new(&cli.device, true).context("Couldn't open the V4L2 Device.")?;

    let queue = dev
        .get_queue(QueueType::Capture)
        .context("Couldn't open the V4L2 Capture Queue")?;

    let dradis = Dradis {
        cfg: test_config,
        dev: &dev,
        heap: &heap,
        queue: &queue,
    };

    subscribe_event(dradis.dev, V4L2_EVENT_SOURCE_CHANGE)
        .context("Couldn't subscribe to our V4L2 Events")?;

    for test in &dradis.cfg.tests {
        test_display_one_mode(&dradis, test)?;
    }

    Ok(())
}
