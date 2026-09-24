# OpenLLVE Models

This directory is the **single source of truth** for all model files.
Model files are external to the application: OpenLLVE is an engine that
plugs in a `.tflite` model and runs it — model training, conversion, and
validation happen in the upstream model repository, not in this repo.

The Android app does **not** store its own copy:
`app/platforms/android/src/main/assets/models/` contains **symlinks** to the
files here. Gradle's `mergeAssets` task follows symlinks (`copyFollowsLinks`
defaults to `true`), so the packaged APK still contains the real model bytes.
On a PC, the Rust core (`ModelRunner`, see `docs/dev/architecture/ARCHITECTURE.md` §11)
loads the file directly from this path — no asset machinery needed.

## Default test model

`external/models/zero-dce-int8.tflite` is the single model committed to this
repo so the app and the benchmark harness always have something real to run
against (symlinked into app assets as
`app/platforms/android/src/main/assets/models/zero-dce-int8.tflite`):

| Property | Value |
| --- | --- |
| Model | Zero-DCE (DCE-Net) low-light image enhancement, INT8-quantized |
| Source | [raspberrypi/AI_enhance](https://github.com/raspberrypi/AI_enhance) (BSD-2-Clause) |
| Input | `(1, 256, 256, 4)` float32 — RGB normalized to `[0, 1]` plus a brightness-guidance channel |
| Output | `(1, 256, 256, 24)` float32 — 8 iterations of per-channel curve parameters |
| Size | ~58 KB |

### Fixed input shape (256×256)

The upstream model ships with a **1×1×1** input tensor that is meant to be
resized at runtime (`TfLiteInterpreterResizeInputTensor`). The LiteRT C
runtime used by the Rust core (`tflite-c-rs`, per `docs/dev/architecture/ARCHITECTURE.md` §11) segfaults on that
resize call for this model, so the committed file is a **re-derived** variant
with the input shape patched in place from `[1, 1, 1, 4]` to
`[1, 256, 256, 4]` (a 16-byte flatbuffer edit; no retraining, no weight
change). The rest of the graph is byte-identical to upstream.

This is verified bit-exact: running the patched model (no resize) through the
LiteRT runtime produces output **identical** to running the original model
with a resize to 256×256. The Rust `ModelRunner` therefore never calls the
resize API — it loads the static-shape model and runs it directly.

Frames larger than 256×256 are handled by the pipeline itself: it tiles them
into 256×256 patches with 16 px overlap (reflect padding at edges, linear-ramp
reassembly), mirroring the upstream `network.py`.

> **Known workaround, not a fix.** The resize segfault's root cause is not yet
> identified. The in-place flatbuffer patch is a workaround. Follow-up: either
> re-export the model with the correct static input shape, or root-cause the
> `TfLiteInterpreterResizeInputTensor` segfault (tracked in `TODO.md`, P1.2).

`zero-dce-int8-upstream.tflite` keeps the **original** 1×1×1 file (byte
identical to upstream) for provenance. It only works with runtimes that
support `ResizeInputTensor` (e.g. the LiteRT Python runtime); the Rust core
uses the fixed-shape file above.

## Model manifests

Every model file has a **manifest** next to it: `<model-file>.manifest.json`
(for submodule models: a `manifest.json` at the submodule repo root).
The manifest is the single machine-readable description of a model — what
`BenchmarkRun` (P3.6) will record as model id/version, and what model
validation (P1.2) checks against.

Fields (all required unless noted):

| Field | Meaning |
| --- | --- |
| `manifest_version` | schema version (currently `1`) |
| `id` | stable identifier, e.g. `zero-dce-int8` |
| `name` | human-readable model name |
| `version` | model version (upstream version or local re-derivation) |
| `file` | the `.tflite` file this manifest describes |
| `sha256` | file hash (provenance / tamper check) |
| `source` | upstream repository URL |
| `license` | model license |
| `input` / `output` | `shape` (ints), `dtype`, `channels` (what the channels mean) |
| `quantization` | `int8`, `fp16`, `fp32`, … |
| `supported_delegates` | delegates verified to work (`cpu`, `gpu`, `nnapi`, …) |
| `usable` | `false` if the file is provenance-only (optional) |
| `notes` | anything else (workarounds, caveats) (optional) |

## Adding new models (external submodules)

New models are added as **external git submodules** under this directory.
Only the default test model (and its upstream provenance copy) is committed
as raw files in the main repo; everything else stays in its upstream
repository.

```bash
scripts/add-model-submodule.sh <name> <repo-url>
# e.g. scripts/add-model-submodule.sh mblLEN https://github.com/Lvfeifan/MBLLEN
```

The script adds the submodule and creates the symlink in
`app/platforms/android/src/main/assets/models/` automatically. Afterwards:

1. Validate the model in its upstream repository (e.g. load it with the
   LiteRT Python runtime and check the input/output tensor specs).
2. Add the model's `manifest.json` (see "Model manifests") to the submodule
   repo.
3. Record the model's I/O shape and license in
   `app/platforms/android/src/main/assets/models/README.md`.
