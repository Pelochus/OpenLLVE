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
- **The app does NOT yet compile cleanly.** There are remaining Kotlin compile
  errors in two files (`engine/AndroidLiteRtEngine.kt` and
  `media/VideoFrameProvider.kt`). They are well-understood and bounded (see
  §4). The rest of the project compiles.
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

## 4. Known build/runtime blockers (to fix next)

### 4.1 LiteRT 2.17.0 is a **relocation** (root cause of the engine errors)

`org.tensorflow:tensorflow-lite:2.17.0` is **not a real AAR** — its POM
relocates to `com.google.ai.edge.litert:litert:1.0.1` (the new "LiteRT"
artifact). Consequences:

- The class package may have moved from `org.tensorflow.lite` to
  `com.google.ai.edge.litert`, which is why `TensorBuffer`,
  `FloatTensorBuffer`, `Int8TensorBuffer`, `getBuffer()` are unresolved in
  `AndroidLiteRtEngine.kt`.
- **Fix options (pick one and verify on a device):**
  1. **Pin the classic artifact** that still exposes the `org.tensorflow.lite`
     API: use `org.tensorflow:tensorflow-lite:2.14.0` (or the last version
     before the relocation) — lowest risk, matches the API the code was written
     against.
  2. **Adopt the new LiteRT artifact** `com.google.ai.edge.litert:litert:1.0.1`
     and update imports (`Interpreter`, `TensorBuffer`, delegate classes) to the
     new `com.google.ai.edge.litert` package. More future-proof but a bigger
     change.
- Whichever is chosen, re-verify the delegate API (XNNPACK via
  `Options.setUseXNNPACK`, NNAPI via `Options.setUseNNAPI`, GPU via
  `GpuDelegate`) against the chosen artifact's actual classes.

### 4.2 `engine/AndroidLiteRtEngine.kt` — remaining compile errors

- `TensorBuffer` / `getBuffer()` unresolved (see 4.1). The current code uses
  `TensorBuffer.create(...)` and `getBuffer().asFloatBuffer()`; this must be
  reconciled with the actual tensor-buffer API of the chosen artifact (e.g.
  `FloatTensorBuffer(vararg shape)` + `.floatBuffer`, `Int8TensorBuffer` +
  `.int8Buffer`, or the `ByteBuffer` path).
- Verify the model's **output tensor dtype** (INT8 vs FLOAT32) at runtime; the
  dequantization path (`scale`/`zeroPoint`) must match.

### 4.3 `media/VideoFrameProvider.kt` — remaining compile errors

- `MediaExtractor.setDataSource(...)` has **no `Uri` overload**. Must open the
  SAF `Uri` via `context.contentResolver.openFileDescriptor(uri, "r")` and call
  `setDataSource(fd.fileDescriptor, 0, fd.length)` (or convert to a path).
- `return` inside the `Thread { }` Runnable lambda is a prohibited non-local
  return — restructure to an `if` block (no `return`).
- The remaining errors (`prepare`, `setEncodingFlags`, `setPresentationTime`,
  `BUFFER_FLAG_SYNC_FRAME`, `queueInputBuffer`, `INFO_OUTPUT_EOS`,
  `getHardwareBuffer`, `Bitmap.createBitmap(...)`) are **cascading** from the
  `setDataSource` type mismatch; they should resolve once the source is fixed.
- Verify `Bitmap.wrapHardwareBuffer` + `Bitmap.createBitmap` work for the
  decoder's output format on a real device (API 26+).

### 4.4 Runtime verification (not done — no device/emulator)

- Confirm the model actually loads and produces a visibly enhanced frame.
- Confirm delegate probing reports correct support (esp. NPU/NNAPI, which can
  silently fall back to CPU — documented limitation).
- Confirm the MediaCodec decode loop runs and produces synchronized
  original/enhanced frames.
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
> 1. Read `TODO-app.md` (this file), `TODO.md`, `IMPROVEMENTS.md`,
>    `docs/ARCHITECTURE.md`, and `core/README.md`. Do **not** modify Rust, add
>    C/C++, or start iOS work.
> 2. Fix the build blockers in §4, in this order:
>    a. Decide the LiteRT artifact (§4.1): **recommended** — pin
>       `org.tensorflow:tensorflow-lite:2.14.0` (classic `org.tensorflow.lite`
>       API) to match the existing engine code; alternatively adopt
>       `com.google.ai.edge.litert:litert:1.0.1` and update imports.
>    b. Make `engine/AndroidLiteRtEngine.kt` compile against the chosen
>       artifact's tensor-buffer + delegate API; verify the model output dtype
>       and dequantization.
>    c. Make `media/VideoFrameProvider.kt` compile: fix `MediaExtractor`
>       `setDataSource` (use the `FileDescriptor` overload from the SAF `Uri`),
>       remove the prohibited `return` in the `Thread` lambda; confirm the
>       cascading errors clear.
> 3. Get `./gradlew :app:assembleDebug` to **BUILD SUCCESS** (no compile
>     errors).
> 4. If a device/emulator is available, run the app and verify: image
>     enhancement produces a visibly enhanced frame; the MP4 decode loop runs and
>     shows synchronized original↔enhanced; delegate selection reports the
>     actual backend (never a silent fallback); settings persist.
> 5. Update `TODO-app.md` to mark completed items and add any newly discovered
>     app-side work. Keep `TODO.md` (Rust) and `IMPROVEMENTS.md` in sync.
> 6. Only after the app is functional, consider the deferred items in §3 (e.g.
>    full synchronized enhanced playback with audio, APK signing, KMP lift of the
>    domain layer).
>
> **Constraints (unchanged):** no Rust changes, no C/C++, no iOS, no speculative
> KMP abstractions, no DI framework, no custom decoder, no invented credentials.
