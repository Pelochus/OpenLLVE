# OpenLLVE — Android App TODO (app-side only)

This file tracks **Android/app-side** work only. It does not duplicate the
general Rust TODOs in `TODO.md`. It records what the current Android-first
vertical-slice session implemented, what is intentionally deferred, the known
build/runtime blockers, and a prompt to continue the work in a new session.

---

## 1. Current status (TL;DR)

- **Architecture and UI are complete and coherent.** The full vertical-slice
  structure is in place: UI → ViewModel → domain config → media layer →
  enhancement engine (temporary LiteRT impl). The domain types are shaped so the
  Rust core can consume them later without changing the UI.
- **The app now compiles cleanly.** `./gradlew :app:assembleDebug` is
  **BUILD SUCCESS** (only deprecation warnings remain, no compile errors).
  The LiteRT artifact was pinned to the classic `org.tensorflow.lite` API
  (2.14.0), the engine and media layers were reconciled with the real
  `org.tensorflow.lite` and `MediaCodec`/`MediaExtractor` APIs, and the
  Compose Compiler Gradle plugin was added (required by Kotlin 2.0).
  See §4 for the details of what was fixed.
- **No Rust files were modified.** No C/C++ bridge was added. No iOS work.
  The LiteRT integration is isolated behind `EnhancementEngine`.
- **No device/emulator was available in this environment**, so runtime behavior
  (actual inference output, video decode, delegate selection) is **unverified**.
  Everything is verified only at the compile level.

---

## 2. What was implemented

### Build configuration

- Fixed two **pre-existing** build blockers that prevented *any* build:
  - `settings.gradle.kts`: `pluginManagement.plugins { }` used the wrong DSL
    (`id("...") { id = ...; version = ... }`). Changed to
    `id("...") version "..."`.
  - `app/build.gradle.kts`: the `:app` module had **no `repositories { }`
    block**. Added `google()` + `mavenCentral()`.
- Refreshed dependencies toward a modern toolchain:
  - `org.tensorflow:tensorflow-lite` 2.12.0 → **2.17.0**
  - `org.tensorflow:tensorflow-lite-gpu` 2.12.0 → **2.17.0**
  - Added: `navigation-compose`, `lifecycle-runtime-compose`,
    `lifecycle-viewmodel-compose`, `datastore-preferences`,
    `material-icons-core`, `compose foundation`.
  - Removed unused: CameraX, `constraintlayout`, `tensorflow-lite-support`.
  - `minSdk` 21 → **26** (Android 8.0): floor for hardware-buffer frame
    decoding and the NNAPI (NPU) delegate.
- `AndroidManifest.xml`: fixed the launcher activity name (was
  `.ui.MainActivity` → resolved to the wrong package; now the fully-qualified
  `openllve.android.ui.MainActivity`), registered `OpenLLVEApp`, and removed the
  now-unneeded storage/camera permissions (media is selected via the Storage
  Access Framework).
- `res/values/themes.xml`: switched to `Theme.Material3.DayNight.NoActionBar`.

### Domain layer (`openllve.android.domain`) — the Rust seam

- `ComputeTarget` — enum `CPU / XNNPACK / GPU / NPU` (OpenLLVE concept, not a
  LiteRT delegate class).
- `EnhancementSettings` — `computeTarget`, `ewmaEnabled`,
  `flickerReductionEnabled`. Shaped so the Rust core can consume it directly.
- `MediaInput` — sealed `ImageInput` / `VideoInput` (image and video paths stay
  separate).
- `BackendSelection` — `requested` vs `actual` backend + `reason` (never a
  silent fallback).
- `ProcessingMetrics` — frame count, inference ms, avg/frame, FPS, resolution,
  backend, model, and a `realtimeFactor` (faster/slower than realtime for
  video).
- `EnhancementEngine` — the interface the UI depends on. **This is the seam**
  that a future `RustEngine` (via the C FFI) will implement without changing the
  UI.

### Engine (`openllve.android.engine`) — temporary LiteRT implementation

