# App fixes — record

Archive of the app-side investigation that landed with P1.1 (2026-10); it began as
a root-level scratch file. Kept so agents and humans do not re-litigate the same
dead ends. Open app work lives in `TODO.md`, not here.

## Fixed (app layer)

| Issue | Fix |
| --- | --- |
| CPU: "output tensor not found" on configure | `AndroidLiteRtEngine` queries the output dtype with the **signature** output name (`output_0`) instead of the graph tensor name (`StatefulPartitionedCall_1:0`); verified against the model with the LiteRT C API (`TfLiteSignatureRunnerGetOutputName`). |
| NPU: "not available" on Tensor devices (e.g. Pixel 10 Pro) | LiteRT's `Environment.create(context)` never registers the NPU accelerator, so the app bundles the Google Tensor dispatch runtime (`libLiteRtDispatch_GoogleTensor.so` + `libLiteRtCompilerPlugin_google_tensor.so` from the LiteRT 2.2.0 release zip — the newest release that ships NPU libraries) into `app/platforms/android/src/main/jniLibs/arm64-v8a/` and builds the environment with a `TensorNpuProvider` (`NpuAcceleratorProvider`) exposing the native library directory; the environment is passed to `CompiledModel.create`. |
| Settings screen too similar to the home screen | Settings now has **Default compute target** (selector moved from home), **Enhancement toppings** (EWMA + flicker), **Coming soon** placeholder toggles, **About** (version, model, runtime, device/SoC), and **Privacy**; home shows a read-only "Current configuration" summary that opens Settings. |
| No way to inspect app behaviour on-device | New **Logs** section in Settings: in-memory ring buffer (last 1000 entries, shared KMP `LogBuffer`) with **View logs** (monospace viewer), **Save to file…** (Storage Access Framework, no permissions), and **Clear**; critical areas instrumented (probe results, NPU provider status, configure fallback reasons, per-frame timing, decode progress), every entry mirrored to logcat under tag `OpenLLVE`. |
| Image enhancement hard-crashed (tombstone) on large stills | `OutOfMemoryError` is not an `Exception`, so the image path's catch never saw it: `EnhancementViewModel` now catches it and shows the normal error banner, and `AndroidLiteRtEngine.enhanceFrame` logs the patch-accumulation size **before** allocating, so a run that still dies leaves the number it requested in Settings → Logs. Root cause is the whole-frame accumulator (`hPad * wPad * 24` floats ≈ 1.17 GB at 4000×3000) — banding it is `TODO.md` item 2; this guard makes the failure reportable, it does not make large stills work. |
| Gradle 177 parse errors on `app/build.gradle.kts` | Comment cleanup swallowed the `signingConfig = if (signingConfigs.names.contains("release")) {` line into the comment above it; the if-expression is restored (release signing opt-in unchanged). |
| Placeholder app identity `com.example.openllve` | Namespace/applicationId is now **`app.openllve`** in `app/build.gradle.kts` with the two `BuildConfig` imports updated (`OpenLLVEApp.kt`, `SettingsScreen.kt`); Kotlin package names (`openllve.android.*`, `openllve.shared.*`) are independent and unchanged. |
| EWMA / flicker reduction were UI-only placeholders | Applied by the Rust core through the JNI bridge (P1.1): `openllve/android/core/CoreToppings.kt` owns `openllve_blend_filter_*` + `openllve_ewma_filter_*` handles per media session, `AndroidLiteRtEngine.enhanceFrame` pipes every model output through them, and the `EnhancementSettings` toggles decide which handles exist. `CoreBridge.abiVersion()` is checked at session start; a rejected frame logs once and passes through. |
| The Rust core was never called from the app | `bridge/android/` exports 22 `Java_openllve_android_core_CoreBridge_*` symbols; `scripts/build-bridge.sh` installs `jniLibs/arm64-v8a/libbridge.so` (859,472 bytes) and the APK packages it. |

## Not actionable — vendor/platform constraints (do not re-attempt)

- **NPU unavailable on Android 16 QPR1 (BP2A) builds:** vendor gate — Google's
  own `NpuCompatibilityChecker` excludes `BP2A*` Build.IDs for Tensor devices;
  resolves on QPR2 (BP4A)+. Nothing the app can do.
- **NPU unavailable on non-arm64-v8a ABIs:** Google ships the Tensor dispatch
  runtime for `arm64-v8a` only; `TensorNpuProvider.isLibraryReady()` reports
  this honestly.
- **Tensor JIT is Beta; AOT is the supported path:** AOT needs Google's Tensor
  compiler toolchain plus per-device compilation, which no agent environment
  has. If a device cannot JIT, `configure` falls back to CPU with an explicit
  reason (never silent).
- **XNNPACK has no LiteRT accelerator:** `ComputeTarget.XNNPACK` maps to CPU —
  a LiteRT capability gap, not an app bug.
- **A bare "unexpected error" on the video path is vendor text, not app code:** the
  string is a LiteRT status message (it appears in `libLiteRtCompilerPlugin_google_tensor.so`
  alongside "no error"/"invalid escape sequence"); LiteRT exposes no status code or cause
  for a failed `Interpreter.create`. The app only wraps it (`VideoFrameProvider.kt`
  `"Video processing failed: ${e.message}"`); fixing it means a model the runtime can load,
  not app code.
- **NPU / GPU end-to-end validation waits on hardware:** no Tensor or OpenGL-ES
  device was available; verified by construction (official LiteRT 2.2.0
  libraries + the provider pattern from the official NPU sample). When a device
  exists, Settings → Logs shows the probe verdict, the backend used, and
  per-frame timing (logcat tag `OpenLLVE`).

## Notes (by design, not bugs)

- **Release signing is opt-in.** Without `RELEASE_*` environment variables, release
  builds are signed with the debug keystore.
- **The log buffer is RAM-only** (last 1000 entries, cleared on restart; "Save to
  file" keeps a copy). `LogBuffer` lives in the KMP layer so a future host reuses it
  with its own mirroring and viewer.
- UI labels for the toppings still read "prototype — applied by the native
  pipeline"; reword once they are verified on a device.
