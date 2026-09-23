package openllve.shared.domain

/**
 * Compute backends the enhancement engine can run on. Not tied to any
 * LiteRT or CoreML delegate class; concrete wiring lives behind
 * [EnhancementEngine].
 */
enum class ComputeTarget {
    /** Plain CPU inference (always available). */
    CPU,

    /**
     * CPU with the XNNPACK delegate. LiteRT `CompiledModel` has no separate
     * XNNPACK accelerator, so this maps to [CPU] in `AndroidLiteRtEngine`.
     */
    XNNPACK,

    /** GPU delegate (OpenGL ES). Device/OpenGL dependent. */
    GPU,

    /** NPU via the NNAPI delegate. Requires a compatible device/NPU. */
    NPU,

    ;

    /** Human-readable label for the UI. */
    val label: String
        get() =
            when (this) {
                CPU -> "CPU"
                XNNPACK -> "XNNPACK (CPU)"
                GPU -> "GPU"
                NPU -> "NPU (NNAPI)"
            }
}
