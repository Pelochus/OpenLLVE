package openllve.android.domain

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
            when {
                realtimeFactor > 1.5 -> "faster than realtime (×${"%.1f".format(realtimeFactor)})"
                realtimeFactor >= 0.75 -> "approximately realtime (×${"%.1f".format(realtimeFactor)})"
                else -> "slower than realtime (×${"%.1f".format(realtimeFactor)})"
            }
        }
}
