use criterion::{criterion_group, criterion_main};
use dradis_frame_check::{DecodeCheckArgs, DecodeCheckArgsDump, Metadata, decode_and_check_frame};

const FRAME_WIDTH: usize = 1280;
const FRAME_HEIGHT: usize = 720;
const QRCODE_WIDTH: usize = 128;
const QRCODE_HEIGHT: usize = 128;
const FRAME: &[u8] = include_bytes!("./data//test-qrcode-detection.rgb888.raw");

fn bench_frame_detect(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("frame processing");
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.bench_function("whole", |b| {
        b.iter(|| {
            let data = decode_and_check_frame(
                FRAME,
                Some(DecodeCheckArgs {
                    previous_frame_idx: None,
                    width: FRAME_WIDTH,
                    height: FRAME_HEIGHT,
                    swap_channels: false,
                    dump: DecodeCheckArgsDump::Ignore,
                }),
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
            )
        });
    });
    group.finish();
}

criterion_group!(benches, bench_frame_detect);
criterion_main!(benches);
