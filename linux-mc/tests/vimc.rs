#![allow(missing_docs)]
#![allow(unused_crate_dependencies)]

use std::{fs, path::PathBuf};

use linux_mc::{
    MediaController, MediaControllerEntity, MediaControllerInterface, MediaControllerInterfaceKind,
    MediaControllerInterfaceV4lKind, MediaControllerLink, MediaControllerLinkKind,
    MediaControllerPad, MediaControllerPadKind, media_entity_function,
};
use linux_raw::KernelVersion;
use rstest::{fixture, rstest};
use tracing::{debug, info};

const VIMC_MODEL_NAME: &'static str = "VIMC MDEV";

fn get_vimc_sysfs_dir() -> PathBuf {
    for entry in fs::read_dir("/sys/bus/media/devices/").unwrap() {
        let entry = entry.unwrap();
        let entry_path = entry.path();

        debug!("Found file {}", entry_path.display());

        let model = String::from_utf8(fs::read(entry_path.join("model")).unwrap()).unwrap();
        let model = model.trim();

        debug!("Media Controller Model: {model:#?}");
        if model != VIMC_MODEL_NAME {
            debug!("Model isn't vimc, skipping.");
            continue;
        }

        info!("Media Bus Device is VIMC, returning");
        return entry_path;
    }

    panic!("Missing VIMC device");
}

#[fixture]
fn get_vimc_device_path() -> PathBuf {
    let sysfs_dir = get_vimc_sysfs_dir();

    let uevent = String::from_utf8(fs::read(sysfs_dir.join("uevent")).unwrap()).unwrap();
    for line in uevent.lines() {
        let parts = line.split('=').collect::<Vec<_>>();

        let attr = parts[0];
        if attr != "DEVNAME" {
            continue;
        }

        let dev = PathBuf::from("/dev/").join(parts[1]);
        info!("Found device file {}", dev.display());

        return dev;
    }

    panic!("Couldn't find the associated device node");
}

#[rstest]
#[test_log::test]
#[cfg_attr(not(feature = "vimc"), ignore)]
fn open(#[from(get_vimc_device_path)] vimc: PathBuf) {
    assert!(MediaController::new(&vimc).is_ok());
}

#[rstest]
#[test_log::test]
#[cfg_attr(not(feature = "vimc"), ignore)]
fn info(#[from(get_vimc_device_path)] vimc: PathBuf) {
    let mc = MediaController::new(&vimc).unwrap();
    let info = mc.info().unwrap();

    info!("Found Media Controller Version {:#?}", info);

    assert_eq!(info.driver(), "vimc");
    assert_eq!(info.model(), VIMC_MODEL_NAME);
    assert_eq!(info.serial(), "");
    assert_eq!(info.bus_info(), "platform:vimc");
    assert_eq!(info.hardware_revision(), 0);

    let current_version = KernelVersion::current();
    info!("Current Kernel Version {}", current_version);

    // On Github CI, we might run with a kernel different than the one the modules we load was
    // compiled for. We can't expect total equality, so let's do our best to check the version
    // is somewhat sane.
    assert_eq!(info.driver_version().major(), current_version.major());
    assert_eq!(info.driver_version().minor(), current_version.minor());
    assert_eq!(
        info.media_controller_version().major(),
        current_version.major()
    );
    assert_eq!(
        info.media_controller_version().minor(),
        current_version.minor()
    );
}

struct ExpectedEntityInterfaceTopology {
    kind: MediaControllerInterfaceKind,
}

struct ExpectedEntityPadLinkTopology<'a> {
    kind: MediaControllerLinkKind,
    flags: &'a [&'a str],
    remote_entity: &'a str,
    remote_pad_index: u32,
}

struct ExpectedEntityPadTopology<'a> {
    index: u32,
    kind: MediaControllerPadKind,
    flags: &'a [&'a str],
    links: &'a [ExpectedEntityPadLinkTopology<'a>],
}

struct ExpectedEntityTopology<'a> {
    name: &'a str,
    function: media_entity_function,
    flags: &'a [&'a str],
    interfaces: &'a [ExpectedEntityInterfaceTopology],
    pads: &'a [ExpectedEntityPadTopology<'a>],
}

struct ExpectedTopology<'a> {
    topology_version: u64,
    entities: &'a [ExpectedEntityTopology<'a>],
}