- `AndroidLiteRtEngine : EnhancementEngine` — loads the Zero-DCE model from
  assets, creates a LiteRT interpreter with the requested delegate
  (CPU / XNNPACK / GPU / NNAPI), probes available backends, and runs the model
  path **mirroring the Rust `ModelRunner`** (`core/src/model.rs`): 256×256 patch
  tiling with 16px overlap, reflect padding, linear-ramp reassembly, and the 8
  learned curves. Clearly documented as temporary.

### Media layer (`openllve.android.media`)

- `FramePixels` — `Bitmap` ↔ row-major RGB float `[0,1]` conversion (shared by
  image and video paths).
- `ImageFrameProvider` — loads a SAF-selected image into a `Bitmap`.
- `VideoMetadata` / `VideoMetadataReader` — basic video metadata via
  `MediaMetadataRetriever` (duration, resolution, mime, size).
- `VideoFrameProvider` — decodes an MP4 with the platform hardware
  `MediaCodec` (demuxed via `MediaExtractor`) and runs each decoded frame
  through the engine, producing paired (original, enhanced) bitmaps on a single
  background thread. Audio is dropped (video-only decode).

### Data (`openllve.android.data`)

- `SettingsRepository` — persists `EnhancementSettings` with Jetpack DataStore
  (Preferences). No database.

### UI (`openllve.android.ui`) — Material 3 / Material You

- `OpenLLVETheme` — dynamic (wallpaper-derived) Material You colors on
  Android 12+, base palette fallback otherwise.
- `MainActivity` — single launcher activity; wires `OpenLLVEApp` components into
  the ViewModel; Compose `NavHost` with four destinations.
- `Destination` — `HOME / SETTINGS / IMAGE_RESULT / VIDEO_RESULT`.
- `UiState` — flat presentation state.
- `EnhancementViewModel` — orchestrates settings (persisted), backend probing,
  image enhancement, and the video decode/enhance loop. Exposes only domain
  types.
- Screens: `HomeScreen` (select image/MP4 + settings), `SettingsScreen`,
  `ImageResultScreen` (comparison + metrics + backend + re-run),
  `VideoResultScreen` (metadata + settings + start/stop + live comparison +
  metrics).
- Components: `ComparisonSlider` (original ↔ enhanced crossfade), `MetricsCard`,
  `BackendInfoCard` (requested vs actual + reason), `SettingsCard` (compute
  target selector with unsupported targets disabled/annotated, EWMA + flicker
  toggles labelled "prototype — applied by the native pipeline"), `StatusViews`
  (processing / error / empty states).
- `OpenLLVEApp` — lightweight application-scoped wiring (no DI framework).

### Removed (old stubs)

- `domain/VideoPipelineManager.kt`, `data/BenchmarkResult.kt`,
  `data/SystemMonitor.kt`, `ui/MainScreen.kt`, and the placeholder
  `ui/components` file.

---

## 3. What was intentionally NOT implemented (deferred)

| Item | Why deferred |
| --- | --- |
| **Rust core wiring (FFI)** | Explicitly out of scope. The Rust `ModelRunner`/pipeline is not called; the Kotlin engine is a temporary stand-in. See `TODO.md` P1.1. |
| **C/C++ bridge** | Out of scope for this phase; to be validated later. |
| **iOS** | Out of scope. |
| **Real EWMA / flicker algorithm in Kotlin** | Belongs in the Rust core. The toggles are exposed, persisted, and passed through config only; the UI labels them "applied by the native pipeline". |
| **Camera capture** | Not part of this vertical slice (media is file-selected via SAF). CameraX deps removed. |
| **Full synchronized enhanced playback with audio** | The video path decodes video-only (audio dropped) and processes frames live. Full frame-by-frame synchronized *enhanced* playback with the original audio track is deferred to the Rust/native pipeline. |
| **KMP shared module** | Deliberately not created to avoid speculative multiplatform abstractions with no second platform. The domain layer is KMP-ready and can be lifted into `app/shared/` later. |
| **APK signing / `build-apk.sh`** | Not completed — no signing credentials available and not invented. See §5. |
| **`proguard-rules.pro`** | Referenced by the release build type but not required for `assembleDebug`; left as-is. |

---

## 4. Build blockers — RESOLVED (compile) / remaining (runtime)

### 4.1 LiteRT artifact — **FIXED**

