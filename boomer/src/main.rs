#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::rc::Rc;
use core::hash::Hasher as _;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::Parser;
use frame_check::{Metadata, QRCODE_HEIGHT, QRCODE_WIDTH};
use image::{EncodableLayout as _, GenericImage as _, Rgb, Rgba, imageops::FilterType};
use nucleid::{
    BufferType, Connector, ConnectorStatus, ConnectorUpdate, Device, Format, Framebuffer, Mode,
    ObjectUpdate as _, Output, Plane, PlaneType, PlaneUpdate,
};
use qrcode::QrCode;
use tracing::{Level, debug, debug_span, info};
use tracing_subscriber::fmt::format::FmtSpan;
use twox_hash::XxHash64;

const HEADER_VERSION_MAJOR: u8 = 2;
const HEADER_VERSION_MINOR: u8 = 0;

const NUM_BUFFERS: usize = 3;

const PATTERN: &[u8] = include_bytes!("../resources/smpte-color-bars.png");

#[derive(Parser)]
#[command(about = "KMS Crash Test Pattern", version)]
struct CliArgs {
    #[arg(
        short = 'D',
        long,
        help = "DRM Device Path",
        default_value = "/dev/dri/card0"
    )]
    device: PathBuf,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn find_connector(device: &Device) -> Option<Rc<Connector>> {
    device
        .connectors()
        .find(|con| con.status().unwrap_or(ConnectorStatus::Unknown) == ConnectorStatus::Connected)
}

fn find_mode_for_connector(connector: &Rc<Connector>) -> Result<Mode, nucleid::Error> {
    connector.preferred_mode()
}

fn find_plane_for_output(output: &Output) -> Option<Rc<Plane>> {
    output.planes().into_iter().find(|plane| {
        plane.formats().any(|fmt| fmt == Format::BGR888) && plane.plane_type() == PlaneType::Overlay
    })
}

fn get_framebuffers(
    device: &Device,
    num: usize,
    width: u32,
    height: u32,
    fmt: Format,
    bpp: u32,
) -> Result<Vec<Framebuffer>, nucleid::Error> {
    let mut buffers = Vec::with_capacity(num);

    for _idx in 0..num {
        let buffer = device
            .allocate_buffer(BufferType::Dumb, width, height, bpp)?
            .into_framebuffer(fmt)?;

        buffers.push(buffer);
    }

    Ok(buffers)
}

fn initial_commit(
    output: Output,
    connector: &Rc<Connector>,
    mode: Mode,
    plane: &Rc<Plane>,
    fb: &Framebuffer,
    src: (f32, f32),
    display: (usize, usize),
) -> Result<Output, nucleid::Error> {
    let (src_w, src_h) = src;
    let (display_w, display_h) = display;

    output
        .start_update()
        .set_mode(mode)
        .add_connector(
            ConnectorUpdate::new(connector)
                .set_property("top margin", 0)
                .set_property("bottom margin", 0)
                .set_property("left margin", 0)
                .set_property("right margin", 0),
        )
        .add_plane(
            PlaneUpdate::new(plane)
                .set_framebuffer(fb)
                .set_source_size(src_w, src_h)
                .set_source_coordinates(0.0, 0.0)
                .set_display_size(display_w, display_h)
                .set_display_coordinates(0, 0),
        )
        .commit()
}

fn create_metadata_json(
    width: u32,
    height: u32,
    hash: u64,
    index: usize,
) -> Result<String, serde_json::Error> {
    let metadata = Metadata {
        version: (HEADER_VERSION_MAJOR, HEADER_VERSION_MINOR),
        qrcode_width: QRCODE_WIDTH,
        qrcode_height: QRCODE_HEIGHT,
        width,
        height,
        hash,
        index,
    };

    serde_json::to_string(&metadata)
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(true)
        .with_max_level(match args.verbose {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        })
        .init();

    let device = Device::new(&args.device).context("Couldn't open the KMS device file")?;

    let connector = find_connector(&device).context("No Active Connector")?;
    info!("Running from connector {:#?}", connector);

    let mode =
        find_mode_for_connector(&connector).context("Couldn't find a mode for the connector")?;

    let width = mode.width();
    let height = mode.height();

    info!("Using mode {:#?}", mode);

    let output = device
        .output_from_connector(&connector)
        .context("Couldn't find a valid output for that connector")?;

    let plane =
        find_plane_for_output(&output).context("Couldn't find a plane with the proper format")?;

    let mut img = image::load_from_memory(PATTERN)
        .context("Couldn't load our image")?
        .resize_exact(width.into(), height.into(), FilterType::Nearest);

    for x in 0..QRCODE_WIDTH {
        for y in 0..QRCODE_HEIGHT {
            img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
    }

    let mut img = img.to_rgb8();

    let mut hasher = XxHash64::with_seed(0);
    hasher.write(img.as_bytes());
    let hash = hasher.finish();

    info!("Hash {:#x}", hash);

    let mut buffers = get_framebuffers(
        &device,
        NUM_BUFFERS,
        width.into(),
        height.into(),
        Format::BGR888,
        24,
    )
    .context("Couldn't create our framebuffers")?;

    info!("Setting up the pipeline");

    let mut output = initial_commit(
        output,
        &connector,
        mode,
        &plane,
        &buffers[0],
        (width.into(), height.into()),
        (width.into(), height.into()),
    )?;

    info!("Starting to output");

    let mut index: usize = 0;
    loop {
        let span = debug_span!("Frame Generation");
        let _enter = span.enter();

        let buffer = &mut buffers[index % NUM_BUFFERS];
        let data = buffer.data();

        debug!("Switching to frame {}", index);

        let json = create_metadata_json(width.into(), height.into(), hash, index)
            .context("Metadata JSON serialization failed.")?;

        debug!("Metadata {:#?}", json);

        let qrcode = QrCode::new(json.as_bytes())
            .context("QR Code creation failed")?
            .render::<Rgb<u8>>()
            .min_dimensions(QRCODE_WIDTH, QRCODE_HEIGHT)
            .max_dimensions(QRCODE_WIDTH, QRCODE_HEIGHT)
            .build();

        image::imageops::overlay(&mut img, &qrcode, 0, 0);
        data.copy_from_slice(img.as_bytes());

        output = output
            .start_update()
            .add_plane(PlaneUpdate::new(&plane).set_framebuffer(buffer))
            .commit()?;

        index += 1;
    }
}
