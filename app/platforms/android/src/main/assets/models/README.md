# OpenLLVE Models (app assets)

This directory holds the `.tflite` model files the Android app loads from
assets. The app is external to model development: it just plugs in a model
and runs it.

## Default test model

- **`zero-dce-int8.tflite`** — Zero-DCE (DCE-Net) low-light image enhancement,
  INT8-quantized. Source: [raspberrypi/AI_enhance](https://github.com/raspberrypi/AI_enhance)
  (BSD-2-Clause).
  - Input: `(H, W, 4)` float32 — RGB in `[0, 1]` plus a brightness-guidance
    channel (default patch 256×256).
  - Output: `(H, W, 24)` float32 — 8 iterations of per-channel curve
    parameters.
  - ~58 KB.

This is the only model committed as a raw file in the main repo. It exists so
the app and the benchmark harness always have a real model to test against.

## Adding more models

New models are pulled in as **external git submodules** under `external/models/`
at the repo root — see [`external/models/README.md`](../../../../external/models/README.md). After a
model is validated upstream, copy its `.tflite` here and record its I/O shape
and license in this README.
