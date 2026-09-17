# OpenLLVE — Android App TODO (app-side only)

This file tracks **Android/app-side** work only. It does not duplicate the
general Rust TODOs in `TODO.md`. It records the current status, what is
intentionally deferred, the remaining runtime work, and a prompt to continue
the work in a new session.

---

## 1. Current status (TL;DR)

- **The app compiles cleanly.** `./gradlew :app:assembleDebug` is **BUILD
  SUCCESS** (only deprecation warnings, no compile errors).
- **Currently on the LiteRT 2.2.0 `CompiledModel` API**
  (`com.google.ai.edge.litert:litert:2.2.0`). The engine is rewritten against
  `CompiledModel`/`TensorBuffer`/`Accelerator`; backend probing uses the new
  runtime's `Environment.getAvailableAccelerators()`. Kotlin was bumped
  2.0.21 → 2.3.0 as a prerequisite (the `litert-api` classes carry Kotlin
  2.3.0 metadata). The classic `org.tensorflow.lite` (2.14.0) dependency is
  gone; the APK shrank ~177 MB → ~42.5 MB (NNAPI delegate + TFLite GPU libs
  dropped).
- **Build toolchain bumped (Change 2, §7 — DONE):** AGP 9.4.0, Kotlin (KGP)
  2.4.20, Compose BOM 2026.06.01 (UI 1.11.4), Gradle wrapper 9.7.1,
  compileSdk 36, activity 1.13.0, navigation 2.9.7, lifecycle 2.10.0.
  (lifecycle 2.11.0 / Compose BOM 2026.08.00 (UI 1.12.x) / navigation
  2.10.x / core 1.19.x all require compileSdk 37, which is not yet
  published — stable channel tops out at android-36.1.)
- **Next work: runtime verification (§3)** — needs a device/emulator.
- **No Rust files were modified.** No C/C++ bridge. No iOS work. The LiteRT
  integration is isolated behind `EnhancementEngine`.
- **No device/emulator was available**, so runtime behavior (actual inference
  output, video decode, delegate selection) is **unverified**. Everything is
  verified only at the compile level.

---

## 2. What was intentionally NOT implemented (deferred)

| Item | Why deferred |
| --- | --- |
| **Rust core wiring (FFI)** | Explicitly out of scope. The Rust `ModelRunner`/pipeline is not called; the Kotlin engine is a temporary stand-in. See `TODO.md` P1.1. |
| **C/C++ bridge** | Out of scope for this phase; to be validated later. |
| **iOS** | Out of scope. |
| **Real EWMA / flicker algorithm in Kotlin** | Belongs in the Rust core. The toggles are exposed, persisted, and passed through config only; the UI labels them "applied by the native pipeline". |
| **Camera capture** | Not part of this vertical slice (media is file-selected via SAF). CameraX deps removed. |
| **Full synchronized enhanced playback with audio** | The video path decodes video-only (audio dropped) and processes frames live. Full frame-by-frame synchronized *enhanced* playback with the original audio track is deferred to the Rust/native pipeline. |
| **KMP shared module** | Deliberately not created to avoid speculative multiplatform abstractions with no second platform. The domain layer is KMP-ready and can be lifted into `app/shared/` later. |
| **APK signing / `build-apk.sh`** | Not completed — no signing credentials available and not invented. See §4. |
| **Real `proguard-rules.pro` rules** | A placeholder file now exists (AGP 9.x fails the build when a declared ProGuard file is missing — `android.proguard.failOnMissingFiles` defaults to true). Real keep rules are only needed when R8/minification is enabled. |

---

## 3. Runtime verification (NOT done — no device/emulator)

The compile is done (BUILD SUCCESS). The remaining runtime work:

- Confirm the model actually loads and produces a visibly enhanced frame.
- Confirm backend probing (`Environment.getAvailableAccelerators()`) reports
  correct support, and that `configure`'s requested-vs-actual report (with
  reason) surfaces any per-target compile failure.
- Confirm the MediaCodec decode loop runs and produces synchronized
  original/enhanced frames (the YUV→RGB conversion is untested on a device).
- Confirm settings persist across app restarts.

---

## 4. APK build / signing (deferred, documented)

- **Debug build** works via `./gradlew :app:assembleDebug` (output:
  `app/build/outputs/apk/debug/app-debug.apk`).
- **Signed release APK** is intentionally **not** done:
  - No signing credentials are available and none were invented.
  - A `build-apk.sh` can be added later; it should:
    1. Generate/use a keystore (via `keytool`),
    2. Run `./gradlew :app:assembleRelease`,
    3. Sign with `apksigner` (from the SDK `build-tools`).
  - **Do not** add fake libraries or fake credentials.

