# OpenLLVE — TODO

Work **one task at a time**, in order. After each: mark it ✅, note what
changed, add follow-ups, and keep the `IMPROVEMENTS.md` §0 status table in sync.
Commit and style conventions live in `AGENTS.md`.

## Read first (context)

1. `AGENTS.md` — conventions, tenets, build/test gates, known constraints.
2. `IMPROVEMENTS.md` §0 — status of every item below (source of truth).
3. `docs/README.md` — documentation index; `docs/dev/architecture/ARCHITECTURE.md` — layering tenets, repo layout, data flow.
4. `docs/dev/app-fixes.md` — app fixes already applied + vendor/platform dead ends (**do not re-attempt**).

## Current state (2026-10-11)

- **Rust core**: 48 unit + 3 integration tests green, clippy/fmt clean; FFI hardened (error codes, `openllve_abi_version`), metrics with warm-up, blend semantics fixed.
- **Bridge**: `bridge/android/` crate (P1.1) builds host-side and cross-compiles via `scripts/build-bridge.sh`; `rust-core.yml` gates it with the same three commands as `core/`.
- **Models**: single source of truth in `external/models/` (default `zero-dce-int8.tflite` as a raw file, new models as submodules); app assets symlink to it.
- **Android app**: functional vertical slice (Compose UI, LiteRT engine, MediaCodec decode, DataStore settings) over the `:shared` KMP contracts; EWMA + blend now run in the Rust core per frame.
- **CI**: `android-ci.yml` (ktlint, androidLint, test, assemble — ktlint step red, P2.5), `rust-core.yml` (test, clippy, fmt), `release.yml` (on `v*` tags), `docker-image.yml` (push to `dev`/`main` → GHCR `ghcr.io/pelochus/openllve-dev:latest`).

## Do next (in order)

1. [~] **P1.2 follow-ups — model validation on real frames.**
   - Verify the pipeline on real (non-synthetic) frames; re-confirm the global-mean
     brightness channel against upstream's 4×4 bilinear downscale.
   - ✅ Closed 2026-10-10: the `ResizeInputTensor` "segfault" was call-order misuse,
     not a library bug — `tflite-c-rs` resizes eagerly in `Interpreter::new`, so
     resize must precede allocate (`external/models/README.md`). Fixed-shape model
     file stays; the crate has no resize API.
2. [ ] **P3.8 — Property / FFI-fuzz tests.**
   - `proptest`: EWMA output stays within `[min(prev,curr), max(prev,curr)]` per
     pixel; blend output is a convex combination. Add golden-frame hashes and
     FFI fuzzing against the bridge crate (P1.1 landed, so it is the FFI target).
3. [x] **P1.3 — KMP shared layer.** `:shared` module holds the platform-neutral
   contracts (domain, `FrameImage`/`FramePixels`/`VideoMetadata`, `UiState`,
   `SettingsStore` persistence contract); Android consumes it through thin
   adapters. P3.6 (run record + persistence) rejected as overkill.
4. [x] **P2.4 — Dependency refresh** in `app/build.gradle.kts`: newest toolchain
   (AGP 9.4.0, KGP 2.4.20, Gradle 9.7.1, compileSdk 37, Compose UI 1.12.1 via BOM
   2026.09.00, LiteRT 2.3.0 + `litert-gpu`); `targetSdk` stays 34.
5. [x] **P1.1 — Android native (Rust) wiring** — landed 2026-10-11.
   - Direct JNI, no C++ layer (`docs/dev/architecture/FFI-WIRING.md`): `bridge/android/`
     crate (`jni` 0.22.4 + `jni-macros`) exports 22
     `Java_openllve_android_core_CoreBridge_*` symbols over the existing `core/src/ffi.rs`
     C ABI; `CoreBridge.kt` + `CoreToppings.kt` apply EWMA/blend per frame;
     `scripts/build-bridge.sh` installs `jniLibs/arm64-v8a/libbridge.so`.
   - Full landing record (symbols, sizes, load-library mechanics, gotchas):
     `docs/dev/architecture/FFI-WIRING.md` §4 + `bridge/android/README.md`.
   - Follow-ups are items 7–9 below; pipeline constructors and `process_frame` are
     exported but unused by the app (inference stays Kotlin-side by decision).
