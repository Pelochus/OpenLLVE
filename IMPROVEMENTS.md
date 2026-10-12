# OpenLLVE — Repository Analysis & Suggested Improvements

*Generated from a full read of the repo (Rust core, Android app, docs, CI, build files) plus targeted web verification. Completed items and rejected/deferred items have been removed — this now tracks what remains.*

## 0. Implementation status

Status of the remaining §4 items (finished work is not listed — see `TODO.md`'s
"Recently completed" and `docs/dev/architecture/FFI-WIRING.md`):

| Item | Status |
| --- | --- |
| P1.5 `NativeFrameHandle` | ⚠️ partial — validated (`new_checked` returns `Result`, rejects null ptr / `stride < width`), lifetime + pixel format documented; still **unused**: the bridge passes raw pointers + dimensions and guards them in `bridge/android/src/frames.rs` instead |
| P3.3 crate additions | ⚠️ partial — `bytemuck` + `proptest` added (`serde` no longer needed — P3.6 rejected) |
| P1.4 frame validation shared across host bridges | ⚠️ open — the checks live in `bridge/android/src/frames.rs`, so only the JVM is guarded; move them into the core |
| P4.1 iOS platform (high-level) | ⚠️ open — iOS is a separate host layer, tracked at a high level only (§4 P4); the C ABI in `ffi.rs` is already complete for it |
| P1.8 EWMA/blend weights as settings | ⚠️ open — `CoreToppings` constants `EWMA_ALPHA = 0.7f`, `BLEND_BETA = 0.9f` (re-tuned: mild flicker damping without visible ghosting; 90 % AI-enhanced) |

## 1. What this repo is

**OpenLLVE** (*Open Low-Light Video Enhancement*) is an early proof-of-concept for a real-time, on-device low-light video enhancement engine with a CPU/GPU/NPU benchmark harness. The intended layering is:

| Layer | Status |
| --- | --- |
| `core/` — Rust crate + C FFI (pipelines, filters, metrics) | Real code; **compiles and passes CI** |
| `app/shared/` — KMP shared app logic | Real module (`:shared`): domain/UI-state/media contracts |
| `app/platforms/android/` — Kotlin/Compose app | Functional vertical slice (LiteRT engine, media decode, Compose UI); calls the Rust core through JNI for the temporal toppings |
| `app/platforms/ios/` — iOS | README placeholder only; tracked as P4 (high-level) |

## 2. Architecture: stated tenets vs. actual code

The docs are clear and good; the code contradicts them in two places:

1. **"All business logic must live in the Rust core"** — the temporal toppings do run there through the JNI shim (P1.1); model inference deliberately stays on the Kotlin LiteRT path (`docs/dev/architecture/FFI-WIRING.md` §3), so the core's `process_frame`/pipeline constructors are exported but unused by the app.
2. **"Platform-neutral compute belongs in `core/`"** — the frame geometry/buffer checks run under `cargo test` on a PC yet sit in the Android shim (`bridge/android/src/frames.rs`), so no other host is guarded (P1.4).
3. **"KMP for all shared app logic"** — satisfied: `app/shared/` is a real KMP module (`:shared`) with the domain, UI-state, and media contracts plus the platform-neutral `SettingsStore` persistence contract.
4. **"Zero-copy frame handles"** — `NativeFrameHandle` exists but is unused by the FFI (raw `float*` + dimensions are passed) and has no lifetime/aliasing story for the raw `*mut u8` (P1.5).

## 3. Concrete code-level bugs & smells

The earlier round's stale entries (`VideoPipelineManager.kt`, `SystemMonitor.kt`,
`MainScreen.kt`) no longer exist in the tree — the app was rewritten around
`MainActivity` + `EnhancementViewModel` + the result screens. What remains:

