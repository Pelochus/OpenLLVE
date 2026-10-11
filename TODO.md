# OpenLLVE — TODO (next agent session)

Work **one task at a time**, in order. After each: mark it ✅, note what
changed, add follow-ups if needed, and keep the `IMPROVEMENTS.md` §0 status
table in sync.

## Read first (context)

1. `IMPROVEMENTS.md` — remaining work + §0 status table (source of truth).
2. `README.md` — overview + build commands.
3. `docs/dev/architecture/ARCHITECTURE.md` — layering tenets, repo layout, data flow, inference placement.
4. `core/README.md` — Rust core modules.
5. `external/models/README.md` — model convention (default model + submodules).

## Current state

- **Rust core**: compiles; all tests green (48 unit + 3 integration);
  clippy/fmt clean. FFI hardened (error codes, `openllve_abi_version`),
  metrics (median, f64, warm-up), blend semantics fixed,
  `NativeFrameHandle` validated.
- **Models**: single source of truth in `external/models/` (default
  `zero-dce-int8.tflite` as a raw file; new models as submodules); app
  assets hold symlinks to it.
- **Benchmarks**: the criterion bench (`core/benches/inference_bench.rs`)
  covers the model path with warm-up exclusion; minimal methodology +
  results table in `docs/benchmarks/`.
- **Android app**: functional vertical slice (Compose UI, LiteRT engine,
  MediaCodec decode, DataStore settings); domain/UI-state/media contracts
  live in the `:shared` KMP module. **JNI wiring landed (P1.1)**:
  `bridge/android/` crate exports 22 `Java_openllve_android_core_CoreBridge_*`
  symbols, `CoreBridge.kt`/`CoreToppings.kt` apply EWMA + blend to every
  enhanced frame.
- **Bridge crate**: `bridge/android/` builds host-side (`cargo build`, tests
  green) and cross-compiles with `scripts/build-bridge.sh` inside the dev
  container.
- **CI**: `android-ci.yml` (ktlint, androidLint, test, assemble),
  `rust-core.yml` (test, clippy, fmt), `release.yml` (on `v*` tags:
  test, release APK, GitHub Release), and `docker-image.yml` (on push to
  `dev`/`main`: build the dev image and push it to GHCR — pull with
  `podman pull ghcr.io/pelochus/openllve-dev:latest`).

## Do next (in order)

1. [~] **P1.2 follow-ups — model validation.**
   - Verify the pipeline on real (non-synthetic) frames; re-confirm the
     global-mean brightness channel vs upstream's 4×4 bilinear downscale.
   - ✅ ResizeInputTensor "segfault" root-caused (2026-10-10): not a library
     bug — a call-order misuse. `TfLiteInterpreterResizeInputTensor` returns
     OK, but called after `AllocateTensors` (which `tflite-c-rs` does eagerly
     in `Interpreter::new`) it NULLs the input tensor's data pointer until
     `AllocateTensors` runs again; writing through the tensor dereferences
     NULL → segfault. Verified with the pinned LiteRT C library (v2.17.1) on
     the upstream model: resize → allocate → invoke works; allocate → resize
     → invoke-without-realloc crashes; allocate → resize → re-allocate →
     invoke works. Docs updated (`external/models/README.md`, `core/README.md`);
     the fixed-shape model file stays (the crate has no resize API).
2. [ ] **P3.8 — Property tests for filters.**
   - Add `proptest`; assert EWMA output stays within
     `[min(prev,curr), max(prev,curr)]` per pixel and blend output is a convex
     combination. (Unblocked by P1.1: the bridge crate is the FFI target now.)
3. [~] **P1.3 — KMP shared layer.**
   - Done: `:shared` KMP module with the platform-neutral contracts from the
     Android vertical slice (domain, `FrameImage`/`FramePixels`/`VideoMetadata`,
     `UiState`, `SettingsStore` persistence contract — Android's
     `SettingsRepository` implements it); Android consumes it via thin
     adapters.
   - Remaining: none (P3.6 rejected — see Recently completed).
4. [x] **P2.4 — Dependency refresh** in `app/build.gradle.kts`: done — ML
   runtime (LiteRT 2.3.0 `CompiledModel` + the separate `litert-gpu` package)
   and full toolchain bump (AGP 9.4.0, KGP 2.4.20, Compose BOM 2026.09.00 →
   Compose UI 1.12.1, Gradle 9.7.1, compileSdk 37 in `:app` and `:shared`,
   lifecycle 2.11.0, navigation-compose 2.10.2, core-ktx 1.19.0; `docker/envsetup.sh`
   installs `platforms;android-37.0`). targetSdk stays at 34 (independent of
   compileSdk).
