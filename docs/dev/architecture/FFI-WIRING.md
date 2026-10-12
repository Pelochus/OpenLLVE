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

## 4. Landed: the bridge crate (P1.1, 2026-10-11)

The decision in §2 shipped as a **separate crate**, `bridge/android/`, rather
than JNI exports inside `core/`: `core/src/ffi.rs` remains the single C ABI
contract (22 exports, `OPENLLVE_ABI_VERSION = 3`, `OpenLlveError`
`Ok=0 / NullPointer=1 / InvalidParameter=2 / BufferDimensionMismatch=3 /
Internal=4`) and is **not** replaced, deleted, or duplicated. The bridge crate
depends on `openllve_core` as a normal Rust library (path dep, so the core's
Rust API is called directly — no `dlopen` chain) and adds only the JVM
marshalling layer.

Layout:

| File | Role |
| --- | --- |
| `bridge/android/src/bridge.rs` | one `#[jni_mangle("openllve.android.core.CoreBridge")] pub extern "system" fn` per C ABI export |
| `bridge/android/src/frames.rs` | boundary validation: JVM `Int` dimensions/strides are signed and JVM buffers can be short, both of which the C ABI trusts (`min_stride`, `invalid_stride`, `short_buffer`, `frame_violation`) with `cargo test`-able cases |
| `app/platforms/android/src/main/java/openllve/android/core/CoreBridge.kt` | Kotlin `external fun` contract, one declaration per export |
| `app/platforms/android/src/main/java/openllve/android/core/CoreToppings.kt` | owns the EWMA + blend handles for one media session; `apply()` per frame, `close()` once |
| `scripts/build-bridge.sh` | `cargo ndk -t arm64-v8a` → installs `jniLibs/<abi>/libbridge.so` |

Verified facts (2026-10-11, dev container):

- `nm -gD jniLibs/arm64-v8a/libbridge.so` → 22 `T Java_openllve_android_core_CoreBridge_<method>`
  entries; `jni_mangle` converts the Rust `snake_case` names to the Kotlin
  `lowerCamelCase` methods, so the JVM's lazy symbol lookup resolves them.
- `libbridge.so` is 859,472 bytes and **statically contains the core** —
  `libopenllve_core.so` is not a `DT_NEEDED` of the LiteRT dispatch libraries
  (they need only `libandroid/log/c/m/dl`), so the APK ships one Rust library,
  not two. The build script keeps Cargo's artifact in the gitignored
  `bridge/android/target/` tree and installs only the renamed `libbridge.so`.
- The JVM resolves `System.loadLibrary("bridge")` by **file name**
  (`libbridge.so`), not by `DT_NEEDED`; that rename is why the crate's
  `[lib] name` and the loaded name differ.
- Kotlin side compiles: `docker/run.sh --cmd './gradlew :app:compileDebugKotlin'`
  is green with JNI bindings declared as **instance** members of `CoreBridge`
  (a qualified `CoreBridge.method()` call does not resolve — the bindings are
  instance methods, so Kotlin holds one `CoreBridge()` receiver).

Follow-ups left open: EWMA/blend weights are constants in `CoreToppings`
(expose them as `EnhancementSettings` values later); `process_frame` and the
pipeline constructors are wired but not yet used by the app (the app keeps its
Kotlin LiteRT inference path per §3); `x86_64` ABI is not built (arm64-v8a only);
P3.8 property tests and FFI-fuzz tests can now target the bridge.

## 5. The C header was dropped (2026-10-12)

`core/include/openllve_core.h` (and `core/include/README.md`) are deleted. Nothing
consumed it: no `#include`, no `@cImport`, no build step or CI job read it, and
Kotlin declares its contract in `CoreBridge.kt` instead. It was a hand-written
mirror of `ffi.rs` with nothing checking it for drift.

`core/src/ffi.rs` remains the ABI contract — the `extern "C"` exports, handle
enums, and `OpenLlveError` codes are the source of truth, and each host shim
derives its own host-side declarations from them (JNI via `jni_mangle` today;
a Swift/iOS bridge would generate its interface from the `cdylib` or use a
binding tool, per P4.1).
