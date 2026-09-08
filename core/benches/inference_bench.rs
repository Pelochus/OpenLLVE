use criterion::{black_box, criterion_group, criterion_main, Criterion};
use openllve_core::{InferenceStrategy, LlieStrategy, LlveTemporalStrategy};

fn bench_strategies(c: &mut Criterion) {
    let mut llie_strategy = LlieStrategy::new().unwrap();
    let mut temporal_strategy = LlveTemporalStrategy::new();
    let frame_data = vec![0.5f32; 1280 * 720];

    c.bench_function("llie_720p", |b| {
        b.iter(|| {
            black_box(llie_strategy.process(black_box(&frame_data))).unwrap();
        })
    });

    c.bench_function("temporal_720p", |b| {
        b.iter(|| {
            black_box(temporal_strategy.process(black_box(&frame_data))).unwrap();
        })
    });
}

criterion_group!(benches, bench_strategies);
criterion_main!(benches);
