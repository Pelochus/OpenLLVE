# OpenLLVE Core (`openllve-core`)

The Rust core for **OpenLLVE** is the portable, performance-sensitive layer that owns pipeline execution, frame abstractions, filtering, and telemetry. It is intentionally separate from Android and iOS app code so the same benchmarking logic can run across platforms without duplicating logic.

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
│   ├── model.rs                  # LiteRT ModelRunner (feature `model`)
│   ├── pipelines.rs              # module declaration for llie.rs + temporal.rs
│   ├── pipelines/
│   │   ├── llie.rs                # LLIE pipeline, optionally configured with EWMA
│   │   └── temporal.rs            # LLVE pipeline, optionally configured with blend
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

Platforms pre-allocate two frame buffers and ping-pong them (double buffering). The FFI mirrors this: `openllve_process_frame(pipeline, in_w, in_h, in_ch, in_stride, in, out_w, out_h, out_ch, out_stride, out)` with independent input/output dims (a model may change the channel count, e.g. Zero-DCE 4 → 24).

## Pipeline and topping model

The architecture separates the main model pipeline from optional post-processing "toppings":

- `LliePipeline`: frame-by-frame enhancement, can optionally use `EwmaFilter` as a temporal smoothing layer.
- `LlveTemporalPipeline`: stateful temporal model, may optionally use `FrameBlendFilter`, but should not usually add EWMA because the model already encodes temporal behavior.

Each pipeline reports its `TemporalMode`: `LliePipeline` is `Stateless` and `LlveTemporalPipeline` is `Recurrent`.

- `FrameBlendFilter`: mixes raw input with processed output for stability and exposure control.
- `EwmaFilter`: anti-flicker smoothing for LLIE-style pipelines.

This is more flexible than hard-wiring all filters into every pipeline.

## Model runner (feature `model`)

Inference runs inside the Rust core (see `docs/architecture/ARCHITECTURE.md` §11). The `ModelRunner`
(`src/model.rs`, behind the `model` cargo feature) loads the Zero-DCE model
via `tflite-c-rs`, which dynamically opens `libtensorflowlite_c` at runtime
(no build-time link). The library path comes from the `OPENLLVE_TFLITE_LIB`
environment variable, falling back to the default search path.

- `ModelRunner::new(model_path, num_threads)` loads the model and validates its
  `(1, 256, 256, 4) → (1, 256, 256, 24)` I/O shape.
- `ModelRunner::run_frame(input, output)` maps an RGB `[0, 1]` frame to an
  enhanced RGB `[0, 1]` frame: it derives the 4th (brightness) input channel
  as the frame's global mean clamped to `0.5`, runs the model (tiling frames
  larger than 256×256 into overlapping 256×256 patches with 16 px overlap,
  reflect padding, and linear-ramp reassembly — mirroring the upstream
  `network.py`), and applies the 8 learned curves per pixel.
- `LliePipeline::with_model(path, num_threads)` attaches a runner; `process`
  then runs the model instead of the identity stub. The FFI exposes it via
  `openllve_pipeline_new_llie_with_model(model_path, num_threads)`.

The feature is off by default so `cargo test` / `cargo clippy` / `cargo fmt`
pass without the LiteRT runtime present. To run the real model path on a PC:

```bash
scripts/fetch-tflite-lib.sh core/native          # downloads libtensorflowlite_c
OPENLLVE_TFLITE_LIB=core/native/libtensorflowlite_c.so \
  cargo test --features model
OPENLLVE_TFLITE_LIB=core/native/libtensorflowlite_c.so \
  cargo bench --features model
```

On Android the same cdylib is packaged alongside `libopenllve_core.so` and the
LiteRT shared library (see `docs/architecture/ARCHITECTURE.md` §11); Kotlin only calls the FFI.

**Scope and performance.** The Rust `ModelRunner` exists for PC-side
benchmarking and validation (the tenet: "if it can run on a PC with
`cargo bench`, it's in the right layer"), **not** for production throughput.
It runs the LiteRT C **reference kernel** (`tflite-c-rs` does not expose the
XNNPACK delegate), so its numbers are not representative of on-device
performance: on Android the real LiteRT runtime uses XNNPACK (or GPU/NPU),
which is ~5× faster than the reference kernel. Measured on this desktop:
256×256 ≈ 11 ms (87 FPS) and 1280×720 ≈ 337 ms (3 FPS) under XNNPACK, versus
≈ 1.68 s/frame under the reference kernel. The model is a small-patch
(256×256) network, so it is real-time at its native patch size but tiling it
to 720p (≈18 overlapping patches) is the dominant cost — not the kernel.

**Known workaround (not a fix).** The upstream model ships with a static
`[1,1,1,4]` input shape and is meant to be resized at runtime, but
`TfLiteInterpreterResizeInputTensor` **segfaults** in the LiteRT C library for
this graph (root cause not yet identified). The committed
`zero-dce-int8.tflite` therefore has the `[1,256,256,4]` input shape baked
into the flatbuffer (in-place patch, verified bit-exact vs upstream+resize).
Follow-up: re-export the model with the correct static shape, or root-cause
the resize segfault (see `TODO.md`).

## Design principles

1. **Zero-copy friendly**: frame handles abstract native buffers without redundant copying.
2. **Modular pipelines**: the algorithm pipeline is independent from the temporal tuning layer.
3. **Benchmark purity**: measure only the pure inference path, excluding camera I/O and UI render latency.
4. **Cross-platform portability**: Rust logic stays reusable while the app layer handles JVM/Swift integration.