fn check_interface_topology(
    actual_itf: &MediaControllerInterface,
    expected_itf: &ExpectedEntityInterfaceTopology,
) {
    assert!(actual_itf.id().unwrap() > 0);
    assert_eq!(actual_itf.kind().unwrap(), expected_itf.kind);

    let itf_node = actual_itf.device_node().unwrap().unwrap();
    assert!(fs::exists(itf_node.path()).unwrap());
}

fn check_pad_link_topology(
    actual_pad: &MediaControllerPad,
    actual_pad_link: &MediaControllerLink,
    expected_pad_link: &ExpectedEntityPadLinkTopology<'_>,
) {
    debug!(
        "Testing Pad \"{}\":{}",
        actual_pad.entity().unwrap().name().unwrap(),
        actual_pad.index().unwrap()
    );

    if actual_pad.kind().unwrap() == MediaControllerPadKind::Source {
        assert_eq!(
            actual_pad_link.source_id().unwrap(),
            actual_pad.id().unwrap()
        );
        assert!(actual_pad_link.sink_id().unwrap() > 0);

        let remote_pad = actual_pad_link.sink_pad().unwrap();
        assert_eq!(
            remote_pad.index().unwrap(),
            expected_pad_link.remote_pad_index
        );
        assert_eq!(
            remote_pad.entity().unwrap().name().unwrap(),
            expected_pad_link.remote_entity
        )
    } else {
        assert_eq!(actual_pad_link.sink_id().unwrap(), actual_pad.id().unwrap());
        assert!(actual_pad_link.source_id().unwrap() > 0);

        let remote_pad = actual_pad_link.source_pad().unwrap();
        assert_eq!(
            remote_pad.index().unwrap(),
            expected_pad_link.remote_pad_index
        );
        assert_eq!(
            remote_pad.entity().unwrap().name().unwrap(),
            expected_pad_link.remote_entity
        )
    }

    assert_eq!(actual_pad_link.kind().unwrap(), expected_pad_link.kind);
    assert_eq!(
        actual_pad_link.flag_names().unwrap().collect::<Vec<_>>(),
        expected_pad_link.flags
    );
}

fn check_pad_topology(
    actual_entity: &MediaControllerEntity,
    actual_pad: &MediaControllerPad,
    expected_pad: &ExpectedEntityPadTopology<'_>,
) {
    assert!(actual_pad.id().unwrap() > 0);
    assert_eq!(actual_pad.index().unwrap(), expected_pad.index);
    assert_eq!(actual_pad.kind().unwrap(), expected_pad.kind);
    assert_eq!(actual_pad.entity_id().unwrap(), actual_entity.id().unwrap());
    assert_eq!(
        actual_pad.flag_names().unwrap().collect::<Vec<_>>(),
        expected_pad.flags
    );

    let pad_links = actual_pad.links().unwrap();
    assert_eq!(pad_links.len(), expected_pad.links.len());

    for (pad_link, pad_link_expected) in Iterator::zip(pad_links.iter(), expected_pad.links.iter())
    {
        check_pad_link_topology(actual_pad, pad_link, pad_link_expected);
    }
}

fn check_entity_topology(entity: &MediaControllerEntity, expected: &ExpectedEntityTopology<'_>) {
    assert!(entity.id().unwrap() > 0);
    assert_eq!(entity.name().unwrap(), expected.name);
    assert_eq!(entity.function().unwrap(), expected.function);
    assert_eq!(
        entity.flag_names().unwrap().collect::<Vec<_>>(),
        expected.flags
    );

    let entity_interfaces = entity.interfaces().unwrap();
    assert_eq!(entity_interfaces.len(), expected.interfaces.len());

    for (itf, itf_expected) in Iterator::zip(entity_interfaces.iter(), expected.interfaces.iter()) {
        check_interface_topology(itf, itf_expected);
    }

    let entity_pads = entity.pads().unwrap();
    assert_eq!(entity_pads.len(), expected.pads.len());

    for (pad, pad_expected) in Iterator::zip(entity_pads.iter(), expected.pads.iter()) {
        check_pad_topology(entity, pad, pad_expected);
    }
}

