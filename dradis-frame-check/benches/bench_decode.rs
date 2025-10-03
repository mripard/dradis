#![allow(missing_docs)]
#![allow(unused_crate_dependencies)]

use criterion::{criterion_group, criterion_main};
use dradis_frame_check::{
    DecodeCheckArgs, DecodeCheckArgsDump, Metadata, QRCODE_HEIGHT, QRCODE_WIDTH,
    decode_and_check_frame,
};

const FRAME_WIDTH: u32 = 1280;
const FRAME_HEIGHT: u32 = 720;
const VALID_XXHASH2_FRAME: &[u8] = include_bytes!("../tests/data/valid-frame-ver-2-0.rgb888.raw");

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
                    hash: 0xcddbc559fb8264e6,
                    index: 6
                }
            )
        });
    });
    group.finish();
}

criterion_group!(benches, bench_frame_detect);
criterion_main!(benches);
