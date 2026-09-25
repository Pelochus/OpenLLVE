# OpenLLVE Documentation

Index of the project documentation.

## Development

- [Architecture](dev/architecture/ARCHITECTURE.md) — layering tenets, repo
  layout, data flow, inference placement, threading model
- [Design suggestions](dev/architecture/DESIGN_SUGGESTIONS.md) — open
  design-level follow-ups
- [FFI wiring (P1.1)](dev/architecture/FFI-WIRING.md) — Rust↔Android wiring
  approach and the LiteRT-on-Rust decision
- [Docker dev environment](dev/docker-dev-env.md) — the `docker/` container
  with all build dependencies, and how to use it

## Benchmarks

- [Benchmark methodology](benchmarks/METHODOLOGY.md) — how to run the
  benchmark and how to interpret results
- [Benchmark results](benchmarks/RESULTS.md) — recorded runs

## Other docs

- [README.md](../README.md) — project overview and build commands
- [CONTRIBUTING.md](../CONTRIBUTING.md) — how to contribute
- [TODO.md](../TODO.md) / [IMPROVEMENTS.md](../IMPROVEMENTS.md) — open work
- `core/README.md` — Rust core modules
- `app/shared/README.md` — KMP shared layer
- `app/platforms/android/README.md` — Android platform
- `external/models/README.md` — model convention
