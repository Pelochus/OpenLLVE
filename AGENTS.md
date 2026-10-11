# AGENTS.md — agent guidance for OpenLLVE

Guidance for AI agents (and humans) working in this repository. Read this
before making changes. Keep it updated when conventions change.

## 1. What to read first

- `README.md` — overview + build commands
- `docs/dev/architecture/ARCHITECTURE.md` — layering tenets and data flow (authoritative)
- `TODO.md` — current task list, work one item at a time, in order
- `IMPROVEMENTS.md` — §0 status table is the source of truth for feature status
- `TODO-APPFIXES.md` — app fixes applied last round and what is NOT fixable in the app (vendor gates, missing hardware) — do not re-attempt those items
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
  dispatch libs) are **not app bugs** — see `TODO-APPFIXES.md` before
  "fixing" them.
- EWMA/flicker toggles are real settings but their computation belongs to
  the Rust core; they only become effective once P1.1 (FFI wiring) lands.
- No Android device hardware available: end-to-end NPU/GPU validation
  cannot be done here; never claim device-verified results.