`org.tensorflow:tensorflow-lite:2.17.0` is **not a real AAR** — its POM
relocates to `com.google.ai.edge.litert:litert:1.0.1` (the new "LiteRT"
artifact), and the `org.tensorflow.lite` classes the engine was written
against (including `TensorBuffer`) do not exist in it. **Resolution:**
pinned to the classic artifact `org.tensorflow:tensorflow-lite:2.14.0` (+
`tensorflow-lite-gpu:2.14.0`), which still exposes the `org.tensorflow.lite`
API. The engine was reconciled against the real classic API:

- `TensorBuffer` / `FloatTensorBuffer` / `Int8TensorBuffer` **do not exist** in
  any classic `org.tensorflow.lite` version. Replaced all `TensorBuffer`
  usage with `ByteBuffer.allocateDirect(...)` and `Interpreter.run(Object, Object)`
  (raw `ByteBuffer` in/out).
- `tensor.type()` → `tensor.dataType()`; `tensor.quantization()` →
  `tensor.quantizationParams()`; `.scale` → `.getScale()`; `.zeroPoint` →
  `.getZeroPoint()`.
- GPU delegate uses the no-arg `GpuDelegate()` constructor.
- **Remaining (runtime, §4.4):** verify the model's output tensor dtype
  (INT8 vs FLOAT32) and the dequantization path on a device.

### 4.2 `engine/AndroidLiteRtEngine.kt` — **FIXED**

- Reconciled with the classic `org.tensorflow.lite` API (see 4.1).
- Resolved a conflicting `lastInferenceMs` declaration (the private `var`
  was renamed to `lastInferenceMsValue`; the public `lastInferenceMs` getter
  is preserved for the media layer).

### 4.3 `media/VideoFrameProvider.kt` — **FIXED**

- `MediaExtractor.setDataSource(...)` has **no `Uri` overload**. Opened the
  SAF `Uri` via `context.contentResolver.openFileDescriptor(uri, "r")` and
  called `setDataSource(fd.fileDescriptor, 0, fd.statSize)` (note: it is
  `fd.statSize`, not `fd.length`).
- Removed the prohibited non-local `return` in the `Thread { }` lambda
  (restructured to an `if` block).
- `MediaExtractor.prepare()` **does not exist** in API 34 — removed the call.
- `MediaExtractor.BUFFER_FLAG_SYNC_FRAME` → `MediaExtractor.SAMPLE_FLAG_SYNC`
  (the correct constant name).
- `codec.queueInputBuffer(...)` takes **5** parameters (index, offset, size,
  presentationTime, flags) — fixed the call.
- `MediaCodec.INFO_OUTPUT_EOS` **does not exist** — after feeding the EOS
  flag, `INFO_TRY_AGAIN_LATER` indicates the decoder has drained every frame.
- `ByteBuffer.getHardwareBuffer()` / `MediaCodec.OutputFrame.getHardwareBuffer()`
  and `Surface(HardwareBuffer)` are not available for software output. Switched
  to **software output** (null surface): the decoded frame comes back as a
  `ByteBuffer` in the codec's YUV format (NV12/YV12), converted to an
  ARGB_8888 `Bitmap` via a `yuvToBitmap` helper.
- `extractor.readSampleData(ByteBuffer, int)` requires a `ByteBuffer` (not a
  `ByteArray`) — used `ByteBuffer.wrap(ByteArray)` for the input sample.

### 4.3a `data/SettingsRepository.kt` — **FIXED** (newly discovered)

- `preferencesDataStore(name = "...")` returns a `ReadOnlyProperty<Context,
  DataStore<Preferences>>`; its `getValue` requires a `Context` this-reference,
  so it cannot be used as a plain class property. Added a top-level extension
  property `val Context.settingsDataStore: DataStore<Preferences> by
  preferencesDataStore(name = "openllve_settings")` and used
  `context.settingsDataStore` in the class.
- Added `import androidx.datastore.preferences.core.edit` for the
  `DataStore<Preferences>.edit` extension.

### 4.3b Compose Compiler Gradle plugin — **FIXED** (newly discovered)

