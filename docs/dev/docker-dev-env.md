# Docker dev environment

A self-contained **Ubuntu 26.04** container with everything needed to build
and test the project.

| File | Purpose |
| --- | --- |
| [`docker/Dockerfile`](../../docker/Dockerfile) | slim image: base + env vars + `envsetup.sh` |
| [`docker/envsetup.sh`](../../docker/envsetup.sh) | all package installation (kept separate so version bumps are a one-file edit; also runnable on a host with sudo) |
| [`docker/run.sh`](../../docker/run.sh) | build + run convenience: mounts the repo at `/workspace`, enables the repo's git hooks |

## What's inside

| Tool | Version | Notes |
| --- | --- | --- |
| JDK | 17 (headless) | what CI uses; the minimum AGP 9.4.0 accepts |
| Rust | stable (rustup, minimal profile + `rustfmt`/`clippy`) | `cargo test`/`clippy`/`fmt` for `core/` |
| Rust Android targets | `aarch64`/`armv7`/`x86_64` `linux-android` + `cargo-ndk` | cross-compiling the core cdylib for the APK (P1.1) |
| ktlint | 1.8.0 | same version pinned in `.github/workflows/android-ci.yml` |
| Android SDK | cmdline-tools, platform 36, build-tools, platform-tools | `ANDROID_HOME` is set inside the image, so Gradle finds the SDK without a `local.properties` |
| Android NDK | r30 (current LTS) | `ANDROID_NDK_HOME`; the clang toolchain `cargo-ndk` cross-compiles with |
| git / curl / unzip / build-essential | — | base tooling |

The **Kotlin compiler does not need a separate install**: Gradle/KGP fetches
it from the plugin versions declared in the build files.

Everything the repo needs builds inside the container:

```bash
./gradlew assembleDebug
cargo test --manifest-path core/Cargo.toml
ktlint app/shared/src app/platforms/android/src
cargo ndk -t arm64-v8a -o ./jniLibs build --release   # core cdylib for the APK
```

## Native host setup (no container)

`docker/envsetup.sh` is also runnable on a host with sudo. On a host without
one (e.g. Arch), install the Rust side in userspace instead:

```bash
curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
rustup component add rustfmt clippy
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk
```

`rustup` installs into `~/.cargo` and `~/.rustup` and puts its own `bin/` on
`PATH` (for fish: `source ~/.cargo/env.fish` in `~/.config/fish/config.fish`).
Cross-compiling additionally needs the NDK unzipped somewhere and exported as
`ANDROID_NDK_HOME`. The Android app still needs a JDK + Android SDK on the
host; the container remains the complete environment.

## Build and run

```bash
docker/run.sh
```

That builds the image (`openllve-dev`), mounts the repo at `/workspace`,
and enables the repo's pre-commit hooks (`git config core.hooksPath
.githooks`) — so `git commit` inside the container runs `cargo fmt --check`
and `ktlint` on staged files. Extra run args pass through, e.g.
`docker/run.sh --privileged`.

From inside, the usual commands work:

```bash
./gradlew assembleDebug
cargo test --manifest-path core/Cargo.toml
ktlint app/shared/src app/platforms/android/src
```

## Prebuilt image from GitHub Container Registry

`.github/workflows/docker-image.yml` builds this image on every push to
`dev`/`main` and pushes it to GHCR — the build runs on the GitHub runner,
so no Docker daemon is needed on the host that pushes. Pull the prebuilt
image and run it without rebuilding:

```bash
podman pull ghcr.io/pelochus/openllve-dev:latest
podman tag ghcr.io/pelochus/openllve-dev:latest localhost/openllve-dev:latest
docker/run.sh --skip-build
```

## Notes

- The container runs as **root** so the mounted repo volume is
  readable/writable (rootless podman maps the host user to root inside the
  container). The mounted volume is the only shared state.
- First `./gradlew` run inside the container downloads the Gradle
  distribution and dependencies (cached in `~/.gradle` inside the
  container).
- If the Android SDK releases a newer platform, bump the
  `sdkmanager` package list in `envsetup.sh` together with `compileSdk` in
  `app/build.gradle.kts`.
- On some rootless podman setups the bind mount fails with
  `Permission denied`; add `--privileged` to the `run` command in that
  case (verified on this host).
