//! Dradis Frame checker implementation
//!
//! This crate is meant to run from a raw frame, decode the metadata, and check that the frame is
//! valid.

extern crate alloc;

use alloc::rc::Rc;

use core::{cell::RefCell, hash::Hasher as _, ops::Deref};
use std::{
    fs::{self, File},
    io::{self, BufWriter},
    path::{Path, PathBuf},
};

use pix::{Raster, Region, bgr::Bgr8, gray::Gray8, rgb::Rgb8};
use png::{BitDepth, ColorType, Encoder};
use serde::Deserialize;
use thiserror::Error;
use threads_pool::ThreadPool;
use tracing::{debug, debug_span, error, trace_span, warn};
use twox_hash::XxHash64;

const HEADER_VERSION_MAJOR: u8 = 2;

/// Our Error Type.
#[derive(Debug, Error)]
pub enum FrameError {
    /// Metadata could be decoded properly, but the frame doesn't match what the metadata were
    /// describing.
    #[error("Frame Integrity Check Failed.")]
    IntegrityFailure,

    /// The frame metadata couldn't be decoded.
    #[error("Frame Header is Invalid.")]
    InvalidFrame,
}

/// Frame Metadata
#[allow(dead_code)]
#[derive(Debug, Deserialize, PartialEq)]
pub struct Metadata {
    /// Metadata Version. The first number is the major version, the second number the minor.
    /// Minors are meant to be backward compatible, majors are breaking changes.
    pub version: (u8, u8),

    /// Width of the QR Code area, in pixels.
    pub qrcode_width: u32,

    /// Height of the QR Code area, in pixels.
    pub qrcode_height: u32,

    /// Frame Width, in pixels.
    pub width: u32,

    /// Frame Height, in pixels.
    pub height: u32,

    /// Frame xxHash with the QR Code area zeroed.
    pub hash: u64,

    /// Frame index. Ever increasing.
    pub index: usize,
}

/// A representation of a Raw RGB24 Frame for our use. The pixels are stored left to right, and the
/// R, G, B color components are stored in the same order. This format is called RGB24 by v4l2,
/// BGR888 by DRM.
#[doc(hidden)]
pub struct FrameInner(Raster<Rgb8>);

impl FrameInner {
    fn from_raw_bytes(width: u32, height: u32, bytes: &[u8]) -> Self {
        Self(Raster::with_u8_buffer(width, height, bytes.to_vec()))
    }

    /// Returns the raw framebuffer content, as bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_u8_slice()
    }

    fn crop(&self, width: u32, height: u32) -> Self {
        let region = Region::new(0, 0, 128, 128);

        let mut smaller = Raster::with_clear(width, height);
        smaller.copy_raster((), &self.0, region);

        Self(smaller)
    }

    /// Returns the pixel value located at the given coordinates
    ///
    /// # Panics
    ///
    /// If the pixel coordinates can't be converted to the underlying representation.
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> Rgb8 {
        self.0.pixel(
            i32::try_from(x).expect("Can't convert i32 to u32"),
            i32::try_from(y).expect("Can't convert i32 to u32"),
        )
    }

    fn to_luma(&self) -> Raster<Gray8> {
        Raster::with_raster(&self.0)
    }

    /// Returns the height of the frame, in pixels
    #[must_use]
    pub fn height(&self) -> usize {
        self.0.height() as usize
    }

    /// Returns the width of the frame, in pixels
    #[must_use]
    pub fn width(&self) -> usize {
        self.0.width() as usize
    }

    /// Writes our framebuffer as a png image, to a file identified by the given path.
    ///
    /// # Errors
    ///
    /// If we can't access the path.
    pub fn write_to_png(&self, path: &Path) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let mut encoder = Encoder::new(writer, self.0.width(), self.0.height());
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(self.0.as_u8_slice())?;

        Ok(())
    }

    /// Writes our frame buffer as is, to a file identified by the given path.
    ///
    /// # Errors
    ///
    /// If we can't access the path.
    pub fn write_to_raw(&self, path: &Path) -> io::Result<()> {
        fs::write(path, self.0.as_u8_slice())
    }
}

