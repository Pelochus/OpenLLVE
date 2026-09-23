# OpenLLVE development environment (Ubuntu 26.04).
#
# Build:  podman build -t openllve-dev docker/
# Run:    podman run --rm -it -v "$PWD":/workspace openllve-dev
#
# See docs/dev/docker-dev-env.md for details.
FROM ubuntu:26.04

ENV DEBIAN_FRONTEND=noninteractive

# --- base tools -----------------------------------------------------------
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
        unzip \
        wget \
        build-essential \
        openjdk-17-jdk-headless

# --- Rust (stable) --------------------------------------------------------
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal

# --- ktlint (matches the version pinned in .github/workflows/android-ci.yml)
RUN curl -sSL -o /usr/local/bin/ktlint \
        https://github.com/ktlint/ktlint/releases/download/1.8.0/ktlint \
    && chmod +x /usr/local/bin/ktlint

# --- Android SDK (cmdline tools + platform 36 + build tools) --------------
ENV ANDROID_HOME=/opt/android-sdk
ENV PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin
RUN mkdir -p "$ANDROID_HOME/cmdline-tools" \
    && curl -sSL -o /tmp/cmdline-tools.zip \
        https://dl.google.com/android/repository/commandlinetools-linux-12700392_latest.zip \
    && unzip -q /tmp/cmdline-tools.zip -d "$ANDROID_HOME/cmdline-tools" \
    && mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest" \
    && rm /tmp/cmdline-tools.zip \
    && yes | sdkmanager --licenses > /dev/null \
    && sdkmanager "platforms;android-36" "build-tools;36.0.0" "platform-tools"

# --- workspace -------------------------------------------------------------
# Runs as root so the mounted repo volume is readable/writable (rootless
# podman maps the host user to root inside the container).
WORKDIR /workspace
