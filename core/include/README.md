# FFI Header

This folder exposes the public C ABI used by Android, iOS, and other host runtimes.

- `openllve_core.h` defines the exported Rust functions and opaque handles.
- Keep the ABI stable when evolving the core; prefer additive changes over breaking ones.
- The host app should call into this layer, not directly into the Rust crate internals.
