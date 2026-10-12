# OpenLLVE — Architecture & Design Suggestions

*Design-level follow-ups that are still open. Implemented suggestions and the
premature/push-back ones (hardware-buffer lock protocol, ring-buffer temporal
state, KMP adapter contracts, iOS symmetry, async pipeline in Rust, a first-class
benchmark module / `BenchmarkRun` record, the model manifest) are not listed here —
they live in the module READMEs and `docs/`.*

The layering intent (Rust compute / C ABI / platform adapters / KMP shared) is sound;
these notes are about making the design hold up once real models and real video flow
through it.

## 1. Testing strategy

- **Property tests for filters** — EWMA output stays within
  `[min(prev, curr), max(prev, curr)]` per pixel, blend output is a convex combination.
  Landed in `core/tests/filters_properties.rs`.
- **Golden-frame tests** — fixed input frame → hash of output, pinned per model file.
  This is what makes CPU/GPU/NPU comparison *valid* (same model, same input, comparable
  outputs). A hash pinning the app's default filter weights exists; per-model-file pins
  are still open.
- **FFI fuzzing** — landed on both boundaries (`core/tests/ffi_fuzz.rs` for the C ABI,
  `bridge/android/src/frames.rs::ffi_fuzz` for adversarial JVM `Int` tuples).

## 2. Remaining Rust tooling & crates

| Tool / crate | Purpose | Notes |
| --- | --- | --- |
| `proptest` | property tests for filters | added |
| `miri` (tool, not a dep) | UB detection in the raw-pointer FFI | `cargo miri test` as a CI job — still open |
| `static_assertions` | compile-time invariants | optional; skip until layout constants grow |
| `cargo-ndk` | cross-compile the cdylib to Android ABIs | installed; `scripts/build-bridge.sh` produces `libbridge.so` for `jniLibs/` |

## Quick ranking (effort × value)

| Suggestion | Effort | Verdict |
| --- | --- | --- |
| Golden-frame tests per model file | medium | **the quality bar** — makes backend comparison meaningful |
| `miri` CI job | medium | worth once the FFI grows; slow to run |

### TL;DR

The keystone pieces are in place (in-Rust model runner, `Frame` type, out-buffer API,
hardened FFI, property + fuzz tests). Benchmarking stays deliberately minimal
(`cargo bench` + `BenchmarkMetrics` + the results table in `docs/benchmarks/`). What
remains is the quality bar: per-model golden frames, and `miri` in CI.