6. [ ] **P1.4 — Move frame validation into the core (every host bridge reuses it).**
   - `bridge/android/src/frames.rs` holds platform-neutral checks (positive dims,
     `stride >= width*channels*4`, buffer element count vs `stride*height/4`) that only
     the JVM gets today; `ffi.rs` builds slices with `slice::from_raw_parts(ptr, stride*height/4)`
     from caller numbers, so a C or iOS consumer of `core/include/openllve_core.h` is unguarded.
   - Do: expose the check in the core (declare it in the header beside the frame
     functions, return the same `OpenLlveError` codes), have both bridges call it, and
     keep in the shim only host facts (JVM array element count, signed `jint`).
7. [ ] **P1.7 — Build the `x86_64` bridge ABI.** `scripts/build-bridge.sh` defaults to
   arm64-v8a, so an x86_64 debug run (emulator / PC tester) cannot load `libbridge.so`.
   Still ship exactly one vendor dispatch runtime per APK (AGENTS.md §3).
8. [ ] **P1.8 — Promote EWMA/blend weights to `EnhancementSettings`.** `CoreToppings`
   keeps `EWMA_ALPHA = 0.5f` / `BLEND_BETA = 0.7f` as constants while the UI already
   exposes `ewmaEnabled` / `flickerReductionEnabled`; the weights are the missing half.
9. [ ] **P2.5 — ktlint debt (31 violations) makes `android-ci.yml` red.**
   `function-signature` 20, `blank-line-before-declaration` 3, `class-signature` 2,
   `import-ordering` 2, `spacing-between-declarations-with-comments` 2,
   `string-template` 2 across `app/shared/src` + `app/platforms/android/src`;
   `ktlint --format` fixes most (ktlint 1.8.0 is pinned in the dev image).
10. [ ] **P3.9 — NPU: additional vendor dispatch runtimes (long-term).**
    - Beyond Google Tensor: Qualcomm, then MediaTek if prebuilt libs appear. The newest
      LiteRT release that ships NPU runtime libraries at all is the **2.2.0** zip (2.3.0
      ships none); 2.3.0 added Samsung (Exynos AI LiteCore) and Intel (OpenVINO) support
      without prebuilt libraries.
    - Constraint: LiteRT dlopens the *first* `libLiteRtDispatch*` in the dispatch dir, so
      bundle exactly **one vendor runtime per APK** — Gradle product flavors (per-vendor
      `jniLibs`) or an AAB with Play Feature Delivery.
    - Per-vendor `NpuCompatibilityChecker` + the `isLibraryReady()` existence check;
      dispatch shims need the vendor runtime on-device (Tensor `libedgetpu_litert.so`,
      Qualcomm QAIRT). arm64-v8a only; JIT is Beta per vendor, AOT is the supported path.
11. [ ] **P4 — iOS platform (high-level only; do not pre-write iOS code).**
    - P4.1 `bridge/ios/` Swift shim `@cImport`ing `core/include/openllve_core.h` (core +
      ABI reused unchanged); P4.2 AVFoundation media; P4.3 Core ML / Metal inference
      (LiteRT stays Android-side); P4.4 SwiftUI on the `:shared` contracts.
    - Hard rule: nothing iOS-specific enters `core/` or `app/shared/` — one bridge crate
      per host (`docs/dev/architecture/ARCHITECTURE.md` §3).

## Recently completed (removed from the list)

- **P3.1 — Model manifest**: rejected as overkill (nothing consumes it).
- **P3.6 — `BenchmarkRun` record + persistence**: rejected; runs live in
  `docs/benchmarks/RESULTS.md`.
- **P2.3 — Benchmarks**: criterion bench covers the model path with warm-up exclusion.
- **GHCR dev image**: `docker-image.yml` builds `docker/Dockerfile` on push and pushes
  `ghcr.io/pelochus/openllve-dev`; `docker/run.sh --skip-build` pulls it.
- **P2.1 — CI**: ktlint step in `android-ci.yml` + `release.yml`.
- **P3.2 — Threading model**: decision in `docs/dev/architecture/ARCHITECTURE.md` §12;
  docs restructured into `docs/dev/` + `docs/benchmarks/` with a `docs/README.md` index.
- **P2.5 (docs round) — LiteRT naming** standardized across docs and comments.

## Notes

- Review `docs/dev/architecture/DESIGN_SUGGESTIONS.md` where it is cheap; defer the
  hard/complex suggestions.
- The `android-ci.yml` ktlint gate is red because of P2.5 — Kotlin changes cannot be
  validated green until that debt is cleared.
