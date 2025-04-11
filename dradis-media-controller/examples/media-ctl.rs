use std::path::PathBuf;

use clap::Parser;
use dradis_media_controller::MediaController;

#[derive(Parser)]
struct CliArgs {
    #[arg(short, long, default_value = "/dev/media0")]
    device: PathBuf,
}

fn main() {
    let args = CliArgs::parse();

    let media = MediaController::new(&args.device).unwrap();

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
    println!("Device topology");

    let entities = media.entities().unwrap();
    let interfaces = media.interfaces().unwrap();
    let links = media.links().unwrap();
    let pads = media.pads().unwrap();

    for entity in &entities {
        let interface = links
            .iter()
            .filter(|l| l.sink_id() == entity.id())
            .next()
            .map(|l| interfaces.iter().filter(|i| i.id() == l.source_id()).next())
            .flatten();

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
                entity_pads
                    .iter()
                    .find(|e| e.id() == l.source_id())
                    .is_some()
                    || entity_pads.iter().find(|e| e.id() == l.sink_id()).is_some()
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
            entity.id(),
            entity.name(),
            if !args.is_empty() {
                format!(" ({})", args.join(", "))
            } else {
                String::new()
            }
        );

        let flags = entity.flag_names().collect::<Vec<&str>>().join(", ");
        print!("            {}", entity.function());
        if !flags.is_empty() {
            print!(", Flags: {}", flags);
        }
        println!();

        if let Some(itf) = interface {
            println!(
                "            device node name {}",
                itf.device_node().unwrap().path().display()
            )
        }

        for pad in entity_pads {
            let (link, remote_pad_id) = if pad.is_sink() {
                let link = links.iter().find(|l| l.sink_id() == pad.id()).unwrap();
                (link, link.source_id())
            } else if pad.is_source() {
                let link = links.iter().find(|l| l.source_id() == pad.id()).unwrap();
                (link, link.sink_id())
            } else {
                unreachable!();
            };
            let remote_pad = pads.iter().find(|p| p.id() == remote_pad_id).unwrap();
            let remote_entity = entities
                .iter()
                .find(|e| e.id() == remote_pad.entity_id())
                .unwrap();

            let flags = pad.flags_name().collect::<Vec<&str>>().join(", ");
            print!("	pad{}:", pad.index());
            if !flags.is_empty() {
                print!(" Flags: {flags}");
            }
            println!();

            let flags = link.flags_name().collect::<Vec<&str>>().join(", ");
            println!(
                "		{} \"{}\":{}, Type {}{}",
                if pad.is_sink() { "<-" } else { "->" },
                remote_entity.name(),
                remote_pad.index(),
                link.kind(),
                if !flags.is_empty() {
                    format!(", Flags: {flags}")
                } else {
                    String::new()
                }
            );
        }

        println!()
    }
}
