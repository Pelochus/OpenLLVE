# `bridge/android` — JNI shim over the core's C ABI

The JVM half of the bridge: it exposes the Rust core to Kotlin. `core/src/ffi.rs`
stays the only ABI surface; this crate depends on `openllve_core` as a normal Rust
library and adds nothing but JVM marshalling. `docs/dev/architecture/FFI-WIRING.md`
records why direct JNI (no C++ layer) and why the shim is its own crate.

## Layout

| Path | Role |
| --- | --- |
| `src/bridge.rs` | one `#[jni_mangle("openllve.android.core.CoreBridge")]` export per C ABI function |
| `src/frames.rs` | JVM facts only (`Int` is signed, a Kotlin array reports an element count, not a byte length) checked against the core's frame contract, with `cargo test` coverage |
| `../app/platforms/android/src/main/java/openllve/android/core/CoreBridge.kt` | Kotlin `external fun` contract, one declaration per export |
| `../app/platforms/android/src/main/java/openllve/android/core/CoreToppings.kt` | app-side owner of the EWMA + blend handles per media session |
| `../../scripts/build-bridge.sh` | `cargo ndk -t arm64-v8a` → installs `jniLibs/<abi>/libbridge.so` |

## Build

Host build (validation, tests) — in this directory:

```sh
cargo build && cargo test
cargo clippy -- -D warnings && cargo fmt --check
```

APK artifact — run inside the dev container, where the NDK lives:

```sh
docker/run.sh --cmd 'scripts/build-bridge.sh'   # default abi: arm64-v8a
```

That writes the committed `app/platforms/android/src/main/jniLibs/<abi>/libbridge.so`.
The JVM resolves `System.loadLibrary("bridge")` by **file name**, so that name is
fixed there rather than coming from Cargo's artifact name; rebuild and re-commit it
whenever `bridge/` or `core/src/ffi.rs` changes.

Only `arm64-v8a` ships. Real devices are arm64 and PC-side validation uses the core
directly, so an `x86_64` `jniLibs` flavor is deliberately not built — running the
APK on an x86_64 emulator fails `dlopen` and `CoreToppings` throws
`UnsatisfiedLinkError`.

## Contract rules

- Handles are `jlong` (Kotlin `Long`), never `ULong` — unsigned JVM types change the
  mangled name. Never overload an `external fun`.
- Frames are `jfloatarray` (Kotlin `FloatArray`); strides are **bytes** per row.
- Core-level failures return an `OpenLlveError` code (`0` = ok); JNI-level failures
  (bad array reference, a Rust panic) throw a JVM exception.
- `frames.rs` runs before any pointer crosses the boundary and keeps only JVM facts;
  the frame contract itself (positive dimensions, stride alignment, row width,
  buffer requirement) is the core's `frame.validate_dimensions` +
  `frame.buffer_elements` — the same pair `core/src/ffi.rs` runs before it sizes
  any caller memory. One contract, every host.
- The library loads lazily on first call, so `CoreBridge.abiVersion()` is the startup
  probe: call it early so a missing or mismatched `.so` fails at startup instead of
  mid-frame.
