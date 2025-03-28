use std::hash::Hasher;

use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgb, Rgba};
use rqrr::PreparedImage;
use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, debug_span, trace_span, warn};
use twox_hash::XxHash64;

const HEADER_VERSION_MAJOR: u8 = 2;

#[derive(Debug, Error)]
enum FrameError {
    #[error("Frame Integrity Check Failed.")]
    IntegrityFailure,

    #[error("Frame Header is Invalid.")]
    InvalidFrame,

    #[error("Not Enough Memory.")]
    NotEnoughMemory,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, PartialEq)]
pub struct Metadata {
    pub version: (u8, u8),
    pub qrcode_width: usize,
    pub qrcode_height: usize,
    pub width: usize,
    pub height: usize,
    pub hash: u64,
    pub index: usize,
}

pub fn decode_and_check_frame(
    data: &[u8],
    args: Option<(Option<usize>, usize, usize)>,
) -> std::result::Result<Metadata, Box<dyn std::error::Error>> {
    let args = args.expect("Missing arguments");
    let last_frame_index = args.0;
    let width = u32::try_from(args.1).expect("Width doesn't fit into a u32");
    let height = u32::try_from(args.2).expect("Height doesn't fit into a u32");

    let mut image = trace_span!("Framebuffer Importation").in_scope(|| {
        let pixels = data.to_vec();
        let buffer = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width, height, pixels)
            .ok_or(FrameError::NotEnoughMemory)?;

        Ok::<DynamicImage, FrameError>(DynamicImage::ImageRgb8(buffer))
    })?;

    let luma = image.to_luma8().view(0, 0, 128, 128).to_image();
    let mut prepared = PreparedImage::prepare(luma);

    let grids = prepared.detect_grids();
    if grids.len() != 1 {
        debug!("Didn't find a QR Code");
        return Err(Box::new(FrameError::InvalidFrame));
    }

    let grid = &grids[0];
    let (_, content) = grid.decode().map_err(|_| FrameError::InvalidFrame)?;

    let metadata: Metadata =
        serde_json::from_str(&content).map_err(|_| FrameError::IntegrityFailure)?;

    if metadata.version.0 != HEADER_VERSION_MAJOR {
        warn!("Metadata Version Mismatch");
        return Err(Box::new(FrameError::IntegrityFailure));
    }

    if let Some(last_index) = last_frame_index {
        let index = metadata.index;

        if index < last_index {
            warn!("Frame Index Mismatch");
            return Err(Box::new(FrameError::IntegrityFailure));
        } else if index == last_index {
            debug!("Source cannot keep up?");
        } else if index > last_index + 1 {
            warn!("Dropped Frame!");
        }
    }

    let qrcode_width =
        u32::try_from(metadata.qrcode_width).expect("QR Code Width doesn't fit into a u32");
    let qrcode_height =
        u32::try_from(metadata.qrcode_height).expect("QR Code Height doesn't fit into a u32");

    for x in 0..qrcode_width {
        for y in 0..qrcode_height {
            image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
    }

    let hash = debug_span!("Checksum Computation").in_scope(|| {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(image.as_bytes());
        hasher.finish()
    });

    if hash != metadata.hash {
        warn!(
            "Hash mismatch: {:#x} vs expected {:#x}",
            hash, metadata.hash
        );
        return Err(Box::new(FrameError::IntegrityFailure));
    }

    Ok(metadata)
}
