# AGENTS.md — agent guidance for OpenLLVE

Guidance for AI agents (and humans) working in this repository. Read this
before making changes. Keep it updated when conventions change.

## 1. What to read first

- `README.md` — overview + build commands
- `docs/dev/architecture/ARCHITECTURE.md` — layering tenets and data flow (authoritative)
- `TODO.md` — current task list, work one item at a time, in order
- `IMPROVEMENTS.md` — §0 status table is the source of truth for feature status
- `docs/dev/app-fixes.md` — app fixes already applied and what is NOT fixable in the app (vendor gates, missing hardware) — do not re-attempt those items
- `docs/dev/architecture/FFI-WIRING.md` — decision record for Rust↔Kotlin wiring (direct JNI, no C++ layer)

## 2. Code style — hard rules

- **Self-documenting code first.** Prefer descriptive names, small typed
  functions, and clear structure over explanatory comments.
- **Minimal inline comments.** Comment only where the "why" cannot be read
  from the code (non-obvious workarounds, unsafe-API contracts, vendor
  quirks). One short line max; no narrated blocks.
- **Docs are the right place for explanation.** Module READMEs
  (`core/README.md`, `core/src/filters/README.md`, …) and `docs/` carry the
  narrative. When a comment would explain design, move it to a doc and let
  the code reference the doc by path.
- **Doc comments stay** where they are load-bearing: public Rust API / C ABI
  (`ffi.rs` — contracts for handles, lifetimes, error codes are consumed by
  the C header and Kotlin `external fun`s), and exported Kotlin shared
  contracts in `app/shared/`.
- No commented-out code. Delete it or keep it.
- Follow `.editorconfig` (4-space indent, LF, UTF-8, trailing newline) and
  `.rustfmt.toml` (`edition = 2024`, `max_width = 120`). Run `cargo fmt`
  and respect ktlint (CI enforces both).

## 3. Architecture tenets (do not violate)

- Business logic and compute live in the **Rust core** (`core/`). If it can
  run under `cargo test` on a PC, it belongs there.
- Shared app logic lives in the KMP `:shared` module (`app/shared/`).
- Platform code (`app/platforms/android/`) is thin glue/adapters only — SDK
  types, UI, MediaCodec, LiteRT engine.
- FFI path is direct JNI: Rust `cdylib` → C ABI → Kotlin `external fun`s.
  **No C++ layer.** The JVM shim is its own crate (`bridge/android/`);
  `core/src/ffi.rs` stays the single C ABI contract and the shim calls the
  core's Rust API through a path dependency — it is **not** a second ABI.
- Bundle exactly **one** LiteRT vendor dispatch runtime per APK in
  `jniLibs/` (LiteRT dlopens the first `libLiteRtDispatch*` found).

### Where the code lives

The Kotlin source roots nest to their package directory (`app/shared/…/openllve/shared/`, `app/platforms/android/src/main/java/openllve/android/`), so agents should navigate by this list instead of walking the tree.

`core/src/` (Rust — the one C ABI):

| Path | Role |
| --- | --- |
| `ffi.rs` | the 22 `pub unsafe extern "C"` exports + `OpenLlveError`/`OpenLlvePipeline` contracts |
| `error.rs` | `OpenLlveError` codes (`Ok=0 NullPointer=1 InvalidParameter=2 BufferDimensionMismatch=3 Internal=4`) |
| `frame.rs` | `FrameRef`/`Frame` pixel-buffer views + `NativeFrameHandle` (unused — P1 item 2) |
| `filters.rs` · `filters/{blend,ewma}.rs` | `FrameBlendFilter`, `EwmaFilter` |
| `pipelines.rs` · `pipelines/{llie,temporal}.rs` | `OpenLlvePipeline`, LLIE curve, `TemporalPipeline` |
| `model.rs` | model/LLIE inference types |
| `metrics.rs` | `BenchmarkMetrics` (warm-up budget, median/p95/p99/FPS) |
| `lib.rs` · `tests/` · `benches/` | module wiring, `cargo test` gates |

