use criterion::{Criterion, black_box, criterion_group, criterion_main};
use openllve_core::{Frame, FrameRef, InferenceStrategy, LlieStrategy, LlveTemporalStrategy};

const W: u32 = 1280;
const H: u32 = 720;
const C: u32 = 3;
const STRIDE: usize = (W * C) as usize * 4;

fn bench_strategies(c: &mut Criterion) {
    let mut llie_strategy = LlieStrategy::new().unwrap();
    let mut temporal_strategy = LlveTemporalStrategy::new();

    // Pre-allocated double-buffered frames: steady-state zero allocation.
    let in_data = vec![0.5f32; (W * H * C) as usize];
    let input = FrameRef::new(W, H, C, STRIDE, &in_data).unwrap();
    let mut out_data = vec![0.0f32; (W * H * C) as usize];
    let mut output = Frame::new(W, H, C, STRIDE, &mut out_data).unwrap();

    c.bench_function("llie_720p", |b| {
        b.iter(|| {
            black_box(llie_strategy.process(black_box(&input), black_box(&mut output))).unwrap();
        })
    });

    c.bench_function("temporal_720p", |b| {
        b.iter(|| {
            black_box(temporal_strategy.process(black_box(&input), black_box(&mut output))).unwrap();
        })
    });
}

criterion_group!(benches, bench_strategies);
criterion_main!(benches);