impl core::fmt::Debug for FrameInner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FrameInner")
            .field("width", &self.0.width())
            .field("height", &self.0.height())
            .finish()
    }
}

/// A frame captured by Dradis
///
/// It's likely to have been emitted by Boomer. It contains a QR Code containing the metadata
/// describing the frame.
#[derive(Debug)]
pub struct DradisFrame(FrameInner);

impl DradisFrame {
    /// Creates a [`DradisFrame`] from a raw frame buffer
    #[must_use]
    pub fn from_raw_bytes(width: u32, height: u32, bytes: &[u8]) -> Self {
        Self(FrameInner::from_raw_bytes(width, height, bytes))
    }

    /// Creates a [`DradisFrame`] from a raw frame buffer, with inverted Red and Blue Color Channels
    #[must_use]
    pub fn from_raw_bytes_with_swapped_channels(width: u32, height: u32, bytes: &[u8]) -> Self {
        let bgr = Raster::<Bgr8>::with_u8_buffer(width, height, bytes.to_vec());

        Self(FrameInner(Raster::with_raster(&bgr)))
    }

    /// Decodes the QR Code content found in a [`DradisFrame`]
    ///
    /// # Errors
    ///
    /// IF the QR Code can't be decoded
    pub fn qrcode_content(&self) -> Result<String, FrameError> {
        debug_span!("QR Code Detection").in_scope(|| {
            let cropped = self.0.crop(128, 128);
            let luma = cropped.to_luma();

            let results =
                rxing::helpers::detect_multiple_in_luma(luma.as_u8_slice().to_vec(), 128, 128)
                    .map_err(|_e| FrameError::InvalidFrame)?;

            if results.len() != 1 {
                debug!("Didn't find a QR Code");
                return Err(FrameError::InvalidFrame);
            }

            Ok(results[0].getText().to_owned())
        })
    }

    /// Decodes and parses the [`Metadata`] found in a [`DradisFrame`]
    ///
    /// # Errors
    ///
    /// If the QR Code can't be decoded, or if the [`Metadata`] can't be parsed.
    pub fn metadata(&self) -> Result<Metadata, FrameError> {
        let content = self.qrcode_content()?;

        trace_span!("JSON Payload Parsing")
            .in_scope(|| serde_json::from_str(&content).map_err(|_e| FrameError::IntegrityFailure))
    }

    /// Creates a [`ClearedDradisFrame`] out of a [`DradisFrame`]
    #[must_use]
    pub fn cleared_frame(&self, clear_width: u32, clear_height: u32) -> ClearedDradisFrame {
        let mut cleared = self.0.0.clone();
        let empty = Raster::<Rgb8>::with_color(clear_width, clear_height, Rgb8::new(0, 0, 0));
        cleared.copy_raster(
            Region::new(0, 0, self.0.0.width(), self.0.0.height()),
            &empty,
            (),
        );

        ClearedDradisFrame(FrameInner(cleared))
    }

    /// Creates a [`ClearedDradisFrame`] out of a [`DradisFrame`] using preidentified [`Metadata`]
    #[must_use]
    pub fn cleared_frame_with_metadata(&self, metadata: &Metadata) -> ClearedDradisFrame {
        self.cleared_frame(metadata.qrcode_width, metadata.qrcode_height)
    }
}

impl Deref for DradisFrame {
    type Target = FrameInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A frame captured by Dradis, with the QR Code area cleared.
#[derive(Debug)]
pub struct ClearedDradisFrame(FrameInner);

impl ClearedDradisFrame {
    /// Computes the checksum of [`DradisFrame`], without the QR Code area.
    #[must_use]
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

/// [`DecodeCheckArgs`] Frame Dump Options
#[derive(Clone, Debug)]
pub struct DecodeCheckArgsDumpOptions {
    /// Pool of thread to defer our dumps to.
    pub threads_pool: Rc<RefCell<ThreadPool<()>>>,

    /// Should we dump our frames if corrupted?
    pub dump_on_corrupted_frame: bool,

