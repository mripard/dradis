//! Dradis Frame checker implementation
//!
//! This crate is meant to run from a raw frame, decode the metadata, and check that the frame is
//! valid.

#![allow(unsafe_code)]

extern crate alloc;

use alloc::{borrow::Cow, rc::Rc, sync::Arc};
use core::{cell::RefCell, fmt, ops::Deref};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, BufWriter},
    path::Path,
};

use bon::Builder;
use image::imageops::FilterType;
use pix::{
    Raster, Region,
    bgr::{Bgr8, Bgra8},
    chan::Ch8,
    el::Pixel,
    rgb::Rgb8,
};
use png::{BitDepth, ColorType, Encoder};
use rxing::{
    BarcodeFormat, BinaryBitmap, DecodeHints, LuminanceSource, MultiFormatReader, Reader as _,
    common::GlobalHistogramBinarizer,
};
use serde::{Deserialize, Deserializer, Serialize};
use static_assertions::const_assert_eq;
use thiserror::Error;
use threads_pool::ThreadPool;
use tracing::{debug, error, trace_span, warn};
use twox_hash::{XxHash3_64, XxHash64};

mod asm;
use asm::optimized_memcpy;

const HEADER_VERSION_MAJOR: u8 = 2;
const HEADER_VERSION_MINOR: u8 = 1;

/// Width of the QR Code Area, in pixels.
pub const QRCODE_WIDTH: u32 = 128;

/// Height of the QR Code Area, in pixels.
pub const QRCODE_HEIGHT: u32 = 128;

fn optimized_slice_to_vec<T>(slice: &[T]) -> Vec<T> {
    let mut vec = Vec::with_capacity(slice.len());

    optimized_memcpy(vec.as_mut_ptr(), slice.as_ptr(), slice.len());

    unsafe {
        vec.set_len(slice.len());
    }

    vec
}

/// Our Error Type.
#[derive(Debug, Error, PartialEq)]
pub enum FrameError {
    /// Metadata could be decoded properly, but the frame doesn't match what the metadata were
    /// describing.
    #[error("Frame Integrity Check Failed.")]
    IntegrityFailure,

    /// The frame metadata couldn't be decoded.
    #[error("Frame Header is Invalid.")]
    InvalidFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase", tag = "hash_type", content = "hash_value")]
pub enum HashVariant {
    XxHash2(u64),
    XxHash3(u64),
}

impl fmt::Display for HashVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XxHash2(v) => f.write_fmt(format_args!("xxHash2 {v:#x}")),
            Self::XxHash3(v) => f.write_fmt(format_args!("xxHash3 {v:#x}")),
        }
    }
}

impl<'de> Deserialize<'de> for HashVariant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "lowercase", tag = "hash_type", content = "hash_value")]
        enum EnumVariantHelper {
            XxHash2(u64),
            XxHash3(u64),
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum VariantHelper {
            Variant1(EnumVariantHelper),
            Variant0(u64),
        }

        Ok(match VariantHelper::deserialize(deserializer)? {
            VariantHelper::Variant1(h) => match h {
                EnumVariantHelper::XxHash2(v) => HashVariant::XxHash2(v),
                EnumVariantHelper::XxHash3(v) => HashVariant::XxHash3(v),
            },
            VariantHelper::Variant0(v) => HashVariant::XxHash2(v),
        })
    }
}

/// Frame Metadata
#[allow(dead_code)]
#[derive(Builder, Debug, Deserialize, PartialEq, Serialize)]
pub struct Metadata {
    /// Metadata Version. The first number is the major version, the second number the minor.
    /// Minors are meant to be backward compatible, majors are breaking changes.
    #[builder(skip = (HEADER_VERSION_MAJOR, HEADER_VERSION_MINOR))]
    pub version: (u8, u8),

    /// Width of the QR Code area, in pixels.
    #[builder(skip = QRCODE_WIDTH)]
    pub qrcode_width: u32,

    /// Height of the QR Code area, in pixels.
    #[builder(skip = QRCODE_HEIGHT)]
    pub qrcode_height: u32,

    /// Frame Width, in pixels.
    pub width: u32,

    /// Frame Height, in pixels.
    pub height: u32,

    /// Frame xxHash with the QR Code area zeroed.
    pub hash: HashVariant,

