# OpenLLVE — Architecture & Design Suggestions

*Scope: design-level suggestions only (not build fixes — see `IMPROVEMENTS.md` for those). The existing layering intent (Rust compute / C ABI / platform adapters / KMP shared) is sound; these suggestions are about making the design hold up when real models and real video flow through it.*

---

## 1. The biggest open design decision: where does inference live?

Today the strategy has **no model** (`process` is an identity copy), and the only "inference engine" in the repo is `LiteRTInferenceEngine.kt` on the Android side. That placement contradicts the project's own rule of thumb: *"if the logic can be executed on a standard PC with `cargo test` / `cargo bench`, it is in the correct layer."*

**Suggestion: introduce a `ModelRunner` trait in the Rust core and make it the seam between "pipeline" and "backend".**

```rust
trait ModelRunner {
    fn run(&mut self, input: &Frame, output: &mut Frame) -> Result<()>;
    fn reset(&mut self);
    fn metadata(&self) -> RunnerMetadata; // model id, version, in/out shapes, delegate
}
```

Candidate backends (all implementable today):

| Backend | How | Notes |
| --- | --- | --- |
| `LiteRtRunner` | `tflite-c-rs` (wraps `libtensorflowlite_c`, loaded dynamically via `libloading` — no build-time link) | Runs on Android **and on a PC with the CPU delegate**, so `cargo test`/`cargo bench` work — satisfies the tenet |
| `CoreMlRunner` | Core ML C API (`.mlmodelc`) | iOS |
| `ReferenceRunner` / `IdentityRunner` | pure Rust | unit tests, baseline benchmarks, FFI round-trip tests |

Then a strategy becomes **runner + toppings** composition:

```rust
struct LliePipeline {
    runner: Box<dyn ModelRunner>,
    ewma: Option<EwmaFilter>,
    blend: Option<FrameBlendFilter>,
}
```

Why this matters:

- The CPU/GPU/NPU comparison becomes a `Delegate` parameter on the runner (CPU / GPU / NNAPI on Android, CoreML/Metal on iOS) — which is literally the project's benchmark thesis.
- The benchmark measures a real inference path on a PC, not a memcpy.
- Kotlin stops owning inference; `LiteRTInferenceEngine.kt` and `LlieEwmaEnhancer.kt` both disappear (the Kotlin enhancer currently duplicates `EwmaFilter`/`FrameBlendFilter`).
- iOS gets symmetry: same pipeline, different runner.

**Alternative considered:** the platform supplies the runner via a C callback (Rust defines the contract, Kotlin/Swift implements it). Trade-off: keeps the Rust crate free of any TFLite dependency, but breaks the "cargo test on a PC" tenet, makes the benchmark PC-side impossible, and pushes tensor handling into two platform languages. **Recommendation: in-Rust runners.**

*Worth writing down as a short ADR (`docs/adr/0001-inference-runner-placement.md`) — it's the decision every later piece hangs on.*

---

## 2. Make `Frame` a first-class type (replace `&[f32]`)

Everything currently operates on flat `&[f32]`. Low-light enhancement is a *color image* problem (per-channel gamma, white balance, spatial structure), so a flat slice is the wrong abstraction:

```rust
struct Frame {
    width: u32,
    height: u32,
    channels: u8,      // 1 (gray) | 3 (RGB)
    stride: usize,     // bytes per row
    data: *mut u8,     // borrowed, caller-owned
    format: PixelFormat,
}
```

Consequences:

- Filters/strategies become `fn apply(&mut self, input: &Frame, output: &mut Frame) -> Result<()>` — dimension mismatches become checked errors instead of silent zip truncation.
- `NativeFrameHandle` finally has a job (it's currently dead code); either fold it into `Frame` or make it the platform-side wrapper that *produces* a `Frame`.
- The FFI becomes honest: `openllve_process_frame(strategy, width, height, channels, stride, in_ptr, out_ptr)` instead of an untyped `input_len`.
- Model I/O mapping (e.g. Zero-DCE expects a normalized HWC float tensor) becomes explicit in the runner, not implicit in a Kotlin array.

---

## 3. Out-buffer API — stop returning `Vec<f32>` per frame

Current `Pipeline::process(&mut self, input: &[f32]) -> Result<Vec<f32>>` allocates 2–3 full-frame `Vec`s per frame (input copy, EWMA output, blend output) — at 1080p RGB that's ~36 MB of churn per frame. The FFI already has an output buffer; make the Rust API match it:

```rust
fn process(&mut self, input: &Frame, output: &mut Frame) -> Result<()>;
```

- Platform pre-allocates two frame buffers and ping-pongs them (double buffering) — steady-state zero allocation.
- Enables in-place processing where a stage allows it.
- Makes the benchmark's "inference path only" measurement cleaner (no allocator noise in the hot path).

---

## 4. Zero-copy done right: define the hardware-buffer lock protocol

"Zero-copy" is only real if the buffer is actually shared. On Android, camera frames are `AHardwareBuffer` (GPU-backed); a raw pointer is only valid while the buffer is **locked** (`AHardwareBuffer_lock`), and on iOS `CVPixelBuffer` has the analogous base-address + pixel-data lifetime rules.

Design suggestion: document and encode the protocol explicitly —

1. Platform locks the hardware buffer before the FFI call.
2. The pointer + stride + dimensions are passed to Rust; the buffer must remain locked for the whole call.
3. Platform unlocks after the call; Rust never caches the pointer across calls.

`Frame` (or `NativeFrameHandle`) should carry this contract (e.g. a `locked: bool` invariant or a doc-level ownership rule), and the FFI docs should state it. Otherwise the "zero-copy" claim is aspirational and the unsafe boundary is unsound.

---

## 5. Generalize temporal state

Both strategies hold a single `Option<Vec<f32>>` of the previous frame, and `LliePipeline` holds *two* temporal states (its own `previous_frame` for blending **and** the EWMA's internal previous frame) — duplicated state that can drift.

Suggestions:

- One shared temporal buffer pool per pipeline (input history + filter state), sized by a `history: usize`.
- Support a **ring buffer of N frames** rather than exactly one previous frame — recurrent/temporal LLVE models typically consume a small window (2–4 frames), and this generalizes without API change.
- `reset()` semantics = stream boundary (camera restart, scene cut, resolution change). Make that explicit in the doc; the platform must call it at the right moments.

---

## 6. Make the benchmark a first-class module, not an accumulator

`BENCHMARK_METHODOLOGY.md` describes a rich protocol (warm-up exclusion, thermal stability, environment capture, per-run metadata) but `BenchmarkMetrics` is a flat `Vec<f32>` with mean/fps/p99. Suggested shape:

```rust
struct BenchmarkConfig { model_id, model_version, strategy, toppings, resolution, delegate }
struct BenchmarkRun {
    config: BenchmarkConfig,
    device: DeviceInfo,          // model, SDK, chip
    thermal: Vec<(f64, f32)>,    // (t, temperature) samples
    latencies: Vec<f64>,         // warm-up frames excluded
}
impl BenchmarkRun {
    fn median(&self) -> f64;     // the methodology doc promises a median; the API has none
    fn p99(&self) -> f64;
    fn fps(&self) -> f64;
    fn thermal_drift(&self) -> f64;  // late-run vs early-run latency ratio
}
```

- `f64` for accumulation (f32 loses precision over a long run).
- Warm-up exclusion as an explicit part of the session (the docs require it; the code has no support).
- A **headless PC benchmark** (ReferenceRunner or CPU-delegate LiteRtRunner) so `cargo bench` validates the harness before any device work — this is the payoff of suggestion #1.
- Persist `BenchmarkRun` as JSON/CSV (serde) so runs are comparable and reproducible — the methodology's "document each run with metadata" becomes a data format, not a note.

---

## 7. FFI/ABI design hardening

The opaque-handle + `Box::into_raw` pattern is the right shape; refine it:

- **Generate the C header with `cbindgen`** instead of hand-writing it — the header's typedef names (`OpenLlvePipeline`) already drift from the Rust types (`OpenLlvePipelineEnum`).
- **Error codes**: return `i32` (stable enum) from `process` instead of `bool`, plus `const char* openllve_last_error()` with a documented lifetime (valid until next call). A bool can't distinguish "bad pointer" from "dimension mismatch" from "model failed".
- **ABI versioning**: `uint32_t openllve_abi_version()` so the app can fail fast on a mismatched `.so`.
- **Frame-based call** (see #2) instead of flat `len`.
- **Thread-safety contract**: stateful handles are `!Send`; document "one processing thread per handle" (ties into #9).

---

## 8. KMP shared layer: define adapter contracts, not just "shared logic"

If the KMP tenet is kept, the shared module should be a small set of **platform-adapter interfaces** that Android and iOS implement:

```kotlin
interface VideoSource { fun start(); fun nextFrame(): Frame; fun stop() }      // camera or test source
interface FrameSink   { fun present(frame: Frame) }                            // preview surface
interface AcceleratorProvider { fun select(delegate: Delegate): Delegate }    // CPU/GPU/NPU availability
interface BenchmarkContract { fun start(cfg: BenchmarkConfig); fun stop(): BenchmarkRun }
```

plus shared value types: `StrategyConfig` (strategy, alpha, beta, history), `Delegate`, `BenchmarkConfig/Run`. That makes "shared app logic" concrete and testable (fake `VideoSource` in unit tests) instead of a vague folder of READMEs.

---

## 9. Threading model: decide and document it

Real-time video needs three flows (capture → process → render). Recommended design decision:

- **The Rust core stays synchronous** (one `process` call per frame, no internal threads) — simplest, most testable, and the benchmark stays pure.
- **The platform owns the threads**: capture thread, a dedicated processing thread, render thread.
- **Bounded frame queue with drop-oldest** when processing falls behind (standard real-time video behavior; avoids unbounded latency growth).

Record this as an ADR. The alternative (async pipeline inside Rust) buys nothing at this stage and complicates the FFI.

---

## 10. Model manifest for reproducible benchmarks

The benchmark thesis (CPU vs GPU vs NPU, repeatable runs) requires models to be *identified*, not just loaded. Suggestion:

- Ship models with a manifest (`external/models/manifest.json` or a Rust `ModelSpec`): name, version, input/output shapes, quantization (INT8/FP16), supported delegates, expected latency class.
- `BenchmarkRun` records the model id/version (suggestion #6) — otherwise two runs of "Zero-DCE" with different weights are indistinguishable.
- This also fixes the `model-validation.yml` design: validation reads the manifest and checks each listed model's tensor signature, failing when the manifest and files disagree.

---

## 11. Testing strategy (design-level)

- **Property tests for filters**: EWMA output must stay within `[min(prev,curr), max(prev,curr)]` per pixel; blend output is a convex combination. Cheap, catches the class of bugs in the current stubs.
- **Golden-frame tests** once a real model lands: fixed input frame → hash of output, pinned per model version. This is what makes CPU/GPU/NPU comparison *valid* (same model, same input, comparable outputs).
- **FFI boundary tests**: null handles, `output_len < input_len`, dimension mismatches, double-free — the unsafe boundary is exactly where undefined behavior hides. A small C test harness or Kotlin JNI test covers this.
- **Benchmark self-test**: run the harness with `ReferenceRunner` on CI and assert the latency distribution is sane (catches warm-up/measurement bugs before device runs).

---

## 12. Naming / domain-model cleanup (minor)

- "Strategy" currently conflates *model family* and *temporal mode*. Consider `Pipeline` (composition) with a `TemporalMode { Stateless, Recurrent }` — reads better as model families grow.
- Standardize LiteRT vs TFLite naming in docs/code (the Maven artifact is still `org.tensorflow:tensorflow-lite` even after the rename to LiteRT — note that once so contributors don't "fix" the artifact id to a non-existent one).

---

## Quick ranking (effort × value)

| Suggestion | Effort | Verdict |
| --- | --- | --- |
| Model manifest (model id/version/shapes) | easy | **clear win** — cheap, unblocks reproducible benchmarks |
| `cbindgen` + error codes + ABI version | easy | **clear win** — prevents ABI drift, small change |
| Out-buffer API (`process(input, output)`) | easy | **clear win** — kills per-frame allocations, matches existing FFI |
| `Frame` type (w/h/channels/stride/format) | easy–med | **clear win** — foundational, everything gets safer |
| `ModelRunner` trait + reference runner | medium | **clear win** — the keystone |
| In-Rust LiteRT/CoreML runner vs platform-callback runner | medium | **mixed** — genuine trade-off (PC-testable vs no TFLite dep in crate); pick one, write the ADR |
| Benchmark `Run` record (median, warm-up, thermal, JSON) | medium | **clear win** — docs already promise it |
| Property tests for filters + threading ADR | easy | **clear win** — nearly free |
| Hardware-buffer lock protocol (AHardwareBuffer/CVPixelBuffer) | medium | **mixed** — required for real zero-copy, but only bites once camera frames flow; adds unsafe-boundary complexity |
| KMP adapter contracts (`VideoSource`, `FrameSink`, …) | medium | **mixed** — only worth it if iOS/KMP actually happens; otherwise premature |
| Ring-buffer temporal state (N-frame history) | medium | **push back** — no recurrent model exists yet; single-frame state suffices |
| Golden-frame tests, FFI fuzzing, JNI round-trip | medium | **push back** — need a real model/FFI wiring first |
| iOS symmetry, async pipeline in Rust | high | **push back** — iOS is a placeholder; sync core is the right call now |

## 13. Recommended Rust tooling & crates

| Tool / crate | Purpose | Notes |
| --- | --- | --- |
| `clippy` | linting | Note: **cranky is deprecated** — use clippy directly (`cargo clippy -- -D warnings`) and gate it in CI |
| `rustfmt` | formatting | `.rustfmt.toml` already exists; add `cargo fmt --check` to CI |
| `cbindgen` | generate the C header from Rust | Replaces the hand-written `openllve_core.h`; kills ABI drift (typedef names already mismatched) |
| `serde` + `serde_json` | persist `BenchmarkRun` | makes benchmark runs comparable/reproducible (§6) |
| `tflite-c-rs` | LiteRT runner backend | wraps `libtensorflowlite_c` via `libloading` — no build-time link; CPU delegate runs on a PC, satisfying the cargo-test tenet (§1) |
| `proptest` | property tests for filters | EWMA stays within input bounds, blend is a convex combination (§11) |
| `bytemuck` | zero-copy typed views of `Frame` bytes | safe transmute between `*mut u8` buffers and typed pixel/f32 views (§2) |
| `miri` (tool, not a dep) | UB detection in unsafe FFI code | `cargo miri test` in CI — exactly what `ffi.rs`/`frame.rs` raw-pointer code needs |
| `criterion` | benchmarks | already in use; keep it |
| `static_assertions` | compile-time invariants | optional; e.g. assert layout/stride constants |
| `cargo-ndk` / `cargo-mobile2` | cross-compile the cdylib to Android ABIs | not a crate; CI job that produces `libopenllve_core.so` |

---

## Prioritization

**Do now** (shapes the next implementation step):

1. `ModelRunner` trait + in-Rust runners (§1) — the keystone; write the ADR.
2. `Frame` type + out-buffer API (§2, §3).
3. Model manifest (§10) — cheap, unblocks reproducible benchmarking.
4. `cbindgen` + error codes + ABI version (§7) — cheap, prevents ABI drift.

**Design before the next milestone** (needed once real video flows):

1. Hardware-buffer lock protocol (§4).
2. Benchmark session/run record (§6).
3. KMP adapter contracts (§8) and threading ADR (§9).

**Later** (quality bar once the pipeline is live):

1. Ring-buffer temporal state (§5), property/golden/FFI tests (§11), naming cleanup (§12), crate/tooling setup (§13).

### TL;DR

The architecture's *shape* is right; the main design gap is that **inference has no home**: the strategies are model-less stubs and the only real engine lives in Kotlin, violating the project's own layering rule. Introducing a `ModelRunner` trait in the Rust core (LiteRT via `tflite-c-rs`, Core ML on iOS, a reference runner for tests) with a first-class `Frame` type and an out-buffer API makes the CPU/GPU/NPU benchmark thesis actually implementable on a PC, deletes the duplicated Kotlin filter code, and gives every other design piece (benchmark records, FFI, KMP contracts, threading) a stable thing to hang on.
