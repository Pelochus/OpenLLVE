# OpenLLVE Core (`openllve-core`)

The Rust core for **OpenLLVE** is the portable, performance-sensitive layer that owns strategy execution, frame abstractions, filtering, and telemetry. It is intentionally separate from Android and iOS app code so the same benchmarking logic can run across platforms without duplicating logic.

## Why the C FFI layer exists

The Rust core is not meant to be directly embedded into the Kotlin or Swift host runtime. Instead, it exposes a C-compatible API through `core/include/openllve_core.h` and `src/ffi.rs`.

This is the standard pattern when you need:

- a safe, fast compute layer in Rust
- platform app code in Kotlin/Swift
- a stable ABI boundary across toolchains and operating systems
- low overhead and no dependency on the JVM or Apple runtime internals

In other words: the app owns the mobile runtime, while the core owns the compute and metrics logic.

## Recommended module shape

Modern Rust prefers folder modules without `mod.rs` when using 2018+ edition. This keeps the tree easy to read and avoids a proliferation of `mod.rs` files.

```text
core/
├── Cargo.toml
├── README.md
├── include/
│   └── openllve_core.h
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── frame.rs
│   ├── metrics.rs
│   ├── filters.rs                # module declaration for ewma.rs + blend.rs
│   ├── filters/
│   │   ├── ewma.rs                # temporal anti-flicker filter
│   │   └── blend.rs               # raw/enhanced blending filter
│   ├── strategies.rs             # module declaration for llie.rs + temporal.rs
│   ├── strategies/
│   │   ├── llie.rs                # LLIE strategy, optionally configured with EWMA
│   │   └── temporal.rs            # LLVE strategy, optionally configured with blend
│   ├── ffi.rs                    # C ABI bindings
│   └── lib.rs
├── tests/
│   └── integration_pipeline.rs
├── benches/
│   └── inference_bench.rs
└── Cargo.lock
```

## Frame model

All processing operates on validated frame views, not flat slices:

- `FrameRef` — immutable, zero-copy view of a caller-owned buffer (input frames; C `const float*`).
- `Frame` — mutable, zero-copy view (output frames; C `float*`).
- `OwnedFrame` — owned, compact frame (no stride padding) for temporal model state.

Frames carry `width`, `height`, `channels`, `stride` (bytes per row, multiple of the element size, at least `width * channels * element_size`) and a `FrameFormat` (currently `F32`). Constructors validate dimensions and buffer size; mismatches between frames are checked errors (`CoreError::BufferDimensionMismatch`), not silent truncation. `bytemuck` provides typed views from raw byte buffers (`Frame::from_bytes` / `FrameRef::from_bytes`).

The processing API writes into a caller-owned output buffer — no per-frame allocation:

```rust
fn process(&mut self, input: &FrameRef, output: &mut Frame) -> Result<()>;
```

Platforms pre-allocate two frame buffers and ping-pong them (double buffering). The FFI mirrors this: `openllve_process_frame(strategy, in_w, in_h, in_ch, in_stride, in, out_w, out_h, out_ch, out_stride, out)` with independent input/output dims (a model may change the channel count, e.g. Zero-DCE 4 → 24).

## Strategy and topping model

The architecture separates the main model strategy from optional post-processing "toppings":

- `LlieStrategy`: frame-by-frame enhancement, can optionally use `EwmaFilter` as a temporal smoothing layer.
- `LlveTemporalStrategy`: stateful temporal model, may optionally use `FrameBlendFilter`, but should not usually add EWMA because the model already encodes temporal behavior.
- `FrameBlendFilter`: mixes raw input with processed output for stability and exposure control.
- `EwmaFilter`: anti-flicker smoothing for LLIE-style pipelines.

This is more flexible than hard-wiring all filters into every strategy.

## Design principles

1. **Zero-copy friendly**: frame handles abstract native buffers without redundant copying.
2. **Modular strategies**: the algorithm strategy is independent from the temporal tuning layer.
3. **Benchmark purity**: measure only the pure inference path, excluding camera I/O and UI render latency.
4. **Cross-platform portability**: Rust logic stays reusable while the app layer handles JVM/Swift integration.