    /// Frame index. Ever increasing.
    pub index: usize,
}

#[cfg(test)]
mod metadata_tests {
    use crate::{HEADER_VERSION_MAJOR, HashVariant, Metadata, QRCODE_HEIGHT, QRCODE_WIDTH};

    #[test]
    fn metadata_without_variant() {
        let json = r#"
        {
            "version": [2, 0],
            "qrcode_width": 128,
            "qrcode_height": 128,
            "width": 1280,
            "height": 720,
            "hash": 14833666787937248486,
            "index": 2
        }"#;

        assert_eq!(
            serde_json::from_str::<Metadata>(json).unwrap(),
            Metadata {
                version: (HEADER_VERSION_MAJOR, 0),
                qrcode_width: QRCODE_WIDTH,
                qrcode_height: QRCODE_HEIGHT,
                width: 1280,
                height: 720,
                hash: HashVariant::XxHash2(0xcddbc559fb8264e6),
                index: 2
            }
        );
    }

    #[test]
    fn metadata_with_variant_xxhash2() {
        let json = r#"
        {
            "version": [2, 1],
            "qrcode_width": 128,
            "qrcode_height": 128,
            "width": 1280,
            "height": 720,
            "hash": {
                "hash_type": "xxhash2",
                "hash_value": 14833666787937248486
            },
            "index": 2
        }"#;

        assert_eq!(
            serde_json::from_str::<Metadata>(json).unwrap(),
            Metadata {
                version: (HEADER_VERSION_MAJOR, 1),
                qrcode_width: QRCODE_WIDTH,
                qrcode_height: QRCODE_HEIGHT,
                width: 1280,
                height: 720,
                hash: HashVariant::XxHash2(0xcddbc559fb8264e6),
                index: 2
            }
        );
    }

    #[test]
    fn metadata_with_variant_xxhash3() {
        let json = r#"
        {
            "version": [2, 1],
            "qrcode_width": 128,
            "qrcode_height": 128,
            "width": 1280,
            "height": 720,
            "hash": {
                "hash_type": "xxhash3",
                "hash_value": 14833666787937248486
            },
            "index": 2
        }"#;

        assert_eq!(
            serde_json::from_str::<Metadata>(json).unwrap(),
            Metadata {
                version: (HEADER_VERSION_MAJOR, 1),
                qrcode_width: QRCODE_WIDTH,
                qrcode_height: QRCODE_HEIGHT,
                width: 1280,
                height: 720,
                hash: HashVariant::XxHash3(0xcddbc559fb8264e6),
                index: 2
            }
        );
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Metadata Version {}.{}, Frame Size {}x{}, QR Code Area {}x{}, index {}, hash {}",
            self.version.0,
            self.version.1,
            self.width,
            self.height,
            self.qrcode_width,
            self.qrcode_height,
            self.index,
            self.hash,
        ))
    }
}

struct CustomRgb24Source {
    luma: Box<[u8]>,
    width: u32,
    height: u32,
}

impl CustomRgb24Source {
    fn new_with_region<P>(pixels: &FrameInner<P>, region: Region) -> Self
    where
        P: FramePixel + Pixel<Chan = Ch8>,
    {
        let luma = pixels
            .0
            .rows(region)
            .flatten()
            .map(|p| p.two().into())
            .collect::<Vec<u8>>();

        Self {
            luma: luma.into_boxed_slice(),
            height: region.height(),
            width: region.width(),
        }
    }
}

impl LuminanceSource for CustomRgb24Source {
    const SUPPORTS_CROP: bool = false;
    const SUPPORTS_ROTATION: bool = false;

    fn get_row(&self, y: usize) -> Option<Cow<'_, [u8]>> {
        if y >= self.get_height() {
            return None;
        }

        let width = self.get_width();
        let offset = (y) * width;

        Some(Cow::Borrowed(&self.luma[offset..offset + width]))
    }

    fn get_column(&self, _x: usize) -> Vec<u8> {
        unimplemented!()
    }

    fn get_matrix(&self) -> Vec<u8> {
        self.luma.to_vec()
    }

    fn get_width(&self) -> usize {
        self.width
            .try_into()
            .expect("A u32 should always fit into a usize on a linux platform")
    }

    fn get_height(&self) -> usize {
        self.height
            .try_into()
            .expect("A u32 should always fit into a usize on a linux platform")
    }

    fn invert(&mut self) {
        unimplemented!()
    }

    fn get_luma8_point(&self, _x: usize, _y: usize) -> u8 {
        unimplemented!()
    }
}

