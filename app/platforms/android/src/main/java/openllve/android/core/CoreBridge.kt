package openllve.android.core

/**
 * Kotlin↔Rust JNI contract.
 *
 * Each `external fun` below resolves at runtime to a `Java_openllve_android_core_CoreBridge_<name>`
 * symbol exported by `libbridge.so` (built by the Rust `bridge/android` crate with the
 * `jni` + `jni-macros` crates — see `docs/dev/architecture/FFI-WIRING.md`). The library
 * is loaded with [System.loadLibrary] once per process before the first call.
 *
 * Handles are JVM `Long` (never `ULong`), frames are JVM `FloatArray`, dimensions are
 * `Int`, strides are bytes per row. Every fallible call returns an `OpenLlveError` code
 * (`0` = success) rather than throwing.
 */
class CoreBridge {
    /** The C ABI version the loaded library implements, for startup validation. */
    external fun abiVersion(): Int

    /** Creates a stateless LLIE pipeline. Returns a native handle, `0` on failure. */
    external fun pipelineNewLlie(): Long

    /** Creates a temporal (EWMA + blend) pipeline. Returns a native handle, `0` on failure. */
    external fun pipelineNewTemporal(): Long

    /**
     * Creates an LLIE pipeline bound to a model file.
     *
     * @param modelPath absolute path readable by the process (not an APK asset path)
     * @param numThreads `0` = let the core decide
     * @return a native handle, `0` on failure
     */
    external fun pipelineNewLlieWithModel(
        modelPath: String,
        numThreads: Int,
    ): Long

    /** Releases a pipeline created by this class. */
    external fun pipelineFree(pipeline: Long)

    /**
     * Runs the pipeline over one frame.
     *
     * @param pipeline handle from [pipelineNewLlie] or [pipelineNewTemporal]
     * @param inWidth,inHeight,inChannels,inStride input frame geometry, stride in bytes
     * @param inData input frame, row-major RGB floats in `[0, 1]`
     * @param outWidth,outHeight,outChannels,outStride output frame geometry
     * @param outData output buffer, at least `outStride * outHeight / 4` elements
     * @return `0` on success, otherwise an `OpenLlveError` code
     */
    external fun processFrame(
        pipeline: Long,
        inWidth: Int,
        inHeight: Int,
        inChannels: Int,
        inStride: Int,
        inData: FloatArray,
        outWidth: Int,
        outHeight: Int,
        outChannels: Int,
        outStride: Int,
        outData: FloatArray,
    ): Int

    /** Creates an EWMA filter. Returns a native handle, `0` on failure. */
    external fun ewmaFilterNew(alpha: Float): Long

    /**
     * Smooths one frame against the filter's previous output.
     *
     * @param filter handle from [ewmaFilterNew]
     * @param width,height,channels,stride frame geometry, stride in bytes
     * @param inData current frame
     * @param outData output buffer, at least `stride * height / 4` elements
     * @return `0` on success, otherwise an `OpenLlveError` code
     */
    external fun ewmaFilterApply(
        filter: Long,
        width: Int,
        height: Int,
        channels: Int,
        stride: Int,
        inData: FloatArray,
        outData: FloatArray,
    ): Int

    /** Releases an EWMA filter. */
    external fun ewmaFilterFree(filter: Long)

    /** Creates a blend filter. Returns a native handle, `0` on failure. */
    external fun blendFilterNew(beta: Float): Long

    /**
     * Blends the raw and model-enhanced frames.
     *
     * @param raw the model's input frame
     * @param enhanced the model's output frame
     * @param outData output buffer, at least `stride * height / 4` elements
     * @return `0` on success, otherwise an `OpenLlveError` code
     */
    external fun blendFilterApply(
        filter: Long,
        width: Int,
        height: Int,
        channels: Int,
        stride: Int,
        raw: FloatArray,
        enhanced: FloatArray,
        outData: FloatArray,
    ): Int

    /** Releases a blend filter. */
    external fun blendFilterFree(filter: Long)

    /** Creates benchmark metrics. Returns a native handle, `0` on failure. */
    external fun metricsNew(): Long

    /** Creates metrics that ignore the first `warmupFrames` samples. */
    external fun metricsNewWithWarmup(warmupFrames: Int): Long

    /** Adds one latency sample. */
    external fun metricsRecord(
        metrics: Long,
        latencyMs: Double,
    )

    /** Mean latency in milliseconds, `-1.0` when no samples. */
    external fun metricsAverage(metrics: Long): Double

    /** Median latency in milliseconds, `-1.0` when no samples. */
    external fun metricsMedian(metrics: Long): Double

    /** Mean throughput in frames per second, `-1.0` when no samples. */
    external fun metricsFps(metrics: Long): Double

    /** 99th-percentile latency in milliseconds, `-1.0` when no samples. */
    external fun metricsP99(metrics: Long): Double

    /** Samples recorded so far. */
    external fun metricsCount(metrics: Long): Int

    /** Discards recorded samples. */
    external fun metricsReset(metrics: Long)

    /** Releases metrics created by this class. */
    external fun metricsFree(metrics: Long)
}
