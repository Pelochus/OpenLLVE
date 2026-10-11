#!/usr/bin/env bash
# Cross-compile the JNI bridge (`bridge/android/`) and install it into the app's
# jniLibs tree, so the APK ships the Rust core as a native library.
#
# Usage:   scripts/build-bridge.sh [abi ...]            (default: arm64-v8a)
# Example: scripts/build-bridge.sh arm64-v8a x86_64
#
# Run it where the NDK lives — inside the dev container:
#   docker/run.sh --cmd 'scripts/build-bridge.sh'
#
# Naming: the JVM resolves `System.loadLibrary("bridge")` by *file name*
# (`libbridge.so`) in the APK's native library directory, not by DT_NEEDED, so
# Cargo's artifact (libopenllve_core_bridge.so) is renamed here. The ABI itself
# is the `Java_openllve_android_core_CoreBridge_*` symbol set — check it with
# `nm -gD` (the script prints the count) before shipping a rebuilt library.
#
# The committed `.so` under `app/platforms/android/src/main/jniLibs/` is a build
# output (`.gitignore` negates `*.so` there so the vendored LiteRT dispatch
# libraries stay trackable): rebuild and re-commit it whenever `bridge/` or
# `core/src/ffi.rs` changes.
set -euo pipefail

abis=("$@")
[ ${#abis[@]} -gt 0 ] || abis=(arm64-v8a)

ndk=${ANDROID_NDK_HOME:-/opt/android-ndk}
jni_libs=app/platforms/android/src/main/jniLibs
crate=bridge/android

[ -d "$ndk" ] || {
  echo "NDK not found at $ndk (set ANDROID_NDK_HOME or run in the dev container)" >&2
  exit 1
}

# cargo-ndk 0.1.1 reads Cargo.toml from the current directory, so the build runs
# inside the crate. Artifacts land under the crate's gitignored target/ tree —
# only the renamed libbridge.so below belongs in jniLibs, so jniLibs never holds
# a second copy of the same library under Cargo's name.
for abi in "${abis[@]}"; do
  (
    cd "$crate"
    cargo ndk -t "$abi" -o target/jni-out -- build --release
  )

  built="$crate/target/jni-out/$abi/libopenllve_core_bridge.so"
  [ -f "$built" ] || {
    echo "missing cargo-ndk artifact for $abi: $built" >&2
    exit 1
  }

  install -m 0644 "$built" "$jni_libs/$abi/libbridge.so"
  exports=$(nm -gD "$jni_libs/$abi/libbridge.so" | grep -c ' T Java_')
  echo "jniLibs/$abi/libbridge.so: $(wc -c <"$built") bytes, $exports JNI exports"
done