---

## 5. How to build / run (environment notes)

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

## 6. Continuation prompt (for a new session)

> **Continue the OpenLLVE Android vertical slice — state-of-the-art dependency
> upgrades.**
>
> **Compile status: DONE.** `./gradlew :app:assembleDebug` is **BUILD SUCCESS**
> (no compile errors; only deprecation warnings). **Change 1 (§7) is DONE:**
> the engine runs on LiteRT 2.2.0 `CompiledModel`. **Change 2 (§7) is DONE:**
> the toolchain is bumped (AGP 9.4.0, KGP 2.4.20, Compose BOM 2026.06.01,
> Gradle 9.7.1, compileSdk 36). The remaining work is runtime verification.
>
> 1. Read `TODO-app.md` (this file), `TODO.md`, `IMPROVEMENTS.md`,
>    `docs/ARCHITECTURE.md`, and `core/README.md`. Do **not** modify Rust, add
>    C/C++, or start iOS work.
> 2. ~~Change 1 (§7): migrate the engine to LiteRT 2.2.0 `CompiledModel`.~~
>    **DONE** — see the "as-implemented" notes under §7 Change 1.
> 3. ~~Change 2 (§7): bump the build toolchain.~~ **DONE** — see the
>    "as-implemented" notes under §7 Change 2.
> 4. **Runtime verification (§3)** — if a device/emulator is available, verify:
>    model loads + visibly enhanced frame; MP4 decode loop synchronized
>    original↔enhanced; backend selection reports the actual backend; settings
>    persist.
> 5. Update `TODO-app.md` to mark completed items. Keep `TODO.md` (Rust) and
>    `IMPROVEMENTS.md` in sync.
>
> **Constraints (unchanged):** no Rust changes, no C/C++, no iOS, no speculative
> KMP abstractions, no DI framework, no custom decoder, no invented credentials.

---

## 7. Planned upgrades — state-of-the-art dependencies (two changes, ordered)

**Goal:** keep the app **state-of-the-art**. The ML side moves to the new
**LiteRT** runtime (the classic `org.tensorflow.lite` line is superseded), and
the build toolchain moves to the latest stable versions. The `.tflite` model
asset is **unchanged**; we will make it run on the new runtime (dtype/dequant +
delegate mapping verified on a device).

Two separate changes/commits, ordered from **easier/better** (do first) to
**harder/can-wait** (do second).

### Change 1 — easier / better (do FIRST): migrate the engine to LiteRT 2.2.0 `CompiledModel` — **DONE**

**As-implemented notes (what actually happened vs the plan):**

- **Prerequisite discovered:** Kotlin had to move 2.0.21 → **2.3.0** — every
  class in `litert-api:2.2.0` carries Kotlin 2.3.0 metadata, unreadable by
  older compilers. This is committed as part of Change 1 (Change 2 then
  continues 2.3.0 → 2.4.20).
- **Transitive-dependency exclusions:** `litert-api` pulls the Google Play
  "ai-delivery" stack (play-services/asset-delivery) and androidx.lifecycle
  2.10.x, which transitively force Compose UI 1.9.0 (requiring AGP 8.6.0+).
  The app only uses the `CompiledModel`/`TensorBuffer` path with a model from
  assets — not the AiPack `ModelProvider` download path — so
  `com.google.android.play`, `com.google.android.gms`, and `androidx.lifecycle`
  are excluded from the `litert` dependency (verified: only
  `ModelProvider`/`ModelSelector`/`AiPackModelProvider` reference them). The
  app keeps its own lifecycle stack (2.8.7).
- **Probing:** uses the new runtime's dedicated API,
  `Environment.getAvailableAccelerators()`, instead of per-delegate
  `Interpreter` construction. `configure` still verifies per-target
  compilation and reports requested-vs-actual with a reason (never silent).
- **Output dtype:** verified FLOAT32 `(1, 256, 256, 24)` for
  `zero-dce-int8.tflite`; the new runtime does not expose INT8 quantization
  parameters (`scale`/`zeroPoint`), so an INT8 output is rejected with a clear
  error rather than misread.
- **XNNPACK:** mapped to CPU (documented in `ComputeTarget`, the engine, and
  the settings UI annotation) — a mapping, not a fallback.
- **Result:** BUILD SUCCESS; APK ~177 MB → ~42.5 MB (NNAPI delegate + TFLite
  GPU libs dropped); `libLiteRt.so`, `libLiteRtClGlAccelerator.so`, and
  `liblitert_jni.so` packaged for all ABIs.
