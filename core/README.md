# OpenLLVE Core (`openllve-core`)

The Rust core for **OpenLLVE** is the portable, performance-sensitive layer that owns strategy execution, frame abstractions, filtering, and telemetry. It is intentionally separate from Android and iOS app code so the same benchmarking logic can run across platforms without duplicating logic.

## Why the C FFI layer exists

The Rust core is not meant to be directly embedded into the Kotlin or Swift host runtime. Instead, it exposes a C-compatible API through `core/include/openllve_core.h` and `src/ffi.rs`.

This is the standard pattern when you need:

- a safe, fast compute layer in Rust
- platform app code in Kotlin/Swift
- a stable ABI boundary across toolchains and operating systems
- low overhead and no dependency on the JVM or Apple runtime internals

In other words: the app owns the mobile runtime, while the core owns the compute and metrics logic.

## Recommended module shape

Modern Rust prefers folder modules without `mod.rs` when using 2018+ edition. This keeps the tree easy to read and avoids a proliferation of `mod.rs` files.

```text
core/
├── Cargo.toml
├── README.md
├── include/
│   └── openllve_core.h
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── frame.rs
│   ├── metrics.rs
│   ├── filters.rs                # module declaration for ewma.rs + blend.rs
│   ├── filters/
│   │   ├── ewma.rs                # temporal anti-flicker filter
│   │   └── blend.rs               # raw/enhanced blending filter
│   ├── strategies.rs             # module declaration for llie.rs + temporal.rs
│   ├── strategies/
│   │   ├── llie.rs                # LLIE strategy, optionally configured with EWMA
│   │   └── temporal.rs            # LLVE strategy, optionally configured with blend
│   ├── ffi.rs                    # C ABI bindings
│   └── lib.rs
├── tests/
│   └── integration_pipeline.rs
├── benches/
│   └── inference_bench.rs
└── Cargo.lock
```

## Strategy and topping model

The architecture separates the main model strategy from optional post-processing "toppings":

- `LlieStrategy`: frame-by-frame enhancement, can optionally use `EwmaFilter` as a temporal smoothing layer.
- `LlveTemporalStrategy`: stateful temporal model, may optionally use `FrameBlendFilter`, but should not usually add EWMA because the model already encodes temporal behavior.
- `FrameBlendFilter`: mixes raw input with processed output for stability and exposure control.
- `EwmaFilter`: anti-flicker smoothing for LLIE-style pipelines.

This is more flexible than hard-wiring all filters into every strategy.

## Design principles

1. **Zero-copy friendly**: frame handles abstract native buffers without redundant copying.
2. **Modular strategies**: the algorithm strategy is independent from the temporal tuning layer.
3. **Benchmark purity**: measure only the pure inference path, excluding camera I/O and UI render latency.
4. **Cross-platform portability**: Rust logic stays reusable while the app layer handles JVM/Swift integration.
