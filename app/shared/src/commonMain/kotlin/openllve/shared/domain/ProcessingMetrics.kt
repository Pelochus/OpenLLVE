package openllve.shared.domain

import kotlin.math.roundToLong

/**
 * Minimal, useful performance metrics for a processing run.
 *
 * Kept isolated (plain data) so the numbers can later be replaced by
 * Rust/native measurements from the core's `BenchmarkMetrics` without
 * changing the UI.
 */
data class ProcessingMetrics(
    val frameCount: Int,
    /** Sum of per-frame inference time in ms. */
    val inferenceMs: Long,
    /** Average per-frame inference time in ms. */
    val averageFrameMs: Double,
    /** Throughput in processed frames per second (inference path only). */
    val fps: Double,
    val inputResolution: String,
    val backend: ComputeTarget,
    val modelName: String,
    /**
     * For video only: `videoDurationMs / inferenceMs`.
     *  > 1.0 -> faster than realtime,
     *  ~ 1.0 -> about realtime,
     *  < 1.0 -> slower than realtime.
     *  0.0 for a single image (no duration to compare against).
     */
    val realtimeFactor: Double = 0.0
) {
    val realtimeLabel: String?
        get() = if (realtimeFactor <= 0.0) {
            null
        } else {
            val factor = oneDecimal(realtimeFactor)
            when {
                realtimeFactor > 1.5 -> "faster than realtime (×$factor)"
                realtimeFactor >= 0.75 -> "approximately realtime (×$factor)"
                else -> "slower than realtime (×$factor)"
            }
        }

    /**
     * Common-code one-decimal formatting (JVM `String.format` is not
     * available in Kotlin multiplatform common code).
     */
    private fun oneDecimal(value: Double): String {
        // value is always > 0 here (guarded by the caller), so scaled >= 0
        val scaled = (value * 10).roundToLong()
        return "${scaled / 10}.${scaled % 10}"
    }
}
