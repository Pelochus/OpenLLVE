# OpenLLVE 🚀

**OpenLLVE** (*Open Low-Light Video Enhancement*) is an open-source real-time engine and benchmark designed to compare **CPU**, **GPU**, and **NPU** execution on low-light video enhancement pipelines.

## Project Status

OpenLLVE is an early implementation: the Rust core and the KMP shared layer are in place, and the Android app is a functional vertical slice (Compose UI, LiteRT inference, MediaCodec decode, DataStore settings). JNI/Rust wiring and the iOS platform are next.

## Documentation

Start at the [documentation index](docs/README.md).

- [Architecture](docs/dev/architecture/ARCHITECTURE.md) - boundaries, tenets, data flow, and repository structure
- [Rust core](core/README.md) - modules, pipelines, filters, FFI, tests, and benchmarks
- [Shared app layer](app/shared/README.md) - KMP responsibilities and shared app conventions
- [Android platform](app/platforms/android/README.md) - Android source and runtime responsibilities
- [iOS platform](app/platforms/ios/README.md) - planned iOS host (P4, high-level); nothing implemented yet
- [Benchmark methodology](docs/benchmarks/METHODOLOGY.md) - how to run the benchmark and interpret results
- [Benchmark results](docs/benchmarks/RESULTS.md) - recorded runs

## Repository at a Glance

Source layers (the only things you edit):

- `core/` - platform-independent Rust business logic and compute (the one C ABI)
- `bridge/android/` - JNI marshalling shim for the JVM host (P1.1)
- `app/shared/` - Kotlin Multiplatform shared app logic
- `app/platforms/android/` - Android host: UI, MediaCodec, LiteRT, `jniLibs/`
- `app/platforms/ios/` - reserved for the future iOS host (P4)
- `external/models/` - vendored model + LiteRT NPU dispatch libraries
- `docs/` - documentation (`docs/README.md` is the index)
- `scripts/`, `docker/` - build helpers and the dev container
- `.github/workflows/` - CI gates

Generated, never pushed (`git ls-files` lists only what is pushed):

- `target/`, `bridge/android/target/` - cargo build dirs
- `app/build/`, `build/`, `.gradle/`, `.kotlin/` - Gradle outputs; the debug APK
  is `app/build/outputs/apk/debug/app-debug.apk` once `./gradlew assembleDebug`
  has run

Kotlin sources nest to their package directory (`app/shared/…/openllve/shared/`, `app/platforms/android/src/…/openllve/android/`); the module list in [`AGENTS.md`](AGENTS.md) §3 maps those directories to roles.

## Getting Started

### Android (Linux / macOS / WSL)

```text
./gradlew assembleDebug
```

### Signing the release APK

```text
scripts/sign-release-apk.sh <keystore> <key-alias> <key-password> [store-password]
```

The script is the single source of truth; see its header for details.

### Rust core

```text
cd core
cargo test
cargo bench
```

### Dev environment (Docker)

```text
docker/run.sh
```

Builds the Ubuntu 26.04 container (JDK 17, Rust, ktlint, Android SDK —
see `docker/envsetup.sh`), mounts the repo at `/workspace`, and enables
the `.githooks` pre-commit hook. See
[docs/dev/docker-dev-env.md](docs/dev/docker-dev-env.md) for details.

### Local commit hooks (optional)

```text
git config core.hooksPath .githooks
```

Runs `cargo fmt --check` when Rust files are staged and `ktlint` when Kotlin
files are staged (if ktlint is installed). CI remains the authoritative gate.

See the [architecture documentation](docs/dev/architecture/ARCHITECTURE.md) before adding code to a new layer.
