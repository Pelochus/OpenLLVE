#!/usr/bin/env bash
# Install the OpenLLVE development toolchain (Ubuntu 26.04).
#
# Used by docker/Dockerfile at build time. It can also be run on a host
# (with sudo) for a native setup.
#
# Versions pinned here mirror CI: ktlint 1.8.0
# (.github/workflows/android-ci.yml), JDK 17 (android-ci.yml, the minimum
# AGP 9.4.0 accepts), Android platform 36 (app/build.gradle.kts compileSdk).
# The image is complete: Rust core (build/test/clippy/fmt), the Android app
# (Gradle + SDK), and the P1.1 cross-compile path (NDK + cargo-ndk + Android
# rustup targets for the core cdylib).
# No pipefail: `yes | sdkmanager` ends with `yes` getting SIGPIPE, which
# is expected and must not fail the script.
set -eu

export DEBIAN_FRONTEND=noninteractive

CARGO_HOME=${CARGO_HOME:-/usr/local/cargo}
RUSTUP_HOME=${RUSTUP_HOME:-/usr/local/rustup}
ANDROID_HOME=${ANDROID_HOME:-/opt/android-sdk}
ANDROID_NDK_HOME=${ANDROID_NDK_HOME:-/opt/android-ndk}
export ANDROID_NDK_HOME

# --- base tools -----------------------------------------------------------
apt-get update
apt-get install -y --no-install-recommends \
	ca-certificates \
	curl \
	git \
	unzip \
	wget \
	build-essential \
	openjdk-17-jdk-headless

# --- Rust (stable, minimal profile) ----------------------------------------
export PATH="$CARGO_HOME/bin:$PATH"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
	sh -s -- -y --default-toolchain stable --profile minimal
# rustfmt: needed by the repo's pre-commit hook (cargo fmt --check).
# clippy: needed by rust-core.yml CI (`cargo clippy -- -Dwarnings`).
rustup component add rustfmt clippy
# Android targets + cargo-ndk: cross-compiling the core cdylib into the APK
# (docs/dev/architecture/FFI-WIRING.md, TODO.md P1.1). cargo-ndk drives the
# NDK clang toolchain installed further below.
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk

# --- ktlint (same version as CI) -------------------------------------------
curl -sSL -o /usr/local/bin/ktlint \
	https://github.com/ktlint/ktlint/releases/download/1.8.0/ktlint
chmod +x /usr/local/bin/ktlint

# --- Android SDK (cmdline tools + platform 36 + build tools) --------------
mkdir -p "$ANDROID_HOME/cmdline-tools"
curl -sSL -o /tmp/cmdline-tools.zip \
	https://dl.google.com/android/repository/commandlinetools-linux-12700392_latest.zip
unzip -q /tmp/cmdline-tools.zip -d "$ANDROID_HOME/cmdline-tools"
# The zip extracts a `cmdline-tools/` dir; the standard layout is
# cmdline-tools/latest/.
mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
rm /tmp/cmdline-tools.zip

SDKMGR="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
yes | "$SDKMGR" --licenses >/dev/null
"$SDKMGR" "platforms;android-36" "build-tools;36.0.0" "platform-tools"

# --- Android NDK (cargo-ndk cross-compiles of the core cdylib) -------------
# NDK r30 (current LTS). cargo-ndk uses its clang toolchain for the
# aarch64/x86_64 Android targets installed above; the resulting
# libopenllve_core.so is packaged into the APK's jniLibs (P1.1).
# Large layer (~740 MB download) — the image is built on the GitHub runner
# (docker-image.yml), so the host that pushes never needs the space.
curl -sSL -o /tmp/ndk.zip \
	https://dl.google.com/android/repository/android-ndk-r30-linux.zip
unzip -q /tmp/ndk.zip -d /tmp/ndk
mv /tmp/ndk/android-ndk-r30 "$ANDROID_NDK_HOME"
rm -rf /tmp/ndk /tmp/ndk.zip

echo "envsetup: toolchain installed (JDK 17, Rust stable + cargo-ndk + Android targets, ktlint 1.8.0, Android SDK + NDK r30)"