#[doc(hidden)]
pub trait FramePixel: Pixel<Chan = Ch8> {
    fn from_raw_bytes(bytes: &[u8]) -> &[Self] {
        let ptr = bytes.as_ptr();
        let len = bytes.len();
        assert_eq!(len % size_of::<Self>(), 0);

        #[allow(unsafe_code)]
        unsafe {
            std::slice::from_raw_parts(ptr as *const Self, len / size_of::<Self>())
        }
    }
}

// The pixels are stored left to right, and the B, G, R color components are stored in the same
// order. This format is called BGR24 by v4l2, RGB888 by DRM.
const_assert_eq!(size_of::<Bgr8>(), 3);
const_assert_eq!(align_of::<Bgr8>(), 1);
impl FramePixel for Bgr8 {}

#[cfg(test)]
mod bgr8_framepixel_tests {
    use pix::bgr::Bgr8;

    use crate::FramePixel as _;

    #[test]
    fn transmute() {
        assert_eq!(
            Bgr8::from_raw_bytes(&[42, 21, 128, 84, 46, 72]),
            &[Bgr8::new(42, 21, 128), Bgr8::new(84, 46, 72),]
        )
    }
}

// The pixels are stored left to right, and the R, G, B color components are stored in the same
// order. This format is called RGB24 by v4l2, BGR888 by DRM.
const_assert_eq!(size_of::<Rgb8>(), 3);
const_assert_eq!(align_of::<Rgb8>(), 1);
impl FramePixel for Rgb8 {}

#[cfg(test)]
mod rgb8_framepixel_tests {
    use pix::rgb::Rgb8;

    use crate::FramePixel as _;

    #[test]
    fn transmute() {
        assert_eq!(
            Rgb8::from_raw_bytes(&[42, 21, 128, 84, 46, 72]),
            &[Rgb8::new(42, 21, 128), Rgb8::new(84, 46, 72),]
        )
    }
}

// The pixels are stored left to right, and the R, G, B, A components are stored in the same order.
// This format is called ABGR32 by v4l2, and  ARGB8888 by KMS.
const_assert_eq!(size_of::<Bgra8>(), 4);
const_assert_eq!(align_of::<Bgra8>(), 1);
impl FramePixel for Bgra8 {}

#[cfg(test)]
mod bgra8_framepixel_tests {
    use pix::bgr::Bgra8;

    use crate::FramePixel as _;

    #[test]
    fn transmute() {
        assert_eq!(
            Bgra8::from_raw_bytes(&[42, 21, 128, 63, 84, 46, 72, 124]),
            &[Bgra8::new(42, 21, 128, 63), Bgra8::new(84, 46, 72, 124),]
        )
    }
}

/// A representation of a raw RGB Frame with 8 bits per components.
#[doc(hidden)]
pub struct FrameInner<P>(Raster<P>)
where
    P: FramePixel + Pixel<Chan = Ch8>;

impl<P> FrameInner<P>
where
    P: FramePixel + Pixel<Chan = Ch8>,
{
    fn from_raw_bytes(width: u32, height: u32, bytes: &[u8]) -> Self {
        Self(Raster::<P>::with_u8_buffer(
            width,
            height,
            optimized_slice_to_vec(bytes),
        ))
    }

    /// Returns the raw framebuffer content, as bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_u8_slice()
    }

    fn clear(&self, width: u32, height: u32) -> Self {
        let empty_pixel = Rgb8::new(0, 0, 0).convert();

        let mut cleared = Raster::<P>::with_pixels(
            self.0.width(),
            self.0.height(),
            optimized_slice_to_vec(self.0.pixels()),
        );

        let empty = Raster::<P>::with_color(width, height, empty_pixel);
        cleared.copy_raster(
            Region::new(0, 0, self.0.width(), self.0.height()),
            &empty,
            (),
        );

        FrameInner(cleared)
    }

    /// Returns the pixel value located at the given coordinates
    ///
    /// # Panics
    ///
    /// If the pixel coordinates can't be converted to the underlying representation.
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> P {
        self.0.pixel(
            i32::try_from(x).expect("Can't convert i32 to u32"),
            i32::try_from(y).expect("Can't convert i32 to u32"),
        )
    }

    /// Returns the height of the frame, in pixels
    #[must_use]
    pub fn height(&self) -> usize {
        self.0.height() as usize
    }

    fn to_pixel_format<D>(&self) -> FrameInner<D>
    where
        D: FramePixel + Pixel<Chan = Ch8>,
    {
        FrameInner(Raster::with_raster(&self.0))
    }

    /// Returns the width of the frame, in pixels
    #[must_use]
    pub fn width(&self) -> usize {
        self.0.width() as usize
    }

    /// Writes our frame buffer as is, to a file identified by the given path.
    ///
    /// # Errors
    ///
    /// If we can't access the path.
    pub fn write_to_raw<PA>(&self, path: PA) -> io::Result<()>
    where
        PA: AsRef<Path>,
    {
        fs::write(path, self.0.as_u8_slice())
    }
}

