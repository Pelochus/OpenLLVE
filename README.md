# OpenLLVE 🚀

**OpenLLVE** (*Open Low-Light Video Enhancement*) is an open-source real-time
engine and benchmark for comparing **CPU**, **GPU** and **NPU** execution on
low-light video enhancement pipelines.

## Project status

An early implementation. The Rust core and the KMP `:shared` layer are in place;
the Android app is a functional vertical slice (Compose UI, LiteRT inference,
MediaCodec decode, DataStore settings) that calls the Rust core through the JNI
bridge. The iOS host is the next platform and is tracked at a high level only.

## Documentation

Start at the [documentation index](docs/README.md).

- [Architecture](docs/dev/architecture/ARCHITECTURE.md) — boundaries, tenets, data flow, repo layout
- [Rust core](core/README.md) — modules, pipelines, filters, FFI, tests, benchmarks
- [JNI shim](bridge/android/README.md) — Kotlin `external fun` contract
- [Shared app layer](app/shared/README.md) — KMP responsibilities and conventions
- [Android platform](app/platforms/android/README.md) — host sources and responsibilities
- [iOS platform](app/platforms/ios/README.md) — planned host, nothing implemented
- [Benchmark methodology](docs/benchmarks/METHODOLOGY.md) / [results](docs/benchmarks/RESULTS.md)

## Repository at a Glance

Source layers (the only things you edit):

- `core/` — platform-independent Rust business logic and compute (the one C ABI)
- `bridge/android/` — JNI marshalling shim for the JVM host
- `app/shared/` — Kotlin Multiplatform shared app logic
- `app/platforms/android/` — Android host: UI, MediaCodec, LiteRT, `jniLibs/`
- `app/platforms/ios/` — reserved for the future iOS host
- `external/models/` — vendored models + LiteRT NPU dispatch libraries
- `docs/` — documentation (`docs/README.md` is the index)
- `scripts/`, `docker/` — build helpers and the dev container
- `.github/workflows/` — CI gates

Generated output is gitignored: `target/`, `app/build/`, `build/`, `.gradle/`,
`.kotlin/` (the debug APK lands at
`app/build/outputs/apk/debug/app-debug.apk`).

## Getting Started

```text
./gradlew assembleDebug                                   # Android app
scripts/sign-release-apk.sh <keystore> <key-alias> <key-password> [store-password]
cd core && cargo test                                     # Rust core
cd core && cargo bench                                    # benchmarks
docker/run.sh                                             # dev container (JDK, SDK, ktlint, NDK)
git config core.hooksPath .githooks                       # optional local fmt/ktlint gate
```

CI is the authoritative gate; see [CONTRIBUTING.md](CONTRIBUTING.md) and
[AGENTS.md](AGENTS.md) for conventions.
