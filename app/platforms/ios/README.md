# iOS Platform Layer

Placeholder — no iOS code yet (tracked as P4, high-level only). A future host
would add AVFoundation capture, a SwiftUI view layer, Core ML / Metal inference,
and adapters onto the `:shared` contracts, with a `bridge/ios/` shim reusing the
core's existing C ABI. Nothing iOS-specific may enter `core/` or `app/shared/`.