5. [x] **P1.1 — Android native (Rust) wiring** — landed 2026-10-11.
   - Investigation: `docs/dev/architecture/FFI-WIRING.md` — direct JNI (Rust
     cdylib → C ABI → Kotlin `external fun`s), **no C++ layer**; Rust-side
     LiteRT runner stays a non-core optional feature (PC-side testing),
     app production inference stays on the Kotlin-side LiteRT engine.
   - Done: `bridge/android/` crate (`jni` 0.22.4 + `jni-macros` 0.22.4,
     `#[jni_mangle("openllve.android.core.CoreBridge")]`) — 22 JNI exports
     (`Java_openllve_android_core_CoreBridge_*`, confirmed with `nm -gD`),
     `src/frames.rs` validates every JVM buffer against the declared
     dimensions before a pointer crosses (the C ABI trusts its dimension
     arguments), `src/bridge.rs` marshals `jlong` handles and `jfloatarray`
     frames. `core/src/ffi.rs` unchanged: the bridge calls its Rust API
     through a path dependency, it is not a second ABI.
   - Kotlin: `openllve/android/core/CoreBridge.kt` (one `external fun` per
     export, declared as instance members) + `CoreToppings.kt` (owns the
     EWMA/blend handles per media session, `apply()` per frame, `close()`
     once); `AndroidLiteRtEngine.kt` pipes `enhanceFrame()` through the
     toppings and releases them in `release()`. `:app:compileDebugKotlin`
     is green in the dev container.
   - Artifact: `scripts/build-bridge.sh` cross-compiles and installs
     `jniLibs/arm64-v8a/libbridge.so` (859,472 bytes, 22 exports);
     `System.loadLibrary("bridge")` resolves that file name, and the
     library statically contains the core, so the APK ships one Rust
     library.
   - Follow-ups: EWMA/blend weights are constants in `CoreToppings`
     (promote to `EnhancementSettings`); `x86_64` ABI not built; pipeline
     constructors/`processFrame` exported but unused by the app.
   - Separate debt (not P1.1): the `android-ci.yml` ktlint step fails on 31
     pre-existing violations (`function-signature` 20, `blank-line-before-declaration`
     3, `class-signature` 2, `import-ordering` 2, `spacing-between-declarations-with-comments`
     2, `string-template` 2) in `app/shared/src` + `app/platforms/android/src`; the
     new `core/` files are clean, `ktlint --format` fixes most of the rest.

6. [ ] **P3.9 — NPU: additional vendor dispatch runtimes (long-term).**
   - Extend NPU beyond Google Tensor (the only bundled dispatch runtime)
     to Qualcomm and, if prebuilt libs become available, MediaTek. The
     newest LiteRT release that ships NPU runtime libraries at all is the
     **2.2.0** zip — the 2.3.0 (October) release ships none — so Qualcomm
     dispatch + compiler plugin (QNN v69–v81) come from the 2.2.0 zip and
     pair with the 2.3.0 Kotlin runtime; MediaTek is not in the zip.
     2.3.0 added Samsung (Exynos AI LiteCore) and Intel (OpenVINO) vendor
     support, but no prebuilt libraries for them.
   - Constraint: LiteRT dlopens the *first* `libLiteRtDispatch*` in the
     dispatch library dir (multiple found → warning + unspecified first
     pick), so bundle exactly **one vendor runtime per APK** — Gradle
     product flavors (per-vendor `jniLibs`) or an AAB with Play Feature
     Delivery (the official sample's approach).
   - Per-vendor `NpuCompatibilityChecker` (Qualcomm/Mediatek/Default) plus
     the `isLibraryReady()` existence check; dispatch shims require the
     vendor runtime on the device (Tensor: `libedgetpu_litert.so`,
     Qualcomm: QAIRT). arm64-v8a only; JIT is Beta per vendor, AOT is the
     supported production path.

## Recently completed (removed from the list)

- **P3.1 — Model manifest**: rejected as overkill (model variations are
  rare; nothing consumes the manifest) — removed.
- **P3.6 — `BenchmarkRun` record + persistence**: rejected as overkill —
  runs are recorded in a simple `docs/benchmarks/RESULTS.md` table instead.
- **P2.3 — Benchmarks**: the criterion bench benchmarks the model path with
  warm-up exclusion; minimal methodology + results table in
  `docs/benchmarks/`.
- **GHCR dev image**: `docker-image.yml` builds `docker/Dockerfile` on push
  to `dev`/`main` and pushes it to
  `ghcr.io/pelochus/openllve-dev` (build runs on the GitHub runner — no local
  Docker daemon needed); `docker/run.sh --skip-build` runs the pulled image.
- **P2.1 — CI**: ktlint step in `android-ci.yml` + `release.yml` workflow.
- **P3.2 — Threading model**: decision in `docs/dev/architecture/ARCHITECTURE.md` §12;
  docs restructured into `docs/dev/` + `docs/benchmarks/` with a
  `docs/index.md` entry point; minimal `CONTRIBUTING.md` added.
- **P2.5 — Docs**: LiteRT naming standardized across docs and comments.

## Notes

- Don't build a complex CI pipeline until the app is functional.
- Keep commits small and scoped (one task per commit).
- Review `docs/dev/architecture/DESIGN_SUGGESTIONS.md` for further improvements where
  applicable (or defer them if not recommended or too hard/complex).
