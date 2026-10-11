//! JNI bridge over the core's C ABI, for the Kotlin app layer.
//!
//! The core keeps one ABI surface (`core/src/ffi.rs` + `core/include/openllve_core.h`);
//! this crate is the thin JVM shim over it — see docs/dev/architecture/FFI-WIRING.md
//! for the decision record, and bridge/android/README.md for the Kotlin side.

mod bridge;
mod frames;
