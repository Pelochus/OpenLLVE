#!/usr/bin/env bash
# Fetch the TensorFlow Lite C shared library for PC-side development and
# benchmarking (the `model` cargo feature of the Rust core).
#
# The Rust core loads this library at runtime via `tflite-c-rs` (libloading);
# it is not linked at build time. On Android the library is packaged into the
# APK (jniLibs) instead — this script is for desktop use only.
#
# Usage:   scripts/fetch-tflite-lib.sh [dest-dir]
# Example: scripts/fetch-tflite-lib.sh core/native
#
# The library is placed in <dest-dir>/libtensorflowlite_c.so (or the platform
# equivalent). Point the `OPENLLVE_TFLITE_LIB` environment variable at it when
# running `cargo test --features model` / `cargo bench --features model`.
set -euo pipefail

dest=${1:-core/native}
mkdir -p "$dest"

# Pinned TFLite C release (matches the runtime the model was validated against).
version=2.17.1
base="https://github.com/tphakala/tflite_c/releases/download/v${version}"

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
Linux)
  case "$arch" in
  x86_64) asset="tflite_c_v${version}_linux_amd64.tar.gz" ;;
  aarch64) asset="tflite_c_v${version}_linux_arm64.tar.gz" ;;
  *)
    echo "error: unsupported architecture $arch" >&2
    exit 1
    ;;
  esac
  lib_name="libtensorflowlite_c.so"
  ;;
Darwin)
  case "$arch" in
  x86_64) asset="tflite_c_v${version}_darwin_amd64.tar.gz" ;;
  arm64) asset="tflite_c_v${version}_darwin_arm64.tar.gz" ;;
  *)
    echo "error: unsupported architecture $arch" >&2
    exit 1
    ;;
  esac
  lib_name="libtensorflowlite_c.dylib"
  ;;
*)
  echo "error: unsupported OS $os (desktop fetch supports Linux/macOS)" >&2
  exit 1
  ;;
esac

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "Downloading ${asset} ..."
curl -fSL -o "$tmp/$asset" "${base}/${asset}"

echo "Extracting ..."
tar -xzf "$tmp/$asset" -C "$tmp"

# Find the shared library inside the extracted tree.
found="$(find "$tmp" -name "$lib_name" | head -n 1)"
if [ -z "$found" ]; then
  echo "error: $lib_name not found in $asset" >&2
  exit 1
fi

cp "$found" "$dest/$lib_name"
echo "Wrote $dest/$lib_name"
echo "Run with: OPENLLVE_TFLITE_LIB=$dest/$lib_name cargo test --features model"
