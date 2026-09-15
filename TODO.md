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
- **Models**: single source of truth in `external/models/` (default
  `zero-dce-int8.tflite` as a raw file; new models as submodules); app
  assets hold symlinks to it.
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
2. [ ] **P3.4+P3.5 — `Frame` type + out-buffer API (blocks P1.2).**
   - Replace flat `&[f32]` with a validated `Frame { width, height, channels,
     stride, data, format }`; `process(input: &Frame, output: &mut Frame)`
     instead of returning `Vec<f32>` per frame (~36 MB churn/frame at 1080p).
   - Honest FFI signature: `openllve_process_frame(strategy, w, h, channels,
     stride, in, out)`; dimension mismatches become checked errors.
   - Do both together; `ModelRunner` (P1.2) builds on this. Add `bytemuck`
     here for typed views (P3.3). Optionally rename `Strategy` → `Pipeline`
     - `TemporalMode { Stateless, Recurrent }` in the same rework (P3.7).
3. [ ] **P1.2 — Integrate the model where the ADR says (highest priority).**
   - If Option A: add a `ModelRunner` in the Rust core loading
     `zero-dce-int8.tflite`; strategies call it; benchmark the model path.
   - If Option B: wire the model into `LiteRTInferenceEngine.kt` (assets,
     tensor mapping, CPU delegate) and run it in `VideoPipelineManager`.
   - Goal: frame in → enhanced frame out.
   - Model I/O is `(H,W,4) → (H,W,24)` (Zero-DCE: RGB + brightness channel in;
     8×RGB curve params out — apply the curves per `external/models/README.md` /
     raspberrypi/AI_enhance).
4. [ ] **P3.1 — Model manifest.**
   - `manifest.json` (or a Rust `ModelSpec`) next to each model in
     `external/models/`: id, version, in/out shapes, quantization, supported
     delegates. Unblocks reproducible benchmarks (P2.3) and P3.6.
5. [ ] **P3.2 — Threading model ADR.**
   - Short ADR: Rust core stays synchronous (one `process` per frame); the
     platform owns capture/process/render threads; bounded frame queue with
     drop-oldest when processing falls behind. Doc only, zero code.
6. [ ] **P3.8 — Property tests for filters.**
   - Add `proptest`; assert EWMA output stays within
     `[min(prev,curr), max(prev,curr)]` per pixel and blend output is a convex
     combination. (Golden-frame and FFI-fuzz tests wait for P1.2.)
7. [ ] **P1.3 — KMP shared layer.**
   - Create `app/shared/` KMP module: benchmark definitions, UI state
     contracts, strategy/topping config model. (Or soften tenet #1 in docs.)
8. [ ] **P2.4 — Dependency refresh** in `app/build.gradle.kts`:
   TFLite 2.12.0 → current, CameraX 1.2.2 → current, Material 1.10.0 → current.
9. [ ] **P2.1 — CI**: add a ktlint step; add a small release workflow.
10. [ ] **P3.6 — `BenchmarkRun` record + persistence.**
    - `BenchmarkConfig` + `BenchmarkRun { config, device, thermal samples,
      latencies }` with `median`/`p99`/`fps`/`thermal_drift`; persist runs as
      JSON/CSV (add `serde`/`serde_json` here, P3.3). Needs P3.1; builds on
      P1.6 metrics (done).
11. [ ] **P2.3 — Benchmarks**: benchmark the *model path* (not memcpy); keep
    warm-up exclusion; record device/thermal/battery metadata per run.
12. [ ] **P1.1 — Android native (Rust) wiring** (re-attempt once the app is
    functional and the NDK is available): cross-compile cdylib (cargo-ndk),
    Kotlin `external fun`s, C JNI glue, package `.so` into the APK.
13. [ ] **P2.5 — Docs**: standardize LiteRT/TFLite naming.

## Notes

- Don't build a complex CI pipeline until the app is functional.
- Keep commits small and scoped (one task per commit).
- Review `docs/DESIGN_SUGGESTIONS.md` for further improvements where
  applicable (or defer them if not recommended or too hard/complex).
