package openllve.shared.domain

/**
 * The seam between the UI and the inference backend.
 *
 * The UI depends only on this interface and the domain types
 * ([ComputeTarget], [EnhancementSettings], [BackendSelection],
 * [ProcessingMetrics]) — it never sees LiteRT/TFLite or CoreML classes.
 *
 * Current (temporary) implementation:
 * [openllve.android.engine.AndroidLiteRtEngine], which runs the model directly
 * on the LiteRT runtime on Android.
 *
 * Later: a `RustEngine` implementation of this same interface will call the
 * Rust core via the C FFI (`openllve_*`), and the UI will not change. The
 * domain configuration ([EnhancementSettings]) is shaped so the Rust core can
 * consume it directly (delegate selection, EWMA/flicker toppings).
 *
 * Platform-neutral: the interface takes no host types (no Android `Context`,
 * no iOS `URLSession`); a host implementation receives what it needs at
 * construction time.
 */
interface EnhancementEngine {

    /** The model file (asset path) this engine runs. Exposed for display only. */
    val modelAssetPath: String

    /** The model's display name (for the metrics screen). */
    val modelName: String

    /**
     * Probes which [ComputeTarget]s are actually usable on this device and
     * returns a note for each unsupported one. Used to populate/disable the
     * compute-target selector in the UI.
     */
    suspend fun probeBackends(): BackendProbeResult

    /**
     * Creates (or re-creates) the interpreter for [settings.computeTarget].
     * Returns the [BackendSelection] describing which backend was actually
     * used and, if it fell back, why.
     */
    suspend fun configure(settings: EnhancementSettings): BackendSelection

    /**
     * Enhances a single RGB frame.
     *
     * @param input row-major RGB floats in `[0, 1]`, size `width * height * 3`.
     * @return enhanced RGB floats in `[0, 1]`, same size.
     */
    fun enhanceFrame(input: FloatArray, width: Int, height: Int): FloatArray

    /** Inference time of the last [enhanceFrame] call, in ms (0 if unknown). */
    val lastInferenceMs: Long

    /** Releases native resources. Safe to call more than once. */
    fun release()
}

/** Result of [EnhancementEngine.probeBackends]. */
data class BackendProbeResult(
    val supported: Set<ComputeTarget>,
    /** Reason a target is unsupported (only present for unsupported targets). */
    val notes: Map<ComputeTarget, String>
) {
    fun isSupported(target: ComputeTarget): Boolean = target in supported
    fun noteFor(target: ComputeTarget): String? = notes[target]
}