    /// Should we dump our frames if valid?
    pub dump_on_valid_frame: bool,
}

/// [`DecodeCheckArgs`] Frame Dump Options, if any
#[derive(Debug)]
pub enum DecodeCheckArgsDump {
    /// Should we dump frames, ...
    Dump(DecodeCheckArgsDumpOptions),

    /// ... Or not?
    Ignore,
}

/// [`decode_and_check_frame`] Arguments
#[derive(Debug)]
pub struct DecodeCheckArgs {
    /// Previous processed frame index, if any.
    pub previous_frame_idx: Option<usize>,

    /// Width of the frame, in pixels.
    pub width: u32,

    /// Width of the frame, in pixels.
    pub height: u32,

    /// Are the Red and Blue color channels inverted?
    pub swap_channels: bool,

    /// Frame Dump options.
    pub dump: DecodeCheckArgsDump,
}

fn dump_image_to_file(
    frame_with_qr: &DradisFrame,
    frame_without_qr: &ClearedDradisFrame,
    valid: bool,
    idx: usize,
) {
    let base_path = PathBuf::from(format!(
        "dumped-buffer-{}-{idx}",
        if valid { "valid" } else { "broken" }
    ));

    if let Err(e) = frame_with_qr.write_to_raw(&base_path.with_extension("with.rgb888.raw")) {
        error!("Error writing file: {e}");
    }

    if let Err(e) = frame_without_qr.write_to_raw(&base_path.with_extension("without.rgb888.raw")) {
        error!("Error writing file: {e}");
    }

    if let Err(e) = frame_with_qr.write_to_png(&base_path.with_extension("with.png")) {
        error!("Error writing file: {e}");
    }

    if let Err(e) = frame_without_qr.write_to_raw(&base_path.with_extension("without.png")) {
        error!("Error writing file: {e}");
    }
}

/// Decodes a raw frame buffer and checks whether the frame is valid or not.
///
/// To consider a frame valid, the frame needs to:
/// - Have a QR Code that can be decoded and parsed into [`Metadata`]
/// - Its version must match our current version expectations
/// - Its index must be in sequence compared to the previous frame, if any.
/// - Its hash must be identical.
///
/// # Errors
///
/// If the frame metadata can't be decoded, or if the frame is invalid.
pub fn decode_and_check_frame(data: &[u8], args: DecodeCheckArgs) -> Result<Metadata, FrameError> {
    let last_frame_index = args.previous_frame_idx;

    let image = trace_span!("Framebuffer Importation").in_scope(|| {
        if args.swap_channels {
            DradisFrame::from_raw_bytes_with_swapped_channels(args.width, args.height, data)
        } else {
            DradisFrame::from_raw_bytes(args.width, args.height, data)
        }
    });

    let metadata = image.metadata()?;
    if metadata.version.0 != HEADER_VERSION_MAJOR {
        warn!("Metadata Version Mismatch");
        return Err(FrameError::IntegrityFailure);
    }

    if let Some(last_index) = last_frame_index {
        let index = metadata.index;

        if index < last_index {
            warn!("Frame Index Mismatch");
            return Err(FrameError::IntegrityFailure);
        } else if index == last_index {
            debug!("Source cannot keep up?");
        } else if index > last_index + 1 {
            warn!("Dropped Frame!");
        }
    }

    let cleared = image.cleared_frame_with_metadata(&metadata);
    let hash = debug_span!("Checksum Computation").in_scope(|| cleared.compute_checksum());

    if let DecodeCheckArgsDump::Dump(o) = args.dump {
        if o.dump_on_corrupted_frame {
            o.threads_pool.borrow_mut().spawn_and_queue(move || {
                dump_image_to_file(&image, &cleared, hash == metadata.hash, metadata.index);
            });
        }
    }

    if hash != metadata.hash {
        warn!(
            "Hash mismatch: {:#x} vs expected {:#x}",
            hash, metadata.hash
        );

        return Err(FrameError::IntegrityFailure);
    }

    Ok(metadata)
}
