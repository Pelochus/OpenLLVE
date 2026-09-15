# OpenLLVE — Repository Analysis & Suggested Improvements

*Generated from a full read of the repo (Rust core, Android app, docs, CI, build files) plus targeted web verification.*

## 0. Implementation status

Status of the §5 items after the implementation pass:

| Item | Status |
| --- | --- |
| P0.1 `ffi.rs` edition-2024 + CI gate | ✅ done — `#[unsafe(no_mangle)]`, explicit `unsafe {}` blocks, `# Safety:` docs; pre-existing blend test failure fixed (f32 rounding); `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` all green |
| P0.2 Gradle repair | ✅ done — stray root `build.gradle.kts` + fake `gradlew.sh` removed, `include(":core")` dropped, real Gradle wrapper (8.9), plugin pins via `pluginManagement` (AGP 8.5.2, Kotlin 2.0.21), `local.properties` untracked, `gradle.properties` cleaned, manifest fixed (namespace, real `MainActivity`, `res/` with strings/theme/launcher icons), Compose deps added to `app/build.gradle.kts` |
| P0.3 `model-validation.yml` | ✅ resolved — **removed by design**: model validation is external to the app (the app just plugs in a model). Default test model `zero-dce-int8.tflite` (Zero-DCE, INT8, from raspberrypi/AI_enhance, BSD-2-Clause) committed in assets; new models are added as external submodules under `models/` (see `models/README.md`) |
| P1.1–P1.7 | ⬜ remaining — see §5 |
| P1.8 cbindgen header generation | ⬜ proposed — see §5 |
| P2.1 CI | ⚠️ partial — clippy/fmt added to `rust-core.yml`; wrapper makes `android-ci.yml` runnable; ktlint step + release workflow remaining |
| P2.2 tests | ⬜ remaining |
| P2.3 benchmarks | ⬜ remaining |
| P2.4 dependency refresh | ⬜ remaining |
| P2.5 docs | ⚠️ partial — README build commands fixed; `ARCHITECTURE.md` tree reconciled with the actual tree; LiteRT/TFLite naming note remaining |
| P2.6 cargo hygiene | ✅ done — dead `std`/`ffi` features removed |

Also done outside §5: version-controlled pre-commit hook (`.githooks/pre-commit`: `cargo fmt --check` on staged Rust, optional `ktlint`), line endings standardized to LF (industry standard), `gradlew.bat` removed (no Windows support — Linux/macOS/WSL only), and `*.tflite` added to `.gitattributes` as binary.

## 1. What this repo is

**OpenLLVE** (*Open Low-Light Video Enhancement*) is an early proof-of-concept for a real-time, on-device low-light video enhancement engine with a CPU/GPU/NPU benchmark harness. The intended layering is:

| Layer | Status |
| --- | --- |
| `core/` — Rust crate + C FFI (strategies, filters, metrics) | Real code; **compiles and passes CI** (P0.1 done — see §0; §2.1 documents the original failure) |
| `app/shared/` — KMP shared app logic | **Empty** (READMEs only) |
| `app/platforms/android/` — Kotlin/Compose app | Stubs: placeholder engine, no-op UI, no camera wiring, **no FFI calls** |
| `app/platforms/ios/` — iOS | Placeholder README only |

Git history is 4 commits ("Initial project template created with AI") — this is a scaffold, and the analysis below is scoped to that reality.

## 2. Critical: the project cannot build as-is

### 2.1 Rust core fails to compile (CI `rust-core.yml` will be red)

`cargo test` / `cargo build` in `core/` fails with **17 errors + 18 warnings**:

- 17 × `error: unsafe attribute used without unsafe` on every `#[no_mangle]` in `core/src/ffi.rs`.
  In Rust **edition 2024** (RFC 3325, stable since 1.82), `no_mangle` is an *unsafe attribute* and must be written as `#[unsafe(no_mangle)]`.
- 18 × `warning[E0133]` (`unsafe_op_in_unsafe_fn`): raw-pointer derefs, `slice::from_raw_parts(_mut)`, and `Box::from_raw` inside `unsafe extern "C" fn`s need explicit `unsafe { }` blocks in edition 2024.

