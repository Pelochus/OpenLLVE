# TODO-APPFIXES

App fixes investigated and applied in this round, plus the remaining work that
the app can still act on. Items gated by vendor/OS or requiring device
hardware are tracked as one-liners under "Not actionable" — they were
investigated and have no app-side fix; do not re-litigate them.

## Fixed in this round

| Issue | Fix |
| --- | --- |
| CPU: "output tensor not found" on configure | `AndroidLiteRtEngine` now queries the output dtype with the **signature** output name (`output_0`) instead of the graph tensor name (`StatefulPartitionedCall_1:0`). Verified against the model with the LiteRT C API (`TfLiteSignatureRunnerGetOutputName`). |
| NPU: "not available" on Tensor devices (e.g. Pixel 10 Pro) | LiteRT's `Environment.create(context)` never registers the NPU accelerator (the dispatch library is not statically linked into `libLiteRt.so`). The app now bundles the Google Tensor dispatch runtime (`libLiteRtDispatch_GoogleTensor.so` + `libLiteRtCompilerPlugin_google_tensor.so`, from the official LiteRT 2.2.0 release zip — the newest release that ships NPU runtime libraries; the 2.3.0 October release ships none) into `app/platforms/android/src/main/jniLibs/arm64-v8a/`, and builds the environment with a `TensorNpuProvider` (`NpuAcceleratorProvider`) that exposes the native library directory. The environment is passed to `CompiledModel.create`. |
| Settings screen too similar to the home screen | Settings now has: **Default compute target** (selector moved from home), **Enhancement toppings** (EWMA + flicker), **Coming soon** placeholder toggles (adaptive exposure, temporal noise suppression, automatic backend selection), **About** (version, model, runtime, device/SoC), and **Privacy**. The home screen shows a read-only "Current configuration" summary that opens Settings. |
| No way to inspect app behaviour on-device | New **Logs** section in Settings: in-memory ring buffer (last 1000 entries, shared KMP `LogBuffer`) with **View logs** (monospace viewer screen), **Save to file…** (Storage Access Framework, no permissions), and **Clear**. Critical areas are instrumented (probe results, NPU provider status, configure fallback reasons, per-frame timing, video decode progress/errors); every entry is also mirrored to logcat under tag `OpenLLVE`. |
| Placeholder app identity `com.example.openllve` | App namespace/applicationId is now **`app.openllve`** in `app/build.gradle.kts`, with the two `BuildConfig` imports updated (`OpenLLVEApp.kt`, `SettingsScreen.kt`). Kotlin package names (`openllve.android.*`, `openllve.shared.*`) are independent and unchanged. |

## Remaining app work

1. **EWMA / flicker reduction are UI placeholders.** The toggles are exposed,
   persisted, and passed through the configuration, but the temporal processing
   belongs to the Rust core (`core/src/filters/ewma.rs`, the `openllve_ewma_filter_*`
   and `openllve_blend_filter_*` C ABI in `core/src/ffi.rs`), which is not
   wired into the Android app yet (TODO.md P1.1). They are labelled
   "prototype — applied by the native pipeline" in the UI until the FFI
   wiring lands.

## Not actionable — vendor/platform constraints (investigated, no app-side fix)

- **NPU unavailable on Android 16 QPR1 (BP2A) builds:** vendor gate — Google's
  own `NpuCompatibilityChecker` excludes `BP2A*` Build.IDs for Tensor devices;
  resolves on QPR2 (BP4A)+. Nothing the app can do.
- **NPU unavailable on non-arm64-v8a ABIs:** Google distributes the Tensor
  dispatch runtime only for `arm64-v8a`; no libraries exist for
  `armeabi-v7a`/`x86`/`x86_64`. `TensorNpuProvider.isLibraryReady()` reports
  this honestly.
- **Tensor JIT is Beta; AOT is the supported path:** AOT needs the Google
  Tensor compiler toolchain + per-device compilation, unavailable in any
  agent environment. If a device cannot JIT, `configure` falls back to CPU
  with an explicit reason (never silent).
- **XNNPACK has no LiteRT accelerator:** `ComputeTarget.XNNPACK` maps to CPU —
  a LiteRT capability gap, not an app bug.
- **NPU / GPU end-to-end validation waits on hardware:** no Tensor device or
  OpenGL-ES device was available. Verified by construction (official LiteRT
  2.2.0 runtime libraries + the provider pattern from the official NPU
  sample). Validation path when a device exists: Settings → Logs shows the
  probe verdict, the backend used, and per-frame timing (logcat tag
  `OpenLLVE`).

## Notes (by design, not bugs)

- **Release signing is opt-in.** Without `RELEASE_*` environment variables,
  release builds are signed with the debug keystore (see
  `scripts/sign-release-apk.sh` and the signing block in `app/build.gradle.kts`).
- **The log buffer is RAM-only** (last 1000 entries, cleared on app
  restart); "Save to file" keeps a copy. The shared `LogBuffer` lives in
  the KMP layer (`openllve.shared.log`) so the future iOS host reuses it
  with its own mirroring (os.log) and viewer.
