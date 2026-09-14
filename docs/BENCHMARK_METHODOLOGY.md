# Benchmark Methodology for OpenLLVE

## Objectives

- Evaluate inference latency of low-light video enhancement models
- Evaluate throughput (frames per second)
- Record thermal stability and throttling effects
- Compare CPU, GPU, and NPU performance under repeatable workloads

## Metrics

- **Median Inference Latency**: average processing time per frame in ms
- **FPS**: throughput in frames per second
- **P99 Latency**: tail-latency behavior under load
- **Thermal Stability**: change in latency and FPS over time

## Methodology

- Measure only the core inference path, not camera capture or UI render overhead
- Discard the first few warm-up frames to avoid allocator and JIT-like initialization effects
- Keep the same resolution, codec, and frame source across benchmark runs
- Run the benchmark on a consistent device state and repeat enough times for confidence

## Benchmarking rules

1. **Isolate core latency**: avoid including color conversion or display time
2. **Use static test sets**: fixed scenes and frame sequences across runs
3. **Benchmark all strategies**: LLIE with optional EWMA versus LLVE temporal strategy
4. **Capture environment**: device model, temperature, battery state, and accelerator selection

## Best practices

- Document each run with metadata about model, strategy, and filter toppings
- Maintain a warm-up phase before recording metrics
- Prefer the same build configuration across all device comparisons
- Use the metrics API in `core::BenchmarkMetrics` for standardized output
