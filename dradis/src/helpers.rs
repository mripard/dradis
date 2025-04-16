use core::{
    cmp::{max, min},
    ops::{Add, Div, Mul, Rem, Sub},
};
use std::{
    io,
    os::{fd::AsFd, unix::io::RawFd},
    thread::sleep,
    time::{Duration, Instant},
};

use num_traits::{One, ToPrimitive, Zero};
use redid::{
    EdidChromaticityPoint, EdidChromaticityPoints, EdidDescriptorDetailedTiming,
    EdidDetailedTimingDigitalSeparateSync, EdidDetailedTimingDigitalSync,
    EdidDetailedTimingDigitalSyncKind, EdidDetailedTimingStereo, EdidDetailedTimingSync,
    EdidDisplayColorType, EdidDisplayTransferCharacteristics, EdidEstablishedTiming, EdidExtension,
    EdidExtensionCTA861, EdidExtensionCTA861ColorimetryDataBlock, EdidExtensionCTA861HdmiDataBlock,
    EdidExtensionCTA861Revision3, EdidExtensionCTA861Revision3DataBlock,
    EdidExtensionCTA861VideoCapabilityDataBlock, EdidExtensionCTA861VideoCapabilityQuantization,
    EdidExtensionCTA861VideoCapabilityScanBehavior, EdidFilterChromaticity, EdidManufactureDate,
    EdidR3BasicDisplayParametersFeatures, EdidR3Descriptor, EdidR3DigitalVideoInputDefinition,
    EdidR3DisplayRangeLimits, EdidR3DisplayRangeVideoTimingsSupport, EdidR3FeatureSupport,
    EdidR3ImageSize, EdidR3VideoInputDefinition, EdidRelease3, EdidScreenSize, IntoBytes,
};
use rustix::io::Errno;
use tracing::{debug, info};
use v4l2_raw::{
    raw::{v4l2_buf_type, v4l2_ioctl_dqbuf, v4l2_ioctl_qbuf, v4l2_ioctl_reqbufs, v4l2_memory},
    wrapper::{
        v4l2_dv_timings, v4l2_ioctl_query_dv_timings, v4l2_ioctl_s_dv_timings, v4l2_ioctl_s_edid,
        v4l2_ioctl_streamoff, v4l2_ioctl_streamon,
    },
};
use v4lise::{Device, v4l2_buffer, v4l2_requestbuffers};

use crate::{BUFFER_TYPE, Dradis, MEMORY_TYPE, PipelineItem, SetupError, TestEdid};

const HFREQ_TOLERANCE_KHZ: u32 = 5;
const VFREQ_TOLERANCE_HZ: u32 = 1;

const VIC_1_HFREQ_HZ: u32 = 31_469;
const VIC_1_VFREQ_HZ: u32 = 60;

pub(crate) fn dequeue_buffer(dev: &Device) -> io::Result<u32> {
    let mut raw_struct = v4l2_buffer {
        type_: BUFFER_TYPE.into(),
        memory: MEMORY_TYPE.into(),
        ..v4l2_buffer::default()
    };

    raw_struct = v4l2_ioctl_dqbuf(dev.as_fd(), raw_struct)?;

    Ok(raw_struct.index)
}

pub(crate) fn queue_buffer(dev: &Device, idx: u32, fd: RawFd) -> io::Result<()> {
    let mut raw_struct = v4l2_buffer {
        index: idx,
        type_: BUFFER_TYPE.into(),
        memory: MEMORY_TYPE.into(),
        ..v4l2_buffer::default()
    };
    raw_struct.m.fd = fd;

    let _ = v4l2_ioctl_qbuf(dev.as_fd(), raw_struct)?;

    Ok(())
}

fn round_up<T>(val: T, multiple: T) -> T
where
    T: Add<T, Output = T> + Copy + Div<T, Output = T> + Mul<T, Output = T> + One,
{
    ((val / multiple) + T::one()) * multiple
}

#[cfg(test)]
mod tests_round_up {
    use super::round_up;

    #[test]
    fn test_unaligned() {
        assert_eq!(round_up(42, 5), 45);
    }

    #[test]
    fn test_aligned() {
        assert_eq!(round_up(40, 5), 45);
    }
}

fn round_down<T>(val: T, multiple: T) -> T
where
    T: Copy
        + Div<T, Output = T>
        + Mul<T, Output = T>
        + Rem<T, Output = T>
        + Sub<T, Output = T>
        + Zero,
{
    if (val % multiple).is_zero() {
        return val - multiple;
    }

    (val / multiple) * multiple
}

#[cfg(test)]
mod tests_round_down {
    use super::round_down;

    #[test]
    fn test_unaligned() {
        assert_eq!(round_down(42, 5), 40);
    }

    #[test]
    fn test_aligned() {
        assert_eq!(round_down(40, 5), 35);
    }
}