- **Still unverified:** on-device runtime (model inference, probe accuracy,
  decode loop, settings persistence) — no device/emulator available.

- **Why:** LiteRT **2.2.0** (released 2026-08-14) is the current Google AI Edge
  runtime. The classic `org.tensorflow.lite` API (2.14.0) is superseded. The
  `CompiledModel` API is the modern standard (CPU/GPU/NPU) and is the
  state-of-the-art path. (The new runtime's `Interpreter` API is **CPU only** —
  no XNNPACK/GPU/NNAPI — so `CompiledModel` is the correct target, not
  `Interpreter`.)
- **What changes:**
  - `app/build.gradle.kts`: replace `org.tensorflow:tensorflow-lite:2.14.0`
    (+ `tensorflow-lite-gpu:2.14.0`) with `com.google.ai.edge.litert:litert:2.2.0`.
  - `engine/AndroidLiteRtEngine.kt`: rewrite against the `CompiledModel` API:
    - `CompiledModel.create(modelPath, CompiledModel.Options(Accelerator.X))`
      instead of `Interpreter(model, options)`.
    - `createInputBuffers()` / `createOutputBuffers()` +
      `compiledModel.run(inputBuffers, outputBuffers)` instead of raw
      `ByteBuffer` + `Interpreter.run(Object, Object)`.
    - Map the `ComputeTarget` enum (CPU / XNNPACK / GPU / NPU) to LiteRT
      `Accelerator` values (CPU / GPU / NPU). **Note:** LiteRT `CompiledModel`
      does not expose XNNPACK as a separate accelerator; the experimental
      YNNPACK CPU accelerator is a build/runtime flag, not a delegate. So
      `XNNPACK` maps to CPU (documented) or is dropped/annotated in the UI.
    - Re-verify the model's **output dtype** (INT8 vs FLOAT32) and the
      dequantization path (`scale`/`zeroPoint`) against the new runtime.
    - Re-verify **delegate probing** (the engine probes available backends and
      reports requested vs actual — the probing API differs on the new runtime;
      never a silent fallback).
  - **Carry over unchanged:** the 256×256 patch tiling, 16px overlap, reflect
    padding, linear-ramp reassembly, and the 8 learned curves (mirroring the
    Rust `ModelRunner`) — that is model logic, not runtime logic.
- **Risk:** contained (behind the `EnhancementEngine` seam). UI/ViewModel/domain
  are untouched. Main risk is the model running correctly on the new runtime
  (dtype/dequant + delegate mapping) — verify on a device.
- **Verify:** `./gradlew :app:assembleDebug` BUILD SUCCESS **and** on-device:
  model loads, produces a visibly enhanced frame, delegate probing reports the
  actual backend.

### Change 2 — harder / can wait (do SECOND): bump the build toolchain — **DONE**

**As-implemented notes (what actually happened vs the plan):**

- **AGP 9.4.0** (latest stable, Sept 2026), **Gradle wrapper 8.9 → 9.7.1**
  (≥ 9.6.0 requirement satisfied).
- **AGP 9.x built-in Kotlin:** the `org.jetbrains.kotlin.android` plugin is
  no longer applied (incompatible with the AGP 9 new DSL). KGP is pinned to
  **2.4.20** via a `buildscript { classpath(...) }` block in
  `app/build.gradle.kts` (it must be on the module buildscript, not the root,
  so KGP and AGP share a buildscript classloader). The Compose compiler
  plugin tracks the KGP version (`org.jetbrains.kotlin.plugin.compose:2.4.20`).
- **`kotlin { compilerOptions { jvmTarget } }` block removed** — with AGP 9.x
  built-in Kotlin, `jvmTarget` defaults to `android.compileOptions
  .targetCompatibility` (17). Kotlin source dirs moved from `java.srcDirs`
  to `kotlin.srcDirs`; the empty `test`/`androidTest` source-set overrides
  were dropped.
- **compileSdk 34 → 36** (required by e.g. activity-compose 1.13.0);
  `targetSdk` stays 34. **JDK 17** still works.
- **Compose BOM 2024.09.02 → 2026.06.01 (UI 1.11.4)** — the newest BOM usable
  with the newest stable SDK platform (android-36). BOM 2026.08.00
  (UI 1.12.x) requires compileSdk 37, not yet published.
