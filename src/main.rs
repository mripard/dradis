#![warn(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]
#![allow(clippy::unreadable_literal)]

use std::hash::Hasher;
use anyhow::{Context, Result};
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use clap::App;
use clap::Arg;
use image::imageops::FilterType;
use nucleid::{BufferType, ConnectorStatus, ConnectorUpdate, Device, Format, ObjectUpdate, PlaneType, PlaneUpdate};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use twox_hash::XxHash32;

const HEADER_VERSION_MAJOR: u8 = 1;
const HEADER_VERSION_MINOR: u8 = 0;
const HEADER_MAGIC: u32 = u32::from_ne_bytes(*b"CRNO");

const NUM_BUFFERS: u32 = 3;

fn main() -> Result<()> {
    let matches = App::new("KMS Crash Test Pattern")
        .arg(Arg::with_name("device")
                .short("D")
                .help("DRM Device Path")
                .default_value("/dev/dri/card0"))
        .arg(Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("Enables debug log level"))
        .arg(Arg::with_name("trace")
                .long("trace")
                .short("t")
                .conflicts_with("debug")
                .help("Enables trace log level"))
        .arg(Arg::with_name("image")
             .required(true))
        .get_matches();

    let log_level = if matches.is_present("trace") {
        LevelFilter::Trace
    } else if matches.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let _ = TermLogger::init(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto
    )
    .context("Couldn't setup our logger.")?;

    let img_path = matches.value_of("image").unwrap();
    let dev_path = matches.value_of("device").unwrap();
    let device = Device::new(dev_path).unwrap();

    let connector = device
        .connectors()
        .into_iter()
        .find(|con| {
            con.status().unwrap_or(ConnectorStatus::Unknown) == ConnectorStatus::Connected
        })
        .context("No Active Connector")?;

    log::info!("Running from connector {:#?}", connector);

    // let mode = connector
    //     .preferred_mode()
    //     .context("Couldn't find a mode for the connector")?;

    let mode = connector.modes()
        .context("Couldn't retrieve the connector modes")?
        .into_iter()
        // .find(|mode| mode.width() == 640 && mode.height() == 480 && mode.refresh() == 60)
        .find(|mode| mode.width() == 1280 && mode.height() == 720 && mode.refresh() == 60)
        .context("Couldn't find our mode")?;

    let width = mode.width();
    let height = mode.height();

    log::info!("Using mode {:#?}", mode);

    let output = device
        .output_from_connector(&connector)
        .context("Couldn't find a valid output for that connector")?;

    let plane = output
        .planes()
        .into_iter()
        .find(|plane| {
            plane.formats().any(|fmt| fmt == Format::RGB888)
                && plane.plane_type() == PlaneType::Overlay
        })
        .context("Couldn't find a plane with the proper format")?;

    let img = image::open(img_path).unwrap()
                                   .resize_exact(width as u32,
                                                 height as u32,
                                                 FilterType::Nearest);
    let img_data = img.to_bgr8().into_vec();

    log::info!("Opened image {}", img_path);

    let mut hasher = XxHash32::with_seed(0);
    hasher.write(&img_data[16..]);
    let hash = hasher.finish() as u32;

    log::info!("Hash {:#x}", hash);

    let mut buffers: Vec<_> = Vec::new();
    for _idx in 0..NUM_BUFFERS {
        let mut buffer = device
            .allocate_buffer(BufferType::Dumb, width, height, 24)
            .unwrap()
            .into_framebuffer(Format::RGB888)
            .unwrap();

        let data = buffer.data();
        // data.copy_from_slice(&img_data);

        data[0] = HEADER_VERSION_MAJOR;
        data[1] = HEADER_VERSION_MINOR;
        data[2] = 0;
        data[3] = 0;
        LittleEndian::write_u32(&mut data[4..8], HEADER_MAGIC);
        LittleEndian::write_u32(&mut data[8..12], 0);
        LittleEndian::write_u32(&mut data[12..16], hash);

        buffers.push(buffer);
    }

    log::info!("Setting up the pipeline");

    let first = &buffers[0];
    let mut output = output
        .start_update()
        .set_mode(mode)
        .add_connector(
            ConnectorUpdate::new(&connector)
                .set_property("top margin", 0)
                .set_property("bottom margin", 0)
                .set_property("left margin", 0)
                .set_property("right margin", 0),
        )
        .add_plane(
            PlaneUpdate::new(&plane)
                .set_framebuffer(&first)
                .set_source_size(width as f32, height as f32)
                .set_source_coordinates(0.0, 0.0)
                .set_display_size(width, height)
                .set_display_coordinates(0, 0),
        )
        .commit()?;

    log::info!("Starting to output");

    let mut index = 0;
    loop {
        let buffer = &mut buffers[(index % NUM_BUFFERS) as usize];
        let data = buffer.data();

        LittleEndian::write_u32(&mut data[8..12], index);

        log::debug!("Switching to frame {}", index);

        output = output
            .start_update()
            .add_plane(PlaneUpdate::new(&plane)
                .set_framebuffer(&buffer)
            )
            .commit()?;

        index = index + 1;
    }
}
