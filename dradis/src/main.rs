#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::unreadable_literal)]

extern crate alloc;
use alloc::rc::Rc;
use core::{cell::RefCell, fmt, time::Duration};
use std::{
    fs::File,
    io,
    os::{fd::AsFd as _, unix::io::AsRawFd as _},
    path::PathBuf,
    thread::sleep,
    time::Instant,
};

use anyhow::Context as _;
use clap::{Parser, ValueEnum};
use dma_buf::{DmaBuf, MappedDmaBuf};
use dma_heap::{Heap, HeapKind};
use frame_check::{DecodeCheckArgs, DecodeCheckArgsDump, decode_and_check_frame};
use linux_mc::{MediaController, MediaControllerEntity, MediaControllerPad, media_entity_function};
use redid::EdidTypeConversionError;
use rustix::io::Errno;
use serde::Deserialize;
use serde_with::{DurationSeconds, serde_as};
use thiserror::Error;
use threads_pool::ThreadPool;
use tracing::{Level, debug, debug_span, error, info, trace, warn};
use tracing_subscriber::fmt::format::FmtSpan;
use v4l2_raw::{
    format::v4l2_pix_fmt,
    raw::{v4l2_buf_type, v4l2_buffer, v4l2_field, v4l2_ioctl_querybuf, v4l2_memory},
    wrapper::{
        v4l2_event_subscription, v4l2_event_subscription_type, v4l2_event_type, v4l2_format,
        v4l2_ioctl_dqevent, v4l2_ioctl_subdev_s_fmt, v4l2_ioctl_subscribe_event,
        v4l2_subdev_format,
    },
};
use v4lise::{Device, Queue};

mod built_info {
    #![allow(unreachable_pub)]
    #![allow(clippy::doc_markdown)]
    #![allow(clippy::needless_raw_strings)]

    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

mod helpers;
use crate::helpers::{
    bridge_set_edid, dequeue_buffer, queue_buffer, start_streaming, wait_and_set_dv_timings,
};

const BUFFER_TYPE: v4l2_buf_type = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE;
const MEMORY_TYPE: v4l2_memory = v4l2_memory::V4L2_MEMORY_DMABUF;
const NUM_BUFFERS: u32 = 5;

const FRAMES_DEQUEUED_TIMEOUT: Duration = Duration::from_secs(10);

const fn default_timeout() -> Duration {
    Duration::from_secs(10)
}

#[derive(Error, Debug)]
enum SetupError {
    #[error("I/O Error {0}")]
    Io(#[from] io::Error),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Value Error: {0}")]
    Value(String),
}

impl<T> From<EdidTypeConversionError<T>> for SetupError
where
    T: fmt::Display,
{
    fn from(value: EdidTypeConversionError<T>) -> Self {
        Self::Value(value.to_string())
    }
}

#[derive(Debug, Error)]
enum TestError {
    #[error("Test Needs to be Started Again")]
    Retry,

    #[error("No Frame Received")]
    NoFrameReceived,

    #[error("Test Setup Failed: {0}")]
    SetupFailed(#[from] SetupError),
}

fn find_endpoint_predicate(
    entity: MediaControllerEntity,
) -> Option<Result<MediaControllerEntity, io::Error>> {
    if entity.function().valid() != media_entity_function::MEDIA_ENT_F_VID_IF_BRIDGE {
        return None;
    }

    match entity.num_pads().valid() {
        Ok(num_pads) => {
            if num_pads > 1 {
                return None;
            }
        }
        Err(e) => return Some(Err(e)),
    }

    match entity.pad(0).valid() {
        Ok(Some(p)) => {
            if p.is_source().valid() {
                Some(Ok(entity))
            } else {
                None
            }
        }
        Ok(None) => None,
        Err(e) => Some(Err(e)),
    }
}

fn find_internal_routed_pad(
    entity: &MediaControllerEntity,
    pad: &MediaControllerPad,
) -> Result<Option<MediaControllerPad>, io::Error> {
    let other = pad.kind().valid().other();

    let mut other_pads = entity
        .pads()
        .valid()?
        .into_iter()
        .filter(|p| p.kind().valid() == other)
        .collect::<Vec<_>>();

    Ok(match other_pads.len() {
        0 => None,
        1 => Some(other_pads.remove(0)),
        2.. => {
            // If it's a subdev, we should be calling g_routing. It's not enabled by default though,
            // so we'll just take the first pad in the list, and hope for the best.
            Some(other_pads.remove(0))
        }
    })
}

#[derive(Debug)]
struct MediaPipelineItem {
    sink_pad: Option<MediaControllerPad>,
    entity: MediaControllerEntity,
    source_pad: Option<MediaControllerPad>,
}

fn find_dev_and_subdev(mc: &MediaController) -> Result<Vec<MediaPipelineItem>, io::Error> {
    let mut outputs = Vec::new();
    let entities = mc.entities()?;

    debug!("Starting to discover our pipeline.");

    let mut endpoints = entities.into_iter().filter_map(find_endpoint_predicate);
    let hdmi_bridge = endpoints
        .next()
        .transpose()?
        .ok_or(io::Error::from(io::ErrorKind::NotFound))?;

    if endpoints.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Found more than one HDMI bridge",
        ));
    }