// Yes, VBLANK is similar to HBLANK
#[allow(clippy::too_many_lines, clippy::similar_names)]
pub(crate) fn set_edid(dev: &impl AsFd, edid: &TestEdid) -> Result<(), SetupError> {
    let TestEdid::DetailedTiming(ref dtd) = edid;

    let mode_hfreq_khz: u32 =
        dtd.clock_khz / u32::from(dtd.hfp + dtd.hdisplay + dtd.hbp + dtd.hsync);
    let mode_hfreq_hz = mode_hfreq_khz * 1000;
    let min_hfreq_khz = round_down(
        min(mode_hfreq_hz, VIC_1_HFREQ_HZ) / 1000,
        HFREQ_TOLERANCE_KHZ,
    )
    .to_u8()
    .ok_or(SetupError::Value(String::from(
        "Min Horizontal Frequency wouldn't fit in an u8",
    )))?;

    let max_hfreq_khz = round_up(
        max(mode_hfreq_hz, VIC_1_HFREQ_HZ) / 1000,
        HFREQ_TOLERANCE_KHZ,
    )
    .to_u8()
    .ok_or(SetupError::Value(String::from(
        "Max Horizontal Frequency wouldn't fit in an u8",
    )))?;

    let mode_vfreq_hz = mode_hfreq_hz
        / u32::from(u16::from(dtd.vfp) + dtd.vdisplay + u16::from(dtd.vbp) + u16::from(dtd.vsync));
    let min_vfreq_hz = round_down(min(mode_vfreq_hz, VIC_1_VFREQ_HZ), VFREQ_TOLERANCE_HZ)
        .to_u8()
        .ok_or(SetupError::Value(String::from(
            "Min Vertical Frequency wouldn't fit in an u8",
        )))?;
    let max_vfreq_hz = round_up(max(mode_vfreq_hz, VIC_1_VFREQ_HZ), VFREQ_TOLERANCE_HZ)
        .to_u8()
        .ok_or(SetupError::Value(String::from(
            "Min Vertical Frequency wouldn't fit in an u8",
        )))?;

    let test_edid = EdidRelease3::builder()
        .manufacturer("CRN".try_into()?)
        .product_code(0x42)
        .serial_number(Some(0x42424242.into()))
        .date(EdidManufactureDate::try_from(2024)?)
        .display_parameters_features(
            EdidR3BasicDisplayParametersFeatures::builder()
                .video_input(EdidR3VideoInputDefinition::Digital(
                    EdidR3DigitalVideoInputDefinition::builder()
                        .dfp1_compatible(true)
                        .build(),
                ))
                .display_transfer_characteristic(EdidDisplayTransferCharacteristics::try_from(2.2)?)
                .feature_support(
                    EdidR3FeatureSupport::builder()
                        .display_type(EdidDisplayColorType::RGBColor)
                        .build(),
                )
                .size(EdidR3ImageSize::Size(
                    EdidScreenSize::builder()
                        .horizontal_cm(160.try_into()?)
                        .vertical_cm(90.try_into()?)
                        .build(),
                ))
                .build(),
        )
        .filter_chromaticity(EdidFilterChromaticity::Color(
            EdidChromaticityPoints::builder()
                .red(EdidChromaticityPoint::try_from((0.627, 0.341))?)
                .green(EdidChromaticityPoint::try_from((0.292, 0.605))?)
                .blue(EdidChromaticityPoint::try_from((0.149, 0.072))?)
                .white(EdidChromaticityPoint::try_from((0.283, 0.297))?)
                .build(),
        ))
        .add_established_timing(EdidEstablishedTiming::ET_640_480_60hz)
        .add_descriptor(EdidR3Descriptor::DetailedTiming(
            EdidDescriptorDetailedTiming::builder()
                .pixel_clock(dtd.clock_khz.try_into()?)
                .horizontal_front_porch(dtd.hfp.try_into()?)
                .horizontal_addressable(dtd.hdisplay.try_into()?)
                .horizontal_blanking((dtd.hfp + dtd.hsync + dtd.hbp).try_into()?)
                .horizontal_sync_pulse(dtd.hsync.try_into()?)
                .horizontal_border(0.try_into()?)
                .horizontal_size(1600.try_into()?)
                .vertical_front_porch(dtd.vfp.try_into()?)
                .vertical_addressable(dtd.vdisplay.try_into()?)
                .vertical_blanking(u16::from(dtd.vfp + dtd.vsync + dtd.vbp).try_into()?)
                .vertical_sync_pulse(dtd.vsync.try_into()?)
                .vertical_border(0.try_into()?)
                .vertical_size(900.try_into()?)
                .sync_type(EdidDetailedTimingSync::Digital(
                    EdidDetailedTimingDigitalSync::builder()
                        .kind(EdidDetailedTimingDigitalSyncKind::Separate(
                            EdidDetailedTimingDigitalSeparateSync::builder()
                                .vsync_positive(true)
                                .build(),
                        ))
                        .hsync_positive(true)
                        .build(),
                ))
                .stereo(EdidDetailedTimingStereo::None)
                .build(),
        ))
        .add_descriptor(EdidR3Descriptor::ProductName("Dradis".try_into()?))
        .add_descriptor(EdidR3Descriptor::DisplayRangeLimits(
            EdidR3DisplayRangeLimits::builder()
                .timings_support(EdidR3DisplayRangeVideoTimingsSupport::DefaultGTF)
                .min_hfreq(min_hfreq_khz.try_into()?)
                .max_hfreq(max_hfreq_khz.try_into()?)
                .min_vfreq(min_vfreq_hz.try_into()?)
                .max_vfreq(max_vfreq_hz.try_into()?)
                .max_pixelclock(80.try_into()?)
                .build(),
        ))
        .add_extension(EdidExtension::CTA861(EdidExtensionCTA861::Revision3(
            EdidExtensionCTA861Revision3::builder()
                .native_formats(1)
                .underscan_it_formats_by_default(true)
                .add_data_block(EdidExtensionCTA861Revision3DataBlock::Colorimetry(
                    EdidExtensionCTA861ColorimetryDataBlock::builder().build(),
                ))
                .add_data_block(EdidExtensionCTA861Revision3DataBlock::VideoCapability(
                    EdidExtensionCTA861VideoCapabilityDataBlock::builder()
                        .qs_quant(EdidExtensionCTA861VideoCapabilityQuantization::Selectable)
                        .ce_scan(EdidExtensionCTA861VideoCapabilityScanBehavior::Underscanned)
                        .it_scan(EdidExtensionCTA861VideoCapabilityScanBehavior::Underscanned)
                        .build(),
                ))
                .add_data_block(EdidExtensionCTA861Revision3DataBlock::HDMI(
                    EdidExtensionCTA861HdmiDataBlock::builder()
                        .source_physical_address([1, 0, 0, 0].try_into()?)
                        .build(),
                ))
                .build(),
        )))
        .build();

    let mut bytes = test_edid.into_bytes();

    v4l2_ioctl_s_edid(dev.as_fd(), &mut bytes)?;

    Ok(())
}

