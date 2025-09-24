use core::{
    cmp::{max, min},
    ops::{Add, Div, Mul, Rem, Sub},
    time::Duration,
};
use std::{
    fs, io,
    os::{fd::AsFd as _, unix::io::RawFd},
    thread::sleep,
    time::Instant,
};

use num_traits::{One, ToPrimitive as _, Zero};
use redid::{
    EdidChromaticityPoint, EdidChromaticityPoints, EdidDescriptorDetailedTiming,
    EdidDescriptorDetailedTimingHorizontal, EdidDescriptorDetailedTimingVertical,
    EdidDetailedTimingDigitalSeparateSync, EdidDetailedTimingDigitalSync,
    EdidDetailedTimingDigitalSyncKind, EdidDetailedTimingStereo, EdidDetailedTimingSync,
    EdidDisplayColorType, EdidDisplayRangeLimitsRangeFreq, EdidDisplayTransferCharacteristics,
    EdidEstablishedTiming, EdidExtension, EdidExtensionCTA861,
    EdidExtensionCTA861ColorimetryDataBlock, EdidExtensionCTA861HdmiDataBlock,
    EdidExtensionCTA861Revision3, EdidExtensionCTA861Revision3DataBlock,
    EdidExtensionCTA861VideoCapabilityDataBlock, EdidExtensionCTA861VideoCapabilityQuantization,
    EdidExtensionCTA861VideoCapabilityScanBehavior, EdidFilterChromaticity, EdidManufactureDate,
    EdidR3BasicDisplayParametersFeatures, EdidR3Descriptor, EdidR3DigitalVideoInputDefinition,
    EdidR3DisplayRangeLimits, EdidR3DisplayRangeVideoTimingsSupport, EdidR3FeatureSupport,
    EdidR3ImageSize, EdidR3VideoInputDefinition, EdidRelease3, EdidScreenSize, IntoBytes as _,
};
use rustix::io::Errno;
use tracing::{debug, info};
use v4l2_raw::{
    raw::{
        v4l2_buf_type, v4l2_buffer, v4l2_ioctl_dqbuf, v4l2_ioctl_qbuf, v4l2_ioctl_reqbufs,
        v4l2_memory, v4l2_requestbuffers,
    },
    wrapper::{
        v4l2_dv_timings, v4l2_ioctl_query_dv_timings, v4l2_ioctl_s_dv_timings, v4l2_ioctl_s_edid,
        v4l2_ioctl_streamoff, v4l2_ioctl_streamon, v4l2_ioctl_subdev_query_dv_timings,
        v4l2_ioctl_subdev_s_dv_timings, v4l2_ioctl_subdev_s_edid,
    },
};
use v4lise::Device;

use crate::{
    BUFFER_TYPE, Cli, Dradis, MEMORY_TYPE, PipelineItem, SetupError, TestEdid, V4l2EntityWrapper,
};

const HFREQ_TOLERANCE_KHZ: u32 = 5;
const VFREQ_TOLERANCE_HZ: u32 = 1;

const VIC_1_HFREQ_HZ: u32 = 31_469;
const VIC_1_VFREQ_HZ: u32 = 60;

pub(crate) fn dequeue_buffer(dev: &Device) -> io::Result<v4l2_buffer> {
    let mut raw_struct = v4l2_buffer {
        type_: BUFFER_TYPE.into(),
        memory: MEMORY_TYPE.into(),
        ..v4l2_buffer::default()
    };

    raw_struct = v4l2_ioctl_dqbuf(dev.as_fd(), raw_struct)?;

    Ok(raw_struct)
}