- Kotlin 2.0 **requires** the Compose Compiler Gradle plugin when Compose is
  enabled. Without it, the build fails with a backend internal error when
  inlining `androidx.lifecycle.viewmodel.compose.viewModel` ("couldn't find
  inline method"). Added `id("org.jetbrains.kotlin.plugin.compose")` (version
  2.0.21, matching the Kotlin compiler) to `settings.gradle.kts` and
  `app/build.gradle.kts`, plus `buildFeatures { compose = true }` and
  `composeOptions { kotlinCompilerExtensionVersion = "2.0.21" }`.

### 4.4 Runtime verification (NOT done — no device/emulator)

- Confirm the model actually loads and produces a visibly enhanced frame.
- Confirm delegate probing reports correct support (esp. NPU/NNAPI, which can
  silently fall back to CPU — documented limitation).
- Confirm the MediaCodec decode loop runs and produces synchronized
  original/enhanced frames (the YUV→RGB conversion is untested on a device).
- Confirm settings persist across app restarts.

---

## 5. APK build / signing (deferred, documented)

- **Debug build** works via `./gradlew :app:assembleDebug` once §4 is fixed
  (output: `app/build/outputs/apk/debug/app-debug.apk`).
- **Signed release APK** is intentionally **not** done:
  - No signing credentials are available and none were invented.
  - A `build-apk.sh` can be added later; it should:
    1. Generate/use a keystore (via `keytool`),
    2. Run `./gradlew :app:assembleRelease`,
    3. Sign with `apksigner` (from the SDK `build-tools`).
  - **Do not** add fake libraries or fake credentials.

---

## 6. How to build / run (environment notes)

The dev machine here is a Fedora-based (Bazzite/UBI) host with **no system
JDK/Android SDK**. A working build environment was set up:

- Portable JDK 17: `/var/home/pelochus/tools/jdk-17.0.13+11`
- Android SDK: `/var/home/pelochus/android-sdk` (platform-34, build-tools 34.0.0,
  platform-tools, cmdline-tools). `local.properties` points to it.
- Build inside the `llama-build` distrobox (Fedora):

  ```bash
  distrobox enter llama-build -- bash -c '
    export JAVA_HOME=/var/home/pelochus/tools/jdk-17.0.13+11
    export PATH=$JAVA_HOME/bin:$PATH
    cd /var/home/pelochus/Documents/OpenLLVE
    ./gradlew :app:assembleDebug --console=plain
  '
  ```

- To run on a device/emulator: `adb install app/build/outputs/apk/debug/app-debug.apk`.

---

## 7. Continuation prompt (for a new session)

> **Continue the OpenLLVE Android vertical slice.**
>
> **Compile status: DONE.** `./gradlew :app:assembleDebug` is **BUILD SUCCESS**
> (no compile errors; only deprecation warnings). The LiteRT artifact is pinned
> to the classic `org.tensorflow.lite` API (2.14.0), the engine and media layers
> are reconciled with the real APIs, and the Compose Compiler Gradle plugin is
> in place. See §4 for the details.
>
> 1. Read `TODO-app.md` (this file), `TODO.md`, `IMPROVEMENTS.md`,
>    `docs/ARCHITECTURE.md`, and `core/README.md`. Do **not** modify Rust, add
>    C/C++, or start iOS work.
> 2. **Runtime verification (§4.4)** — the remaining work. If a device/emulator
>    is available, run the app and verify:
>    - The model loads and produces a visibly enhanced frame (image path).
>    - The MP4 decode loop runs and shows synchronized original↔enhanced frames
>      (video path); the YUV→RGB conversion is correct.
>    - Delegate selection reports the actual backend (never a silent fallback;
>      esp. NPU/NNAPI, which can silently fall back to CPU).
>    - The model's output tensor dtype (INT8 vs FLOAT32) and the dequantization
>      path are correct.
>    - Settings persist across app restarts.
> 3. Update `TODO-app.md` to mark completed items and add any newly discovered
>    app-side work. Keep `TODO.md` (Rust) and `IMPROVEMENTS.md` in sync.
> 4. Only after the app is functional, consider the deferred items in §3 (e.g.
>    full synchronized enhanced playback with audio, APK signing, KMP lift of the
>    domain layer).
>
> **Constraints (unchanged):** no Rust changes, no C/C++, no iOS, no speculative
> KMP abstractions, no DI framework, no custom decoder, no invented credentials.
