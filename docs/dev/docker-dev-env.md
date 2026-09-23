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
| JDK | 17 (headless) | what CI uses; Gradle/AGP run on it |
| Rust | stable (rustup, minimal profile) | `cargo test`/`clippy`/`fmt` for `core/` |
| ktlint | 1.8.0 | same version pinned in `.github/workflows/android-ci.yml` |
| Android SDK | cmdline-tools, platform 36, build-tools, platform-tools | `ANDROID_HOME` is set inside the image, so Gradle finds the SDK without a `local.properties` |
| git / curl / unzip / build-essential | — | base tooling |

The **Kotlin compiler does not need a separate install**: Gradle/KGP fetches
it from the plugin versions declared in the build files.

The **NDK is not installed yet** — it becomes needed for P1.1
(`cargo-ndk` cross-compilation of the Rust cdylib).

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
