# Contributing to OpenLLVE

OpenLLVE is an early-stage project; contributions are welcome.

## Ground rules

Agent-facing conventions (code style, environment limits, gotchas) live in
[AGENTS.md](AGENTS.md) — read it before contributing.

- **One small, scoped commit per task.** Keep changes reviewable.
- **Follow the layering**: Rust core for business logic and compute, KMP
  `:shared` for shared app logic, platform code only under `app/platforms/`.
- **CI is the authoritative gate**: Rust (`cargo test`, clippy, fmt) and Kotlin
  (ktlint, androidLint, tests) must be clean. Local pre-commit hooks are optional
  (`git config core.hooksPath .githooks`).
- Never push to a protected branch without review; open a PR instead.

## Building

```text
cd core && cargo test          # Rust core
./gradlew assembleDebug        # Android app (needs JDK 17 + Android SDK — see docs/dev/docker-dev-env.md)
```

## Open work

See [TODO.md](TODO.md) and [IMPROVEMENTS.md](IMPROVEMENTS.md) for the current task
list and status.
