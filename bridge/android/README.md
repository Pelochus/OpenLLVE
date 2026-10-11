# `bridge/android` — JNI shim over the core's C ABI

The JVM half of P1.1: it exposes the Rust core to Kotlin. The core itself keeps
one ABI surface (`core/src/ffi.rs` + `core/include/openllve_core.h`); nothing
here redefines it — the crate depends on `openllve_core` as a normal Rust
library and calls its functions directly. See
`docs/dev/architecture/FFI-WIRING.md` for the decision record.

## Why a separate crate

`core/` publishes a C ABI for consumers that dlopen it (LiteRT dispatch
runtimes, a future Swift bridge). The JVM needs a different thing: JVM
signatures, JVM object marshalling, and validation of caller-supplied
dimensions. Keeping that in `bridge/android/` means `core/` still compiles and
tests on a PC with no JVM types in scope, and the two ABI layers stay
reviewable separately. It also means the shipped library is one file: the core
is statically linked into `libbridge.so`.

## Layout

| Path | Role |
| --- | --- |
| `src/bridge.rs` | one export per `core/src/ffi.rs` function, named by `#[jni_mangle("openllve.android.core.CoreBridge")]` |
| `src/frames.rs` | boundary checks for JVM-supplied frame geometry + buffers, with `cargo test` coverage |
| `../app/platforms/android/src/main/java/openllve/android/core/CoreBridge.kt` | the Kotlin `external fun` contract (must match 1:1) |
| `../app/platforms/android/src/main/java/openllve/android/core/CoreToppings.kt` | app-side owner of the filter handles |

## Build

Host build (validation, tests):

```sh
cargo build && cargo test          # in this directory
cargo clippy -- -D warnings && cargo fmt --check
```

Android artifact for the APK — run inside the dev container, where the NDK
lives:

```sh
docker/run.sh --cmd 'scripts/build-bridge.sh'   # default abi: arm64-v8a
```

That writes `app/platforms/android/src/main/jniLibs/<abi>/libbridge.so`, which
is committed: the JVM resolves `System.loadLibrary("bridge")` by **file name**,
so the name is fixed there, not by Cargo's artifact name. Rebuild and re-commit
it whenever `bridge/` or `core/src/ffi.rs` changes.

Only `arm64-v8a` ships. Real devices are arm64, and PC-side validation uses the
core directly (`cargo test`/`bench --features model`, which also builds this crate
under `rust-core.yml`), so an `x86_64` `jniLibs` flavor is deliberately not built:
running the APK on an x86_64 emulator fails `dlopen` and `CoreToppings` throws
`UnsatisfiedLinkError`. Add the flavor only if emulator debugging is ever needed.

## Contract rules

- Handles are `jlong` (Kotlin `Long`). Not `ULong` — unsigned JVM types change
  the mangled name. Never overload an `external fun`.
- Frames are `jfloatarray` (Kotlin `FloatArray`); strides are **bytes** per row.
- Core-level failures return an `OpenLlveError` code (`0` = success); JNI-level
  failures (bad array reference, a Rust panic) throw a JVM exception.
- `frames.rs` runs before any pointer crosses the boundary: the C ABI derives
  buffer lengths from its dimension arguments and trusts the caller, so a short
  Kotlin array would otherwise be read past its allocation.
- The library is loaded lazily on first call. `CoreBridge.abiVersion()` is the
  startup probe: call it early so a missing or mismatched `.so` fails at startup
  instead of mid-frame.
