# OpenLLVE — Repository Analysis & Suggested Improvements

*Generated from a full read of the repo (Rust core, Android app, docs, CI, build files) plus targeted web verification. Completed items and rejected/deferred items have been removed — this now tracks what remains.*

## 0. Implementation status

Status of the remaining §4 items:

| Item | Status |
| --- | --- |
| P1.1 wire Rust core into Android | ✅ done — `bridge/android/` crate (`jni` 0.22.4 + `jni-macros`) exports 22 `Java_openllve_android_core_CoreBridge_*` symbols over the existing `core/src/ffi.rs` C ABI; Kotlin `CoreBridge.kt` + `CoreToppings.kt` run EWMA/blend per frame; `scripts/build-bridge.sh` installs `jniLibs/arm64-v8a/libbridge.so` (see `docs/dev/architecture/FFI-WIRING.md` §4) |
| P1.3 KMP shared layer | ✅ done — `:shared` KMP module with domain/UI-state/media contracts + `SettingsStore` persistence contract (P3.6 rejected) |
| P1.5 `NativeFrameHandle` | ⚠️ partial — validation added (`new()` now returns `Result`, rejects null ptr / `stride < width`), manual `Debug` impl, lifetime/pixel-format documented; still not wired into the FFI (deferred to P1.1) |
| P2.3 benchmarks | ✅ done — criterion bench covers the model path with warm-up exclusion; results table in `docs/benchmarks/` |
| P2.4 dependency refresh | ✅ done — newest everything: LiteRT 2.3.0 + `litert-gpu`, AGP 9.4.0, KGP 2.4.20, Compose BOM 2026.09.00 (UI 1.12.1), Gradle 9.7.1, compileSdk 37 (app + `:shared`), lifecycle 2.11.0, navigation-compose 2.10.2, core-ktx 1.19.0 |
| P3.3 crate additions | ⚠️ partial — `bytemuck` added; `proptest` waits for P3.8 (`serde` no longer needed — P3.6 rejected) |

## 1. What this repo is

**OpenLLVE** (*Open Low-Light Video Enhancement*) is an early proof-of-concept for a real-time, on-device low-light video enhancement engine with a CPU/GPU/NPU benchmark harness. The intended layering is:

| Layer | Status |
| --- | --- |
| `core/` — Rust crate + C FFI (pipelines, filters, metrics) | Real code; **compiles and passes CI** |
| `app/shared/` — KMP shared app logic | Real module (`:shared`): domain/UI-state/media contracts |
| `app/platforms/android/` — Kotlin/Compose app | Functional vertical slice (LiteRT engine, media decode, Compose UI); calls the Rust core through JNI for the temporal toppings |
| `app/platforms/ios/` — iOS | Placeholder README only |

## 2. Architecture: stated tenets vs. actual code

The docs are clear and good, but the code still contradicts them in places:

1. **"All business logic must live in the Rust core"** — the EWMA and blend filters now run in the Rust core and are called from Kotlin through the `bridge/android` JNI shim (P1.1). Model inference deliberately stays on the Kotlin LiteRT path (`docs/dev/architecture/FFI-WIRING.md` §3), so the core's `process_frame`/pipeline constructors are exported but not yet exercised by the app.
2. **"KMP for all shared app logic"** — `app/shared/` is now a real KMP module (`:shared`) with the domain, UI-state, and media contracts, plus the platform-neutral `SettingsStore` persistence contract.
3. **"Zero-copy frame handles"** — `NativeFrameHandle` exists but is **never used by the FFI** (the FFI takes raw `float*` + dimensions) and has no lifetime/aliasing story for the raw `*mut u8`.

## 3. Concrete code-level bugs & smells

The earlier round's stale entries (`VideoPipelineManager.kt`, `SystemMonitor.kt`,
`MainScreen.kt`) no longer exist in the tree — the app was rewritten around
`MainActivity` + `EnhancementViewModel` + the result screens. The remaining
real gap is the FFI:

| Location | Issue |
| --- | --- |
| `core/src/ffi.rs` | C ABI is consumed by the `bridge/android` JNI shim; the pipeline constructors and `process_frame` are exported but unused by the app, which keeps its Kotlin inference path. |
| `AndroidLiteRtEngine.kt` | Kotlin-side LiteRT engine is a parallel implementation of the Rust `ModelRunner`; the Rust core is not consulted. |
| `CoreToppings.kt` | Applies the EWMA/flicker toppings through the Rust core per frame; the alpha/beta weights are still constants rather than `EnhancementSettings` values. |

## 4. Suggested improvements (prioritized, remaining)

### P1 — Make the architecture real (align code with the tenets)

1. ~~Wire the Rust core into Android end-to-end~~ — done (P1.1): the JNI shim lives in its own crate (`bridge/android/`, `libbridge.so` in `jniLibs/arm64-v8a`), and model inference deliberately stays on the Kotlin LiteRT path. `x86_64` is not built (arm64-v8a only).
2. **Either wire `NativeFrameHandle` into the FFI** (pass pointer + stride + pixel format, validate dimensions) **or delete it** until it's used. The bridge validates JVM buffers against declared dimensions in `bridge/android/src/frames.rs` instead, so `NativeFrameHandle` is still unused.
2. **Either wire `NativeFrameHandle` into the FFI** (pass pointer + stride + pixel format, validate dimensions) **or delete it** until it's used.

### P2 — Quality, tests, and process

Nothing remaining — P2.4 (dependency refresh) is done: compileSdk 37 with the
newest AndroidX lines (lifecycle 2.11.0, navigation-compose 2.10.2, core-ktx
1.19.0, Compose 1.12.1 via BOM 2026.09.00), LiteRT 2.3.0 + `litert-gpu`, AGP
9.4.0, KGP 2.4.20, Gradle 9.7.1; the dev image installs `platforms;android-37`.

### P3 — Design follow-ups from `DESIGN_SUGGESTIONS.md`

Ranked easiest/most-recommended → hardest/least-useful right now. Nothing here
blocks the build.

| # | Item (`DESIGN_SUGGESTIONS.md` §) | Effort | Verdict | Notes |
| --- | --- | --- | --- | --- |
| P3.3 | Crate additions (§2) | easy | enabler | Add each crate when its paired item lands: `proptest` → P3.8; `miri` as a CI job later; `static_assertions` optional. (`serde` no longer needed — P3.6 rejected.) |
| P3.8 | Property / FFI-fuzz tests (§1) | med | partial now | proptest for filter invariants (EWMA stays within `[min,max]` per pixel, blend is a convex combination) can start now; golden-frame hashes and FFI fuzzing/miri need P1.1 first (FFI wiring). |

## 5. What's already good

- Clear, consistent architectural docs (`ARCHITECTURE.md`, `docs/benchmarks/`, per-module READMEs) with explicit tenets and data flow.
- Correct layering *intent*: Rust compute + C ABI + platform adapters is the right shape for this problem.
- Sensible Rust module layout (folder modules, no `mod.rs`), unit tests per module, integration test, criterion bench, and a clean C header with opaque handles.
- Pipeline/topping composition (EWMA as an LLIE topping, not a universal layer) is a genuinely good design decision.
- `.gitignore` correctly excludes `target/`, `*.so`, `local.properties`.

---

### TL;DR

The build is fixed and the Rust core is real (model path included). The **core promise — Kotlin calling the Rust core — is still not implemented** (P1.1: the FFI is dead code today). The highest-leverage work remaining is: (1) cross-compile the cdylib and call it from Android, (2) record benchmark runs in `docs/benchmarks/RESULTS.md` (P2.3 done), and (3) property tests (P3.8).