    debug!("Found an HDMI bridge: {}", hdmi_bridge.name());

    let hdmi_bridge_pad = hdmi_bridge
        .pad(0)
        .valid()?
        .ok_or(io::Error::from(io::ErrorKind::NotFound))?;

    debug!(
        "Found our pipeline source: {}, pad {}",
        hdmi_bridge.name(),
        hdmi_bridge_pad.index()
    );

    outputs.push(MediaPipelineItem {
        sink_pad: None,
        entity: hdmi_bridge,
        source_pad: Some(hdmi_bridge_pad.clone()),
    });

    let mut prev_source_pad = hdmi_bridge_pad;
    loop {
        let sink_pad = prev_source_pad
            .remote_pad()
            .valid()?
            .ok_or(io::Error::from(io::ErrorKind::NotFound))?;

        let entity = sink_pad.entity().valid();

        let source_pad = find_internal_routed_pad(&entity, &sink_pad)?;

        outputs.push(MediaPipelineItem {
            sink_pad: Some(sink_pad.clone()),
            entity: entity.clone(),
            source_pad: source_pad.clone(),
        });

        // Should we test whether it's a v4l2 device and / or a default controller?
        if let Some(source_pad) = source_pad {
            debug!(
                "Found an intermediate entity {}, input pad {}, output pad {}",
                entity.name(),
                sink_pad.index(),
                source_pad.index()
            );
            prev_source_pad = source_pad;
        } else {
            debug!(
                "Found the root entity {}, input pad {}",
                entity.name(),
                sink_pad.index()
            );

            break;
        }
    }

    outputs.reverse();
    Ok(outputs)
}

