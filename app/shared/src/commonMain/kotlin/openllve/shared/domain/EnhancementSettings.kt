package openllve.shared.domain

/**
 * User-facing enhancement configuration, shaped so the Rust core can consume
 * it directly: [computeTarget] -> delegate selection, [ewmaEnabled] ->
 * `EwmaFilter` topping, [flickerReductionEnabled] -> anti-flicker blend.
 *
 * The two toppings run in the Rust core: `CoreToppings` creates the native
 * filters for a media session, so the toggles decide which filters exist and
 * the weights are the filter constructor arguments — [ewmaAlpha] weights the
 * current frame (never below 0.25, i.e. at most 75 % of the previous frame),
 * [blendBeta] weights the AI-enhanced frame.
 */
data class EnhancementSettings(
    val computeTarget: ComputeTarget = ComputeTarget.CPU,
    val ewmaEnabled: Boolean = false,
    val flickerReductionEnabled: Boolean = false,
    val ewmaAlpha: Float = 0.8f,
    val blendBeta: Float = 0.9f,
)
