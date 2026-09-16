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

#[cfg(feature = "model")]
fn bench_model(c: &mut Criterion) {
    // Benchmark the *real* inference path (Zero-DCE model), not a memcpy.
    // Requires the `model` feature and `libtensorflowlite_c` at runtime.
    let model_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../external/models/zero-dce-int8.tflite");
    let mut strategy = match LlieStrategy::new().unwrap().with_model(&model_path, 4) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("model bench skipped (set OPENLLVE_TFLITE_LIB): {e}");
            return;
        }
    };

    let in_data = vec![0.5f32; (W * H * C) as usize];
    let input = FrameRef::new(W, H, C, STRIDE, &in_data).unwrap();
    let mut out_data = vec![0.0f32; (W * H * C) as usize];
    let mut output = Frame::new(W, H, C, STRIDE, &mut out_data).unwrap();

    c.bench_function("llie_model_720p", |b| {
        b.iter(|| {
            black_box(strategy.process(black_box(&input), black_box(&mut output))).unwrap();
        })
    });
}

#[cfg(feature = "model")]
fn bench_all(c: &mut Criterion) {
    bench_strategies(c);
    bench_model(c);
}

#[cfg(not(feature = "model"))]
fn bench_all(c: &mut Criterion) {
    bench_strategies(c);
}

criterion_group!(benches, bench_all);
criterion_main!(benches);
