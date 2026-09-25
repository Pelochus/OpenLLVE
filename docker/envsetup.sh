#!/usr/bin/env bash
# Install the OpenLLVE development toolchain (Ubuntu 26.04).
#
# Used by docker/Dockerfile at build time. It can also be run on a host
# (with sudo) for a native setup.
#
# Versions pinned here mirror CI: ktlint 1.8.0
# (.github/workflows/android-ci.yml), JDK 17 (android-ci.yml),
# Android platform 36 (app/build.gradle.kts compileSdk).
# No pipefail: `yes | sdkmanager` ends with `yes` getting SIGPIPE, which
# is expected and must not fail the script.
set -eu

export DEBIAN_FRONTEND=noninteractive

CARGO_HOME=${CARGO_HOME:-/usr/local/cargo}
RUSTUP_HOME=${RUSTUP_HOME:-/usr/local/rustup}
ANDROID_HOME=${ANDROID_HOME:-/opt/android-sdk}

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
rustup component add rustfmt

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

echo "envsetup: toolchain installed (JDK 17, Rust stable, ktlint 1.8.0, Android SDK)"
