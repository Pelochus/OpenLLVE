# OpenLLVE development environment (Ubuntu 26.04).
#
# All package installation lives in envsetup.sh (kept separate so version
# bumps are a one-file edit and the script can also run on a host).
#
# Build:  docker/run.sh
# Run:    docker/run.sh
#
# See docs/dev/docker-dev-env.md for details.
FROM ubuntu:26.04

# Toolchain locations (shared with docker/envsetup.sh).
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV ANDROID_HOME=/opt/android-sdk
ENV PATH=/usr/local/cargo/bin:/opt/android-sdk/cmdline-tools/latest/bin:$PATH

COPY envsetup.sh /usr/local/src/envsetup.sh
RUN /usr/local/src/envsetup.sh

# Runs as root so the mounted repo volume is readable/writable (rootless
# podman maps the host user to root inside the container).
WORKDIR /workspace