impl FrameInner<Rgb8> {
    /// Writes our framebuffer as a png image, to a file identified by the given path.
    ///
    /// # Errors
    ///
    /// If we can't access the path.
    pub fn write_to_png<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let mut encoder = Encoder::new(writer, self.0.width(), self.0.height());
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(self.0.as_u8_slice())?;

        Ok(())
    }
}

impl<P> fmt::Debug for FrameInner<P>
where
    P: FramePixel + Pixel<Chan = Ch8>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameInner")
            .field("width", &self.0.width())
            .field("height", &self.0.height())
            .finish()
    }
}

/// A Frame with an the QR Code area cleared, and an embedded QR Code in that area.
///
/// It's likely to have been emitted by Boomer, and received by Dradis. The QR Code contains the
/// metadata describing the frame.
#[derive(Debug)]
pub struct QRCodeFrame<P>(FrameInner<P>)
where
    P: FramePixel;

impl<P> QRCodeFrame<P>
where
    P: FramePixel,
{
    /// Creates a [`QRCodeFrame`] from a raw frame buffer
    #[must_use]
    pub fn from_raw_bytes(width: u32, height: u32, bytes: &[u8]) -> Self {
        Self(FrameInner::from_raw_bytes(width, height, bytes))
    }

    /// Decodes the QR Code content found in a [`QRCodeFrame`]
    ///
    /// # Errors
    ///
    /// IF the QR Code can't be decoded
    pub fn qrcode_content(&self) -> Result<String, FrameError> {
        trace_span!("QR Code Detection").in_scope(|| {
            let mut reader = MultiFormatReader::default();

            let results = reader
                .decode_with_hints(
                    &mut BinaryBitmap::new(GlobalHistogramBinarizer::new(
                        CustomRgb24Source::new_with_region(
                            &self.0,
                            Region::new(0, 0, QRCODE_WIDTH, QRCODE_HEIGHT),
                        ),
                    )),
                    &DecodeHints {
                        PossibleFormats: Some(HashSet::from([BarcodeFormat::QR_CODE])),
                        TryHarder: Some(true),

                        ..Default::default()
                    },
                )
                .map_err(|_e| {
                    warn!("Couldn't detect a QR Code.");
                    FrameError::InvalidFrame
                })?;

            Ok(results.getText().to_owned())
        })
    }

    /// Converts a [`QRCodeFrame`] pixel type to another
    #[must_use]
    pub fn to_pixel_format<D>(&self) -> QRCodeFrame<D>
    where
        D: FramePixel + Pixel<Chan = Ch8>,
    {
        QRCodeFrame(self.0.to_pixel_format())
    }

    /// Decodes and parses the [`Metadata`] found in a [`QRCodeFrame`]
    ///
    /// # Errors
    ///
    /// If the QR Code can't be decoded, or if the [`Metadata`] can't be parsed.
    pub fn metadata(&self) -> Result<Metadata, FrameError> {
        let content = self.qrcode_content()?;

        debug!("JSON Payload: {content}");

        trace_span!("JSON Payload Parsing").in_scope(|| {
            serde_json::from_str(&content).map_err(|_e| {
                warn!("Couldn't parse JSON content.");
                FrameError::IntegrityFailure
            })
        })
    }

    /// Creates a [`ClearedFrame`] out of a [`QRCodeFrame`]
    #[must_use]
    pub fn cleared_frame(&self, clear_width: u32, clear_height: u32) -> ClearedFrame<P> {
        ClearedFrame(self.0.clear(clear_width, clear_height))
    }

    /// Creates a [`ClearedFrame`] out of a [`QRCodeFrame`] using preidentified [`Metadata`]
    #[must_use]
    pub fn cleared_frame_with_metadata(&self, metadata: &Metadata) -> ClearedFrame<P> {
        self.cleared_frame(metadata.qrcode_width, metadata.qrcode_height)
    }
}

