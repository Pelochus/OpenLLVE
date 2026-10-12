//! JNI bridge over the core's C ABI, for the Kotlin app layer.
//!
//! The core keeps one ABI surface; this crate is the thin JVM shim over it.

mod bridge;
mod frames;
