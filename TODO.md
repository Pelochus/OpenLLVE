# OpenLLVE — TODO (next agent session)

Work **one task at a time**, in order. After each: mark it ✅, note what
changed, add follow-ups if needed, and keep the `IMPROVEMENTS.md` §0 status
table in sync.

## Read first (context)

1. `IMPROVEMENTS.md` — remaining work + §0 status table (source of truth).
2. `README.md` — overview + build commands.
3. `docs/architecture/ARCHITECTURE.md` — layering tenets, repo layout, data flow, inference placement.
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
- **Android app**: functional vertical slice (Compose UI, LiteRT engine,
  MediaCodec decode, DataStore settings); domain/UI-state/media contracts
  live in the `:shared` KMP module. **No JNI/Rust wiring** (re-attempt in
  P1.1).
- **CI**: small `android-ci.yml` (lint, test, assemble) + `rust-core.yml`
  (test, clippy, fmt).

## Do next (in order)

1. [ ] **P1.2 follow-ups — model validation.**
   - Verify the pipeline on real (non-synthetic) frames; re-confirm the
     global-mean brightness channel vs upstream's 4×4 bilinear downscale.
   - Re-export the model with the correct static input shape, or root-cause
     the `TfLiteInterpreterResizeInputTensor` segfault (see
     `external/models/README.md`).
2. [ ] **P3.1 — Model manifest.**
   - `manifest.json` (or a Rust `ModelSpec`) next to each model in
     `external/models/`: id, version, in/out shapes, quantization, supported
     delegates. Unblocks reproducible benchmarks (P2.3) and P3.6.
3. [x] **P3.2 — Threading model ADR**: done (simplified) — decision recorded
   as §12 in `docs/architecture/ARCHITECTURE.md` (Rust core synchronous, one
   `process` per frame; platform owns capture/process/render threads; bounded
   frame queue with drop-oldest). Docs restructured into `docs/architecture/`,
   `docs/design/`, `docs/guides/`; minimal `CONTRIBUTING.md` added.
4. [ ] **P3.8 — Property tests for filters.**
   - Add `proptest`; assert EWMA output stays within
     `[min(prev,curr), max(prev,curr)]` per pixel and blend output is a convex
     combination. (Golden-frame and FFI-fuzz tests wait for P1.1.)
5. [~] **P1.3 — KMP shared layer.**
   - Done: `:shared` KMP module with the platform-neutral contracts from the
     Android vertical slice (domain, `FrameImage`/`FramePixels`/`VideoMetadata`,
     `UiState`); Android consumes it via thin adapters.
   - Remaining: benchmark definitions (P3.6) and a platform-neutral settings
     persistence contract.
6. [~] **P2.4 — Dependency refresh** in `app/build.gradle.kts`: ML runtime
   done (LiteRT 2.2.0 `CompiledModel`) and toolchain bump done (AGP 9.4.0,
   KGP 2.4.20, Compose BOM 2026.06.01, Gradle 9.7.1, compileSdk 36).
   Remaining: the compileSdk 37 lines
   (lifecycle 2.11.0, Compose UI 1.12.x, navigation 2.10.x, core 1.19.x)
   once android-37 is published.
7. [x] **P2.1 — CI**: done — real ktlint step in `android-ci.yml` (prebuilt
   binary, lints `app/shared/src` + `app/platforms/android/src`); new
   `release.yml` workflow (on `v*` tag: test, build release APK, upload to
   GitHub Release).
8. [ ] **P3.6 — `BenchmarkRun` record + persistence.**
   - `BenchmarkConfig` + `BenchmarkRun { config, device, thermal samples,
     latencies }` with `median`/`p99`/`fps`/`thermal_drift`; persist runs as
     JSON/CSV (add `serde`/`serde_json` here, P3.3). Needs P3.1; builds on
     the metrics API (done).
9. [ ] **P2.3 — Benchmarks**: benchmark the *model path* (not memcpy); keep
   warm-up exclusion; record device/thermal/battery metadata per run.
10. [~] **P1.1 — Android native (Rust) wiring**.
   - Investigation done: `docs/design/FFI-WIRING.md` — direct JNI (Rust
     cdylib → C ABI → Kotlin `external fun`s), **no C++ layer**; Rust-side
     LiteRT runner stays a non-core optional feature (PC-side testing),
     app production inference stays on the Kotlin-side LiteRT engine.
   - Remaining: cross-compile cdylib (cargo-ndk), Kotlin `external fun`s,
     package `.so` into the APK.
11. [x] **P2.5 — Docs**: done — LiteRT naming standardized across docs, READMEs, and
   comments (kept literal names: `tflite-c-rs`, `libtensorflowlite_c`, `.tflite`,
   `TfLite*` C API; app already on `com.google.ai.edge.litert`).

## Notes

- Don't build a complex CI pipeline until the app is functional.
- Keep commits small and scoped (one task per commit).
- Review `docs/design/DESIGN_SUGGESTIONS.md` for further improvements where
  applicable (or defer them if not recommended or too hard/complex).
