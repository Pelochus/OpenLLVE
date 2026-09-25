# Benchmark Methodology

OpenLLVE benchmarks the **inference path only** — not camera capture,
color conversion, or UI rendering.

## Metrics

- **Median inference latency** (ms) per frame
- **FPS** (throughput)
- **P99 latency** (tail behavior under load)

Warm-up frames are excluded from all statistics
(`BenchmarkMetrics::with_warmup`).

## How to run

PC (Rust core):

```bash
cd core
cargo bench --features model   # real model path (needs libtensorflowlite_c)
cargo bench                    # pipeline only (no model)
```

Android: not yet — the app's benchmark path is not implemented; use the
Rust core on a PC for now.

## Rules

1. Measure only the core inference path (no camera I/O, no display time).
2. Use a static test set: fixed frames across runs.
3. Keep the same resolution, model, and settings when comparing runs.
4. Run long enough for a stable median (hundreds of frames).

## Interpreting results

- Numbers are only comparable **on the same device** (same model,
  resolution, and settings).
- Across devices/OS, treat results as **directional** (which backend is
  relatively faster), not absolute: clocks, thermals, and drivers differ.
- Record runs in [RESULTS.md](RESULTS.md).
