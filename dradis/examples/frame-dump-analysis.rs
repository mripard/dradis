use std::{fmt::Display, fs, path::PathBuf};

use clap::Parser;
use frame_check::{FrameError, QRCodeFrame};
use pix::{chan::Ch8, el::Pixel, rgb::Rgb8};
use tracelimit::{error_ratelimited, warn_ratelimited};
use tracing::{Level, debug, error, info, warn};

const LIMITED_RGB_LO_LEVEL: u8 = 16;
const LIMITED_RGB_HI_LEVEL: u8 = 235;
const LIMITED_RGB_DET_LEVEL: usize = 0;

fn within_limited_rgb_bounds(ch: u8) -> bool {
    (LIMITED_RGB_LO_LEVEL..=LIMITED_RGB_HI_LEVEL).contains(&ch)
}

fn limited_to_full_channel(ch: Ch8) -> Ch8 {
    let lim_u8 = <Ch8 as Into<u8>>::into(ch);
    if !(LIMITED_RGB_LO_LEVEL..=LIMITED_RGB_HI_LEVEL).contains(&lim_u8) {
        return ch;
    }

    let lim_u32: u32 = <Ch8 as Into<u8>>::into(ch).into();
    let full = (lim_u32 - 16) * 255 / 219;

    Ch8::new(u8::try_from(full).unwrap())
}

#[derive(Clone, Copy)]
struct FullRangeRgb8(Rgb8);

impl FullRangeRgb8 {
    fn to_limited_range(&self) -> LimitedRangeRgb8 {
        let full_channels: Vec<_> = self
            .0
            .channels()
            .iter()
            .map(|ch| limited_to_full_channel(*ch))
            .collect();

        LimitedRangeRgb8(Rgb8::from_channels(&full_channels))
    }
}

impl Display for FullRangeRgb8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "R: {}, G: {}, B: {}",
            <pix::chan::Ch8 as Into<u8>>::into(pix::rgb::Rgb::red(self.0)),
            <pix::chan::Ch8 as Into<u8>>::into(pix::rgb::Rgb::green(self.0)),
            <pix::chan::Ch8 as Into<u8>>::into(pix::rgb::Rgb::blue(self.0))
        ))
    }
}

impl From<Rgb8> for FullRangeRgb8 {
    fn from(value: Rgb8) -> Self {
        Self(value)
    }
}

impl PartialEq for FullRangeRgb8 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

struct LimitedRangeRgb8(Rgb8);

impl PartialEq for LimitedRangeRgb8 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

fn eq_ignore_limited_threshold(a: FullRangeRgb8, b: FullRangeRgb8) -> bool {
    for (ch_a, ch_b) in a.0.channels().iter().zip(b.0.channels().iter()) {
        let ch_a_u8 = u8::from(*ch_a);
        let ch_b_u8 = u8::from(*ch_b);

        if within_limited_rgb_bounds(ch_a_u8) && !within_limited_rgb_bounds(ch_b_u8) {
            return false;
        }

        if !within_limited_rgb_bounds(ch_a_u8) && within_limited_rgb_bounds(ch_b_u8) {
            return false;
        }

        if within_limited_rgb_bounds(ch_a_u8)
            && within_limited_rgb_bounds(ch_b_u8)
            && (ch_a_u8 != ch_b_u8)
        {
            return false;
        }
    }

    true
}

/// Check that a Boomer frame is correct.
///
/// We expect the bytes to be stored with RGB left-to-right (ie, RGB24 for v4l2, BGR24 for KMS),
/// and with pixels left-to-right, scanlines being top to bottom.
fn check_frame(
    bytes: &[u8],
    width: u32,
    height: u32,
    swap_channels: bool,
) -> Result<u64, Box<dyn std::error::Error>> {
    let frame = if swap_channels {
        QRCodeFrame::from_raw_bytes_with_swapped_channels(width, height, bytes)
    } else {
        QRCodeFrame::from_raw_bytes(width, height, bytes)
    };

    let _content = match frame.qrcode_content() {
        Ok(s) => {
            debug!("Found the QRcode!");
            s
        }
        Err(e) => {
            error!("The QRCode cannot be found or decoded");
            return Err(e.into());
        }
    };

    let metadata = match frame.metadata() {
        Ok(m) => {
            debug!("Metadata can be decoded");
            m
        }
        Err(e) => {
            error!("Metadata are invalid.");
            return Err(e.into());
        }
    };

    let cleared = frame.cleared_frame_with_metadata(&metadata);
    let hash = cleared.compute_checksum();
    if hash == metadata.hash {
        debug!("Hash is valid.");
    } else {
        error!(
            "Hash mismatch: {:#x} vs expected {:#x}",
            hash, metadata.hash
        );
        return Err(FrameError::IntegrityFailure.into());
    }

    Ok(hash)
}

