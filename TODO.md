# OpenLLVE — TODO (next agent session)

Work **one task at a time**, in order. After each: mark it ✅, note what
changed, add follow-ups if needed, and keep the `IMPROVEMENTS.md` §0 status
table in sync.

## Read first (context)

1. `IMPROVEMENTS.md` — remaining work + §0 status table (source of truth).
2. `README.md` — overview + build commands.
3. `docs/dev/architecture/ARCHITECTURE.md` — layering tenets, repo layout, data flow, inference placement.
4. `core/README.md` — Rust core modules.
5. `external/models/README.md` — model convention (default model + submodules).

## Current state

- **Rust core**: compiles; all tests green (48 unit + 3 integration);
  clippy/fmt clean. FFI hardened (error codes, `openllve_abi_version`),
  metrics (median, f64, warm-up), blend semantics fixed,
  `NativeFrameHandle` validated.
- **Models**: single source of truth in `external/models/` (default
  `zero-dce-int8.tflite` as a raw file; new models as submodules); app
  assets hold symlinks to it.
- **Benchmarks**: the criterion bench (`core/benches/inference_bench.rs`)
  covers the model path with warm-up exclusion; minimal methodology +
  results table in `docs/benchmarks/`.
- **Android app**: functional vertical slice (Compose UI, LiteRT engine,
  MediaCodec decode, DataStore settings); domain/UI-state/media contracts
  live in the `:shared` KMP module. **No JNI/Rust wiring** (re-attempt in
  P1.1).
- **CI**: `android-ci.yml` (ktlint, androidLint, test, assemble),
  `rust-core.yml` (test, clippy, fmt), `release.yml` (on `v*` tags:
  test, release APK, GitHub Release), and `docker-image.yml` (on push to
  `dev`/`main`: build the dev image and push it to GHCR — pull with
  `podman pull ghcr.io/pelochus/openllve-dev:latest`).

## Do next (in order)

1. [ ] **P1.2 follow-ups — model validation.**
   - Verify the pipeline on real (non-synthetic) frames; re-confirm the
     global-mean brightness channel vs upstream's 4×4 bilinear downscale.
   - Re-export the model with the correct static input shape, or root-cause
     the `TfLiteInterpreterResizeInputTensor` segfault (see
     `external/models/README.md`).
2. [ ] **P3.8 — Property tests for filters.**
   - Add `proptest`; assert EWMA output stays within
     `[min(prev,curr), max(prev,curr)]` per pixel and blend output is a convex
     combination. (Golden-frame and FFI-fuzz tests wait for P1.1.)
3. [~] **P1.3 — KMP shared layer.**
   - Done: `:shared` KMP module with the platform-neutral contracts from the
     Android vertical slice (domain, `FrameImage`/`FramePixels`/`VideoMetadata`,
     `UiState`, `SettingsStore` persistence contract — Android's
     `SettingsRepository` implements it); Android consumes it via thin
     adapters.
   - Remaining: none (P3.6 rejected — see Recently completed).
4. [~] **P2.4 — Dependency refresh** in `app/build.gradle.kts`: ML runtime
   done (LiteRT 2.2.0 `CompiledModel`) and toolchain bump done (AGP 9.4.0,
   KGP 2.4.20, Compose BOM 2026.06.01, Gradle 9.7.1, compileSdk 36).
   Remaining: the compileSdk 37 lines
   (lifecycle 2.11.0, Compose UI 1.12.x, navigation 2.10.x, core 1.19.x)
   once android-37 is published.
5. [~] **P1.1 — Android native (Rust) wiring**.
   - Investigation done: `docs/dev/architecture/FFI-WIRING.md` — direct JNI (Rust
     cdylib → C ABI → Kotlin `external fun`s), **no C++ layer**; Rust-side
     LiteRT runner stays a non-core optional feature (PC-side testing),
     app production inference stays on the Kotlin-side LiteRT engine.
   - Remaining: cross-compile cdylib (cargo-ndk), Kotlin `external fun`s,
     package `.so` into the APK.

## Recently completed (removed from the list)

- **P3.1 — Model manifest**: rejected as overkill (model variations are
  rare; nothing consumes the manifest) — removed.
- **P3.6 — `BenchmarkRun` record + persistence**: rejected as overkill —
  runs are recorded in a simple `docs/benchmarks/RESULTS.md` table instead.
- **P2.3 — Benchmarks**: the criterion bench benchmarks the model path with
  warm-up exclusion; minimal methodology + results table in
  `docs/benchmarks/`.
- **GHCR dev image**: `docker-image.yml` builds `docker/Dockerfile` on push
  to `dev`/`main` and pushes it to
  `ghcr.io/pelochus/openllve-dev` (build runs on the GitHub runner — no local
  Docker daemon needed); `docker/run.sh --skip-build` runs the pulled image.
- **P2.1 — CI**: ktlint step in `android-ci.yml` + `release.yml` workflow.
- **P3.2 — Threading model**: decision in `docs/dev/architecture/ARCHITECTURE.md` §12;
  docs restructured into `docs/dev/` + `docs/benchmarks/` with a
  `docs/index.md` entry point; minimal `CONTRIBUTING.md` added.
- **P2.5 — Docs**: LiteRT naming standardized across docs and comments.

## Notes

- Don't build a complex CI pipeline until the app is functional.
- Keep commits small and scoped (one task per commit).
- Review `docs/dev/architecture/DESIGN_SUGGESTIONS.md` for further improvements where
  applicable (or defer them if not recommended or too hard/complex).