- **Bumped to the latest lines usable with compileSdk 36:** activity
  1.9.2 → 1.13.0, navigation 2.8.5 → 2.9.7, lifecycle 2.8.7 → 2.10.0,
  core-ktx 1.12.0 → 1.18.0, material 1.10.0 → 1.14.0, datastore-preferences
  1.1.0 → 1.2.1, test ext junit 1.1.5 → 1.3.0, espresso 3.5.1 → 3.7.0.
  (lifecycle 2.11.0 / navigation 2.10.x / core 1.19.x / Compose UI 1.12.x all
  require compileSdk 37 — deferred until android-37 is published.)
- **`androidx.lifecycle` exclusion dropped from the `litert` dependency** —
  no longer needed now that the app declares its own (newer) lifecycle and
  Compose versions; the `com.google.android.play` / `com.google.android.gms`
  exclusions remain (AiPack download path unused).
- **AGP 9.x breaking changes handled:** `android.enableJetifier` dropped
  (deprecated in AGP 9, app uses only AndroidX); `android.uniquePackageNames`
  set to `false` in `gradle.properties` because `litert` and `litert-api`
  (both from `litert:2.2.0`) share the namespace `com.google.ai.edge.litert`,
  which fails manifest merger while it defaults to `true`.
- **`app/proguard-rules.pro` placeholder added** — AGP 9.x fails the build when
  a declared ProGuard file is missing (`android.proguard.failOnMissingFiles`
  defaults to `true`); no real keep rules yet (`isMinifyEnabled = false`).
- **Result:** `./gradlew :app:assembleDebug` **BUILD SUCCESS** (only
  deprecation warnings); APK ~43.5 MB (`app-debug.apk`).
- **Still unverified:** on-device runtime — no device/emulator available.

- **Why:** the toolchain is several versions behind the latest stable. Bumping
  it keeps the app modern and picks up security/perf fixes.
- **What changes (coupled — do as ONE commit):**
  - **Kotlin** 2.3.0 → **2.4.20** (latest stable, 2026-09-07). 2.3.0 was
    already bumped as a Change 1 prerequisite (`litert-api` metadata).
  - **AGP** 8.5.2 → **9.4.0** (latest stable, Sept 2026). AGP 9.x has
    **breaking changes** (namespace, source-set, and DSL changes) — expect to
    adjust `app/build.gradle.kts` and `settings.gradle.kts`.
  - **Gradle wrapper** 8.9 → **≥ 9.6.0** (hard requirement of AGP 9.4.0).
  - **Compose BOM** 2024.09.02 (Compose UI 1.7.x) → latest (Compose UI **1.12.1**
    line).
  - **lifecycle** 2.8.7 → **2.11.0** (latest stable, 2026-06-17). Requires
    Compose UI 1.7.0+ **and AGP 9.2.0+** (satisfied by the AGP 9.4.0 bump).
  - **Compose compiler:** bundled with the Kotlin compiler since Kotlin 2.0,
    so `org.jetbrains.kotlin.plugin.compose` just tracks the Kotlin version
    and `composeOptions { kotlinCompilerExtensionVersion }` is **not** needed
    (already removed in Change 1).
- **Risk:** higher — AGP 9.x is a major bump with breaking changes, and the
  bumps are coupled (lifecycle 2.11.0 needs AGP 9.2.0+; Compose BOM needs a
  matching compiler plugin; Kotlin 2.4.x may surface new warnings/strictness).
  Can wait; do it **after** Change 1 so the ML migration is isolated and
  verifiable on its own.
- **Verify:** `./gradlew :app:assembleDebug` BUILD SUCCESS — **DONE** (see
  as-implemented notes above) **and** on-device smoke test (UI navigation,
  settings persistence, image + video enhancement) — pending a device.

### Ordering rationale

- **Change 1 first** — it is the *better* change (state-of-the-art ML) and is
  self-contained (isolated behind `EnhancementEngine`), so it can be verified on
  its own.
- **Change 2 second** — it is the *harder* change (coupled AGP 9.x + Kotlin +
  Compose + lifecycle bumps with breaking changes) and can wait; doing it after
  Change 1 keeps the two concerns separate and each independently verifiable.

### Also missing / to keep in mind

- **Model compatibility:** the `.tflite` (Zero-DCE, int8) is unchanged; confirm
  it loads and infers correctly on LiteRT 2.2.0 (op coverage, dtype, dequant)
  **on a device** — compile-level only so far.
- ~~**XNNPACK mapping:**~~ resolved — `ComputeTarget.XNNPACK` maps to CPU,
  documented in `ComputeTarget`, the engine, and the settings UI annotation.
- ~~**Delegate probing API:**~~ resolved — `Environment.getAvailableAccelerators()`
  + requested-vs-actual reporting in `configure` (never silent).
- **Commit hygiene:** each change is its own commit with an accurate message.