pub(crate) fn wait_and_set_dv_timings(
    suite: &Dradis<'_>,
    width: u32,
    height: u32,
) -> Result<(), SetupError> {
    let PipelineItem(_, root, _) =
        suite
            .pipeline
            .first()
            .ok_or(SetupError::from(io::Error::new(
                Errno::NODEV.kind(),
                "Missing Root Entity",
            )))?;

    let root_device = root.device.as_ref().ok_or(SetupError::from(io::Error::new(
        Errno::NODEV.kind(),
        "Missing V4L2 Root Device",
    )))?;

    let start = Instant::now();

    loop {
        if start.elapsed() > suite.cfg.link_timeout {
            return Err(SetupError::Timeout(String::from(
                "Timed out waiting for source to emit the proper resolution.",
            )));
        }

        let timings = v4l2_ioctl_query_dv_timings(root_device.as_fd());
        match timings {
            Ok(timings) => {
                if let v4l2_dv_timings::Bt_656_1120(bt) = timings {
                    if bt.width == width && bt.height == height {
                        info!("Source started to transmit the proper resolution.");
                        v4l2_ioctl_s_dv_timings(root_device.as_fd(), timings)?;
                        return Ok(());
                    }
                }
            }
            Err(e) => match Errno::from_io_error(&e) {
                Some(Errno::NOLCK) => {
                    debug!("Link detected but unstable.");
                }
                Some(Errno::NOLINK) => {
                    debug!("No link detected.");
                }
                Some(Errno::RANGE) => {
                    debug!("Timings out of range.");
                }
                _ => return Err(e.into()),
            },
        }

        sleep(Duration::from_millis(100));
    }
}

pub(crate) fn clear_buffers(
    device: &Device,
    buf_type: v4l2_buf_type,
    mem_type: v4l2_memory,
) -> io::Result<()> {
    let rbuf = v4l2_requestbuffers {
        count: 0,
        type_: buf_type.into(),
        memory: mem_type.into(),
        ..Default::default()
    };

    v4l2_ioctl_reqbufs(device.as_fd(), rbuf)?;

    Ok(())
}

pub(crate) struct StreamingDevice<'a> {
    device: &'a Device,
    buf_type: v4l2_buf_type,
}

impl Drop for StreamingDevice<'_> {
    fn drop(&mut self) {
        info!("Stopping Streaming");

        v4l2_ioctl_streamoff(self.device.as_fd(), self.buf_type).expect("Couldn't stop streaming");

        clear_buffers(self.device, self.buf_type, v4l2_memory::V4L2_MEMORY_DMABUF)
            .expect("Couldn't free our buffers.");
    }
}

pub(crate) fn start_streaming(
    device: &Device,
    buf_type: v4l2_buf_type,
) -> io::Result<StreamingDevice<'_>> {
    info!("Starting Streaming");

    v4l2_ioctl_streamon(device.as_fd(), buf_type)?;

    Ok(StreamingDevice { device, buf_type })
}
