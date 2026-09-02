package openllve.android.data

data class BenchmarkResult(
    val inferenceLatency: Long, // Latency in milliseconds
    val throughput: Double, // Frames per second
    val thermalStability: Double // Stability metric for thermal performance
)