fn check_topology(mc: &MediaController, expected_topology: &ExpectedTopology<'_>) {
    assert_eq!(
        mc.topology_version().unwrap(),
        expected_topology.topology_version
    );

    let entities = mc.entities().unwrap();
    assert_eq!(entities.len(), expected_topology.entities.len());

    for expected_ent in expected_topology.entities {
        let entity = entities
            .iter()
            .find(|e| e.name().unwrap() == expected_ent.name)
            .unwrap();

        info!("Found entity {}. Checking.", expected_ent.name);

        check_entity_topology(entity, &expected_ent);
    }
}

#[rstest]
#[test_log::test]
#[cfg_attr(not(feature = "vimc"), ignore)]
fn topology(#[from(get_vimc_device_path)] vimc: PathBuf) {
    let mc = MediaController::new(&vimc).unwrap();

    check_topology(
        &mc,
        &ExpectedTopology {
            topology_version: 16,
            entities: &[
                ExpectedEntityTopology {
                    name: "Sensor A",
                    function: media_entity_function::MEDIA_ENT_F_CAM_SENSOR,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Subdev,
                        ),
                    }],
                    pads: &[ExpectedEntityPadTopology {
                        index: 0,
                        kind: MediaControllerPadKind::Source,
                        flags: &["SOURCE"],
                        links: &[
                            ExpectedEntityPadLinkTopology {
                                kind: MediaControllerLinkKind::Data,
                                flags: &["ENABLED", "IMMUTABLE"],
                                remote_entity: "Debayer A",
                                remote_pad_index: 0,
                            },
                            ExpectedEntityPadLinkTopology {
                                kind: MediaControllerLinkKind::Data,
                                flags: &["ENABLED", "IMMUTABLE"],
                                remote_entity: "Raw Capture 0",
                                remote_pad_index: 0,
                            },
                        ],
                    }],
                },
                ExpectedEntityTopology {
                    name: "Sensor B",
                    function: media_entity_function::MEDIA_ENT_F_CAM_SENSOR,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Subdev,
                        ),
                    }],
                    pads: &[ExpectedEntityPadTopology {
                        index: 0,
                        kind: MediaControllerPadKind::Source,
                        flags: &["SOURCE"],
                        links: &[
                            ExpectedEntityPadLinkTopology {
                                kind: MediaControllerLinkKind::Data,
                                flags: &["ENABLED", "IMMUTABLE"],
                                remote_entity: "Debayer B",
                                remote_pad_index: 0,
                            },
                            ExpectedEntityPadLinkTopology {
                                kind: MediaControllerLinkKind::Data,
                                flags: &["ENABLED", "IMMUTABLE"],
                                remote_entity: "Raw Capture 1",
                                remote_pad_index: 0,
                            },
                        ],
                    }],
                },
                ExpectedEntityTopology {
                    name: "Debayer A",
                    function: media_entity_function::MEDIA_ENT_F_PROC_VIDEO_PIXEL_ENC_CONV,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Subdev,
                        ),
                    }],
                    pads: &[
                        ExpectedEntityPadTopology {
                            index: 0,
                            kind: MediaControllerPadKind::Sink,
                            flags: &["SINK"],
                            links: &[ExpectedEntityPadLinkTopology {
                                kind: MediaControllerLinkKind::Data,
                                flags: &["ENABLED", "IMMUTABLE"],
                                remote_entity: "Sensor A",
                                remote_pad_index: 0,
                            }],
                        },
                        ExpectedEntityPadTopology {
                            index: 1,
                            kind: MediaControllerPadKind::Source,
                            flags: &["SOURCE"],
                            links: &[ExpectedEntityPadLinkTopology {
                                kind: MediaControllerLinkKind::Data,
                                flags: &["ENABLED"],
                                remote_entity: "Scaler",
                                remote_pad_index: 0,
                            }],
                        },
                    ],
                },
                ExpectedEntityTopology {
                    name: "Debayer B",
                    function: media_entity_function::MEDIA_ENT_F_PROC_VIDEO_PIXEL_ENC_CONV,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Subdev,
                        ),
                    }],
                    pads: &[
                        ExpectedEntityPadTopology {
                            index: 0,
                            kind: MediaControllerPadKind::Sink,
                            flags: &["SINK"],
                            links: &[ExpectedEntityPadLinkTopology {
                                kind: MediaControllerLinkKind::Data,
                                flags: &["ENABLED", "IMMUTABLE"],
                                remote_entity: "Sensor B",
                                remote_pad_index: 0,
                            }],
                        },
                        ExpectedEntityPadTopology {
                            index: 1,
                            kind: MediaControllerPadKind::Source,
                            flags: &["SOURCE"],
                            links: &[ExpectedEntityPadLinkTopology {
                                kind: MediaControllerLinkKind::Data,
                                flags: &[],
                                remote_entity: "Scaler",
                                remote_pad_index: 0,
                            }],
                        },
                    ],
                },
                ExpectedEntityTopology {
                    name: "Raw Capture 0",
                    function: media_entity_function::MEDIA_ENT_F_IO_V4L,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Video,
                        ),
                    }],
                    pads: &[ExpectedEntityPadTopology {
                        index: 0,
                        kind: MediaControllerPadKind::Sink,
                        flags: &["SINK"],
                        links: &[ExpectedEntityPadLinkTopology {
                            kind: MediaControllerLinkKind::Data,
                            flags: &["ENABLED", "IMMUTABLE"],
                            remote_entity: "Sensor A",
                            remote_pad_index: 0,
                        }],
                    }],
                },
                ExpectedEntityTopology {
                    name: "Raw Capture 1",
                    function: media_entity_function::MEDIA_ENT_F_IO_V4L,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Video,
                        ),
                    }],
                    pads: &[ExpectedEntityPadTopology {
                        index: 0,
                        kind: MediaControllerPadKind::Sink,
                        flags: &["SINK"],
                        links: &[ExpectedEntityPadLinkTopology {
                            kind: MediaControllerLinkKind::Data,
                            flags: &["ENABLED", "IMMUTABLE"],
                            remote_entity: "Sensor B",
                            remote_pad_index: 0,
                        }],
                    }],
                },
                ExpectedEntityTopology {
                    name: "RGB/YUV Input",
                    function: media_entity_function::MEDIA_ENT_F_CAM_SENSOR,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Subdev,
                        ),
                    }],
                    pads: &[ExpectedEntityPadTopology {
                        index: 0,
                        kind: MediaControllerPadKind::Source,
                        flags: &["SOURCE"],
                        links: &[ExpectedEntityPadLinkTopology {
                            kind: MediaControllerLinkKind::Data,
                            flags: &[],
                            remote_entity: "Scaler",
                            remote_pad_index: 0,
                        }],
                    }],
                },
                ExpectedEntityTopology {
                    name: "Scaler",
                    function: media_entity_function::MEDIA_ENT_F_PROC_VIDEO_SCALER,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Subdev,
                        ),
                    }],
                    pads: &[
                        ExpectedEntityPadTopology {
                            index: 0,
                            kind: MediaControllerPadKind::Sink,
                            flags: &["SINK"],
                            links: &[
                                ExpectedEntityPadLinkTopology {
                                    kind: MediaControllerLinkKind::Data,
                                    flags: &["ENABLED"],
                                    remote_entity: "Debayer A",
                                    remote_pad_index: 1,
                                },
                                ExpectedEntityPadLinkTopology {
                                    kind: MediaControllerLinkKind::Data,
                                    flags: &[],
                                    remote_entity: "Debayer B",
                                    remote_pad_index: 1,
                                },
                                ExpectedEntityPadLinkTopology {
                                    kind: MediaControllerLinkKind::Data,
                                    flags: &[],
                                    remote_entity: "RGB/YUV Input",
                                    remote_pad_index: 0,
                                },
                            ],
                        },
                        ExpectedEntityPadTopology {
                            index: 1,
                            kind: MediaControllerPadKind::Source,
                            flags: &["SOURCE"],
                            links: &[ExpectedEntityPadLinkTopology {
                                kind: MediaControllerLinkKind::Data,
                                flags: &["ENABLED", "IMMUTABLE"],
                                remote_entity: "RGB/YUV Capture",
                                remote_pad_index: 0,
                            }],
                        },
                    ],
                },
                ExpectedEntityTopology {
                    name: "RGB/YUV Capture",
                    function: media_entity_function::MEDIA_ENT_F_IO_V4L,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Video,
                        ),
                    }],
                    pads: &[ExpectedEntityPadTopology {
                        index: 0,
                        kind: MediaControllerPadKind::Sink,
                        flags: &["SINK"],
                        links: &[ExpectedEntityPadLinkTopology {
                            kind: MediaControllerLinkKind::Data,
                            flags: &["ENABLED", "IMMUTABLE"],
                            remote_entity: "Scaler",
                            remote_pad_index: 1,
                        }],
                    }],
                },
                ExpectedEntityTopology {
                    name: "Lens A",
                    function: media_entity_function::MEDIA_ENT_F_LENS,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Subdev,
                        ),
                    }],
                    pads: &[],
                },
                ExpectedEntityTopology {
                    name: "Lens B",
                    function: media_entity_function::MEDIA_ENT_F_LENS,
                    flags: &[],
                    interfaces: &[ExpectedEntityInterfaceTopology {
                        kind: MediaControllerInterfaceKind::V4L(
                            MediaControllerInterfaceV4lKind::Subdev,
                        ),
                    }],
                    pads: &[],
                },
            ],
        },
    );
}

