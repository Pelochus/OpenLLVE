# OpenLLVE — TODO (next agent session)

Work **one task at a time**, in order. After each: mark it ✅, note what
changed, add follow-ups if needed, and keep the `IMPROVEMENTS.md` §0 status
table in sync.

## Read first (context)

1. `IMPROVEMENTS.md` — full analysis + §0 status table (source of truth).
2. `README.md` — overview + build commands.
3. `docs/ARCHITECTURE.md` — layering tenets, repo layout, data flow.
4. `core/README.md` — Rust core modules.
5. `external/models/README.md` — model convention (default model + submodules).

## Current state

- **Rust core**: compiles; 25 tests green; clippy/fmt clean. FFI hardened
  (error codes, `openllve_abi_version`), metrics (median, f64, warm-up),
  blend semantics fixed, `NativeFrameHandle` validated.
- **Models**: default `zero-dce-int8.tflite` in app assets; external
  submodules convention under `external/models/`.
- **Android app**: compiles as a stub (Compose UI + placeholder pipeline using
  Kotlin `LlieEwmaEnhancer`). **No JNI/Rust wiring** (reverted — premature).
- **CI**: small `android-ci.yml` (lint, test, assemble) + `rust-core.yml`
  (test, clippy, fmt).

## Do next (in order)

1. [x] **P1.0 — Architecture decision: where does inference live?** ✅ 2026-09-15 — `docs/ADR-0001-inference-runner.md` written; Option A accepted (in-Rust `ModelRunner` via `tflite-c-rs`, CPU delegate; `LiteRTInferenceEngine.kt` to be deleted in P1.2). Follow-up: P1.2 implements the runner per the ADR.
   - Write a short ADR in `docs/` (e.g. `docs/ADR-0001-inference-runner.md`).
   - Option A (recommended): in-Rust runner via `tflite-c-rs` (CPU delegate) —
     satisfies the "cargo bench on a PC" tenet, enables PC-side benchmarking,
     `LiteRTInferenceEngine.kt` disappears, Kotlin only calls the FFI.
   - Option B: platform-side LiteRT in Kotlin — simpler short-term, but
     violates tenet #2 and blocks PC benchmarking.
   - P1.2 follows whichever option is chosen.
2. [ ] **P1.2 — Integrate the model where the ADR says (highest priority).**
   - If Option A: add a `ModelRunner` in the Rust core loading
     `zero-dce-int8.tflite`; strategies call it; benchmark the model path.
   - If Option B: wire the model into `LiteRTInferenceEngine.kt` (assets,
     tensor mapping, CPU delegate) and run it in `VideoPipelineManager`.
   - Goal: frame in → enhanced frame out.
   - Model I/O is `(H,W,4) → (H,W,24)` (Zero-DCE: RGB + brightness channel in;
     8×RGB curve params out — apply the curves per `external/models/README.md` /
     raspberrypi/AI_enhance).
3. [ ] **P1.3 — KMP shared layer.**
   - Create `app/shared/` KMP module: benchmark definitions, UI state
     contracts, strategy/topping config model. (Or soften tenet #1 in docs.)
4. [ ] **P2.4 — Dependency refresh** in `app/build.gradle.kts`:
   TFLite 2.12.0 → current, CameraX 1.2.2 → current, Material 1.10.0 → current.
5. [ ] **P2.1 — CI**: add a ktlint step; add a small release workflow.
6. [ ] **P2.3 — Benchmarks**: benchmark the *model path* (not memcpy); keep
   warm-up exclusion; record device/thermal/battery metadata per run.
7. [ ] **P1.1 — Android native (Rust) wiring** (re-attempt once the app is
   functional and the NDK is available): cross-compile cdylib (cargo-ndk),
   Kotlin `external fun`s, C JNI glue, package `.so` into the APK.
8. [ ] **P2.5 — Docs**: standardize LiteRT/TFLite naming.

## Notes

- Don't build a complex CI pipeline until the app is functional.
- Keep commits small and scoped (one task per commit).
