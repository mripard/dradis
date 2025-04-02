use std::hash::Hasher;

use pix::{bgr::Bgr8, gray::Gray8, Raster, Region};
use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, debug_span, trace_span, warn};
use twox_hash::XxHash64;

const HEADER_VERSION_MAJOR: u8 = 2;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("Frame Integrity Check Failed.")]
    IntegrityFailure,

    #[error("Frame Header is Invalid.")]
    InvalidFrame,
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

pub struct DecodeCheckArgs {
    pub previous_frame_idx: Option<usize>,
    pub width: usize,
    pub height: usize,
}

pub fn decode_and_check_frame(
    data: &[u8],
    args: Option<DecodeCheckArgs>,
) -> std::result::Result<Metadata, Box<dyn std::error::Error>> {
    let args = args.expect("Missing arguments");
    let last_frame_index = args.previous_frame_idx;
    let width = u32::try_from(args.width).expect("Width doesn't fit into a u32");
    let height = u32::try_from(args.height).expect("Height doesn't fit into a u32");

    let mut image = trace_span!("Framebuffer Importation").in_scope(|| {
        let pixels = data.to_vec();

        Raster::<Bgr8>::with_u8_buffer(width, height, pixels)
    });

    let content = debug_span!("QRCode Detection").in_scope(|| {
        let region = Region::new(0, 0, 128, 128);

        let mut qr_raster = Raster::with_clear(128, 128);
        qr_raster.copy_raster((), &image, region);

        let luma = Raster::<Gray8>::with_raster(&qr_raster);
        let results =
            rxing::helpers::detect_multiple_in_luma(luma.as_u8_slice().to_vec(), 128, 128)
                .map_err(|_| FrameError::InvalidFrame)?;

        if results.len() != 1 {
            debug!("Didn't find a QR Code");
            return Err(Box::new(FrameError::InvalidFrame));
        }

        Ok(results[0].getText().to_string())
    })?;

    let metadata: Metadata = trace_span!("JSON Payload Parsing")
        .in_scope(|| serde_json::from_str(&content).map_err(|_| FrameError::IntegrityFailure))?;

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

    let empty_raster = Raster::<Bgr8>::with_color(qrcode_width, qrcode_height, Bgr8::new(0, 0, 0));
    image.copy_raster(
        Region::new(0, 0, qrcode_width, qrcode_height),
        &empty_raster,
        (),
    );

    let hash = debug_span!("Checksum Computation").in_scope(|| {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(image.as_u8_slice());
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
