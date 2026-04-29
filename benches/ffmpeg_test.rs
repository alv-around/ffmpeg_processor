use criterion::{Criterion, criterion_group, criterion_main};
use ffmpeg_processor::ffmpeg::run_ffmpeg;
use std::hint::black_box;
use tokio::runtime::Runtime;

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("performance of ffmeg wrapper function", |b| {
        b.to_async(&rt).iter(|| {
            run_ffmpeg(
                black_box("tests/data/trial_video.mp4".to_string()),
                black_box("tmp/test_output.mp4".to_string()),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
