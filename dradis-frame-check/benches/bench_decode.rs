#![allow(missing_docs)]
#![allow(unused_crate_dependencies)]

use criterion::{criterion_group, criterion_main};
use dradis_frame_check::{
    DecodeCheckArgs, DecodeCheckArgsDump, HashVariant, Metadata, QRCODE_HEIGHT, QRCODE_WIDTH,
    decode_and_check_frame,
};

const FRAME_WIDTH: u32 = 1280;
const FRAME_HEIGHT: u32 = 720;
const VALID_XXHASH2_FRAME: &[u8] = include_bytes!("../tests/data/valid-frame-ver-2-0.rgb888.raw");
const SWAPPED_XXHASH2_FRAME: &[u8] = include_bytes!("../tests/data/valid-frame-ver-2-0.bgr888.raw");
const VALID_XXHASH3_FRAME: &[u8] =
    include_bytes!("../tests/data/valid-frame-ver-2-1.xxhash3.rgb888.raw");
const SWAPPED_XXHASH3_FRAME: &[u8] =
    include_bytes!("../tests/data/valid-frame-ver-2-1.xxhash3.bgr888.raw");

fn bench_frame_detect(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("decode_and_check_frame");
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.bench_function("xxhash2/valid", |b| {
        b.iter(|| {
            let data = decode_and_check_frame(
                VALID_XXHASH2_FRAME,
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
                    hash: HashVariant::XxHash2(0xcddbc559fb8264e6),
                    index: 6
                }
            )
        });
    });
    group.bench_function("xxhash2/swapped", |b| {
        b.iter(|| {
            let data = decode_and_check_frame(
                SWAPPED_XXHASH2_FRAME,
                DecodeCheckArgs {
                    sequence: 42,
                    previous_frame_idx: None,
                    width: FRAME_WIDTH,
                    height: FRAME_HEIGHT,
                    swap_channels: true,
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
                    hash: HashVariant::XxHash2(0xcddbc559fb8264e6),
                    index: 39
                }
            )
        });
    });
    group.bench_function("xxhash3/valid", |b| {
        b.iter(|| {
            let data = decode_and_check_frame(
                VALID_XXHASH3_FRAME,
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
                    version: (2, 1),
                    qrcode_width: QRCODE_WIDTH,
                    qrcode_height: QRCODE_HEIGHT,
                    width: FRAME_WIDTH,
                    height: FRAME_HEIGHT,
                    hash: HashVariant::XxHash3(0xaab295ae5a7690c3),
                    index: 97
                }
            )
        });
    });
    group.bench_function("xxhash3/swapped", |b| {
        b.iter(|| {
            let data = decode_and_check_frame(
                SWAPPED_XXHASH3_FRAME,
                DecodeCheckArgs {
                    sequence: 42,
                    previous_frame_idx: None,
                    width: FRAME_WIDTH,
                    height: FRAME_HEIGHT,
                    swap_channels: true,
                    dump: DecodeCheckArgsDump::Never,
                },
            )
            .unwrap();
            assert_eq!(
                data,
                Metadata {
                    version: (2, 1),
                    qrcode_width: QRCODE_WIDTH,
                    qrcode_height: QRCODE_HEIGHT,
                    width: FRAME_WIDTH,
                    height: FRAME_HEIGHT,
                    hash: HashVariant::XxHash3(0xaab295ae5a7690c3),
                    index: 277
                }
            )
        });
    });
    group.finish();
}

criterion_group!(benches, bench_frame_detect);
criterion_main!(benches);