#[expect(
    clippy::missing_asserts_for_indexing,
    reason = "windows() guarantees the slice size, but it looks like we can't disable the lint locally"
)]
#[expect(clippy::too_many_lines, reason = "Yup, this function is long indeed.")]
fn test_prepare_queue(
    suite: &Dradis<'_>,
    queue: &Queue<'_>,
    test: &TestItem,
) -> Result<(), SetupError> {
    wait_and_set_dv_timings(suite, test.expected_width, test.expected_height)?;

    let _: v4l2_pix_fmt = queue
        .get_pixel_formats()
        .find(|fmt| *fmt == v4l2_pix_fmt::V4L2_PIX_FMT_RGB24)
        .expect("Couldn't find our format");

    let pix_fmt = if let v4l2_format::VideoCapture(pix_fmt) = queue
        .get_current_format()
        .expect("Couldn't get our queue format")
    {
        pix_fmt
            .set_width(test.expected_width)
            .set_height(test.expected_height)
            // Reset the bytes per line field to avoid inheriting the one from the previous format.
            .set_bytes_per_line(0)
            .set_pixel_format(v4l2_pix_fmt::V4L2_PIX_FMT_RGB24)
            .set_field(v4l2_field::V4L2_FIELD_NONE)
    } else {
        unreachable!()
    };

    for PipelineItem {
        source_pad,
        entity: wrapper,
        sink_pad,
    } in &suite.pipeline
    {
        trace!("Trying to set format on entity {}", wrapper.entity.name());

        if !wrapper.entity.is_v4l2_sub_device().valid()? {
            trace!("Entity {} isn't a subdev, skipping.", wrapper.entity.name());
            continue;
        }
        let Some(subdev) = wrapper.device.as_ref() else {
            continue;
        };

        let mbus_fmt = pix_fmt
            .to_v4l2_mbus_framefmt(pix_fmt.pixel_format().to_mipi_csi2_mbus_pixelcode().ok_or(
                SetupError::from(io::Error::new(
                    Errno::INVAL.kind(),
                    "Couldn't find a mediabus format for the pixel format",
                )),
            )?)
            .map_err(|_e| {
                SetupError::from(io::Error::new(
                    Errno::INVAL.kind(),
                    "Couldn't convert v4l2_pix_format to v4l2_mbus_framefmt",
                ))
            })?;

        if let Some(source_pad) = source_pad {
            let subdev_fmt = v4l2_subdev_format::new_active()
                .set_pad(source_pad.index().valid())
                .set_stream(0)
                .set_format(mbus_fmt);

            debug!(
                "Entity {}, Pad {} (Source): Setting {}",
                wrapper.entity.name(),
                source_pad.index(),
                subdev_fmt
            );

            if let Err(e) = v4l2_ioctl_subdev_s_fmt(subdev.as_fd(), subdev_fmt) {
                match Errno::from_io_error(&e) {
                    Some(Errno::PERM) => {
                        debug!("Subdev is Read-Only. Ignoring.");
                    }
                    _ => {
                        return Err(e.into());
                    }
                }
            }
        }

        if let Some(sink_pad) = sink_pad {
            let subdev_fmt = v4l2_subdev_format::new_active()
                .set_pad(sink_pad.index().valid())
                .set_stream(0)
                .set_format(mbus_fmt);

            debug!(
                "Entity {}, Pad {} (Source): Setting {}",
                wrapper.entity.name(),
                sink_pad.index(),
                subdev_fmt
            );

            if let Err(e) = v4l2_ioctl_subdev_s_fmt(subdev.as_fd(), subdev_fmt) {
                match Errno::from_io_error(&e) {
                    Some(Errno::PERM) => {
                        debug!("Subdev is Read-Only. Ignoring.");
                    }
                    _ => {
                        return Err(e.into());
                    }
                }
            }
        }
    }

    for items_slice in suite.pipeline.windows(2) {
        let sink_item = &items_slice[0];
        let source_item = &items_slice[1];

        debug!("Found Sink {}, Source {}", sink_item, source_item);

        if let (Some(source), Some(sink)) = (&source_item.source_pad, &sink_item.sink_pad) {
            debug!(
                "Enabling link between source {}:{} and sink {}:{}",
                source_item.entity.entity.name(),
                source.index(),
                sink_item.entity.entity.name(),
                sink.index()
            );

            let link = suite
                .mc
                .find_data_link_by_pads(source, sink)
                .valid()?
                .expect("Missing link between pads.");

            link.enable().valid()?;
        } else {
            unreachable!();
        }
    }

    let ret_fmt = queue
        .set_format(v4l2_format::VideoCapture(pix_fmt))
        .expect("Couldn't change our queue format");

    let ret_pix_fmt = ret_fmt
        .as_v4l2_pix_format()
        .expect("Queue Format isn't what we set.");

    if ret_pix_fmt.bytes_per_line()
        != ret_pix_fmt.width() * u32::from(ret_pix_fmt.pixel_format().bytes_per_pixel())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "We don't support any line padding",
        )
        .into());
    }

    debug!("Format set {:#?}", ret_fmt);

    Ok(())
}

