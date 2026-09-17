# OpenLLVE — Android App TODO (app-side only)

This file tracks **Android/app-side** work only. It does not duplicate the
general Rust TODOs in `TODO.md`. It records the current status, what is
intentionally deferred, the remaining runtime work, and a prompt to continue
the work in a new session.

---

## 1. Current status (TL;DR)

- **The app compiles cleanly.** `./gradlew :app:assembleDebug` is **BUILD
  SUCCESS** (only deprecation warnings, no compile errors).
- **Currently on the classic `org.tensorflow.lite` API (2.14.0).** The engine
  and media layers are reconciled with the real `org.tensorflow.lite`,
  `MediaCodec`/`MediaExtractor`, and DataStore APIs. The Compose Compiler
  Gradle plugin is in place (required by Kotlin 2.0).
- **Next work: two state-of-the-art dependency upgrades (§7):** (1) migrate the
  engine to LiteRT 2.2.0 `CompiledModel`, (2) bump the build toolchain
  (Kotlin 2.4.20 / AGP 9.4.0 / Compose BOM / lifecycle 2.11.0). Ordered
  easy→hard.
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
| **`proguard-rules.pro`** | Referenced by the release build type but not required for `assembleDebug`; left as-is. |

---

## 3. Runtime verification (NOT done — no device/emulator)

The compile is done (BUILD SUCCESS). The remaining runtime work:

- Confirm the model actually loads and produces a visibly enhanced frame.
- Confirm delegate probing reports correct support (esp. NPU/NNAPI, which can
  silently fall back to CPU — documented limitation).
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
> (no compile errors; only deprecation warnings). The app currently uses the
> classic `org.tensorflow.lite` API (2.14.0). The next work is the two planned
> upgrades in §7, in order (easy→hard).
>
> 1. Read `TODO-app.md` (this file), `TODO.md`, `IMPROVEMENTS.md`,
>    `docs/ARCHITECTURE.md`, and `core/README.md`. Do **not** modify Rust, add
>    C/C++, or start iOS work.
> 2. **Change 1 (§7, do FIRST): migrate the engine to LiteRT 2.2.0
>    `CompiledModel`.**
>    - Replace `org.tensorflow:tensorflow-lite:2.14.0` (and the `-gpu` dep)
>      with `com.google.ai.edge.litert:litert:2.2.0`.
>    - Rewrite `AndroidLiteRtEngine` against the `CompiledModel` API
>      (`CompiledModel.create`, `createInputBuffers`/`createOutputBuffers`,
>      `run`). Map `ComputeTarget` to LiteRT `Accelerator` (CPU/GPU/NPU;
>      XNNPACK → CPU, documented).
>    - Verify the model's output dtype (INT8 vs FLOAT32) and the dequantization
>      path (`scale`/`zeroPoint`).
>    - Re-derive delegate probing (requested vs actual, never a silent
>      fallback).
>    - Keep the 256×256 patch tiling, reflect padding, linear-ramp reassembly,
>      and the 8 learned curves (model logic, not runtime logic).
>    - Commit as its own commit with an accurate message.
> 3. **Change 2 (§7, do SECOND): bump the build toolchain.**
>    - Kotlin 2.0.21 → 2.4.20; AGP 8.5.2 → 9.4.0; Compose BOM → latest
>      (UI 1.12.1 line); lifecycle 2.8.7 → 2.11.0; Compose Compiler plugin →
>      2.4.20 (and `composeOptions { kotlinCompilerExtensionVersion }` must
>      match).
>    - Handle AGP 9.x breaking changes (namespace, source-set, DSL).
>    - Commit as its own commit.
> 4. **Runtime verification (§3)** — if a device/emulator is available, verify:
>    model loads + visibly enhanced frame; MP4 decode loop synchronized
>    original↔enhanced; delegate selection reports the actual backend; settings
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

### Change 1 — easier / better (do FIRST): migrate the engine to LiteRT 2.2.0 `CompiledModel`

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

### Change 2 — harder / can wait (do SECOND): bump the build toolchain

- **Why:** the toolchain is several versions behind the latest stable. Bumping
  it keeps the app modern and picks up security/perf fixes.
- **What changes (coupled — do as ONE commit):**
  - **Kotlin** 2.0.21 → **2.4.20** (latest stable, 2026-09-07).
  - **AGP** 8.5.2 → **9.4.0** (latest stable, Sept 2026). AGP 9.x has
    **breaking changes** (namespace, source-set, and DSL changes) — expect to
    adjust `app/build.gradle.kts` and `settings.gradle.kts`.
  - **Compose BOM** 2024.09.02 (Compose UI 1.7.x) → latest (Compose UI **1.12.1**
    line).
  - **lifecycle** 2.8.7 → **2.11.0** (latest stable, 2026-06-17). Requires
    Compose UI 1.7.0+ **and AGP 9.2.0+** (satisfied by the AGP 9.4.0 bump).
  - **Compose Compiler plugin** version must track the Kotlin version
    (`org.jetbrains.kotlin.plugin.compose` → **2.4.20**), and
    `composeOptions { kotlinCompilerExtensionVersion }` must match.
- **Risk:** higher — AGP 9.x is a major bump with breaking changes, and the
  bumps are coupled (lifecycle 2.11.0 needs AGP 9.2.0+; Compose BOM needs a
  matching compiler plugin; Kotlin 2.4.x may surface new warnings/strictness).
  Can wait; do it **after** Change 1 so the ML migration is isolated and
  verifiable on its own.
- **Verify:** `./gradlew :app:assembleDebug` BUILD SUCCESS **and** on-device
  smoke test (UI navigation, settings persistence, image + video enhancement).

### Ordering rationale

- **Change 1 first** — it is the *better* change (state-of-the-art ML) and is
  self-contained (isolated behind `EnhancementEngine`), so it can be verified on
  its own.
- **Change 2 second** — it is the *harder* change (coupled AGP 9.x + Kotlin +
  Compose + lifecycle bumps with breaking changes) and can wait; doing it after
  Change 1 keeps the two concerns separate and each independently verifiable.

### Also missing / to keep in mind

- **Model compatibility:** the `.tflite` (Zero-DCE, int8) is unchanged; confirm
  it loads and infers correctly on LiteRT 2.2.0 (op coverage, dtype, dequant).
- **XNNPACK mapping:** LiteRT `CompiledModel` has no XNNPACK accelerator; decide
  whether `ComputeTarget.XNNPACK` maps to CPU (documented) or is removed from the
  selector. Update the UI annotation accordingly.
- **Delegate probing API:** the new runtime's backend-probing API differs from the
  classic `org.tensorflow.lite` one; re-derive the "requested vs actual" report
  so it never silently falls back.
- **Commit hygiene:** each change is its own commit with an accurate message
  (the current HEAD is already `Android vertical slice: fix build to BUILD
  SUCCESS (commit 2/2)`).