`bridge/android/src/` (JNI shim — JVM marshalling only):

| Path | Role |
| --- | --- |
| `bridge.rs` | 22 `Java_openllve_android_core_CoreBridge_*` exports |
| `frames.rs` | JVM frame-buffer guards run before pointers cross |
| `lib.rs` | module wiring + ABI probe |

`app/shared/src/commonMain/kotlin/openllve/shared/` (KMP — no platform API):

| Path | Role |
| --- | --- |
| `domain/` | `BackendSelection`, `ComputeTarget`, `EnhancementEngine`, `EnhancementSettings`, `MediaInput`, `ProcessingMetrics`, `SettingsStore` |
| `log/` | `LogBuffer` ring buffer, `LogEntry`, `LogLevel`, `LogLock` (+ `androidMain/`, `iosMain/` mirrors) |
| `media/` | `FrameImage`, `FramePixels`, `VideoMetadata` |
| `ui/` | `UiState` |

`app/platforms/android/src/main/java/openllve/android/` (thin host):

| Path | Role |
| --- | --- |
| `core/` | `CoreBridge` (`external fun`s), `CoreToppings` (filter handles per session) |
| `engine/` | `AndroidLiteRtEngine` (LiteRT inference, `enhanceFrame`) |
| `media/` | `VideoFrameProvider`, `ImageFrameProvider`, `BitmapFrameAdapter`, `VideoMetadataReader` |
| `ui/` | `MainActivity`, `OpenLLVEApp`, `screens/`, `components/`, `viewmodel/`, `navigation/`, `theme/` |
| `data/` · `log/` | DataStore settings (`SettingsRepository`), logcat mirror (`AppLog`) |

`app/platforms/android/src/main/` non-code: `AndroidManifest.xml`, `res/` (icons,
strings, themes), `assets/models/` (`zero-dce-int8.tflite`),
`jniLibs/arm64-v8a/` (`libbridge.so` + LiteRT NPU dispatch libs), `src/test/`.

## 4. Build & test

- Rust: `cargo test --manifest-path core/Cargo.toml`,
  `cargo clippy … -- -D warnings`, `cargo fmt --check` (all three are CI gates).
- Android: `./gradlew assembleDebug` / `testDebugUnitTest` — needs JDK 17 +
  Android SDK. The host has a JDK but no Android SDK, so Kotlin builds run in
  the dev container (`docker/run.sh`, see `docs/dev/docker-dev-env.md`); CI
  (`.github/workflows/android-ci.yml`) still gates Kotlin changes. Make Kotlin
  changes conservative and ktlint-clean.
- Rust cross-compile for Android: `cargo-ndk` is installed locally; the JNI
  shim is packaged into the APK by `scripts/build-bridge.sh` (run it in the
  dev container, where the NDK lives) — see `bridge/android/README.md`.
  `rust-core.yml` gates `core/` and `bridge/android/` with the same three
  commands.

## 5. Commit discipline

- One small, scoped commit per task; commit message states task + what
  changed.
- After completing a task: mark it ✅ in `TODO.md`, sync the
  `IMPROVEMENTS.md` §0 table, add follow-ups if any.
- Never mark done with failing tests, partial implementation, or
  unresolved errors.

## 6. Known constraints & gotchas

- `tflite-c-rs` resizes eagerly: `ResizeInputTensor` after `AllocateTensors`
  NULLs the input data pointer — order must be resize → allocate → invoke
  (see `external/models/README.md`).
- NPU items gated by vendor/OS (BP2A builds, non-arm64-v8a, missing
  dispatch libs) are **not app bugs** — see `docs/dev/app-fixes.md` before
  "fixing" them.
- EWMA/flicker toggles are real settings and their computation now runs in
  the Rust core through the JNI bridge (P1.1); the weights are `CoreToppings`
  constants until P1.8 promotes them to `EnhancementSettings`.
- No Android device hardware available: end-to-end NPU/GPU validation
  cannot be done here; never claim device-verified results.
