package openllve.android.domain

/**
 * User-facing enhancement configuration, expressed in OpenLLVE concepts.
 *
 * This is the configuration shape the eventual Rust core should be able to
 * consume without changing the UI. The Rust pipeline would map:
 * - [computeTarget] -> delegate selection (Rust `ModelRunner` / FFI option),
 * - [ewmaEnabled] -> attach the `EwmaFilter` topping,
 * - [flickerReductionEnabled] -> anti-flicker behaviour.
 *
 * NOTE (prototype): in this Android-first slice the EWMA / flicker toppings
 * are **not** implemented in Kotlin (that logic belongs in the Rust core per
 * the architecture tenets). The toggles are exposed, persisted, and passed
 * through here so the UI and configuration are real; the actual temporal
 * processing is deferred to the Rust pipeline (see TODO.md P1.1).
 */
data class EnhancementSettings(
    val computeTarget: ComputeTarget = ComputeTarget.CPU,
    val ewmaEnabled: Boolean = false,
    val flickerReductionEnabled: Boolean = false
)
