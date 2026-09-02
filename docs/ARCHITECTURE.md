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
├── docs/
├── .github/
├── .gitignore
├── .editorconfig
├── .gitattributes
├── .rustfmt.toml
├── .cargo/
├── README.md
├── build.gradle.kts
├── settings.gradle.kts
├── gradlew
├── gradlew.bat
├── local.properties
└── gradle.properties
```

The important architectural rule is that `core/` stays independent from the app runtime, and Android-specific code is kept under `app/platforms/android` rather than being mixed into the shared layer.

## 4. Platform Boundary

### Android / Kotlin layer
- capture, UI, camera session control, benchmark orchestration
- delegates to the Rust core via the generated C ABI
- manages Android thread dispatch and lifecycle

### Shared app layer (KMP)
- reusable app behavior and contracts that are not tied to a host SDK
- benchmark definitions and UI model/state abstractions
- platform-neutral orchestration glue between the presentation layer and the host platform adapters

### Rust Core layer
- frame abstraction and buffer compatibility
- strategy selection (`LlieStrategy`, `LlveTemporalStrategy`)
- optional post-processing toppings (`EwmaFilter`, `FrameBlendFilter`)
- benchmark metrics and pure timing logic

### Why C FFI
The C ABI is the compatibility layer between the app runtime and the Rust core. It avoids coupling the Rust crate to Kotlin or Swift internals while still allowing fast, cross-platform calls for performance-critical paths.

## 5. Application Technology Responsibilities

- **KMP shared app layer**: shared app state, domain models, benchmark orchestration, and UI contracts
- **Android platform layer**: Kotlin, Jetpack Compose, CameraX / Media3, Android lifecycle, and device-specific accelerator integration
- **Future iOS platform layer**: SwiftUI, AVFoundation, and Apple-specific runtime integration
- **Inference runtimes**: LiteRT / TFLite delegates on Android; Core ML / Metal or MPS on iOS when that platform is implemented
- **Rust core**: business logic, enhancement strategies, filters, frame abstractions, telemetry, and the C FFI surface

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

This is the key architectural decision: **EWMA is a topping for LLIE, not a universal layer for all strategies.**

## 8. Core Components

- `NativeFrameHandle`: abstraction over platform buffer handles
- `BenchmarkMetrics`: tracks latency, FPS, and p99 values
- `LlieStrategy`: LLIE execution strategy
- `LlveTemporalStrategy`: temporal execution strategy
- `EwmaFilter`: temporal smoothing filter
- `FrameBlendFilter`: raw-vs-enhanced blending filter
- `ffi.rs`: C bridging layer for Android/iOS interop

## 9. Metrics and Benchmarking

The benchmark must isolate the inference path from camera I/O, color conversion, rendering, and other host overhead. The primary measurements are:

- **Inference latency**: milliseconds per processed frame
- **Throughput**: processed frames per second
- **P99 latency**: tail latency under load
- **Thermal stability**: behavior during sustained processing and throttling

Warm-up frames should be excluded from reported averages. See [BENCHMARK_METHODOLOGY.md](BENCHMARK_METHODOLOGY.md) for the detailed procedure.

## 10. Design Principles

- **Zero-copy efficiency** wherever possible, especially at the native boundary
- **Strategy modularity** so model families can be swapped without rewriting the pipeline
- **Optional toppings** to keep algorithms composable rather than hard-coded
- **Pure benchmarking** for the inference path only
- **Cross-platform reuse** of core logic across mobile runtimes