**Fix:** replace `#[no_mangle]` → `#[unsafe(no_mangle)]` (17 sites in `ffi.rs`), wrap the unsafe ops in `unsafe {}`, then gate CI on `cargo clippy -- -D warnings` + `cargo fmt --check`.

### 2.2 Gradle build is broken in at least five independent ways

1. **`settings.gradle.kts` includes `:core`**, but `core/` is a Rust crate with **no `build.gradle.kts`** → Gradle fails on an included project without a build file.
2. **Root `build.gradle.kts` applies `com.android.application` to the root project** (no Android sources/manifest at root) and duplicates the `:app` module with the same `applicationId` (`com.example.openllve`). It looks like a stray copy of the app build file.
3. **`plugins {}` blocks declare no versions** (`com.android.application`, `kotlin("android")`) and there is no `pluginManagement` in `settings.gradle.kts` → plugin resolution fails.
4. **No Gradle wrapper exists.** The repo has only a hand-written `gradlew.sh` that expects a `gradle/` directory (absent). Standard `gradlew` / `gradlew.bat` are missing, so:
   - the README commands (`gradlew.bat assembleDebug`, `./gradlew.sh assembleDebug`) fail, and
   - CI `android-ci.yml` fails at `chmod +x gradlew`.
5. **`local.properties` is committed** (it's in `git ls-files` even though `.gitignore` lists it). It contains a placeholder `sdk.dir=/path/to/your/sdk`.

Additional build-file issues:

- `gradle.properties` contains non-standard/ineffective properties (`gradle.version=7.4.2`, `org.gradle.scan=true`, `android.compileSdk=31`, …), stale `-XX:MaxPermSize=512m` (removed since Java 8), and `kotlin.version=1.6.10` while the app pins `kotlin-stdlib:1.7.10` — inconsistent and mostly dead config.
- `app/build.gradle.kts` pins old dependency versions (TFLite **2.12.0**, CameraX **1.2.2**, Material **1.10.0**); current TFLite is ~2.17+ and CameraX ~1.4+.
- `AndroidManifest.xml` references resources that don't exist (`@mipmap/ic_launcher`, `@string/app_name`, `@style/Theme.OpenLLVE` — `src/main/res/` is **empty**) and declares `.ui.MainScreen` as an `<activity>`, but `MainScreen` is a `@Composable` function, not an `Activity`. The app cannot compile or launch.

### 2.3 `model-validation.yml` is effectively a no-op job

- No `.tflite` files exist under `assets/models/` (only a README), so the `for model in ...*.tflite` loop iterates **zero times** and the job "passes" silently. It should fail fast when no models are found.
- It uses the legacy Python API `tf.lite.Interpreter` (deprecated; removed in TF 2.13+ in favor of `tf.lite.interpreter`), with no pinned TensorFlow version.
- It calls `model.allocate_tensors()` **before** `model.get_input_details()`, which is the wrong order for the TFLite Python runtime.

## 3. Architecture: stated tenets vs. actual code

The docs are clear and good, but the code contradicts them:

1. **"All business logic must live in the Rust core"** — violated. `LlieEwmaEnhancer.kt` re-implements EWMA + blending in Kotlin (`app/platforms/android/.../domain/LlieEwmaEnhancer.kt`), duplicating `EwmaFilter`/`FrameBlendFilter` from the core. **The C FFI is never called from anywhere in the Android app** — the entire `ffi.rs` surface is dead code today.
2. **"KMP for all shared app logic"** — `app/shared/` contains only READMEs; there is no KMP module, no `shared/build.gradle.kts`, no contracts.
3. **"Zero-copy frame handles"** — `NativeFrameHandle` exists but is **never used by the FFI** (the FFI takes raw `float*` + length) and has no validation (e.g., `stride >= width * channels`), no lifetime/aliasing story for the raw `*mut u8`, and no way to express pixel format.
4. **Strategy model is an identity stub** — `LlieStrategy::process` and `LlveTemporalStrategy::process` just copy the input; no model is invoked. Consequently `benches/inference_bench.rs` measures a memcpy, not inference, and the benchmark harness (warm-up exclusion, thermal tracking) is not wired to any real workload.

## 4. Concrete code-level bugs & smells

| Location | Issue |
| --- | --- |
| `core/src/strategies/llie.rs` `process()` | Blend uses `self.previous_frame` (the **previous** raw frame) as the "raw" input, but the doc comment says "Blends the **raw input frame** with the AI-enhanced frame". Same pattern in `temporal.rs` (`self.state`). Either the semantics or the docs are wrong. |
| `core/src/strategies/llie.rs` `Default` | `Self::new().unwrap()` in a `Default` impl — panics on an infallible path; fine today, but a footgun. |
| `core/src/metrics.rs` | `percentile_p99` on 1 sample returns the sample (ok), but f32 accumulation of many latencies loses precision; the methodology doc promises **median** latency, yet there is no median API. No warm-up exclusion support in the metrics API (docs say warm-up frames must be discarded). |
| `core/src/ffi.rs` | FFI returns only `bool` — no error codes, no ABI version function, no documented thread-safety (stateful strategies must not be shared across threads). C header uses opaque typedefs (`OpenLlveStrategy`) whose names don't match the Rust type (`OpenLlveStrategyEnum`) — harmless but confusing. `openllve_strategy_process` silently accepts `output_len > input_len` and copies `min(...)`. |
| `core/Cargo.toml` | Features are dead: `default = ["std"]`, `ffi = ["std"]`, but nothing is feature-gated — `ffi.rs` always compiles. |
| `LiteRTInferenceEngine.kt` | `loadModelFile()` returns `ByteArray(0)` → `Interpreter(ByteArray(0))` throws at runtime. Not `AutoCloseable`; `VideoPipelineManager` never even instantiates it. |
| `VideoPipelineManager.kt` | `startPipeline()`/`stopPipeline()` are empty comments; "temporal" strategy is `frame.copyOf()`; no engine, no threading, no lifecycle. |
| `SystemMonitor.kt` | `getCpuUsage()` is a hardcoded `0.0f` placeholder; `getMemoryUsage()` uses the deprecated `ActivityManager.getMemoryInfo` path. |
| `MainScreen.kt` | Button is a no-op; no camera preview, no result surface, no state. |
| Naming | "LiteRT" (class name) vs `org.tensorflow:tensorflow-lite` artifacts: Google renamed TensorFlow Lite → **LiteRT** (Sept 2024), but the Maven artifact IDs and APIs are unchanged. Docs/code mix both; pick one and note the artifact ID is still `tensorflow-lite`. |

## 5. Suggested improvements (prioritized)

### P0 — Make the project build (unblocks everything)

1. **Fix `core/src/ffi.rs` for edition 2024**: `#[no_mangle]` → `#[unsafe(no_mangle)]`; wrap unsafe ops in `unsafe {}`. Verify with `cargo test && cargo clippy -- -D warnings && cargo fmt --check`.
2. **Repair the Gradle setup**:
   - Delete the stray root `build.gradle.kts` (root should be an empty container project) **or** keep one app module only.
   - Remove `include(":core")` from `settings.gradle.kts` (Rust is not a Gradle module) — or create a small Gradle module whose job is to cross-compile the cdylib (see P1.1).
   - Add a real Gradle wrapper (`gradlew`, `gradlew.bat`, `gradle/wrapper/`) and pin plugin versions via `pluginManagement` in settings.
   - `git rm --cached local.properties` (keep it local-only).
   - Clean `gradle.properties` (remove bogus properties, drop `-XX:MaxPermSize`).
   - Fix `AndroidManifest.xml`: add the missing `res/` resources (theme, strings, launcher icons) and a real `Activity` that hosts the Compose screen.
3. **Fix `model-validation.yml`**: pin `tensorflow`, use `tf.lite.interpreter`, correct call order, and **fail when no `.tflite` files exist**.

### P1 — Make the architecture real (align code with the tenets)

1. **Wire the Rust core into Android end-to-end** (the single highest-value step):
   - Cross-compile `openllve-core` to Android ABIs (`arm64-v8a`, `x86_64`) via `cargo-ndk` or a CI matrix job, packaging the `cdylib` as `libopenllve_core.so`.
   - Load it with `System.loadLibrary("openllve_core")` and call `openllve_*` from Kotlin via `external` functions.
   - **Delete `LlieEwmaEnhancer.kt`** — the Kotlin re-implementation of EWMA/blend.
2. **Integrate a real model** so the benchmark measures something: load an actual low-light model (the docs already name **Zero-DCE** and **MBLLEN**) from assets via LiteRT, map input/output tensors, and expose delegate selection (CPU / GPU / NNAPI) as the benchmark axis.
3. **Implement the KMP shared layer** (even minimally): benchmark definitions, UI state contracts, and the strategy/topping configuration model — or soften tenet #1 in the docs until it exists.
4. **Fix blend semantics** in `llie.rs`/`temporal.rs`: blend with the *current* raw frame (matching the documented intent) or rename/document the previous-frame behavior.
5. **Either wire `NativeFrameHandle` into the FFI** (pass pointer + stride + pixel format, validate dimensions) **or delete it** until it's used.
6. **Metrics API**: add `median()` (promised by `BENCHMARK_METHODOLOGY.md`), consider `f64` accumulation, and add warm-up-frame exclusion support.
7. **FFI hardening**: return error codes (not just `bool`), add an ABI version function, document single-threaded ownership of strategy handles, and match C typedef names to Rust type names.
8. **Generate the C header with cbindgen**: `cbindgen` emits `openllve_core.h` from the `#[unsafe(no_mangle)] extern "C"` functions in `ffi.rs`, replacing the hand-maintained header and keeping it in sync. Setup: `cargo install cbindgen` (verify the version parses the edition-2024 `#[unsafe(no_mangle)]` syntax — 0.26+), add `[package.metadata.cbindgen]` to `core/Cargo.toml`, run `cbindgen core -h include/openllve_core.h` (wire into CI or the pre-commit hook). Note: cbindgen only generates the *header* — `ffi.rs` itself stays hand-written, and the Kotlin side still needs hand-written `external fun`s (or JNA).

### P2 — Quality, tests, and process

1. **CI**: add `cargo clippy`/`fmt` to `rust-core.yml`; fix `android-ci.yml` (wrapper first); add a Kotlin lint step (ktlint); add a release workflow.
2. **Tests**: FFI round-trip tests (Kotlin JNI or a small C test harness), metrics edge cases (empty, 1 sample, p99 with ties), and a test that the blend output actually differs from input when `beta ∈ (0,1)`.
3. **Benchmarks**: once a real model is in, benchmark the *model path* (not memcpy), keep warm-up exclusion, and record device/temperature/battery metadata per run as the methodology doc requires.
4. **Dependency refresh**: TFLite 2.12.0 → current, CameraX 1.2.2 → current, Material → current; keep `minSdk`/`targetSdk` consistent (root says 33, app says 34).
5. **Docs**: fix README build commands (only `gradlew.sh` exists today), reconcile the repo-layout tree in `ARCHITECTURE.md` with the actual tree (it lists `gradlew`, `gradlew.bat`, `app/build.gradle.kts` under `app/`), and standardize LiteRT vs TFLite naming.
6. **Cargo hygiene**: drop the dead `std`/`ffi` features or actually gate `ffi.rs` behind the `ffi` feature; keep `Cargo.lock` committed (fine for reproducible builds).

## 6. What's already good

- Clear, consistent architectural docs (`ARCHITECTURE.md`, `BENCHMARK_METHODOLOGY.md`, per-module READMEs) with explicit tenets and data flow.
- Correct layering *intent*: Rust compute + C ABI + platform adapters is the right shape for this problem.
- Sensible Rust module layout (folder modules, no `mod.rs`), unit tests per module, integration test, criterion bench, and a clean C header with opaque handles.
- Strategy/topping composition (EWMA as an LLIE topping, not a universal layer) is a genuinely good design decision.
- `.gitignore` correctly excludes `target/`, `*.so`, `local.properties` (the file just needs to be untracked).

---

### TL;DR

The repo is a well-documented scaffold whose **build was broken in three independent places** (Rust edition-2024 FFI errors, a broken Gradle setup with no wrapper, and a no-op model-validation job) — the first two are fixed and the third was **removed by design** in favor of a committed default test model (`zero-dce-int8.tflite`) plus external model submodules under `models/`. The **core promise — Kotlin calling the Rust core — is still not implemented** (the FFI is dead code and the same filter logic is duplicated in Kotlin). The highest-leverage work remaining is: (1) cross-compile the cdylib and call it from Android, (2) wire the LiteRT model into the pipeline so the benchmark measures inference instead of a memcpy, (3) align the remaining P1 items (blend semantics, metrics, FFI hardening, cbindgen).
