# FFI wiring: Rust core ↔ Android (P1.1 investigation)

*Investigation and decision record for wiring the Rust core into the
Android app. Supersedes the open P1.1 question "wire Rust to Kotlin
directly, or via an intermediate C++ layer?"*

## 1. Question

How should Kotlin call the Rust core?

- **Option A — direct JNI**: Rust `cdylib` with a C ABI; Kotlin declares
  `external fun`s and loads the `.so` via `System.loadLibrary`.
- **Option B — intermediate C++ layer**: Rust → C ABI → a C++ shim → JNI
  (`jni.h`) → Kotlin.

## 2. Findings

- JNI is itself a **C API** (`JavaVM`/`JNIEnv` are function-table
  pointers). The JVM loads any `.so` that exports C symbols — no C++ is
  required on the path.
- The standard, well-trodden Rust→Android setup is: `cargo-ndk` (cross-
  compile `arm64-v8a`/`x86_64`), a Rust `cdylib`, Kotlin `external fun`s,
  `.so` packaged in `jniLibs`. AOSP documents this as the canonical Rust
  pattern; the `jni` crate is only needed if Rust code must call *back*
  into the JVM or handle JVM objects.
- A C++ shim adds a third language, a third build step, and no capability
  we need: our ABI is plain `float*` + dimensions + opaque handles +
  error codes. C++ would only be justified by C++-only libraries or heavy
  object marshalling — neither applies.

**Decision: Option A.** Direct JNI: cross-compile the existing `ffi.rs`
C ABI with `cargo-ndk`, package `libopenllve_core.so`, and call it from
Kotlin `external fun`s. No C++ layer.

## 3. LiteRT on the Rust side: core or not?

The Rust `ModelRunner` (feature `model`) runs the model via `tflite-c-rs`,
which loads `libtensorflowlite_c` at runtime (no build-time link).

Concerns:

- The C library is an **external, version-sensitive dependency**: it must
  be vendored/fetched (`scripts/fetch-tflite-lib.sh`) and kept in step with
  the model and LiteRT releases. A stale or mismatched library is a
  breakage vector over time (the `TfLiteInterpreterResizeInputTensor`
  segfault is already one symptom of this surface).
- The Rust runner executes the **reference kernel** (no XNNPACK/NNAPI
  delegate), so it is not representative of on-device performance anyway.
- The app already has a production-quality LiteRT path on the Kotlin side
  (`com.google.ai.edge.litert`, XNNPACK-capable).

**Decision: keep the Rust-side LiteRT runner non-core.** It stays an
optional cargo feature (`model`), off by default, used for PC-side
testing and benchmarking of the pipeline. The app's production inference
stays on the Kotlin-side LiteRT engine; the Rust core is wired into the
app via the FFI for the compute path (pipelines, filters, metrics), not
for model inference on device.

## 4. Remaining implementation (P1.1, still open)

- `cargo-ndk` cross-compile of the `cdylib` (`arm64-v8a`, `x86_64`).
- Kotlin `external fun`s for the `openllve_*` FFI surface +
  `System.loadLibrary("openllve_core")`.
- Package the `.so` into the APK (`jniLibs`), then replace the Kotlin-side
  placeholder processing with FFI calls.
