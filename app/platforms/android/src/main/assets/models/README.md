# Model assets (app side)

The Android app loads `.tflite` files from here, but the files are **symlinks**
into [`../../../../../../../external/models/README.md`](../../../../../../../external/models/README.md)
— the app keeps no copies. Gradle's `mergeAssets` follows symlinks
(`copyFollowsLinks` defaults to `true`), so the APK still gets real model bytes.

Default model `zero-dce-int8.tflite` (Zero-DCE / DCE-Net, INT8-quantized, ~58 KB):

- Input `(1, 256, 256, 4)` float32 — RGB in `[0, 1]` plus a brightness-guidance
  channel.
- Output `(1, 256, 256, 24)` float32 — 8 iterations of per-channel curve
  parameters.

It is the only model committed as a raw file, so the app and the benchmark
harness always have something real to run. New models come in as external
submodules under `external/models/` (`scripts/add-model-submodule.sh` also
creates the symlink here); record each model's I/O shape and license in that
README.
