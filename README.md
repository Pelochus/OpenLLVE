# OpenLLVE 🚀

**OpenLLVE** (*Open Low-Light Video Enhancement*) is an open-source real-time engine and benchmark designed to compare **CPU**, **GPU**, and **NPU** execution on low-light video enhancement pipelines.

## Project Status

OpenLLVE is currently a proof-of-concept architecture and early implementation. The Rust core and Android project structure are being established before production inference and iOS work.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - boundaries, tenets, data flow, and repository structure
- [Rust core](core/README.md) - modules, pipelines, filters, FFI, tests, and benchmarks
- [Shared app layer](app/shared/README.md) - KMP responsibilities and shared app conventions
- [Android platform](app/platforms/android/README.md) - Android source and runtime responsibilities
- [iOS platform](app/platforms/ios/README.md) - reserved placeholder for future iOS development
- [Benchmark methodology](docs/BENCHMARK_METHODOLOGY.md) - measurement rules and interpretation

## Repository at a Glance

- `core/` - platform-independent Rust business logic and compute
- `app/shared/` - Kotlin Multiplatform shared app logic
- `app/platforms/android/` - Android-specific source tree and integration
- `app/platforms/ios/` - future iOS integration

## Getting Started

### Android (Linux / macOS / WSL)

```text
./gradlew assembleDebug
```

### Rust core

```text
cd core
cargo test
cargo bench
```

### Local commit hooks (optional)

```text
git config core.hooksPath .githooks
```

Runs `cargo fmt --check` when Rust files are staged and `ktlint` when Kotlin
files are staged (if ktlint is installed). CI remains the authoritative gate.

See the [architecture documentation](docs/ARCHITECTURE.md) before adding code to a new layer.
