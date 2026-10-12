# OpenLLVE Core (`openllve-core`)

Portable, performance-sensitive Rust layer: pipeline execution, frame
abstractions, filtering, telemetry. Deliberately separate from app code so the
same compute and benchmarking logic runs on every platform.

## Why the C FFI layer exists

The core is not embedded in the Kotlin/Swift runtime. `src/ffi.rs` exposes a C-compatible
API — its `extern "C"` exports, handle types and error codes are the contract every
host bridge reuses (see `bridge/android/README.md` for the JVM half). The app owns
the mobile runtime; the core owns compute and metrics.

## Module shape

Folder modules, no `mod.rs`:

```text
core/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # module wiring + ABI probe
│   ├── error.rs                  # OpenLlveError codes
│   ├── frame.rs                  # FrameRef / Frame / OwnedFrame + frame contract
│   ├── metrics.rs                # BenchmarkMetrics
│   ├── filters.rs                # declarations for filters/
│   ├── filters/{ewma,blend}.rs
│   ├── model.rs                  # LiteRT ModelRunner (cargo feature `model`)
│   ├── pipelines.rs              # declarations for pipelines/
│   ├── pipelines/{llie,temporal}.rs
│   └── ffi.rs                    # the one C ABI
├── tests/                        # integration + property + FFI fuzz gates
├── benches/inference_bench.rs
└── Cargo.lock
```

## Frame model

Processing operates on validated views of caller-owned memory, never flat slices:

- `FrameRef` — immutable zero-copy view (input; C `const float*`)
- `Frame` — mutable zero-copy view (output; C `float*`)
- `OwnedFrame` — owned, compact frame (no stride padding), for temporal state

Frames carry `width`, `height`, `channels`, `stride` (bytes per row: a multiple of
the element size, at least `width * channels * element_size`) and a `FrameFormat`
(currently `F32`). Mismatched frames are check errors (`CoreError::BufferDimensionMismatch`),
not silent truncation; `bytemuck` gives typed views from raw bytes
(`Frame::from_bytes` / `FrameRef::from_bytes`).

Two functions are the frame contract every consumer runs: `validate_dimensions`
(geometry) and `buffer_elements` (buffer requirement, in `u64` so an absurd
`stride * height` cannot wrap). The frame-taking `ffi.rs` exports run them
*before* sizing caller memory, host bridges call the same pair, and
`Frame::new`/`FrameRef::new` are thin wrappers — so the safe API and the C ABI
cannot drift apart.

Processing writes into a caller-owned buffer, so platforms pre-allocate two
buffers and ping-pong them:

```rust
fn process(&mut self, input: &FrameRef, output: &mut Frame) -> Result<()>;
```

The FFI mirrors this with independent input/output geometry
(`openllve_process_frame(pipeline, in_w, in_h, in_ch, in_stride, in, out_w, out_h, out_ch, out_stride, out)`)
because a model may change the channel count (Zero-DCE 4 → 24).

## Pipeline and topping model

- `LliePipeline` — frame-by-frame enhancement, `TemporalMode::Stateless`; may
  attach `EwmaFilter` for anti-flicker and `FrameBlendFilter` for exposure control.
- `LlveTemporalPipeline` — stateful temporal model, `TemporalMode::Recurrent`; may
  attach `FrameBlendFilter`, and usually should not add EWMA, since the model
  already encodes temporal behaviour.

Toppings stay separate from pipelines so each combination is intentional.

## Model runner (cargo feature `model`)

`src/model.rs` runs inference in Rust via `tflite-c-rs`, which opens
`libtensorflowlite_c` at runtime (no build-time link); the library comes from
`OPENLLVE_TFLITE_LIB`, falling back to the default search path.

- `ModelRunner::new(model_path, num_threads)` loads a model and validates its
  `(1, 256, 256, 4) → (1, 256, 256, 24)` I/O shape.
- `ModelRunner::run_frame(input, output)` maps an RGB `[0, 1]` frame to an
  enhanced RGB `[0, 1]` frame: it derives the 4th (brightness) channel as the
  frame's global mean clamped to `0.5`, tiles frames larger than 256×256 into
  overlapping 256×256 patches (16 px overlap, reflect padding, linear-ramp
  reassembly — mirroring upstream `network.py`), and applies the 8 learned curves
  per pixel.
- `LliePipeline::with_model(path, num_threads)` attaches a runner, so `process`
  runs the model instead of the identity stub; the FFI exposes it as
  `openllve_pipeline_new_llie_with_model`.

The feature is off by default so plain `cargo test`/`clippy`/`fmt` need no LiteRT
runtime. To exercise the real path on a PC:

```bash
scripts/fetch-tflite-lib.sh core/native
OPENLLVE_TFLITE_LIB=core/native/libtensorflowlite_c.so cargo test --features model
OPENLLVE_TFLITE_LIB=core/native/libtensorflowlite_c.so cargo bench --features model
```

**Scope.** This runner exists for PC-side validation and benchmarking, not
production throughput: it uses the LiteRT C reference kernel (`tflite-c-rs`
exposes no XNNPACK delegate), so its numbers understate on-device LiteRT, which
measured ~5× faster here (256×256 ≈ 11 ms / 87 FPS, 1280×720 ≈ 337 ms / 3 FPS
under XNNPACK versus ≈ 1.68 s/frame on the reference kernel). The dominant cost
at 720p is tiling a 256×256-patch network (~18 overlapping patches), not the
kernel. On Android the app infers in its own LiteRT runtime and the core ships
inside `libbridge.so` in `jniLibs/<abi>/` — statically linked, so the APK carries
one Rust library. Kotlin calls it through `CoreBridge`.

## Design principles

1. Zero-copy frame handles over native buffers.
2. Pipelines independent of temporal tuning.
3. Benchmark only the inference path — no camera I/O or render latency.
4. Rust logic stays reusable while app layers handle JVM/Swift integration.
