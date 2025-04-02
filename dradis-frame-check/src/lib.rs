use std::{hash::Hasher, ops::Deref};

use pix::{gray::Gray8, rgb::Rgb8, Raster, Region};
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

#[doc(hidden)]
/// A representation of a Raw RGB24 Frame for our use
pub struct FrameInner(Raster<Rgb8>);

impl FrameInner {
    fn from_raw_bytes(width: usize, height: usize, bytes: &[u8]) -> Self {
        Self(Raster::with_u8_buffer(
            width as u32,
            height as u32,
            bytes.to_vec(),
        ))
    }

    fn crop(&self, width: usize, height: usize) -> Self {
        let region = Region::new(0, 0, 128, 128);

        let mut smaller = Raster::with_clear(width as u32, height as u32);
        smaller.copy_raster((), &self.0, region);

        Self(smaller)
    }

    fn to_luma(&self) -> Raster<Gray8> {
        Raster::with_raster(&self.0)
    }
}

/// A frame captured by Dradis
///
/// It's likely to have been emitted by Boomer. It contains a QRCode containing the metadata
/// describing the frame.
pub struct DradisFrame(FrameInner);

impl DradisFrame {
    pub fn from_raw_bytes(width: usize, height: usize, bytes: &[u8]) -> Self {
        Self(FrameInner::from_raw_bytes(width, height, bytes))
    }

    pub fn qrcode_content(&self) -> Result<String, FrameError> {
        debug_span!("QRCode Detection").in_scope(|| {
            let cropped = self.0.crop(128, 128);
            let luma = cropped.to_luma();

            let results =
                rxing::helpers::detect_multiple_in_luma(luma.as_u8_slice().to_vec(), 128, 128)
                    .map_err(|_| FrameError::InvalidFrame)?;

            if results.len() != 1 {
                debug!("Didn't find a QR Code");
                return Err(FrameError::InvalidFrame);
            }

            Ok(results[0].getText().to_string())
        })
    }

    pub fn metadata(&self) -> Result<Metadata, FrameError> {
        let content = self.qrcode_content()?;

        trace_span!("JSON Payload Parsing")
            .in_scope(|| serde_json::from_str(&content).map_err(|_| FrameError::IntegrityFailure))
    }

    pub fn cleared_frame_with_metadata(&self, metadata: &Metadata) -> ClearedDradisFrame {
        let mut cleared = self.0.0.clone();
        let empty = Raster::<Rgb8>::with_color(
            metadata.qrcode_width as u32,
            metadata.qrcode_height as u32,
            Rgb8::new(0, 0, 0),
        );
        cleared.copy_raster(
            Region::new(0, 0, metadata.width as u32, metadata.height as u32),
            &empty,
            (),
        );

        ClearedDradisFrame(FrameInner(cleared))
    }
}

impl Deref for DradisFrame {
    type Target = FrameInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A frame captured by Dradis, with the QRCode area cleared.
pub struct ClearedDradisFrame(FrameInner);

impl ClearedDradisFrame {
    pub fn compute_checksum(&self) -> u64 {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(self.0.0.as_u8_slice());
        hasher.finish()
    }
}

impl Deref for ClearedDradisFrame {
    type Target = FrameInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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

    let image = trace_span!("Framebuffer Importation")
        .in_scope(|| DradisFrame::from_raw_bytes(args.width, args.height, data));

    let metadata = image.metadata()?;
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

    let cleared = image.cleared_frame_with_metadata(&metadata);
    let hash = debug_span!("Checksum Computation").in_scope(|| cleared.compute_checksum());
    if hash != metadata.hash {
        warn!(
            "Hash mismatch: {:#x} vs expected {:#x}",
            hash, metadata.hash
        );
        return Err(Box::new(FrameError::IntegrityFailure));
    }

    Ok(metadata)
}
