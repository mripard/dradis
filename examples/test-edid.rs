use core::{
    cmp::{max, min},
    ops::{Add, Div, Mul, Rem, Sub},
};
use std::io::Write;

use num_traits::{One, Zero};
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

const HFREQ_TOLERANCE_KHZ: u32 = 5;
const VFREQ_TOLERANCE_HZ: u32 = 1;

const VIC_1_HFREQ_HZ: u32 = 31_469;
const VIC_1_VFREQ_HZ: u32 = 60;

macro_rules! cast_panic {
    ($x:expr) => {
        $x.try_into().unwrap()
    };
}

fn round_up<T>(val: T, multiple: T) -> T
where
    T: Add<T, Output = T> + Copy + Div<T, Output = T> + Mul<T, Output = T> + One,
{
    ((val / multiple) + T::one()) * multiple
}

#[cfg(test)]
mod tests_round_up {
    use crate::round_up;

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
    use crate::round_down;

    #[test]
    fn test_unaligned() {
        assert_eq!(round_down(42, 5), 40);
    }

    #[test]
    fn test_aligned() {
        assert_eq!(round_down(40, 5), 35);
    }
}

fn main() {
    let mut stdout = std::io::stdout();

    let clock_khz = 74250;
    let hfp = 110;
    let hdisplay = 1280;
    let hbp = 220;
    let hsync = 40;
    let vfp = 5;
    let vdisplay = 720;
    let vbp = 20;
    let vsync = 5;

    let mode_hfreq_khz = clock_khz / (hfp + hdisplay + hbp + hsync);
    dbg!(&mode_hfreq_khz);
    let mode_hfreq_hz = mode_hfreq_khz * 1000;
    dbg!(&mode_hfreq_hz);
    let min_hfreq_khz = round_down(
        min(mode_hfreq_hz - 1, VIC_1_HFREQ_HZ) / 1000,
        HFREQ_TOLERANCE_KHZ,
    ) as u8;
    dbg!(&min_hfreq_khz);
    let max_hfreq_khz = round_up(
        max(mode_hfreq_hz + 1, VIC_1_HFREQ_HZ) / 1000,
        HFREQ_TOLERANCE_KHZ,
    ) as u8;
    dbg!(&max_hfreq_khz);

    let mode_vfreq_hz = mode_hfreq_hz / (vfp + vdisplay + vbp + vsync);
    dbg!(&mode_vfreq_hz);
    let min_vfreq_hz = round_down(min(mode_vfreq_hz - 1, VIC_1_VFREQ_HZ), VFREQ_TOLERANCE_HZ) as u8;
    dbg!(&min_vfreq_hz);
    let max_vfreq_hz: u8 =
        round_up(max(mode_vfreq_hz + 1, VIC_1_VFREQ_HZ), VFREQ_TOLERANCE_HZ) as u8;
    dbg!(&max_vfreq_hz);

    let test_edid = EdidRelease3::builder()
        .manufacturer("CRN".try_into().unwrap())
        .product_code(0x42)
        .serial_number(Some(0x42424242.try_into().unwrap()))
        .date(EdidManufactureDate::try_from(2024).unwrap())
        .display_parameters_features(
            EdidR3BasicDisplayParametersFeatures::builder()
                .video_input(EdidR3VideoInputDefinition::Digital(
                    EdidR3DigitalVideoInputDefinition::builder()
                        .dfp1_compatible(true)
                        .build(),
                ))
                .display_transfer_characteristic(
                    EdidDisplayTransferCharacteristics::try_from(2.2).unwrap(),
                )
                .feature_support(
                    EdidR3FeatureSupport::builder()
                        .display_type(EdidDisplayColorType::RGBColor)
                        .build(),
                )
                .size(EdidR3ImageSize::Size(
                    EdidScreenSize::builder()
                        .horizontal_cm(cast_panic!(160))
                        .vertical_cm(cast_panic!(90))
                        .build(),
                ))
                .build(),
        )
        .filter_chromaticity(EdidFilterChromaticity::Color(
            EdidChromaticityPoints::builder()
                .red(EdidChromaticityPoint::try_from((0.627, 0.341)).unwrap())
                .green(EdidChromaticityPoint::try_from((0.292, 0.605)).unwrap())
                .blue(EdidChromaticityPoint::try_from((0.149, 0.072)).unwrap())
                .white(EdidChromaticityPoint::try_from((0.283, 0.297)).unwrap())
                .build(),
        ))
        .add_established_timing(EdidEstablishedTiming::ET_640_480_60hz)
        .add_descriptor(EdidR3Descriptor::DetailedTiming(
            EdidDescriptorDetailedTiming::builder()
                .pixel_clock(cast_panic!(clock_khz))
                .horizontal_front_porch(cast_panic!(hfp as u16))
                .horizontal_addressable(cast_panic!(hdisplay as u16))
                .horizontal_blanking(cast_panic!((hfp + hsync + hbp) as u16))
                .horizontal_sync_pulse(cast_panic!(hsync as u16))
                .horizontal_border(cast_panic!(0))
                .horizontal_size(cast_panic!(1600))
                .vertical_front_porch(cast_panic!(vfp as u8))
                .vertical_addressable(cast_panic!(vdisplay as u16))
                .vertical_blanking(cast_panic!((vfp + vsync + vbp) as u16))
                .vertical_sync_pulse(cast_panic!(vsync as u8))
                .vertical_border(cast_panic!(0))
                .vertical_size(cast_panic!(900))
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
        .add_descriptor(EdidR3Descriptor::ProductName("Dradis".try_into().unwrap()))
        .add_descriptor(EdidR3Descriptor::DisplayRangeLimits(
            EdidR3DisplayRangeLimits::builder()
                .timings_support(EdidR3DisplayRangeVideoTimingsSupport::DefaultGTF)
                .min_hfreq(cast_panic!(min_hfreq_khz))
                .max_hfreq(cast_panic!(max_hfreq_khz))
                .min_vfreq(cast_panic!(min_vfreq_hz))
                .max_vfreq(cast_panic!(max_vfreq_hz))
                .max_pixelclock(cast_panic!(80))
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
                        .source_physical_address(cast_panic!([1, 0, 0, 0]))
                        .build(),
                ))
                .build(),
        )))
        .build();

    let output = test_edid.into_bytes();
    stdout.write_all(&output).unwrap();
    stdout.flush().unwrap();
}
