#![allow(missing_docs)]
#![allow(unused_crate_dependencies)]

use std::fs;

use dradis_frame_check::{
    DecodeCheckArgs, DecodeCheckArgsDump, FrameError, Metadata, QRCODE_HEIGHT, QRCODE_WIDTH,
    decode_and_check_frame,
};

const FRAME_WIDTH: u32 = 1280;
const FRAME_HEIGHT: u32 = 720;

#[test]
fn valid_qrcode() {
    let data = fs::read("tests/data/test-qrcode-detection.rgb888.raw").unwrap();

    let data = decode_and_check_frame(
        &data,
        DecodeCheckArgs {
            sequence: 42,
            previous_frame_idx: None,
            width: FRAME_WIDTH,
            height: FRAME_HEIGHT,
            swap_channels: false,
            dump: DecodeCheckArgsDump::Never,
        },
    )
    .unwrap();

    assert_eq!(
        data,
        Metadata {
            version: (2, 0),
            qrcode_width: QRCODE_WIDTH,
            qrcode_height: QRCODE_HEIGHT,
            width: FRAME_WIDTH,
            height: FRAME_HEIGHT,
            hash: 0xcddbc559fb8264e6,
            index: 0
        }
    );
}

#[test]
fn invalid_qrcode_hash() {
    let data = fs::read("tests/data/test-qrcode-hash-mismatch.rgb888.raw").unwrap();

    assert_eq!(
        decode_and_check_frame(
            &data,
            DecodeCheckArgs {
                sequence: 42,
                previous_frame_idx: None,
                width: FRAME_WIDTH,
                height: FRAME_HEIGHT,
                swap_channels: false,
                dump: DecodeCheckArgsDump::Never,
            },
        ),
        Err(FrameError::IntegrityFailure)
    );
}
