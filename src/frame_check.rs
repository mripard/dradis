use std::hash::Hasher;

use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgb, Rgba};
use log::{debug, warn};
use rqrr::PreparedImage;
use serde::Deserialize;
use strum_macros::Display;
use twox_hash::XxHash64;

const HEADER_VERSION_MAJOR: u8 = 2;

#[derive(Display, Debug)]
enum FrameError {
    Invalid,
}

impl std::error::Error for FrameError {}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Metadata {
    version: (u8, u8),
    qrcode_width: usize,
    qrcode_height: usize,
    width: usize,
    height: usize,
    hash: u64,
    index: usize,
}

pub fn decode_and_check_frame(
    data: &[u8],
    args: Option<(Option<usize>, usize, usize)>,
) -> std::result::Result<usize, Box<dyn std::error::Error>> {
    let args = args.unwrap();
    let last_frame_index = args.0;
    let width = args.1;
    let height = args.2;

    let pixels = data.to_vec();
    let buffer =
        ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width as u32, height as u32, pixels).unwrap();

    let mut image = DynamicImage::ImageRgb8(buffer);
    let luma = image.to_luma8().view(0, 0, 128, 128).to_image();
    let mut prepared = PreparedImage::prepare(luma);

    let grids = prepared.detect_grids();
    if grids.len() != 1 {
        warn!("Didn't find a QR Code");
        return Err(FrameError::Invalid)?;
    }

    let grid = &grids[0];
    let (_, content) = grid.decode().unwrap();

    let metadata: Metadata = serde_json::from_str(&content).unwrap();

    if metadata.version.0 != HEADER_VERSION_MAJOR {
        warn!("Metadata Version Mismatch");
        return Err(FrameError::Invalid)?;
    }

    if let Some(last_index) = last_frame_index {
        let index = metadata.index as usize;

        if index < last_index {
            warn!("Frame Index Mismatch");
            return Err(FrameError::Invalid)?;
        } else if index == last_index {
            debug!("Source cannot keep up?");
        } else if index > last_index + 1 {
            warn!("Dropped Frame!");
        }
    }

    for x in 0..metadata.qrcode_width {
        for y in 0..metadata.qrcode_height {
            image.put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 255]));
        }
    }

    let mut hasher = XxHash64::with_seed(0);
    hasher.write(image.as_bytes());
    let hash = hasher.finish();

    if hash != metadata.hash {
        warn!(
            "Hash mismatch: {:#x} vs expected {:#x}",
            hash, metadata.hash
        );
        return Err(FrameError::Invalid)?;
    }

    Ok(metadata.index)
}
