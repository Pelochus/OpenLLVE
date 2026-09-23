# OpenLLVE — Repository Analysis & Suggested Improvements

*Generated from a full read of the repo (Rust core, Android app, docs, CI, build files) plus targeted web verification. Completed items and rejected/deferred items have been removed — this now tracks what remains.*

## 0. Implementation status

Status of the remaining §5 items:

| Item | Status |
| --- | --- |
| P1.1 wire Rust core into Android | ⬜ remaining — the single highest-value step (cross-compile cdylib, Kotlin `external fun`s) |
| P1.3 KMP shared layer | ⚠️ partial — `:shared` KMP module with domain/UI-state/media contracts; benchmark definitions (P3.6) remaining |
| P1.5 `NativeFrameHandle` | ⚠️ partial — validation added (`new()` now returns `Result`, rejects null ptr / `stride < width`), manual `Debug` impl, lifetime/pixel-format documented; still not wired into the FFI (deferred to P1.1) |
| P2.1 CI | ✅ done — clippy/fmt in `rust-core.yml`; ktlint step in `android-ci.yml` (prebuilt binary); `release.yml` workflow for `v*` tags |
| P2.3 benchmarks | ⬜ remaining |
| P2.4 dependency refresh | ✅ done — ML runtime migrated to LiteRT 2.2.0 (app) and toolchain bumped (AGP 9.4.0, KGP 2.4.20, Compose BOM 2026.06.01, Gradle 9.7.1, compileSdk 36); only the compileSdk 37 lines remain |
| P2.5 docs | ⚠️ partial — app now uses the real `com.google.ai.edge.litert` artifact; docs naming pass remaining |
| P3.3 crate additions | ⚠️ partial — `bytemuck` added; `serde`/`serde_json` wait for P3.6, `proptest` for P3.8 |

## 1. What this repo is

**OpenLLVE** (*Open Low-Light Video Enhancement*) is an early proof-of-concept for a real-time, on-device low-light video enhancement engine with a CPU/GPU/NPU benchmark harness. The intended layering is:

| Layer | Status |
| --- | --- |
| `core/` — Rust crate + C FFI (pipelines, filters, metrics) | Real code; **compiles and passes CI** |
| `app/shared/` — KMP shared app logic | Real module (`:shared`): domain/UI-state/media contracts |
| `app/platforms/android/` — Kotlin/Compose app | Functional vertical slice (LiteRT engine, media decode, Compose UI); **no FFI calls** |
| `app/platforms/ios/` — iOS | Placeholder README only |

## 2. Architecture: stated tenets vs. actual code

The docs are clear and good, but the code still contradicts them in places:

1. **"All business logic must live in the Rust core"** — the C FFI is never called from anywhere in the Android app; the entire `ffi.rs` surface is dead code today (P1.1).
2. **"KMP for all shared app logic"** — `app/shared/` is now a real KMP module (`:shared`) with the domain, UI-state, and media contracts. Remaining: benchmark definitions (P3.6) and a platform-neutral settings persistence contract.
3. **"Zero-copy frame handles"** — `NativeFrameHandle` exists but is **never used by the FFI** (the FFI takes raw `float*` + dimensions) and has no lifetime/aliasing story for the raw `*mut u8`.

## 3. Concrete code-level bugs & smells

| Location | Issue |
| --- | --- |
| `VideoPipelineManager.kt` | `startPipeline()`/`stopPipeline()` are empty comments; `processFrame` is a placeholder copy; no engine, no threading, no lifecycle. |
| `SystemMonitor.kt` | `getCpuUsage()` is a hardcoded `0.0f` placeholder; `getMemoryUsage()` uses the deprecated `ActivityManager.getMemoryInfo` path. |
| `MainScreen.kt` | Button is a no-op; no camera preview, no result surface, no state. |
| Naming | "LiteRT" (docs) vs `org.tensorflow:tensorflow-lite` artifacts: Google renamed TensorFlow Lite → **LiteRT** (Sept 2024). The app now uses the real LiteRT artifact (`com.google.ai.edge.litert:litert:2.2.0`, `CompiledModel` API), so the app-side naming issue is resolved; remaining: a docs-wide pass to standardize on "LiteRT". |

## 4. Suggested improvements (prioritized, remaining)

### P1 — Make the architecture real (align code with the tenets)