#[rstest]
#[test_log::test]
#[cfg_attr(not(feature = "vimc"), ignore)]
fn setup_link(#[from(get_vimc_device_path)] vimc: PathBuf) {
    let mc = MediaController::new(&vimc).unwrap();

    let entities = mc.entities().unwrap();
    let debayer_b = entities
        .iter()
        .find(|e| e.name().unwrap() == "Debayer B")
        .unwrap();

    let debayer_source_pad = debayer_b.pad(1).unwrap().unwrap();
    assert!(debayer_source_pad.is_source().unwrap());

    let scaler = entities
        .iter()
        .find(|e| e.name().unwrap() == "Scaler")
        .unwrap();

    let scaler_sink_pad = scaler.pad(0).unwrap().unwrap();
    assert!(scaler_sink_pad.is_sink().unwrap());

    let debayer_links = debayer_source_pad.links().unwrap();
    let link_from_debayer = debayer_links.first().unwrap();
    assert_eq!(
        link_from_debayer.source_id().unwrap(),
        debayer_source_pad.id().unwrap()
    );
    assert_eq!(
        link_from_debayer.sink_id().unwrap(),
        scaler_sink_pad.id().unwrap()
    );
    assert!(!link_from_debayer.is_enabled().unwrap());

    link_from_debayer.enable().unwrap();

    assert!(link_from_debayer.is_enabled().unwrap());

    let mc = MediaController::new(&vimc).unwrap();

    let entities = mc.entities().unwrap();
    let debayer_b = entities
        .iter()
        .find(|e| e.name().unwrap() == "Debayer B")
        .unwrap();

    let debayer_source_pad = debayer_b.pad(1).unwrap().unwrap();
    assert!(debayer_source_pad.is_source().unwrap());

    let scaler = entities
        .iter()
        .find(|e| e.name().unwrap() == "Scaler")
        .unwrap();

    let scaler_sink_pad = scaler.pad(0).unwrap().unwrap();
    assert!(scaler_sink_pad.is_sink().unwrap());

    let debayer_links = debayer_source_pad.links().unwrap();
    let link_from_debayer = debayer_links.first().unwrap();
    assert_eq!(
        link_from_debayer.source_id().unwrap(),
        debayer_source_pad.id().unwrap()
    );
    assert_eq!(
        link_from_debayer.sink_id().unwrap(),
        scaler_sink_pad.id().unwrap()
    );
    assert!(link_from_debayer.is_enabled().unwrap());

    // Disable our link to leave the pipeline in the same state we found it.
    link_from_debayer.disable().unwrap();
}

#[rstest]
#[test_log::test]
#[cfg_attr(not(feature = "vimc"), ignore)]
fn find_link_from_pads(#[from(get_vimc_device_path)] vimc: PathBuf) {
    let mc = MediaController::new(&vimc).unwrap();

    let entities = mc.entities().unwrap();
    let debayer_b = entities
        .iter()
        .find(|e| e.name().unwrap() == "Debayer B")
        .unwrap();

    let debayer_source_pad = debayer_b.pad(1).unwrap().unwrap();
    assert!(debayer_source_pad.is_source().unwrap());

    let scaler = entities
        .iter()
        .find(|e| e.name().unwrap() == "Scaler")
        .unwrap();

    let scaler_sink_pad = scaler.pad(0).unwrap().unwrap();
    assert!(scaler_sink_pad.is_sink().unwrap());

    let link = mc
        .find_data_link_by_pads(&debayer_source_pad, &scaler_sink_pad)
        .unwrap();
    assert!(link.is_some());
}
