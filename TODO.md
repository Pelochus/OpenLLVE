# OpenLLVE — TODO (next agent session)

Work **one task at a time**, in order. After each: mark it ✅, note what
changed, add follow-ups if needed, and keep the `IMPROVEMENTS.md` §0 status
table in sync.

## Read first (context)

1. `IMPROVEMENTS.md` — full analysis + §0 status table (source of truth).
2. `README.md` — overview + build commands.
3. `docs/ARCHITECTURE.md` — layering tenets, repo layout, data flow.
4. `core/README.md` — Rust core modules.
5. `models/README.md` — model convention (default model + submodules).

## Current state

- **Rust core**: compiles; 25 tests green; clippy/fmt clean. FFI hardened
  (error codes, `openllve_abi_version`), metrics (median, f64, warm-up),
  blend semantics fixed, `NativeFrameHandle` validated.
- **Models**: default `zero-dce-int8.tflite` in app assets; external
  submodules convention under `models/`.
- **Android app**: compiles as a stub (Compose UI + placeholder pipeline using
  Kotlin `LlieEwmaEnhancer`). **No JNI/Rust wiring** (reverted — premature).
- **CI**: small `android-ci.yml` (lint, test, assemble) + `rust-core.yml`
  (test, clippy, fmt).

## Do next (in order)

1. [ ] **P1.2 — Make the app actually functional (highest priority).**
   - Wire `zero-dce-int8.tflite` into `LiteRTInferenceEngine.kt`: load from
     assets, map input/output tensors (CPU delegate).
   - Make `VideoPipelineManager.processFrame` run the model on a frame.
   - Goal: frame in → enhanced frame out, runnable on device/emulator.
   - Model I/O is `(H,W,4) → (H,W,24)` (Zero-DCE: RGB + brightness channel in;
     8×RGB curve params out — apply the curves per `models/README.md` /
     raspberrypi/AI_enhance).
2. [ ] **P1.3 — KMP shared layer.**
   - Create `app/shared/` KMP module: benchmark definitions, UI state
     contracts, strategy/topping config model. (Or soften tenet #1 in docs.)
3. [ ] **P2.4 — Dependency refresh** in `app/build.gradle.kts`:
   TFLite 2.12.0 → current, CameraX 1.2.2 → current, Material 1.10.0 → current.
4. [ ] **P2.1 — CI**: add a ktlint step; add a small release workflow.
5. [ ] **P2.3 — Benchmarks**: benchmark the *model path* (not memcpy); keep
   warm-up exclusion; record device/thermal/battery metadata per run.
6. [ ] **P1.1 — Android native (Rust) wiring** (re-attempt once the app is
   functional and the NDK is available): cross-compile cdylib (cargo-ndk),
   Kotlin `external fun`s, C JNI glue, package `.so` into the APK.
7. [ ] **P2.5 — Docs**: standardize LiteRT/TFLite naming.

## Notes

- Don't build a complex CI pipeline until the app is functional.
- Keep commits small and scoped (one task per commit).
