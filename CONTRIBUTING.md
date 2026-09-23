# Contributing to OpenLLVE

OpenLLVE is an early-stage project; contributions are welcome.

## Ground rules

- **One small, scoped commit per task.** Keep changes reviewable.
- **Follow the layering** in [docs/dev/architecture/ARCHITECTURE.md](docs/dev/architecture/ARCHITECTURE.md):
  Rust core for business logic and compute, KMP `:shared` for shared app
  logic, platform code only under `app/platforms/`.
- **CI is the authoritative gate**: Rust (`cargo test`, clippy, fmt) and
  Kotlin (ktlint, androidLint, tests) must be clean.
- Local pre-commit hooks are optional (`git config core.hooksPath .githooks`).

## Building

- Rust core: `cargo test` in `core/`
- Android app: `./gradlew assembleDebug`

## Open work

See [TODO.md](TODO.md) and [IMPROVEMENTS.md](IMPROVEMENTS.md) for the
current task list and status.
