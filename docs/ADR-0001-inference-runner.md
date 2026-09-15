# ADR-0001: Where does inference live?

- **Status:** Accepted
- **Date:** 2026-09-15
- **Deciders:** repository maintainers
- **Supersedes:** n/a

## Context

OpenLLVE's core tenet is that **all business logic and compute lives in the Rust
core**, with the rule of thumb: *if it can run on a standard PC with
`cargo test` / `cargo bench`, it is in the right layer*
(see `ARCHITECTURE.md` §2).

Today the strategy layer (`LlieStrategy`, `LlveTemporalStrategy`) is an identity
stub — no model is invoked — and the only model-related code in the repo is
`LiteRTInferenceEngine.kt`, a broken Kotlin stub that loads a zero-length
`ByteArray` and is never instantiated. The default test model
(`zero-dce-int8.tflite`, Zero-DCE, I/O `(H,W,4) → (H,W,24)`) is committed in app
assets.

The question is where the actual model execution belongs, because it determines
what P1.2 (model integration) implements and whether the benchmark harness can
ever measure a real inference path.

## Decision

**Inference runs inside the Rust core, via a `ModelRunner` backed by
`tflite-c-rs` (CPU delegate).**

- A `ModelRunner` in `core/` loads the `.tflite` model (from a path), runs
  inference, and returns the raw output tensor. Strategies call it; the FFI
  exposes it so Android can drive the same code.
- `LiteRTInferenceEngine.kt` is deleted. Kotlin never links TensorFlow Lite
  directly; it only calls the FFI.
- The runner is feature-gated (e.g. a `model` cargo feature) so the core still
  builds and passes `cargo test` without the TFLite runtime present.

## Options considered

### Option A — In-Rust runner via `tflite-c-rs` (chosen)

`tflite-c-rs` is a safe Rust wrapper over the TensorFlow Lite C API. It
dynamically loads `libtensorflowlite_c` (`.so`/`.dylib`/`.dll`) at runtime via
`libloading` — no `build.rs`, no `-sys` crate, no TFLite headers needed at
compile time. It provides an RAII `Interpreter` and an `InterpreterOptions`
builder (thread count, delegates).

Pros:

- Satisfies tenet #2: inference is business logic and lives in Rust.
- Satisfies the "cargo bench on a PC" rule of thumb: the *same* code path is
  benchmarkable with `cargo bench` on a desktop, which the benchmark harness
  and `BENCHMARK_METHODOLOGY.md` require.
- One implementation for all platforms; Android and (future) iOS both call the
  same FFI.
- `LiteRTInferenceEngine.kt` disappears; Kotlin is a thin FFI adapter.
- Delegate selection (CPU now; NNAPI/GPU later) can be exposed as an FFI knob,
  keeping the benchmark axis (delegate) configurable.

Cons:

- `tflite-c-rs` is early-stage (0.0.x). Mitigation: pin the version, keep it
  behind a cargo feature, and fall back to Option B if the crate proves
  unusable.
- The TFLite shared library must be present at runtime: on Android it is
  packaged into the APK (`jniLibs`) alongside the Rust cdylib; on PC it must be
  on the library search path (or loaded from an explicit path).

### Option B — Platform-side LiteRT in Kotlin

Keep inference in `LiteRTInferenceEngine.kt` (or a fixed equivalent) using the
Android LiteRT AAR.

Pros:

- Simpler short-term: no Rust TFLite dependency, uses the well-trodden Android
  AAR.

Cons:

- Violates tenet #2: the main compute of the engine would live in Kotlin.
- Blocks PC benchmarking: `cargo bench` could not exercise the real inference
  path, so the benchmark harness would still measure a stub.
- Duplicates the work the Rust core is supposed to own, and would have to be
  re-implemented for iOS.

Rejected.

## Consequences

1. **P1.2** implements `ModelRunner` in `core/` (load `zero-dce-int8.tflite`,
   map `(H,W,4) → (H,W,24)`, apply the 8×RGB output curves per
   `external/models/README.md`), called by the strategies and exposed through the FFI.
2. `LiteRTInferenceEngine.kt` is deleted as part of P1.2.
3. The Android app must package `libtensorflowlite_c.so` (from the LiteRT AAR
   or a standalone build) in `jniLibs` next to `libopenllve_core.so`.
4. CPU delegate is the default and the only delegate initially; delegate
   selection is a future FFI option (NNAPI on Android, GPU where available).
5. `cargo test` / `cargo bench` run the real model path on a PC once the
   `model` feature is enabled and `libtensorflowlite_c` is available.
6. The model file's single source of truth is `external/models/zero-dce-int8.tflite`;
   the app assets entry is a symlink to it, so PC-side runs load the file
   directly from that path.

## Follow-ups

- P1.2: implement `ModelRunner` per this ADR (highest priority).
- P1.1: Android wiring — the APK will then contain both the Rust cdylib and
  the TFLite shared library.
- P2.3: benchmark the model path with warm-up exclusion and device/thermal
  metadata.
