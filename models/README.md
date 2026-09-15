# OpenLLVE Models

Model files are **external to the application**. OpenLLVE is an engine that
plugs in a `.tflite` model and runs it — model training, conversion, and
validation happen in the upstream model repository, not in this repo.

## Default test model

`app/platforms/android/src/main/assets/models/zero-dce-int8.tflite` is the
single model committed to this repo so the app and the benchmark harness
always have something real to run against:

| Property | Value |
| --- | --- |
| Model | Zero-DCE (DCE-Net) low-light image enhancement, INT8-quantized |
| Source | [raspberrypi/AI_enhance](https://github.com/raspberrypi/AI_enhance) (BSD-2-Clause) |
| Input | `(H, W, 4)` float32 — RGB normalized to `[0, 1]` plus a brightness-guidance channel (default patch 256×256) |
| Output | `(H, W, 24)` float32 — 8 iterations of per-channel curve parameters |
| Size | ~58 KB |

## Adding new models (external submodules)

New models are added as **external git submodules** under this directory.
Only the default test model above is committed as a raw file in the main
repo; everything else stays in its upstream repository.

```bash
scripts/add-model-submodule.sh <name> <repo-url>
# e.g. scripts/add-model-submodule.sh mblLEN https://github.com/Lvfeifan/MBLLEN
```

After adding the submodule:

1. Validate the model in its upstream repository (e.g. load it with the
   LiteRT/TFLite Python runtime and check the input/output tensor specs).
2. Copy the chosen `.tflite` into
   `app/platforms/android/src/main/assets/models/` and commit it, so the app
   can load it from assets.
3. Record the model's I/O shape and license next to the file (see
   `assets/models/README.md`).