fn scan_different_pixels(bytes_a: &[u8], bytes_b: &[u8], width: u32, height: u32) {
    let frame_a = QRCodeFrame::<Rgb8>::from_raw_bytes(width, height, bytes_a);
    let frame_b = QRCodeFrame::<Rgb8>::from_raw_bytes(width, height, bytes_b);

    for row in 0..height {
        for col in 0..width {
            let full_pix_a = FullRangeRgb8::from(frame_a.pixel(col, row));
            let full_pix_b = FullRangeRgb8::from(frame_b.pixel(col, row));

            if full_pix_a == full_pix_b {
                continue;
            }

            let lim_pix_a = full_pix_a.to_limited_range();
            let lim_pix_b = full_pix_b.to_limited_range();

            if lim_pix_a.0 == full_pix_b.0 || full_pix_a.0 == lim_pix_b.0 {
                error_ratelimited!("Pixel at X {col} Y {row} has a quantization range mismatch");
                continue;
            }

            if eq_ignore_limited_threshold(full_pix_a, full_pix_b) {
                warn_ratelimited!(
                    "Pixel at X {col} Y {row} probably has a quantization range mismatch?"
                );
                continue;
            }

            if full_pix_a != full_pix_b {
                error_ratelimited!(
                    "Pixel at X {col} Y {row} is different ({}) vs ({})",
                    full_pix_a,
                    full_pix_b
                );
            }
        }
    }
}

fn compare_two_frames(bytes_a: &[u8], bytes_b: &[u8], width: u32, height: u32) {
    let hash_a = check_frame(bytes_a, width, height, false).ok();
    let hash_b = check_frame(bytes_b, width, height, false).ok();

    match (hash_a, hash_b) {
        // Both frames have a QR Code and are valid. We just need to make sure the hashes match.
        (Some(hash_a), Some(hash_b)) => {
            info!("Both Frames have a QRCode and a valid checksum.");

            if hash_a == hash_b {
                info!("Frames are identical");
            } else {
                error!(
                    "Frames are different. Hash mismatch: {:#x} vs expected {:#x}",
                    hash_a, hash_b
                );
            }
        }
        (None, None) => scan_different_pixels(bytes_a, bytes_b, width, height),
        (Some(_), None) | (None, Some(_)) => {
            let frame_a =
                QRCodeFrame::<Rgb8>::from_raw_bytes(width, height, bytes_a).cleared_frame(128, 128);
            let frame_b =
                QRCodeFrame::<Rgb8>::from_raw_bytes(width, height, bytes_b).cleared_frame(128, 128);

            scan_different_pixels(frame_a.as_bytes(), frame_b.as_bytes(), width, height);
        }
    }
}

#[derive(Parser)]
struct CliArgs {
    frame_a: PathBuf,
    frame_b: Option<PathBuf>,

    #[arg(long)]
    height: u32,

    #[arg(long)]
    width: u32,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() -> Result<(), anyhow::Error> {
    let args = CliArgs::parse();

    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(true)
        .with_max_level(match args.verbose {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        })
        .init();

    match (args.frame_a, args.frame_b) {
        (frame_a, None) => {
            let bytes = fs::read(&frame_a).unwrap();

            if check_frame(&bytes, args.width, args.height, false).is_ok() {
                return Ok(());
            }

            warn!("Frame doesn't match as is. Trying to swap R/B components");

            if check_frame(&bytes, args.width, args.height, true).is_ok() {
                error!("Frame has swapped R/B components");
                return Err(FrameError::IntegrityFailure.into());
            }

            warn!(
                "Frame doesn't match with swapped R/B components either. Trying to see if it uses Limited Range RGB."
            );

            let (num_below, num_above) =
                bytes
                    .iter()
                    .fold((0, 0), |(mut num_below, mut num_above), val| {
                        if *val > LIMITED_RGB_HI_LEVEL {
                            num_above += 1
                        } else if *val < LIMITED_RGB_LO_LEVEL {
                            num_below += 1
                        }

                        (num_below, num_above)
                    });

            if num_below <= LIMITED_RGB_DET_LEVEL || num_above <= LIMITED_RGB_DET_LEVEL {
                error!("Frame is encoded using Limited Range RGB.");
                return Err(FrameError::IntegrityFailure.into());
            }

            info!("Frame looks good to me, but somehow fails integrity check. ¯\\_(ツ)_/¯");
            Err(FrameError::IntegrityFailure.into())
        }
        (frame_a, Some(frame_b)) => {
            let bytes_a = fs::read(&frame_a).unwrap();
            let bytes_b = fs::read(&frame_b).unwrap();

            compare_two_frames(&bytes_a, &bytes_b, args.width, args.height);

            Ok(())
        }
    }
}
