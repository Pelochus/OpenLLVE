# OpenLLVE — Architecture & Design Suggestions

*Scope: design-level follow-ups that are still open. The already-implemented
suggestions (in-Rust `ModelRunner`, `Frame` type, out-buffer API, FFI
hardening, `Pipeline`/`TemporalMode` rename, the threading-model ADR — now
`ARCHITECTURE.md` §12, and the LiteRT naming standardization) and the
push-back/premature ones (hardware-buffer lock protocol, ring-buffer temporal
state, KMP adapter contracts, iOS symmetry, async pipeline in Rust, the
benchmark first-class module / `BenchmarkRun` record, and the model manifest)
have been removed — see `IMPROVEMENTS.md` and the git history. The existing layering
intent (Rust compute / C ABI / platform adapters / KMP shared) is sound; these
suggestions are about making the design hold up when real models and real
video flow through it.*

---

## 1. Testing strategy (design-level)

- **Property tests for filters**: EWMA output must stay within
  `[min(prev,curr), max(prev,curr)]` per pixel; blend output is a convex
  combination. Cheap, catches the class of bugs in the current stubs.
  (Tracked as P3.8.)
- **Golden-frame tests**: fixed input frame → hash of output, pinned per model
  file. This is what makes CPU/GPU/NPU comparison *valid* (same model, same
  input, comparable outputs).
- FFI-fuzzing, miri, and JNI round-trip tests wait for P1.1 (FFI wiring into
  the app).

## 2. Remaining Rust tooling & crates

| Tool / crate | Purpose | Notes |
| --- | --- | --- |
| `proptest` | property tests for filters | EWMA stays within input bounds, blend is a convex combination (§1) |
| `miri` (tool, not a dep) | UB detection in unsafe FFI code | `cargo miri test` in CI — exactly what `ffi.rs`/`frame.rs` raw-pointer code needs |
| `static_assertions` | compile-time invariants | optional; e.g. assert layout/stride constants |
| `cargo-ndk` / `cargo-mobile2` | cross-compile the cdylib to Android ABIs | not a crate; CI job that produces `libopenllve_core.so` (P1.1) |

## Quick ranking (effort × value)

| Suggestion | Effort | Verdict |
| --- | --- | --- |
| Property tests for filters | easy | **clear win** — nearly free |
| Golden-frame tests | medium | now feasible — pinned per model file |

## Prioritization

**Later** (quality bar once the pipeline is live):

1. Property + golden-frame tests (§1), crate/tooling setup (§2).

### TL;DR

The keystone pieces are in place (in-Rust model runner, `Frame` type,
out-buffer API, hardened FFI). Benchmarking is deliberately kept minimal
(`cargo bench` + `BenchmarkMetrics` + a results table in `docs/benchmarks/`).
What remains is the quality bar (property/golden-frame tests, remaining
crates).