impl QRCodeFrame<Rgb8> {
    /// Creates a [`QRCodeFrame`] from a raw frame buffer, with inverted Red and Blue Color
    /// Channels
    #[must_use]
    pub fn from_raw_bytes_with_swapped_channels(width: u32, height: u32, bytes: &[u8]) -> Self {
        Self(FrameInner(Raster::<Rgb8>::with_pixels(
            width,
            height,
            Bgr8::from_raw_bytes(bytes)
                .iter()
                .map(|b| Rgb8::new(b.three(), b.two(), b.one()))
                .collect::<Vec<_>>(),
        )))
    }
}

impl<P> Deref for QRCodeFrame<P>
where
    P: FramePixel,
{
    type Target = FrameInner<P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A Frame with the QR Code area cleared.
#[derive(Debug)]
pub struct ClearedFrame<P>(FrameInner<P>)
where
    P: FramePixel;

impl<P> ClearedFrame<P>
where
    P: FramePixel,
{
    /// Converts a [`ClearedFrame`] pixel type to another.
    #[must_use]
    pub fn to_pixel_format<D>(&self) -> ClearedFrame<D>
    where
        D: FramePixel + Pixel<Chan = Ch8>,
    {
        ClearedFrame(self.0.to_pixel_format())
    }

    /// Adds a QR Code to a [`ClearedFrame`] to create a [`QRCodeFrame`]
    #[must_use]
    pub fn with_qr_code(&self, qr: &Raster<P>) -> QRCodeFrame<P> {
        let mut merged = self.0.0.clone();
        merged.copy_raster((0, 0, QRCODE_WIDTH, QRCODE_HEIGHT), qr, ());

        QRCodeFrame(FrameInner(merged))
    }
}

impl ClearedFrame<Rgb8> {
    /// Computes the XxHash2 checksum of [`QRCodeFrame`], without the QR Code area. Only relevant for RGB24.
    #[must_use]
    pub fn compute_xxhash2_checksum(&self) -> HashVariant {
        HashVariant::XxHash2(XxHash64::oneshot(0, self.0.0.as_u8_slice()))
    }

    /// Computes the XxHash3 checksum of [`QRCodeFrame`], without the QR Code area. Only relevant for RGB24.
    #[must_use]
    pub fn compute_xxhash3_checksum(&self) -> HashVariant {
        HashVariant::XxHash3(XxHash3_64::oneshot_with_seed(0, self.0.0.as_u8_slice()))
    }
}

impl<P> Deref for ClearedFrame<P>
where
    P: FramePixel,
{
    type Target = FrameInner<P>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A Frame.
#[derive(Debug)]
pub struct Frame(FrameInner<Rgb8>);

impl Frame {
    /// Creates a [`ClearedFrame`] out of a [`Frame`]
    #[must_use]
    pub fn clear(self) -> ClearedFrame<Rgb8> {
        ClearedFrame(self.0.clear(QRCODE_WIDTH, QRCODE_HEIGHT))
    }

    /// Creates a [`Frame`] from an SVG, at the given frame size
    ///
    /// # Errors
    ///
    /// If the SVG parsing fails
    pub fn from_svg_with_size(bytes: &[u8], width: u32, height: u32) -> io::Result<Self> {
        Ok(Self(FrameInner(Raster::with_u8_buffer(
            width,
            height,
            image::load_from_memory(bytes)
                .map_err(|_e| io::Error::new(io::ErrorKind::InvalidData, "Invalid SVG"))?
                .resize_exact(width, height, FilterType::Nearest)
                .to_rgb8()
                .to_vec(),
        ))))
    }
}

impl Deref for Frame {
    type Target = FrameInner<Rgb8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// [`DecodeCheckArgs`] Frame Dump Options, if any
#[derive(Debug)]
pub enum DecodeCheckArgsDump {
    /// Always dump received frames. In this case, the farme index used in the file name is the
    /// `v4l2_buffer` sequence number.
    Always(Rc<RefCell<ThreadPool<()>>>),

    /// Dump corrupted frames only
    Corrupted(Rc<RefCell<ThreadPool<()>>>),

    /// Never dump frames
    Never,
}

/// [`decode_and_check_frame`] Arguments
#[derive(Debug)]
pub struct DecodeCheckArgs {
    /// V4L2 Sequence Number
    pub sequence: u32,

    /// Previous processed frame index, if any.
    pub previous_frame_idx: Option<usize>,

    /// Width of the frame, in pixels.
    pub width: u32,

    /// Width of the frame, in pixels.
    pub height: u32,

    /// The frame is actually in the Bgr8 format, we need to swap red and blue channels.
    pub swap_channels: bool,

    /// Frame Dump options.
    pub dump: DecodeCheckArgsDump,
}

/// Decodes a raw RGB8 frame buffer and checks whether the frame is valid or not.
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
#[expect(
    clippy::needless_pass_by_value,
    reason = "Yes, clippy, that's really what we want."
)]
pub fn decode_and_check_frame(data: &[u8], args: DecodeCheckArgs) -> Result<Metadata, FrameError> {
    let last_frame_index = args.previous_frame_idx;

    let image: Arc<QRCodeFrame<Rgb8>> = trace_span!("Framebuffer Importation").in_scope(|| {
        Arc::new(if args.swap_channels {
            QRCodeFrame::<Rgb8>::from_raw_bytes_with_swapped_channels(args.width, args.height, data)
        } else {
            QRCodeFrame::<Rgb8>::from_raw_bytes(args.width, args.height, data)
        })
    });

    if let DecodeCheckArgsDump::Always(pool) = &args.dump {
        let thread_image = image.clone();

        pool.borrow_mut().spawn_and_queue(move || {
            if let Err(e) =
                thread_image.write_to_png(format!("dumped-buffer-{}.png", args.sequence))
            {
                error!("Error writing file: {e}");
            }

            if let Err(e) =
                thread_image.write_to_raw(format!("dumped-buffer-{}.rgb888.raw", args.sequence))
            {
                error!("Error writing file: {e}");
            }
        });
    }

    let metadata = image.metadata()?;
    if metadata.version.0 != HEADER_VERSION_MAJOR {
        warn!("Metadata Version Mismatch");
        return Err(FrameError::IntegrityFailure);
    }

    debug!("Frame {}: Found Metadata {metadata}", metadata.index);

    if let Some(last_index) = last_frame_index {
        let index = metadata.index;

        if index < last_index {
            warn!("Frame {}: Frame Index Mismatch", metadata.index);
            return Err(FrameError::IntegrityFailure);
        } else if index == last_index {
            debug!("Frame {}: Source cannot keep up?", metadata.index);
        } else if index > last_index + 1 {
            warn!("Frame {}: Dropped Frame!", metadata.index);
        }
    }

    let cleared = image.cleared_frame_with_metadata(&metadata);

    let hash = trace_span!("Checksum Computation").in_scope(|| match metadata.hash {
        HashVariant::XxHash2(_) => cleared.compute_xxhash2_checksum(),
        HashVariant::XxHash3(_) => cleared.compute_xxhash3_checksum(),
    });

    if hash != metadata.hash {
        warn!(
            "Frame {}: Hash mismatch: {hash} vs expected {}",
            metadata.index, metadata.hash
        );

        if let DecodeCheckArgsDump::Corrupted(pool) = &args.dump {
            let thread_image = image.clone();

            pool.borrow_mut().spawn_and_queue(move || {
                if let Err(e) = thread_image
                    .write_to_png(format!("dumped-buffer-broken-{}.png", metadata.index))
                {
                    error!("Error writing file: {e}");
                }

                if let Err(e) = thread_image.write_to_raw(format!(
                    "dumped-buffer-broken-{}.rgb888.raw",
                    metadata.index
                )) {
                    error!("Error writing file: {e}");
                }
            });
        }

        return Err(FrameError::IntegrityFailure);
    }

    Ok(metadata)
}
