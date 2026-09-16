# OpenLLVE — TODO (next agent session)

Work **one task at a time**, in order. After each: mark it ✅, note what
changed, add follow-ups if needed, and keep the `IMPROVEMENTS.md` §0 status
table in sync.

## Read first (context)

1. `IMPROVEMENTS.md` — remaining work + §0 status table (source of truth).
2. `README.md` — overview + build commands.
3. `docs/ARCHITECTURE.md` — layering tenets, repo layout, data flow, inference placement.
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
- **Android app**: compiles as a stub (Compose UI + placeholder pipeline
  manager; all enhancement logic lives in the Rust core). **No JNI/Rust
  wiring** (reverted — premature; re-attempt in P1.1).
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
3. [ ] **P3.2 — Threading model ADR.**
   - Short ADR: Rust core stays synchronous (one `process` per frame); the
     platform owns capture/process/render threads; bounded frame queue with
     drop-oldest when processing falls behind. Doc only, zero code.
4. [ ] **P3.8 — Property tests for filters.**
   - Add `proptest`; assert EWMA output stays within
     `[min(prev,curr), max(prev,curr)]` per pixel and blend output is a convex
     combination. (Golden-frame and FFI-fuzz tests wait for P1.1.)
5. [ ] **P1.3 — KMP shared layer.**
   - Create `app/shared/` KMP module: benchmark definitions, UI state
     contracts, pipeline/topping config model. (Or soften tenet #1 in docs.)
6. [ ] **P2.4 — Dependency refresh** in `app/build.gradle.kts`:
   TFLite 2.12.0 → current, CameraX 1.2.2 → current, Material 1.10.0 → current.
7. [ ] **P2.1 — CI**: add a ktlint step; add a small release workflow.
8. [ ] **P3.6 — `BenchmarkRun` record + persistence.**
   - `BenchmarkConfig` + `BenchmarkRun { config, device, thermal samples,
     latencies }` with `median`/`p99`/`fps`/`thermal_drift`; persist runs as
     JSON/CSV (add `serde`/`serde_json` here, P3.3). Needs P3.1; builds on
     the metrics API (done).
9. [ ] **P2.3 — Benchmarks**: benchmark the *model path* (not memcpy); keep
   warm-up exclusion; record device/thermal/battery metadata per run.
10. [ ] **P1.1 — Android native (Rust) wiring** (re-attempt once the app is
   functional and the NDK is available): cross-compile cdylib (cargo-ndk),
   Kotlin `external fun`s, C JNI glue, package `.so` into the APK.
11. [ ] **P2.5 — Docs**: standardize LiteRT/TFLite naming.

## Notes

- Don't build a complex CI pipeline until the app is functional.
- Keep commits small and scoped (one task per commit).
- Review `docs/DESIGN_SUGGESTIONS.md` for further improvements where
  applicable (or defer them if not recommended or too hard/complex).
