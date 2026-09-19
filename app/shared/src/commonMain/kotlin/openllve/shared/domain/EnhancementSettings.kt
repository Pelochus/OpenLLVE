package openllve.shared.domain

/**
 * User-facing enhancement configuration, shaped so the Rust core can consume
 * it directly: [computeTarget] -> delegate selection, [ewmaEnabled] ->
 * `EwmaFilter` topping, [flickerReductionEnabled] -> anti-flicker.
 *
 * Prototype: the EWMA/flicker toppings are not implemented in Kotlin (that
 * logic belongs in the Rust core); the toggles are exposed, persisted, and
 * passed through so the UI and configuration are real.
 */
data class EnhancementSettings(
    val computeTarget: ComputeTarget = ComputeTarget.CPU,
    val ewmaEnabled: Boolean = false,
    val flickerReductionEnabled: Boolean = false
)
