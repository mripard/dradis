#![allow(missing_docs)]
#![allow(unused_crate_dependencies)]

use std::fs;

use dradis_frame_check::{
    DecodeCheckArgs, DecodeCheckArgsDump, FrameError, HashVariant, Metadata, QRCODE_HEIGHT,
    QRCODE_WIDTH, decode_and_check_frame,
};

const TEST_WIDTH: u32 = 1280;
const TEST_HEIGHT: u32 = 720;

#[test_log::test]
fn test_bgr() {
    let data = fs::read("tests/data/valid-frame-ver-2-0.bgr888.raw").unwrap();

    assert_eq!(
        decode_and_check_frame(
            &data,
            DecodeCheckArgs {
                sequence: 0,
                previous_frame_idx: None,
                width: TEST_WIDTH,
                height: TEST_HEIGHT,
                swap_channels: false,
                dump: DecodeCheckArgsDump::Never,
            },
        ),
        Err(FrameError::IntegrityFailure)
    )
}

#[test_log::test]
fn test_bgr_swap_channels() {
    let data = fs::read("tests/data/valid-frame-ver-2-0.bgr888.raw").unwrap();

    assert_eq!(
        decode_and_check_frame(
            &data,
            DecodeCheckArgs {
                sequence: 0,
                previous_frame_idx: None,
                width: TEST_WIDTH,
                height: TEST_HEIGHT,
                swap_channels: true,
                dump: DecodeCheckArgsDump::Never,
            },
        )
        .unwrap(),
        Metadata {
            version: (2, 0),
            qrcode_width: QRCODE_WIDTH,
            qrcode_height: QRCODE_HEIGHT,
            width: TEST_WIDTH,
            height: TEST_HEIGHT,
            hash: HashVariant::XxHash2(0xcddbc559fb8264e6),
            index: 39
        }
    )
}

#[test_log::test]
fn test_bgr_swap_channels_xxhash3() {
    let data = fs::read("tests/data/valid-frame-ver-2-1.xxhash3.bgr888.raw").unwrap();

    assert_eq!(
        decode_and_check_frame(
            &data,
            DecodeCheckArgs {
                sequence: 0,
                previous_frame_idx: None,
                width: TEST_WIDTH,
                height: TEST_HEIGHT,
                swap_channels: true,
                dump: DecodeCheckArgsDump::Never,
            },
        ).unwrap(),
        Metadata {
            version: (2, 1),
            qrcode_width: QRCODE_WIDTH,
            qrcode_height: QRCODE_HEIGHT,
            width: TEST_WIDTH,
            height: TEST_HEIGHT,
            hash: HashVariant::XxHash3(0xaab295ae5a7690c3),
            index: 277
        }
    )
}

#[test_log::test]
fn test_rgb() {
    let data = fs::read("tests/data/valid-frame-ver-2-0.rgb888.raw").unwrap();

    assert_eq!(
        decode_and_check_frame(
            &data,
            DecodeCheckArgs {
                sequence: 0,
                previous_frame_idx: None,
                width: TEST_WIDTH,
                height: TEST_HEIGHT,
                swap_channels: false,
                dump: DecodeCheckArgsDump::Never,
            },
        )
        .unwrap(),
        Metadata {
            version: (2, 0),
            qrcode_width: QRCODE_WIDTH,
            qrcode_height: QRCODE_HEIGHT,
            width: TEST_WIDTH,
            height: TEST_HEIGHT,
            hash: HashVariant::XxHash2(0xcddbc559fb8264e6),
            index: 6
        }
    )
}

#[test_log::test]
fn test_rgb_xxhash3() {
    let data = fs::read("tests/data/valid-frame-ver-2-1.xxhash3.rgb888.raw").unwrap();

    assert_eq!(
        decode_and_check_frame(
            &data,
            DecodeCheckArgs {
                sequence: 0,
                previous_frame_idx: None,
                width: TEST_WIDTH,
                height: TEST_HEIGHT,
                swap_channels: false,
                dump: DecodeCheckArgsDump::Never,
            },
        ).unwrap(),
        Metadata {
            version: (2, 1),
            qrcode_width: QRCODE_WIDTH,
            qrcode_height: QRCODE_HEIGHT,
            width: TEST_WIDTH,
            height: TEST_HEIGHT,
            hash: HashVariant::XxHash3(0xaab295ae5a7690c3),
            index: 97
        }
    )
}

#[test_log::test]
fn test_rgb_swap_channels() {
    let data = fs::read("tests/data/valid-frame-ver-2-0.rgb888.raw").unwrap();

    assert_eq!(
        decode_and_check_frame(
            &data,
            DecodeCheckArgs {
                sequence: 0,
                previous_frame_idx: None,
                width: TEST_WIDTH,
                height: TEST_HEIGHT,
                swap_channels: true,
                dump: DecodeCheckArgsDump::Never,
            },
        ),
        Err(FrameError::IntegrityFailure)
    )
}
