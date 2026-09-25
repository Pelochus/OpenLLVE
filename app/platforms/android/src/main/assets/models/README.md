# OpenLLVE Models (app assets)

This directory is where the Android app loads `.tflite` model files from
assets. The files here are **symlinks** to the single source of truth in
[`external/models/`](../../../../../../../../external/models/README.md) — the app
does not keep its own copies. Gradle's `mergeAssets` task follows symlinks
(`copyFollowsLinks` defaults to `true`), so the packaged APK contains the real
model bytes.

The app is external to model development: it just plugs in a model and runs
it.

## Default test model

- **`zero-dce-int8.tflite`** — Zero-DCE (DCE-Net) low-light image enhancement,
  INT8-quantized. Source: [raspberrypi/AI_enhance](https://github.com/raspberrypi/AI_enhance)
  (BSD-2-Clause).
  - Input: `(H, W, 4)` float32 — RGB in `[0, 1]` plus a brightness-guidance
    channel (default patch 256×256).
  - Output: `(H, W, 24)` float32 — 8 iterations of per-channel curve
    parameters.
  - ~58 KB.

This is the only model committed to the repo (as a raw file in
`external/models/` and a symlink here). It exists so the app and the
benchmark harness always have a real model to test against.

## Adding more models

New models are pulled in as **external git submodules** under
`external/models/` — see
[`external/models/README.md`](../../../../../../../../external/models/README.md).
`scripts/add-model-submodule.sh` adds the submodule and creates the symlink
here automatically. After a model is validated upstream, record its I/O shape
and license in this README.
