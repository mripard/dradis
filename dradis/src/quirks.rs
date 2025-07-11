use std::{collections::HashMap, io};

use linux_mc::MediaController;
use rustix::io::Errno;
use tracing::debug;

use crate::PipelineItem;

/// Type of the functions that apply quirks for a given device and bridge
type QuirkFunction = fn(&MediaController, &[PipelineItem]) -> Result<(), io::Error>;

/// Applies, if required, quirks to a pair of device plus bridge.
pub fn apply_quirks(mc: &MediaController, pipeline: &[PipelineItem]) -> Result<(), io::Error> {
    let quirks: HashMap<&str, QuirkFunction> = HashMap::from([
        // Raspberry Pi 5 + GeekWorm C779
        (
            "rp1-cfe-csi2_ch0:tc358743 11-000f",
            rpi5_geekworm_c779 as QuirkFunction,
        ),
    ]);

    let PipelineItem(_, root, _) = pipeline
        .first()
        .ok_or(io::Error::new(Errno::NODEV.kind(), "Missing Root Entity"))?;

    let PipelineItem(_, bridge, _) = pipeline.last().ok_or(io::Error::new(
        Errno::NODEV.kind(),
        "Missing HDMI Bridge Entity",
    ))?;

    let root_name = root.entity.name().valid();
    let bridge_name = bridge.entity.name().valid();
    let key = format!("{root_name}:{bridge_name}");

    if let Some(quirk_fn) = quirks.get(key.as_str()) {
        debug!("Applying quirks for {}", key);
        quirk_fn(mc, pipeline)?;
    }

    Ok(())
}

/// On Raspberry Pi 5 the C779 bridge requires this link:
/// $ media-ctl -d /dev/mediaX -l ''\''csi2'\'':4 -> '\''rp1-cfe-csi2_ch0'\'':0 [1]'
/// For more information, see: <https://wiki.geekworm.com/CSI_Manual_on_Pi_5>
fn rpi5_geekworm_c779(mc: &MediaController, pipeline: &[PipelineItem]) -> Result<(), io::Error> {
    let PipelineItem(sink, _, _) = pipeline
        .first()
        .ok_or(io::Error::new(Errno::NODEV.kind(), "Missing Root Entity"))?;

    let root_sink_pad = sink
        .as_ref()
        .ok_or(io::Error::new(Errno::NODEV.kind(), "Missing Root Sink Pad"))?;

    let csi2_source_pad = root_sink_pad.remote_pad().valid()?.ok_or(io::Error::new(
        Errno::NODEV.kind(),
        "Missing CSI2 Source Pad",
    ))?;

    mc.setup_link(&csi2_source_pad, root_sink_pad, 1)
}
