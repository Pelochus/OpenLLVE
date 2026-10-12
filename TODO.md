# OpenLLVE — TODO

Work **one task at a time**, in order. When a task finishes, remove it from this
list and drop its row from the `IMPROVEMENTS.md` §0 status table; record the
result where it belongs (module README or `docs/`). Commit and style conventions
live in `AGENTS.md`.

## Read first (context)

1. `AGENTS.md` — conventions, tenets, build/test gates, known constraints.
2. `IMPROVEMENTS.md` §0 — status of every item below (source of truth).
3. `docs/dev/architecture/ARCHITECTURE.md` — layering tenets, repo layout, data flow;
   `docs/dev/app-fixes.md` — fixes already applied and vendor/platform dead ends
   (**do not re-attempt**).

## Current state (2026-10-12)

- **Rust core**: 50 unit + 8 property + 4 FFI-fuzz + 3 filter-property tests green,
  clippy/fmt clean; FFI hardened (error codes, `openllve_abi_version`), the frame
  contract lives here and runs before any caller memory is sized.
- **Bridge**: `bridge/android/` (P1.1) builds host-side and cross-compiles via
  `scripts/build-bridge.sh`; `rust-core.yml` gates it with the same three commands
  as `core/`.
- **Models**: single source of truth in `external/models/` (default `zero-dce-int8.tflite`
  as a raw file, new models as submodules); app assets symlink to it.
- **Android app**: functional vertical slice (Compose UI, LiteRT engine, MediaCodec
  decode, DataStore settings) over the `:shared` KMP contracts; EWMA + blend run in
  the Rust core per frame.
- **CI**: `android-ci.yml` (ktlint, androidLint, test, assemble), `rust-core.yml`
  (test, clippy, fmt), `release.yml` (on `v*` tags), `docker-image.yml` (push to
  `dev`/`main` → GHCR `ghcr.io/pelochus/openllve-dev:latest`).

## Do next (in order)

1. [~] **P1.2 follow-ups — model validation on real frames.**
   - Verify the pipeline on real (non-synthetic) frames; re-confirm the global-mean
     brightness channel against upstream's 4×4 bilinear downscale.
2. [ ] **P1.8 — Promote EWMA/blend weights to `EnhancementSettings`.** `CoreToppings`
   keeps `EWMA_ALPHA = 0.8f` / `BLEND_BETA = 0.9f` (defaults tuned for video: alpha
   weights the current frame — no ghosting, mild flicker damping) as constants while
   the UI already exposes `ewmaEnabled` / `flickerReductionEnabled`; the weights are
   the missing half.
3. [ ] **P3.9 — NPU: additional vendor dispatch runtimes (long-term).**
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
4. [ ] **P4 — iOS platform (high-level only; do not pre-write iOS code).**
   - P4.1 `bridge/ios/` Swift shim over the core's `cdylib` (core + ABI reused unchanged);
     P4.2 AVFoundation media; P4.3 Core ML / Metal inference (LiteRT stays Android-side);
     P4.4 SwiftUI on the `:shared` contracts.
   - Hard rule: nothing iOS-specific enters `core/` or `app/shared/` — one bridge crate
     per host.
5. [ ] **P5 — App UI in the Morphe Manager style** ([MorpheApp/morphe-manager](https://github.com/MorpheApp/morphe-manager)).
   - Look-and-structure reference only: Material 3 surfaces, card + toggle layout, icon-led
     settings rows, step/progress indicators while a session runs.
   - License gate: `morphe-manager` is GPL-3.0, this repo is Apache-2.0 — imitate the design,
     never port its code.
   - Parked: no UI work until asked; UI lives in `app/platforms/android/…/ui/` on top of the
     `:shared` UI-state contracts.

## Notes

- Review `docs/dev/architecture/DESIGN_SUGGESTIONS.md` where it is cheap; defer the
  hard/complex suggestions.
