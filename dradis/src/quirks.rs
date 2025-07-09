use std::{collections::HashMap, io};

use linux_mc::MediaController;
use rustix::io::Errno;
use tracing::debug;

use crate::PipelineItem;

/// Type of the functions that apply quirks for a given device and bridge
type QuirkFunction = fn(&MediaController, &[PipelineItem]) -> Result<(), io::Error>;

/// Applies, if required, quirks to a pair of device plus bridge.
pub fn apply_quirks(mc: &MediaController, pipeline: &[PipelineItem]) -> Result<(), io::Error> {
    let quirks: HashMap<&str, QuirkFunction> = HashMap::from([]);

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
