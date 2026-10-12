# Android Platform Layer

Android-specific runtime code — thin glue only: SAF media selection and
MediaCodec/MediaExtractor decode, Compose UI, lifecycle, device accelerator
integration, and the Kotlin half of the JNI bridge (`CoreBridge` `external fun`s,
`CoreToppings` filter handles).

Rule: anything not tied to SDK types, UI, MediaCodec or the LiteRT engine
belongs in `app/shared/` or the Rust core.
