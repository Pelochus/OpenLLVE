# TODO-APPFIXES

App fixes investigated and applied in this round, plus what could not be fixed
in the app (or is too difficult without device hardware).

## Fixed in this round

| Issue | Fix |
| --- | --- |
| CPU: "output tensor not found" on configure | `AndroidLiteRtEngine` now queries the output dtype with the **signature** output name (`output_0`) instead of the graph tensor name (`StatefulPartitionedCall_1:0`). Verified against the model with the LiteRT C API (`TfLiteSignatureRunnerGetOutputName`). |
| NPU: "not available" on Tensor devices (e.g. Pixel 10 Pro) | LiteRT's `Environment.create(context)` never registers the NPU accelerator (the dispatch library is not statically linked into `libLiteRt.so`). The app now bundles the Google Tensor dispatch runtime (`libLiteRtDispatch_GoogleTensor.so` + `libLiteRtCompilerPlugin_google_tensor.so`, from the official LiteRT 2.2.0 release zip) into `app/platforms/android/src/main/jniLibs/arm64-v8a/`, and builds the environment with a `TensorNpuProvider` (`NpuAcceleratorProvider`) that exposes the native library directory. The environment is passed to `CompiledModel.create`. |
| Settings screen too similar to the home screen | Settings now has: **Default compute target** (selector moved from home), **Enhancement toppings** (EWMA + flicker), **Coming soon** placeholder toggles (adaptive exposure, temporal noise suppression, automatic backend selection), **About** (version, model, runtime, device/SoC), and **Privacy**. The home screen shows a read-only "Current configuration" summary that opens Settings. |

## Not fixed — cannot be fixed in the app

### 1. NPU unavailable on Android 16 QPR1 (BP2A) builds

Google's own `NpuCompatibilityChecker` (used by the app) excludes builds whose
`Build.ID` starts with `BP2A` (the initial Android 16 release) for Google
Tensor devices, per Google's comment that Tensor NPU support in those builds
is not ready. A Pixel 10 Pro running a BP2A build will still report NPU as
unavailable even with the dispatch runtime bundled. This resolves on QPR2
(BP4A) and later builds. Nothing the app can do — it is a vendor/OS gate.

### 2. NPU unavailable on non-arm64-v8a devices

The Google Tensor dispatch runtime is only distributed for `arm64-v8a`
(NDK-built aarch64 libraries). There are no dispatch libraries for
`armeabi-v7a`, `x86`, or `x86_64` (e.g. x86_64 emulators). The app's
`TensorNpuProvider.isLibraryReady()` checks for the dispatch `.so` in the
native library directory, so the probe honestly reports NPU as unavailable on
those ABIs instead of advertising a backend that would fail at model
creation.

### 3. Google Tensor JIT is Beta; AOT is the supported path

On-device **JIT** compilation (the runtime bundled here, matching the
official LiteRT `kotlin_npu/android_jit` sample) is marked *Beta* in the
LiteRT docs; **AOT** (pre-compiled model, `android_aot`) is the supported
Tensor path. Producing an AOT model for Tensor would require the Google
Tensor compiler toolchain and per-device compilation, which is not available
in this environment. If a given Tensor device cannot JIT-compile the
Zero-DCE model, `configure` falls back to CPU with an explicit reason
(the app never falls back silently).

### 4. Tensor NPU model compatibility is unverified

No Tensor device was available for testing. The fix is verified by
construction (official LiteRT 2.2.0 runtime libraries + the same
provider-based environment the official NPU sample uses), but end-to-end
NPU inference on a Pixel 10 Pro (or other Tensor G3–G6 device, Android 16
QPR2+) still needs on-device validation.

### 5. XNNPACK has no accelerator in LiteRT

`CompiledModel` exposes no XNNPACK accelerator; `ComputeTarget.XNNPACK`
therefore maps to CPU (a mapping, not a fallback). This is a LiteRT
capability gap, not an app bug.

### 6. EWMA / flicker reduction are UI placeholders

The toggles are exposed, persisted, and passed through the configuration,
but the actual temporal processing belongs to the Rust core (TODO.md P1.1),
which is not wired into the Android app yet. They are labelled
"prototype — applied by the native pipeline" in the UI.

### 7. GPU (OpenGL ES) is device-dependent and unverified

GPU availability depends on the device's OpenGL ES support; no device was
available to validate. The probe reports it per device via
`getAvailableAccelerators()`.

## Notes (by design, not bugs)

- **Release signing is opt-in.** Without `RELEASE_*` environment variables,
  release builds are signed with the debug keystore (see
  `docs/dev/android-release-signing.md`).
- **`namespace = "com.example.openllve"`** is still a placeholder in
  `app/build.gradle.kts`; the About screen reads `BuildConfig.VERSION_NAME`
  from it.