1. **Wire the Rust core into Android end-to-end** (the single highest-value step):
   - Cross-compile `openllve-core` to Android ABIs (`arm64-v8a`, `x86_64`) via `cargo-ndk` or a CI matrix job, packaging the `cdylib` as `libopenllve_core.so`.
   - Load it with `System.loadLibrary("openllve_core")` and call `openllve_*` from Kotlin via `external` functions.
2. **Implement the KMP shared layer** (even minimally): benchmark definitions, UI state contracts, and the pipeline/topping configuration model — or soften tenet #1 in the docs until it exists.
3. **Either wire `NativeFrameHandle` into the FFI** (pass pointer + stride + pixel format, validate dimensions) **or delete it** until it's used.

### P2 — Quality, tests, and process

1. **CI**: add a Kotlin lint step (ktlint); add a release workflow.
2. **Benchmarks**: benchmark the *model path* (not memcpy), keep warm-up exclusion, and record device/temperature/battery metadata per run as the methodology doc requires.
3. **Dependency refresh**: done — ML runtime on LiteRT 2.2.0 `CompiledModel` and toolchain bumped (AGP 9.4.0, KGP 2.4.20, Compose BOM 2026.06.01, Gradle 9.7.1, compileSdk 36). Only the compileSdk 37 lines remain (lifecycle 2.11.0, Compose UI 1.12.x, navigation 2.10.x, core 1.19.x).
4. **Docs**: standardize LiteRT vs TFLite naming (app-side resolved by the LiteRT migration; docs pass remaining).

### P3 — Design follow-ups from `DESIGN_SUGGESTIONS.md`

Ranked easiest/most-recommended → hardest/least-useful right now. Nothing here
blocks the build.

| # | Item (`DESIGN_SUGGESTIONS.md` §) | Effort | Verdict | Notes |
| --- | --- | --- | --- | --- |
| P3.1 | Model manifest (§2) | easy | **clear win** | `manifest.json` (or a Rust `ModelSpec`) next to each model: id, version, in/out shapes, quantization, supported delegates. Pure data + a tiny loader; makes benchmark runs reproducible and gives model validation something to check against. |
| P3.2 | Threading model ADR (§3) | easy (doc only) | **clear win** | Short ADR: Rust core stays synchronous (one `process` per frame); the platform owns capture/process/render threads; bounded frame queue with drop-oldest when processing falls behind. Zero code; records the decision the FFI and benchmark hang on. |
| P3.3 | Crate additions (§6) | easy | enabler | Add each crate when its paired item lands: `serde`/`serde_json` → P3.6, `proptest` → P3.8; `miri` as a CI job later; `static_assertions` optional. |
| P3.6 | `BenchmarkRun` record + persistence (§1) | med | **clear win** | `BenchmarkConfig` + `BenchmarkRun { config, device, thermal samples, latencies }` with `median`/`p99`/`fps`/`thermal_drift`; persist runs as JSON/CSV via serde so they are comparable and reproducible. Needs P3.1; builds on the metrics API (done). |
| P3.8 | Property / FFI-fuzz tests (§4) | med | partial now | proptest for filter invariants (EWMA stays within `[min,max]` per pixel, blend is a convex combination) can start now; golden-frame hashes and FFI fuzzing/miri need P1.1 first (FFI wiring). |

## 5. What's already good

- Clear, consistent architectural docs (`ARCHITECTURE.md`, `BENCHMARK_METHODOLOGY.md`, per-module READMEs) with explicit tenets and data flow.
- Correct layering *intent*: Rust compute + C ABI + platform adapters is the right shape for this problem.
- Sensible Rust module layout (folder modules, no `mod.rs`), unit tests per module, integration test, criterion bench, and a clean C header with opaque handles.
- Pipeline/topping composition (EWMA as an LLIE topping, not a universal layer) is a genuinely good design decision.
- `.gitignore` correctly excludes `target/`, `*.so`, `local.properties`.

---

### TL;DR

The build is fixed and the Rust core is real (model path included). The **core promise — Kotlin calling the Rust core — is still not implemented** (P1.1: the FFI is dead code today). The highest-leverage work remaining is: (1) cross-compile the cdylib and call it from Android, (2) benchmark the model path with per-run metadata (P2.3), and (3) the P3 design follow-ups (model manifest, threading ADR, property tests, `BenchmarkRun`).