pub(crate) fn queue_buffer(dev: &Device, idx: u32, fd: RawFd) -> io::Result<()> {
    let mut raw_struct = v4l2_buffer {
        index: idx,
        type_: BUFFER_TYPE.into(),
        memory: MEMORY_TYPE.into(),
        ..v4l2_buffer::default()
    };
    raw_struct.m.fd = fd;

    let _: v4l2_buffer = v4l2_ioctl_qbuf(dev.as_fd(), raw_struct)?;

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

fn mc_wrapper_v4l2_s_edid(dev: &V4l2EntityWrapper, edid: &mut [u8]) -> io::Result<()> {
    if let Some(device) = &dev.device {
        if dev.entity.is_v4l2_device().valid()? {
            debug!("Running VIDIOC_S_EDID on entity {}", dev.entity.name());
            v4l2_ioctl_s_edid(device.as_fd(), edid)
        } else if dev.entity.is_v4l2_sub_device().valid()? {
            debug!(
                "Running VIDIOC_SUBDEV_S_EDID on entity {}",
                dev.entity.name()
            );
            v4l2_ioctl_subdev_s_edid(device.as_fd(), edid)
        } else {
            unreachable!()
        }
    } else {
        Ok(())
    }
}

pub(crate) fn mc_wrapper_v4l2_query_dv_timings(
    dev: &V4l2EntityWrapper,
) -> io::Result<v4l2_dv_timings> {
    if let Some(device) = &dev.device {
        if dev.entity.is_v4l2_device().valid()? {
            debug!(
                "Running VIDIOC_QUERY_DV_TIMINGS on entity {}",
                dev.entity.name()
            );
            v4l2_ioctl_query_dv_timings(device.as_fd())
        } else if dev.entity.is_v4l2_sub_device().valid()? {
            debug!(
                "Running VIDIOC_SUBDEV_QUERY_DV_TIMINGS on entity {}",
                dev.entity.name()
            );
            v4l2_ioctl_subdev_query_dv_timings(device.as_fd())
        } else {
            unreachable!()
        }
    } else {
        unimplemented!()
    }
}

pub(crate) fn mc_wrapper_v4l2_s_dv_timings(
    dev: &V4l2EntityWrapper,
    timings: v4l2_dv_timings,
) -> io::Result<()> {
    if let Some(device) = &dev.device {
        if dev.entity.is_v4l2_device().valid()? {
            debug!(
                "Running VIDIOC_S_DV_TIMINGS on entity {}",
                dev.entity.name()
            );
            v4l2_ioctl_s_dv_timings(device.as_fd(), timings)
        } else if dev.entity.is_v4l2_sub_device().valid()? {
            debug!(
                "Running VIDIOC_SUBDEV_S_DV_TIMINGS on entity {}",
                dev.entity.name()
            );
            v4l2_ioctl_subdev_s_dv_timings(device.as_fd(), timings)
        } else {
            unreachable!()
        }
    } else {
        Ok(())
    }
}

// Yes, VBLANK is similar to HBLANK
#[allow(clippy::too_many_lines, clippy::similar_names)]
pub(crate) fn bridge_set_edid(
    args: &Cli,
    dev: &V4l2EntityWrapper,
    edid: &TestEdid,
) -> Result<(), SetupError> {
    let TestEdid::DetailedTiming(dtd) = edid;

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
        / u32::from(u16::from(dtd.vfp) + dtd.vdisplay + dtd.vbp + u16::from(dtd.vsync));
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
                .display_transfer_characteristics(EdidDisplayTransferCharacteristics::try_from(
                    2.2,
                )?)
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
        .preferred_timing(
            EdidDescriptorDetailedTiming::builder()
                .pixel_clock(dtd.clock_khz.try_into()?)
                .horizontal(
                    EdidDescriptorDetailedTimingHorizontal::builder()
                        .active(dtd.hdisplay.try_into()?)
                        .border(0.try_into()?)
                        .front_porch(dtd.hfp.try_into()?)
                        .sync_pulse(dtd.hsync.try_into()?)
                        .back_porch(dtd.hbp.try_into()?)
                        .size_mm(1600.try_into()?)
                        .build(),
                )
                .vertical(
                    EdidDescriptorDetailedTimingVertical::builder()
                        .active(dtd.vdisplay.try_into()?)
                        .border(0.try_into()?)
                        .front_porch(dtd.vfp.try_into()?)
                        .sync_pulse(dtd.vsync.try_into()?)
                        .back_porch(dtd.vbp.try_into()?)
                        .size_mm(900.try_into()?)
                        .build(),
                )
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
        )
        .add_descriptor(EdidR3Descriptor::ProductName("Dradis".try_into()?))
        .add_descriptor(EdidR3Descriptor::DisplayRangeLimits(
            EdidR3DisplayRangeLimits::builder()
                .timings_support(EdidR3DisplayRangeVideoTimingsSupport::DefaultGTF)
                .hfreq_khz(EdidDisplayRangeLimitsRangeFreq::try_from(
                    min_hfreq_khz..max_hfreq_khz,
                )?)
                .vfreq_hz(EdidDisplayRangeLimitsRangeFreq::try_from(
                    min_vfreq_hz..max_vfreq_hz,
                )?)
                .max_pixelclock_mhz(80.try_into()?)
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

    if let Some(folder) = &args.dump_edid {
        if !folder.exists() {
            fs::create_dir(folder)?;
        }

        fs::write(folder.join("test-edid.bin"), &bytes)?;
    }

    mc_wrapper_v4l2_s_edid(dev, &mut bytes)?;

    Ok(())
}

pub(crate) fn wait_and_set_dv_timings(
    suite: &Dradis<'_>,
    width: u32,
    height: u32,
) -> Result<(), SetupError> {
    let PipelineItem { entity: root, .. } =
        suite
            .pipeline
            .first()
            .ok_or(SetupError::from(io::Error::new(
                Errno::NODEV.kind(),
                "Missing Root Entity",
            )))?;

    let PipelineItem { entity: bridge, .. } =
        suite
            .pipeline
            .last()
            .ok_or(SetupError::from(io::Error::new(
                Errno::NODEV.kind(),
                "Missing HDMI Bridge Entity",
            )))?;

    let start = Instant::now();

    let timings = loop {
        if start.elapsed() > suite.cfg.link_timeout {
            return Err(SetupError::Timeout(String::from(
                "Timed out waiting for source to emit the proper resolution.",
            )));
        }

        let timings = mc_wrapper_v4l2_query_dv_timings(bridge);
        match timings {
            Ok(timings) => {
                if let v4l2_dv_timings::Bt_656_1120(bt) = timings {
                    if bt.width == width && bt.height == height {
                        info!("Source started to transmit the proper resolution.");
                        break timings;
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
    };

    let res = mc_wrapper_v4l2_s_dv_timings(bridge, timings);
    match res {
        Ok(()) => Ok(()),
        Err(e) => match Errno::from_io_error(&e) {
            Some(Errno::PERM) => {
                debug!("Bridge is read-only. Trying on the main device.");
                mc_wrapper_v4l2_s_dv_timings(root, timings).map_err(Into::into)
            }
            _ => Err(e.into()),
        },
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
