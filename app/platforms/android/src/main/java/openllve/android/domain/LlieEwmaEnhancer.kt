package openllve.android.domain

/**
 * LLIE pipeline + optional temporal topping.
 *
 * Architectural intent:
 * - `Ewma` is a temporal anti-flicker topping that works well for static LLIE pipelines.
 * - `Frame blending` is an additional output mixing stage that can be applied to either LLIE or temporal models.
 * - LLVE temporal models already carry state and typically do not need an independent EWMA topper.
 */
class LlieEwmaEnhancer(
    private val alpha: Float = 0.35f,
    private val blendAlpha: Float = 0.0f
) {
    private var previousFrame: FloatArray? = null

    fun enhance(currentFrame: FloatArray): FloatArray {
        val baseFrame = if (previousFrame == null) {
            previousFrame = currentFrame.copyOf()
            currentFrame.copyOf()
        } else {
            val smoothed = FloatArray(currentFrame.size)
            for (i in currentFrame.indices) {
                smoothed[i] = alpha * currentFrame[i] + (1f - alpha) * previousFrame!![i]
            }
            previousFrame = smoothed.copyOf()
            smoothed
        }

        return if (blendAlpha > 0f) {
            val blended = FloatArray(baseFrame.size)
            for (i in baseFrame.indices) {
                blended[i] = blendAlpha * baseFrame[i] + (1f - blendAlpha) * currentFrame[i]
            }
            blended
        } else {
            baseFrame
        }
    }
}