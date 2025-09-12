#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::rc::Rc;
use std::{io, path::PathBuf};

use anyhow::{Context as _, Result};
use clap::Parser;
use frame_check::{Frame, Metadata, QRCODE_HEIGHT, QRCODE_WIDTH};
use image::{Rgba, imageops::FilterType};
use nucleid::{
    BufferType, Connector, ConnectorStatus, ConnectorType, ConnectorUpdate, Device, Format,
    Framebuffer, Mode, Object as _, ObjectUpdate as _, Output, Plane, PlaneType, PlaneUpdate,
};
use pix::{Raster, bgr::Bgra8, rgb::Rgba8};
use qrcode::QrCode;
use tracing::{Level, debug, debug_span, info, trace};
use tracing_subscriber::fmt::format::FmtSpan;

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

    #[arg(short = 'C', long, help = "Connector name, for example: HDMI-A-1")]
    connector_name: Option<String>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn find_connector(device: &Device, connector_name: Option<&str>) -> Option<Rc<Connector>> {
    if let Some(name) = connector_name {
        device.connectors().find(|con| con.to_string() == name)
    } else {
        device.connectors().find(|con| {
            con.connector_type() == ConnectorType::HDMIA
                && con.status().unwrap_or(ConnectorStatus::Unknown) == ConnectorStatus::Connected
        })
    }
}

fn find_mode_for_connector(connector: &Rc<Connector>) -> io::Result<Mode> {
    connector.preferred_mode()
}

fn find_plane_for_output(output: &Output) -> Option<Rc<Plane>> {
    output.planes().into_iter().find(|plane| {
        plane.formats().any(|fmt| fmt == Format::XRGB8888)
            && plane.plane_type().expect("Can't get plane type") == PlaneType::Primary
    })
}

fn get_framebuffers(
    device: &Device,
    num: usize,
    width: u32,
    height: u32,
    fmt: Format,
    bpp: u32,
) -> io::Result<Vec<Framebuffer>> {
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
) -> io::Result<Output> {
    let (src_w, src_h) = src;
    let (display_w, display_h) = display;

    output
        .start_update()
        .set_mode(mode)
        .add_connector(if connector.property("top margin")?.is_some() {
            debug!("Driver supports TV margins properties. Using them.");

            ConnectorUpdate::new(connector)
                .set_property("top margin", 0)
                .set_property("bottom margin", 0)
                .set_property("left margin", 0)
                .set_property("right margin", 0)
        } else {
            debug!("KMS Driver doesn't support TV margins properties. Skipping.");

            ConnectorUpdate::new(connector)
        })
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

    debug!("{}", metadata);

    serde_json::to_string(&metadata)
}

fn get_rgb_pattern(width: u32, height: u32) -> Result<Frame, image::ImageError> {
    Ok(Raster::with_u8_buffer(
        width,
        height,
        image::load_from_memory(PATTERN)?
            .resize_exact(width, height, FilterType::Nearest)
            .to_rgb8()
            .to_vec(),
    )
    .into())
}

fn create_qr_code(bytes: &[u8]) -> Result<Raster<Bgra8>, qrcode::types::QrError> {
    let qrcode = QrCode::new(bytes)?
        .render::<Rgba<u8>>()
        .min_dimensions(QRCODE_WIDTH, QRCODE_HEIGHT)
        .max_dimensions(QRCODE_WIDTH, QRCODE_HEIGHT)
        .build();

    let rgba_raster: Raster<Rgba8> =
        Raster::with_u8_buffer(qrcode.width(), qrcode.height(), qrcode.to_vec());

    Ok(Raster::with_raster(&rgba_raster))
}

enum TestResult {
    Error(anyhow::Error),
}

macro_rules! try_anyhow {
    ($e:expr, $c:literal) => {
        match $e.context($c) {
            Ok(v) => v,
            Err(e) => return TestResult::Error(e),
        }
    };
}

fn start_output(device: &Device, connector: &Rc<Connector>) -> TestResult {
    info!("Running from Connector {}", connector);

    let mode = try_anyhow!(
        find_mode_for_connector(connector),
        "Couldn't find a mode for the connector"
    );

    let width = mode.width();
    let height = mode.height();

    info!("Using mode {}", mode);

    let output = try_anyhow!(
        device.output_from_connector(connector),
        "Couldn't find a valid output for that connector"
    );

    info!("Using output: {}", output);

    let plane = try_anyhow!(
        find_plane_for_output(&output),
        "Couldn't find a plane with the proper format"
    );

    let pattern_bgr = try_anyhow!(
        get_rgb_pattern(width.into(), height.into()),
        "Couldn't load our pattern."
    );
    let cleared_pattern_bgr = pattern_bgr.clear();

    let hash = cleared_pattern_bgr.compute_checksum();
    info!("Hash {:#x}", hash);

    let cleared_pattern_xrgb = cleared_pattern_bgr.convert::<Bgra8>();

    let mut buffers = try_anyhow!(
        get_framebuffers(
            device,
            NUM_BUFFERS,
            width.into(),
            height.into(),
            Format::XRGB8888,
            32,
        ),
        "Couldn't create our framebuffers"
    );

    info!("Setting up the pipeline");

    let mut output = try_anyhow!(
        initial_commit(
            output,
            connector,
            mode,
            &plane,
            &buffers[0],
            (width.into(), height.into()),
            (width.into(), height.into()),
        ),
        "Couldn't perform initial commit"
    );

    info!("Starting to output");

    let mut index: usize = 0;
    loop {
        let span = debug_span!("Frame Generation");
        let _enter = span.enter();

        let buffer = &mut buffers[index % NUM_BUFFERS];
        let data = buffer.data();

        debug!("Switching to frame {}", index);

        let json = try_anyhow!(
            create_metadata_json(width.into(), height.into(), hash, index),
            "Metadata JSON serialization failed."
        );

        trace!("Metadata JSON {}", json);

        let qrcode = try_anyhow!(create_qr_code(json.as_bytes()), "QR Code creation failed");

        let merged_buffer = cleared_pattern_xrgb.with_qr_code(&qrcode);
        data.copy_from_slice(merged_buffer.as_bytes());

        output = try_anyhow!(
            output
                .start_update()
                .add_plane(PlaneUpdate::new(&plane).set_framebuffer(buffer))
                .commit(),
            "Commit Failed"
        );

        index += 1;
    }
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

    let device = Device::new(&args.device).context(format!(
        "Couldn't open the KMS device file \"{}\"",
        &args.device.display(),
    ))?;

    let connector =
        find_connector(&device, args.connector_name.as_deref()).context("No Active Connector")?;

    let TestResult::Error(e) = start_output(&device, &connector);
    Err(e)
}
