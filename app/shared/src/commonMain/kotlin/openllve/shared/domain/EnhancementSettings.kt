package openllve.shared.domain

/**
 * User-facing enhancement configuration, shaped so the Rust core can consume
 * it directly: [computeTarget] -> delegate selection, [ewmaEnabled] ->
 * `EwmaFilter` topping, [flickerReductionEnabled] -> anti-flicker blend.
 *
 * The two toppings run in the Rust core: `CoreToppings` creates the native
 * filters for a media session, so these toggles decide which filters exist
 * rather than implementing anything here.
 */
data class EnhancementSettings(
    val computeTarget: ComputeTarget = ComputeTarget.CPU,
    val ewmaEnabled: Boolean = false,
    val flickerReductionEnabled: Boolean = false,
)