| Location | Issue |
| --- | --- |
| `core/src/ffi.rs` | C ABI is consumed by the `bridge/android` JNI shim; the pipeline constructors and `process_frame` are exported but unused by the app, which keeps its Kotlin inference path. |
| `AndroidLiteRtEngine.kt` | Inference is a Kotlin LiteRT implementation parallel to the Rust model path — by design (`FFI-WIRING.md` §3); its `enhanceFrame` output does go through the Rust toppings. |
| `CoreToppings.kt` | Applies the EWMA/flicker toppings through the Rust core per frame; the alpha/beta weights are still constants rather than `EnhancementSettings` values. |

## 4. Suggested improvements (prioritized, remaining)

### P1 — Make the architecture real (align code with the tenets)

1. **Wire `NativeFrameHandle` (P1.5) into the FFI** (pass pointer + stride + pixel format,
   validate dimensions) **or delete it** until it is used.
2. **Move frame validation into the core** (P1.4): the geometry/buffer checks today live in
   the Android shim, so a C or iOS consumer of the core's C ABI gets no guard
   while `ffi.rs` builds slices from caller-supplied numbers.
3. **Expose the EWMA/blend weights as `EnhancementSettings`** (P1.8) — they are `CoreToppings`
   constants today.

### P3 — Design follow-ups from `DESIGN_SUGGESTIONS.md`

Ranked easiest/most-recommended → hardest/least-useful right now. Nothing here
blocks the build.

| # | Item (`DESIGN_SUGGESTIONS.md` §) | Effort | Verdict | Notes |
| --- | --- | --- | --- | --- |
| P3.3 | Crate additions (§2) | easy | enabler | Add each crate when its paired item lands: `proptest` → P3.8; `miri` as a CI job later; `static_assertions` optional. (`serde` no longer needed — P3.6 rejected.) |
| P3.8 | Property / FFI-fuzz tests (§1) | med | partial | Done in `core/tests/filters_properties.rs`: EWMA-bounds, blend-convexity, blend-endpoint proptests + golden-frame hash pinning the app defaults (EWMA 0.7 / blend 0.9). Remaining: FFI fuzzing against the bridge crate. |

### P4 — iOS platform (deliberately high-level)

iOS is a separate host: no iOS code exists today (`app/platforms/ios/` is a
README placeholder) and nothing here should be pre-implemented until someone
actually starts that platform. Keep these as one-liners.

| # | Item | Notes |
| --- | --- | --- |
| P4.1 | iOS bridge over the existing C ABI | `bridge/ios/` Swift shim over the core's `cdylib` (Swift `extern`/`@cImport` of a generated interface, or a JNI-style binding tool); the core and its ABI are reused unchanged |
| P4.2 | Media decode/encode | AVFoundation in `app/platforms/ios/` |
| P4.3 | Inference runtime on iOS | Core ML / Metal; LiteRT stays Android-side |
| P4.4 | UI on the shared contracts | SwiftUI screens over the `:shared` KMP contracts (domain, UI state, `SettingsStore`, `LogBuffer`) |

Hard rule: nothing iOS-specific may enter `core/` or `app/shared/` — each host
gets its own bridge crate (`docs/dev/architecture/ARCHITECTURE.md` §3).

## 5. What's already good

- Clear, consistent architectural docs (`ARCHITECTURE.md`, `docs/benchmarks/`, per-module READMEs) with explicit tenets and data flow.
- Correct layering *intent*: Rust compute + C ABI + platform adapters is the right shape for this problem.
- Sensible Rust module layout (folder modules, no `mod.rs`), unit tests per module, integration test, criterion bench, and a clean C ABI with opaque handles.
- Pipeline/topping composition (EWMA as an LLIE topping, not a universal layer) is a genuinely good design decision.
- `.gitignore` correctly excludes `target/`, `*.so`, `local.properties`.

---

### TL;DR

The Rust core is real and the app now calls it: P1.1 landed (22 JNI exports, `libbridge.so` in the APK, EWMA + blend applied per frame). Highest-leverage work remaining: (1) move frame validation into the core (P1.4 — only the JVM is guarded today), (2) property/FFI-fuzz tests (P3.8, unblocked).
