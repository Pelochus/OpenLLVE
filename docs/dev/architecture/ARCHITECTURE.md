# OpenLLVE Architecture Documentation

## 1. Overview

OpenLLVE is a high-performance system for real-time low-light video enhancement. The architecture is intentionally split into a reusable Rust core, a Kotlin Multiplatform shared app layer, and host-specific platform adapters so the compute logic is not tied to Android or iOS implementation details.

## 2. Core Architectural Tenets

1. **Kotlin Multiplatform (KMP) for all shared app logic**: All app logic that is shared across Android and iOS must live in KMP so it is reusable without duplication. This includes shared domain models, UI state contracts, benchmark definitions, and app-side orchestration logic.
2. **Rust for all business logic and compute**: All real algorithmic logic, filtering, strategy selection, telemetry logic, and buffer/compute orchestration must live in the Rust core.
   - **Strict Rule of Thumb**: if the logic can be executed on a standard PC with `cargo test` / `cargo bench`, it is likely in the correct layer. If it depends on Android or iOS SDK types directly, it is probably in the wrong layer and should be pushed behind a platform boundary.

## 3. Repository Layout

```text
OpenLLVE/
├── core/                          # Portable Rust core + C FFI; platform-agnostic compute
├── app/
│   ├── shared/                    # KMP shared app logic: contracts, UI state, domain, benchmark orchestration
│   ├── platforms/
│   │   ├── android/               # Android-specific runtime sources and glue
│   │   │   └── src/
│   │   └── ios/                   # Future iOS SwiftUI / AVFoundation layer (empty placeholder for now)
│   └── build.gradle.kts
├── external/                      # External model submodules (see external/models/README.md)
├── docs/
├── scripts/                      # Local dev tooling (not part of CI)
├── .github/
├── .gitignore
├── .editorconfig
├── .gitattributes
├── .rustfmt.toml
├── .cargo/
├── gradle/
├── README.md
├── settings.gradle.kts
├── gradlew
└── gradle.properties
```

The important architectural rule is that `core/` stays independent from the app runtime, and Android-specific code is kept under `app/platforms/android` rather than being mixed into the shared layer.

## 4. Platform Boundary

### Android / Kotlin layer

- media selection (SAF), MediaCodec/MediaExtractor decode, UI, benchmark orchestration
- delegates to the Rust core via the generated C ABI
- manages Android thread dispatch and lifecycle

### Shared app layer (KMP)

- reusable app behavior and contracts that are not tied to a host SDK
- benchmark definitions and UI model/state abstractions
- platform-neutral orchestration glue between the presentation layer and the host platform adapters

### Rust Core layer

- frame abstraction and buffer compatibility
- pipeline selection (`LliePipeline`, `LlveTemporalPipeline`)
- optional post-processing toppings (`EwmaFilter`, `FrameBlendFilter`)
- benchmark metrics and pure timing logic

### Why C FFI

The C ABI is the compatibility layer between the app runtime and the Rust core. It avoids coupling the Rust crate to Kotlin or Swift internals while still allowing fast, cross-platform calls for performance-critical paths.

## 5. Application Technology Responsibilities

- **KMP shared app layer**: shared app state, domain models, benchmark orchestration, and UI contracts
- **Android platform layer**: Kotlin, Jetpack Compose, MediaCodec/MediaExtractor, Android lifecycle, and device-specific accelerator integration
- **Future iOS platform layer**: SwiftUI, AVFoundation, and Apple-specific runtime integration
- **Inference runtimes**: LiteRT delegates on Android; Core ML / Metal or MPS on iOS when that platform is implemented
- **Rust core**: business logic, enhancement pipelines, filters, frame abstractions, telemetry, and the C FFI surface

## 6. Data Flow

1. **Video Capture**: the host platform captures frames from camera or a test source.
2. **Frame Hand-off**: frame handles are mapped into a Rust-compatible abstraction.
3. **Inference Execution**: a strategy runs the enhancement model or temporal logic.
4. **Optional Post-Processing**: a topping may smooth the result or blend with the original frame.
5. **Rendering**: the processed frame is handed to the platform UI pipeline.

## 7. Strategy Model

The strategy is responsible for the main inference path. Toppings are optional and should be attached only when they are relevant to the chosen strategy.

### LLIE strategy

- best for static frame-wise enhancement models
- may attach `EwmaFilter` for anti-flicker stabilization
- may also attach `FrameBlendFilter` when blending with the original signal is desired

### LLVE temporal strategy

- best for stateful or recurrent temporal models
- may accept a `FrameBlendFilter`
- usually should not add `EwmaFilter` unless the model explicitly requires it

This is the key architectural decision: **EWMA is a topping for LLIE, not a universal layer for all pipelines.**

## 8. Core Components

- `NativeFrameHandle`: abstraction over platform buffer handles
- `BenchmarkMetrics`: tracks latency, FPS, and p99 values
- `LliePipeline`: LLIE execution pipeline (`TemporalMode::Stateless`)
- `LlveTemporalPipeline`: temporal execution pipeline (`TemporalMode::Recurrent`)
- `EwmaFilter`: temporal smoothing filter
- `FrameBlendFilter`: raw-vs-enhanced blending filter
- `ffi.rs`: C bridging layer for Android/iOS interop

## 9. Metrics and Benchmarking

The benchmark must isolate the inference path from camera I/O, color conversion, rendering, and other host overhead. The primary measurements are:

- **Inference latency**: milliseconds per processed frame
- **Throughput**: processed frames per second
- **P99 latency**: tail latency under load
- **Thermal stability**: behavior during sustained processing and throttling

Warm-up frames should be excluded from reported averages. See [METHODOLOGY.md](../../benchmarks/METHODOLOGY.md) for the procedure.

## 10. Design Principles

- **Zero-copy efficiency** wherever possible, especially at the native boundary
- **Strategy modularity** so model families can be swapped without rewriting the pipeline
- **Optional toppings** to keep algorithms composable rather than hard-coded
- **Pure benchmarking** for the inference path only
- **Cross-platform reuse** of core logic across mobile runtimes

## 11. Inference placement (decision, accepted 2026-09-15)

Inference runs **inside the Rust core**, not on the platform: a `ModelRunner`
(feature `model` in `core/`) loads the `.tflite` model via `tflite-c-rs`
(CPU delegate, dynamically loaded at runtime — no build-time link). The
platform never links LiteRT directly; Kotlin/Swift only call the C
FFI. This satisfies tenet 2 and the "cargo bench on a PC" rule: the same
inference path is benchmarkable on a desktop, and there is one implementation
for all platforms.

- The model file's single source of truth is `external/models/`; the app
  assets entry is a symlink to it.
- On Android, `libtensorflowlite_c.so` is packaged in `jniLibs` alongside the
  Rust cdylib; on a PC it is loaded from `OPENLLVE_TFLITE_LIB`
  (`scripts/fetch-tflite-lib.sh`).
- CPU delegate is the default and the only delegate initially; delegate
  selection (NNAPI/GPU) is a future FFI option.

## 12. Threading model (decision)

Real-time video has three flows — capture, process, render — and the split is
fixed as follows:

- **The Rust core stays synchronous**: one `process` call per frame, no
  internal threads. Simplest to test, keeps the benchmark pure, and avoids
  threading across the FFI.
- **The platform owns the threads**: a capture thread, a dedicated processing
  thread, and a render thread.
- **Bounded frame queue with drop-oldest**: when processing falls behind, the
  oldest queued frame is dropped — standard real-time video behavior, and it
  avoids unbounded latency growth.

An async pipeline inside Rust was considered and rejected: it buys nothing at
this stage and complicates the FFI.
