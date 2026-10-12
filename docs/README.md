# OpenLLVE Documentation

Index of the project documentation. Module READMEs live beside their code
(`core/README.md`, `bridge/android/README.md`, `app/shared/README.md`,
`app/platforms/*/README.md`, `external/models/README.md`).

## Development

- [Architecture](dev/architecture/ARCHITECTURE.md) — layering tenets, repo layout,
  data flow, inference placement, threading model
- [FFI wiring](dev/architecture/FFI-WIRING.md) — Rust↔Kotlin decision record
  (direct JNI, no C++ layer; LiteRT stays host-side for production inference)
- [Design suggestions](dev/architecture/DESIGN_SUGGESTIONS.md) — open design-level
  follow-ups
- [Docker dev environment](dev/docker-dev-env.md) — the container with every build
  dependency, and how to use it
- [App fixes record](dev/app-fixes.md) — what the app layer already fixed and the
  vendor/platform dead ends **not to re-attempt**

## Benchmarks

- [Methodology](benchmarks/METHODOLOGY.md) — how to run and how to read results
- [Results](benchmarks/RESULTS.md) — recorded runs

## Project files

- [../README.md](../README.md) — overview and build commands
- [../AGENTS.md](../AGENTS.md) — agent conventions (style, gates, gotchas)
- [../CONTRIBUTING.md](../CONTRIBUTING.md) — how to contribute
- [../TODO.md](../TODO.md) / [../IMPROVEMENTS.md](../IMPROVEMENTS.md) — open work and status
