# OpenLLVE development environment (Ubuntu 26.04).
#
# All package installation lives in envsetup.sh (kept separate so version
# bumps are a one-file edit and the script can also run on a host); each
# phase is its own layer so a phase edit re-runs only that phase.
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
ENV ANDROID_NDK_HOME=/opt/android-ndk
ENV PATH=/usr/local/cargo/bin:/opt/android-sdk/cmdline-tools/latest/bin:$PATH

COPY envsetup.sh /usr/local/src/envsetup.sh
RUN /usr/local/src/envsetup.sh base
RUN /usr/local/src/envsetup.sh rust
RUN /usr/local/src/envsetup.sh ktlint
RUN /usr/local/src/envsetup.sh sdk
RUN /usr/local/src/envsetup.sh ndk

# World-writable so cargo/rustup lockfiles and AGP's runtime SDK installs
# work under either runner (podman runs as root, docker as the host uid).
RUN chmod -R a+rwX /usr/local/cargo /usr/local/rustup /opt/android-sdk /opt/android-ndk

# Runs as root (podman) or the host uid (docker) so the mounted repo volume
# is readable/writable.
WORKDIR /workspace
