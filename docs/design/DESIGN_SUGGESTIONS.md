# OpenLLVE — Architecture & Design Suggestions

*Scope: design-level follow-ups that are still open. The already-implemented
suggestions (in-Rust `ModelRunner`, `Frame` type, out-buffer API, FFI
hardening, `Pipeline`/`TemporalMode` rename) and the push-back/premature ones
(hardware-buffer lock protocol, ring-buffer temporal state, KMP adapter
contracts, iOS symmetry, async pipeline in Rust) have been removed — see
`IMPROVEMENTS.md` and the git history. The existing layering intent (Rust
compute / C ABI / platform adapters / KMP shared) is sound; these suggestions
are about making the design hold up when real models and real video flow
through it.*

---

## 1. Make the benchmark a first-class module, not an accumulator

`BENCHMARK_METHODOLOGY.md` describes a rich protocol (warm-up exclusion,
thermal stability, environment capture, per-run metadata) but
`BenchmarkMetrics` is a flat `Vec<f64>` with mean/median/p99/fps. Suggested
shape:

```rust
struct BenchmarkConfig { model_id, model_version, pipeline, toppings, resolution, delegate }
struct BenchmarkRun {
    config: BenchmarkConfig,
    device: DeviceInfo,          // model, SDK, chip
    thermal: Vec<(f64, f32)>,    // (t, temperature) samples
    latencies: Vec<f64>,         // warm-up frames excluded
}
impl BenchmarkRun {
    fn median(&self) -> f64;
    fn p99(&self) -> f64;
    fn fps(&self) -> f64;
    fn thermal_drift(&self) -> f64;  // late-run vs early-run latency ratio
}
```

- Keep `f64` accumulation and warm-up exclusion (already in
  `BenchmarkMetrics`) and lift them into the run record.
- Persist `BenchmarkRun` as JSON/CSV (serde) so runs are comparable and
  reproducible — the methodology's "document each run with metadata" becomes
  a data format, not a note.
- Needs the model manifest (§2) so runs can name the model id/version.

## 2. Model manifest for reproducible benchmarks

The benchmark thesis (CPU vs GPU vs NPU, repeatable runs) requires models to
be *identified*, not just loaded. Suggestion:

- Ship models with a manifest (`external/models/manifest.json` or a Rust
  `ModelSpec`): name, version, input/output shapes, quantization (INT8/FP16),
  supported delegates, expected latency class.
- `BenchmarkRun` records the model id/version (§1) — otherwise two runs of
  "Zero-DCE" with different weights are indistinguishable.

## 3. Threading model: decide and document it

Real-time video needs three flows (capture → process → render). Recommended
design decision:

- **The Rust core stays synchronous** (one `process` call per frame, no
  internal threads) — simplest, most testable, and the benchmark stays pure.
- **The platform owns the threads**: capture thread, a dedicated processing
  thread, render thread.
- **Bounded frame queue with drop-oldest** when processing falls behind
  (standard real-time video behavior; avoids unbounded latency growth).

Record this as an ADR. The alternative (async pipeline inside Rust) buys
nothing at this stage and complicates the FFI.

## 4. Testing strategy (design-level)

- **Property tests for filters**: EWMA output must stay within
  `[min(prev,curr), max(prev,curr)]` per pixel; blend output is a convex
  combination. Cheap, catches the class of bugs in the current stubs.
  (Tracked as P3.8.)
- **Golden-frame tests**: fixed input frame → hash of output, pinned per model
  version (needs the model manifest, §2). This is what makes CPU/GPU/NPU
  comparison *valid* (same model, same input, comparable outputs).
- FFI-fuzzing, miri, and JNI round-trip tests wait for P1.1 (FFI wiring into
  the app).

## 5. Naming (done)

Docs and code are standardized on **LiteRT** (Google's rename of
TensorFlow Lite, Sept 2024). The literal names `tflite-c-rs`,
`libtensorflowlite_c`, the `.tflite` file extension, and the `TfLite*` C
API are kept as-is — they are the real crate/library names, not stale
spelling.

## 6. Remaining Rust tooling & crates

| Tool / crate | Purpose | Notes |
| --- | --- | --- |
| `serde` + `serde_json` | persist `BenchmarkRun` | makes benchmark runs comparable/reproducible (§1) |
| `proptest` | property tests for filters | EWMA stays within input bounds, blend is a convex combination (§4) |
| `miri` (tool, not a dep) | UB detection in unsafe FFI code | `cargo miri test` in CI — exactly what `ffi.rs`/`frame.rs` raw-pointer code needs |
| `static_assertions` | compile-time invariants | optional; e.g. assert layout/stride constants |
| `cargo-ndk` / `cargo-mobile2` | cross-compile the cdylib to Android ABIs | not a crate; CI job that produces `libopenllve_core.so` (P1.1) |

## Quick ranking (effort × value)

| Suggestion | Effort | Verdict |
| --- | --- | --- |
| Model manifest (model id/version/shapes) | easy | **clear win** — cheap, unblocks reproducible benchmarks |
| Threading ADR | easy (doc only) | **clear win** — nearly free |
| Property tests for filters | easy | **clear win** — nearly free |
| Benchmark `Run` record (median, warm-up, thermal, JSON) | medium | **clear win** — docs already promise it |
| Golden-frame tests | medium | now feasible — needs the model manifest (§2) |

## Prioritization

**Do now** (shapes the next implementation step):

1. Model manifest (§2) — cheap, unblocks reproducible benchmarking.

**Design before the next milestone** (needed once real video flows):

1. Benchmark session/run record (§1).
2. Threading ADR (§3).

**Later** (quality bar once the pipeline is live):

1. Property + golden-frame tests (§4), naming cleanup (§5), crate/tooling setup (§6).

### TL;DR

The keystone pieces are in place (in-Rust model runner, `Frame` type,
out-buffer API, hardened FFI). What remains is the benchmark data model
(`BenchmarkRun` + model manifest), a short threading ADR, and the quality bar
(property/golden-frame tests, remaining crates).
