#![allow(missing_docs)]
#![allow(unused_crate_dependencies)]

use std::{io, path::PathBuf};

use clap::Parser;
use linux_mc::{
    MediaController, MediaControllerPadKind, RevocableResult, RevocableValue, try_result, try_value,
};

fn dump_topology(media: &MediaController) -> RevocableResult<(), io::Error> {
    let info = media.info().unwrap();
    println!(
        "Media controller API version {}",
        info.media_controller_version()
    );
    println!();
    println!("Media device information");
    println!("------------------------");
    println!("driver\t\t{}", info.driver());
    println!("model\t\t{}", info.model());
    println!("serial\t\t{}", info.serial());
    println!("bus info\t{}", info.bus_info());
    println!("hw revision\t{:#x}", info.hardware_revision());
    println!("driver version\t{}", info.driver_version());
    println!();
    println!(
        "Device topology version {}",
        media.topology_version().unwrap()
    );

    let entities = media.entities().unwrap();
    let links = media.links().unwrap();
    let pads = media.pads().unwrap();

    for entity in &entities {
        let entity_interfaces = try_result!(entity.interfaces());

        let entity_pads: Vec<_> = pads
            .iter()
            .filter(|p| p.entity_id() == entity.id())
            .collect();

        let mut args = Vec::new();
        if !entity_pads.is_empty() {
            args.push(if entity_pads.len() > 1 {
                format!("{} pads", entity_pads.len())
            } else {
                format!("{} pad", entity_pads.len())
            });
        }

        let outbound_links: Vec<_> = links
            .iter()
            .filter(|l| {
                entity_pads.iter().any(|e| e.id() == l.source_id())
                    || entity_pads.iter().any(|e| e.id() == l.sink_id())
            })
            .collect();
        if !outbound_links.is_empty() {
            args.push(if outbound_links.len() > 1 {
                format!("{} links", outbound_links.len())
            } else {
                format!("{} link", outbound_links.len())
            });
        }

        println!(
            "- entity {}: {}{}",
            try_value!(entity.id()),
            try_value!(entity.name()),
            if args.is_empty() {
                String::new()
            } else {
                format!(" ({})", args.join(", "))
            }
        );

        let flags = try_value!(entity.flag_names())
            .collect::<Vec<&str>>()
            .join(", ");

        print!("            {}", try_value!(entity.function()));
        if !flags.is_empty() {
            print!(", Flags: {flags}");
        }
        println!();

        if let Some(itf) = entity_interfaces.first() {
            let device_node = try_result!(itf.device_node());

            println!(
                "            device node name {}",
                device_node.unwrap().path().display()
            );
        }

        for pad in entity_pads {
            let pad_kind = try_value!(pad.kind());

            let link = match pad_kind {
                MediaControllerPadKind::Sink => {
                    links.iter().find(|l| l.sink_id() == pad.id()).unwrap()
                }
                MediaControllerPadKind::Source => {
                    links.iter().find(|l| l.source_id() == pad.id()).unwrap()
                }
            };

            let remote_pad = try_result!(pad.remote_pad()).unwrap();
            let remote_entity = try_result!(remote_pad.entity());

            let flags = try_value!(pad.flag_names())
                .collect::<Vec<&str>>()
                .join(", ");
            print!("	pad{}:", try_value!(pad.index()));
            if !flags.is_empty() {
                print!(" Flags: {flags}");
            }
            println!();

            let flags = try_value!(link.flag_names())
                .collect::<Vec<&str>>()
                .join(", ");
            println!(
                "		{} \"{}\":{}, Type {}{}",
                match pad_kind {
                    MediaControllerPadKind::Sink => "<-",
                    MediaControllerPadKind::Source => "->",
                },
                try_value!(remote_entity.name()),
                try_value!(remote_pad.index()),
                try_value!(link.kind()),
                if flags.is_empty() {
                    String::new()
                } else {
                    format!(", Flags: {flags}")
                }
            );
        }

        println!();
    }

    RevocableResult::Ok(())
}

#[derive(Parser)]
struct CliArgs {
    #[arg(short, long, default_value = "/dev/media0")]
    device: PathBuf,
}

fn main() -> Result<(), io::Error> {
    let args = CliArgs::parse();

    let media = MediaController::new(&args.device).unwrap();

    loop {
        match dump_topology(&media) {
            RevocableResult::Ok(()) => break,
            RevocableResult::Revoked => {}
            RevocableResult::Err(e) => return Err(e),
        }
    }

    Ok(())
}
