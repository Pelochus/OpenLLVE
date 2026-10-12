# OpenLLVE — Repository Analysis & Suggested Improvements

*Completed items are removed (their results live in module READMEs and `docs/`);
this file tracks what remains.*

## 0. Implementation status

| Item | Status |
| --- | --- |
| P1.5 `NativeFrameHandle` | ⚠️ open — validated (`new_checked` returns `Result`, rejects null ptr / `stride < width`), lifetime + pixel format documented; still **unused** by the FFI (raw `float*` + dimensions cross instead) |
| P3.3 crate additions | ⚠️ partial — `bytemuck` + `proptest` added (`serde` no longer needed — P3.6 rejected) |
| P4.1 iOS platform (high-level) | ⚠️ open — iOS is a separate host layer tracked at a high level only (§4 P4); the C ABI in `ffi.rs` is already complete for it |
| P1.8 EWMA/blend weights as settings | ⚠️ open — `CoreToppings` constants `EWMA_ALPHA = 0.8f`, `BLEND_BETA = 0.9f` (80 % of the current frame: flicker damped without visible ghosting; 90 % AI-enhanced) |

## 1. What this repo is

**OpenLLVE** (*Open Low-Light Video Enhancement*) is an early proof-of-concept for
a real-time, on-device low-light video enhancement engine with a CPU/GPU/NPU
benchmark harness.

| Layer | Status |
| --- | --- |
| `core/` — Rust crate + C FFI (pipelines, filters, metrics) | Real code; compiles and passes CI |
| `app/shared/` — KMP shared app logic | Real `:shared` module: domain / UI-state / media contracts |
| `app/platforms/android/` — Kotlin/Compose app | Functional vertical slice; calls the Rust core through JNI for the temporal toppings |
| `app/platforms/ios/` — iOS | README placeholder only; tracked as P4 (high-level) |

## 2. Architecture: stated tenets vs. actual code

1. **"All business logic must live in the Rust core"** — the temporal toppings do
   run there through the JNI shim; model inference deliberately stays on the Kotlin
   LiteRT path for production, so the core's `process_frame` and pipeline
   constructors are exported but unused by the app.
2. **"Platform-neutral compute belongs in `core/`"** — satisfied for frame checks:
   the frame contract lives in `core/src/frame.rs` and both the C ABI and the JVM
   shim run it, so a shim carries only its host's own facts.
3. **"KMP for all shared app logic"** — satisfied: `app/shared/` holds the domain,
   UI-state, media and platform-neutral `SettingsStore` contracts.
4. **"Zero-copy frame handles"** — `NativeFrameHandle` exists but is unused by the
   FFI (raw `float*` + dimensions are passed) and has no lifetime/aliasing story for
   the raw `*mut u8` (P1.5).

## 3. Concrete code-level smells

| Location | Issue |
| --- | --- |
| `core/src/ffi.rs` | The pipeline constructors and `process_frame` are exported but unused by the app, which keeps its Kotlin inference path. |
| `AndroidLiteRtEngine.kt` | Inference is a Kotlin LiteRT implementation parallel to the Rust model path — by design; its `enhanceFrame` output does go through the Rust toppings. |
| `CoreToppings.kt` | Applies EWMA/flicker through the Rust core per frame, but the alpha/beta weights are constants rather than `EnhancementSettings` values. |

## 4. Suggested improvements (prioritized, remaining)

### P1 — Make the architecture real

1. **Wire `NativeFrameHandle` (P1.5) into the FFI** (pass pointer + stride + pixel
   format, validate dimensions) **or delete it** until it is used.
2. **Expose the EWMA/blend weights as `EnhancementSettings`** (P1.8) — they are
   `CoreToppings` constants today.

### P3 — Design follow-ups (easiest first)

Nothing here blocks the build.

| # | Item | Effort | Verdict | Notes |
| --- | --- | --- | --- | --- |
| P3.3 | Crate additions | easy | enabler | Add each crate when its paired item lands: `proptest` → P3.8; `miri` as a CI job later; `static_assertions` optional. |
| P3.8 | Property / FFI-fuzz tests | med | done | `core/tests/filters_properties.rs`: EWMA-bounds, blend-convexity, blend-endpoint proptests + golden-frame hash pinning the app defaults (EWMA 0.8 / blend 0.9). FFI fuzzing landed on both boundaries: `core/tests/ffi_fuzz.rs` (C-ABI handles, dims, strides, weights — nulls/dims/params classified, no UB) and `bridge/android/src/frames.rs::ffi_fuzz` (adversarial JVM `Int` tuples against an independent oracle). The bridge fuzz caught the guard accepting a stride that is not a whole number of `f32`, whose requirement rounds down past the allocation — now rejected there as in the core. |

### P4 — iOS platform (deliberately high-level)

No iOS code exists today and nothing here should be pre-implemented until that
platform actually starts. Keep these as one-liners.

| # | Item | Notes |
| --- | --- | --- |
| P4.1 | iOS bridge over the existing C ABI | `bridge/ios/` Swift shim over the core's `cdylib` (Swift `extern`/`@cImport` of a generated interface, or a JNI-style binding tool); core and ABI reused unchanged |
| P4.2 | Media decode/encode | AVFoundation in `app/platforms/ios/` |
| P4.3 | Inference runtime on iOS | Core ML / Metal; LiteRT stays Android-side |
| P4.4 | UI on the shared contracts | SwiftUI screens over the `:shared` contracts (domain, UI state, `SettingsStore`, `LogBuffer`) |

Hard rule: nothing iOS-specific may enter `core/` or `app/shared/` — each host gets
its own bridge crate.

## 5. What's already good

- Clear architectural docs with explicit tenets and data flow.
- Correct layering *intent*: Rust compute + C ABI + platform adapters is the right
  shape for this problem.
- Sensible Rust module layout (folder modules, no `mod.rs`), unit tests per module,
  integration tests, criterion bench, and a clean C ABI with opaque handles.
- Pipeline/topping composition (EWMA as an LLIE topping, not a universal layer) is
  a genuinely good design decision.
- `.gitignore` correctly excludes `target/`, `*.so`, `local.properties`.

### TL;DR

The Rust core is real and the app calls it: P1.1 landed (22 JNI exports, `libbridge.so`
in the APK, EWMA + blend applied per frame) and P1.4 landed (one frame contract in
`core/src/frame.rs`, run by the C ABI and the JVM shim alike). Highest-leverage work
remaining: (1) model validation on real frames (P1.2 follow-ups), (2) promote the
EWMA/blend weights to `EnhancementSettings` (P1.8), (3) wire `NativeFrameHandle`
into the FFI or delete it (P1.5).