#[expect(clippy::too_many_lines)]
fn test_run(
    cli: &Cli,
    suite: &Dradis<'_>,
    queue: &Queue<'_>,
    test: &TestItem,
) -> Result<(), TestError> {
    let PipelineItem { entity: root, .. } =
        suite
            .pipeline
            .first()
            .ok_or(SetupError::from(io::Error::new(
                Errno::NODEV.kind(),
                "Missing Root Entity",
            )))?;

    let root_device = root.device.as_ref().ok_or(SetupError::from(io::Error::new(
        Errno::NODEV.kind(),
        "Missing V4L2 Root Device",
    )))?;

    let PipelineItem { entity: bridge, .. } =
        suite
            .pipeline
            .last()
            .ok_or(SetupError::from(io::Error::new(
                Errno::NODEV.kind(),
                "Missing HDMI Bridge Entity",
            )))?;

    let bridge_device = bridge
        .device
        .as_ref()
        .ok_or(SetupError::from(io::Error::new(
            Errno::NODEV.kind(),
            "Missing V4L2 HDMI Bridge Device",
        )))?;

    test_prepare_queue(suite, queue, test)?;

    queue
        .request_buffers(v4l2_memory::V4L2_MEMORY_DMABUF, NUM_BUFFERS)
        .expect("Couldn't request our buffers");

    let mut buffers = Vec::with_capacity(NUM_BUFFERS as usize);
    let pool = if cli.dump_frames_limit > 0 {
        Rc::new(RefCell::new(ThreadPool::new(Some(cli.dump_frames_limit))))
    } else {
        Rc::new(RefCell::new(ThreadPool::new(None)))
    };

    for idx in 0..NUM_BUFFERS {
        let mut rbuf = v4l2_buffer {
            index: idx,
            type_: BUFFER_TYPE.into(),
            memory: MEMORY_TYPE.into(),
            ..v4l2_buffer::default()
        };

        rbuf = v4l2_ioctl_querybuf(root_device.as_fd(), rbuf).expect("Couldn't query our buffer");

        let len = rbuf.length as usize;
        let buffer = suite
            .heap
            .allocate(len)
            .expect("Couldn't allocate our dma-buf buffer");

        let buffer: MappedDmaBuf = DmaBuf::from(buffer)
            .memory_map()
            .expect("Couldn't map our dma-buf buffer");

        queue_buffer(root_device, idx, buffer.as_raw_fd()).expect("Couldn't queue our buffer");
        buffers.push(buffer);
    }

    let _stream = start_streaming(root_device, BUFFER_TYPE).expect("Couldn't start streaming");

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

        let vbuf = loop {
            if frame_dequeue_start.elapsed() > FRAMES_DEQUEUED_TIMEOUT {
                return Err(TestError::NoFrameReceived);
            }

            let evt = v4l2_ioctl_dqevent(bridge_device.as_fd());
            if let Ok(e) = evt {
                if let v4l2_event_type::SourceChange(_) = e.kind() {
                    debug! {"Source Changed: seq: {}, rem: {}", e.sequence(), e.pending()};
                    return Err(TestError::Retry);
                }

                trace!("Igoring event {e:#?}");
            } else {
                debug!("No Event to Dequeue.");
            }

            let res = dequeue_buffer(root_device);
            match &res {
                Ok(_) => break res,
                Err(e) => match Errno::from_io_error(e) {
                    Some(Errno::AGAIN) => {
                        debug!("No buffer to dequeue.");
                    }
                    _ => break res,
                },
            }

            sleep(Duration::from_millis(5));
        }
        .expect("Couldn't dequeue our buffer");

        let idx = vbuf.index;
        let buf = &buffers[idx as usize];
        debug_span!("Frame Processing").in_scope(|| {
            if let Ok(metadata) = buf.read(
                |b, a| {
                    decode_and_check_frame(
                        &b[..(vbuf.bytesused as usize)],
                        a.expect("Missing arguments"),
                    )
                    .map_err(Into::into)
                },
                Some(DecodeCheckArgs {
                    sequence: vbuf.sequence,
                    previous_frame_idx: last_frame_index,
                    width: test.expected_width,
                    height: test.expected_height,
                    // The RaspberryPi driver advertises the RGB24 v4l2 format (Red first), but
                    // actually stores the CSI format (blue first). We need to
                    // do a conversion to make it meaningful to us.
                    swap_channels: true,
                    dump: match cli.dump_frames {
                        CliDump::Always => DecodeCheckArgsDump::Always(pool.clone()),
                        CliDump::Corrupted => DecodeCheckArgsDump::Corrupted(pool.clone()),
                        CliDump::Never => DecodeCheckArgsDump::Never,
                    },
                }),
            ) {
                debug!("Frame {} Valid", metadata.index);
                if first_frame_valid.is_none() {
                    first_frame_valid = Some(Instant::now());
                    info!("Source started to transmit a valid frame");
                }

                last_frame_index = Some(metadata.index);
                last_frame_valid = Some(Instant::now());
            } else {
                debug!("Frame Invalid.");
                last_frame_index = None;
            }
        });

        queue_buffer(root_device, idx, buf.as_raw_fd()).expect("Couldn't queue our buffer");

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

fn test_display_one_mode(args: &Cli, suite: &Dradis<'_>, test: &TestItem) -> Result<(), TestError> {
    let PipelineItem { entity: root, .. } =
        suite
            .pipeline
            .first()
            .ok_or(SetupError::from(io::Error::new(
                Errno::NODEV.kind(),
                "Missing Root Entity",
            )))?;

    let root_device = root.device.as_ref().ok_or(SetupError::from(io::Error::new(
        Errno::NODEV.kind(),
        "Missing V4L2 Root Device",
    )))?;

    let PipelineItem { entity: bridge, .. } =
        suite
            .pipeline
            .last()
            .ok_or(SetupError::from(io::Error::new(
                Errno::NODEV.kind(),
                "Missing HDMI Bridge Entity",
            )))?;

    let bridge_device = bridge
        .device
        .as_ref()
        .ok_or(SetupError::from(io::Error::new(
            Errno::NODEV.kind(),
            "Missing V4L2 HDMI Bridge Device",
        )))?;

    let queue = root_device
        .get_queue(v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE)
        .map_err(SetupError::from)?;

    v4l2_ioctl_subscribe_event(
        bridge_device.as_fd(),
        v4l2_event_subscription::new(v4l2_event_subscription_type::SourceChange),
    )
    .map_err(SetupError::from)?;

    bridge_set_edid(args, bridge, &test.edid)?;

    loop {
        match test_run(args, suite, &queue, test) {
            Ok(()) => break,
            Err(e) => match e {
                TestError::Retry => {
                    warn!("Test needs to be restarted.");
                }
                TestError::NoFrameReceived | TestError::SetupFailed(_) => {
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
    vbp: u16,
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
    expected_height: u32,

    #[serde(rename = "expected-width")]
    expected_width: u32,

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
struct V4l2EntityWrapper {
    entity: MediaControllerEntity,
    device: Option<Device>,
}

#[derive(Debug)]
struct PipelineItem {
    source_pad: Option<MediaControllerPad>,
    entity: V4l2EntityWrapper,
    sink_pad: Option<MediaControllerPad>,
}

impl fmt::Display for PipelineItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.sink_pad, &self.source_pad) {
            (Some(source), Some(sink)) => f.write_fmt(format_args!(
                "Entity {} Source Pad {} Sink Pad {}",
                self.entity.entity.name(),
                source,
                sink
            )),
            (Some(sink), None) => f.write_fmt(format_args!(
                "Entity {} Sink Pad {}",
                self.entity.entity.name(),
                sink,
            )),
            (None, Some(source)) => f.write_fmt(format_args!(
                "Entity {} Source Pad {}",
                self.entity.entity.name(),
                source
            )),
            (None, None) => f.write_fmt(format_args!("Entity {}", self.entity.entity.name(),)),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Dradis<'a> {
    cfg: Test,
    mc: MediaController,
    pipeline: Vec<PipelineItem>,
    heap: &'a Heap,
}

#[derive(Clone, ValueEnum)]
enum CliDump {
    /// Dump All Received Frames
    Always,

    /// Dump Corrupted Frames Only
    Corrupted,

    /// Never Dump Any Frame
    Never,
}

#[derive(Parser)]
#[command(version, about = "DRADIS DRM/KMS Test Program")]
struct Cli {
    #[arg(
        default_value = "/dev/media0",
        help = "Media Controller Device File",
        long,
        short
    )]
    device: PathBuf,

    #[arg(long = "dump-edid", help = "Folder to dump test EDIDs in.")]
    dump_edid: Option<PathBuf>,

    #[arg(long = "dump-frames", value_enum, default_value_t = CliDump::Never, help = "Dump Received Frames")]
    dump_frames: CliDump,

    #[arg(
        long = "dump-frames-limit",
        default_value_t = 10,
        help = "Maximum Number of Frames to Dump. 0 for no limit."
    )]
    dump_frames_limit: usize,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(help = "Test Configuration File")]
    test: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(true)
        .with_max_level(match cli.verbose {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        })
        .init();

    info!(
        "Running {} {}",
        built_info::PKG_NAME,
        if let Some(version) = built_info::GIT_VERSION {
            version
        } else {
            built_info::PKG_VERSION
        }
    );

    let test_file =
        File::open(cli.test.clone()).context("Couldn't open the test description file.")?;

    let test_config: Test =
        serde_yaml::from_reader(test_file).context("Couldn't parse the test description file.")?;

    let heap = Heap::new(HeapKind::Cma).context("Couldn't open the DMA-Buf Heap")?;

    debug!("Running from media controller {}", cli.device.display());
    let mc = MediaController::new(&cli.device)?;
    let pipeline = find_dev_and_subdev(&mc)?
        .into_iter()
        .map(
            |MediaPipelineItem {
                 sink_pad,
                 entity,
                 source_pad,
             }| {
                let node = if let Some(itf) = entity.interfaces().valid()?.first() {
                    if let Some(node) = itf.device_node().valid() {
                        Some(Device::new(node.path(), true)?)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(PipelineItem {
                    source_pad,
                    entity: V4l2EntityWrapper {
                        entity,
                        device: node,
                    },
                    sink_pad,
                })
            },
        )
        .collect::<Result<Vec<_>, io::Error>>()?;

    let dradis = Dradis {
        cfg: test_config,
        mc,
        pipeline,
        heap: &heap,
    };

    for test in &dradis.cfg.tests {
        test_display_one_mode(&cli, &dradis, test)?;
    }

    Ok(())
}
