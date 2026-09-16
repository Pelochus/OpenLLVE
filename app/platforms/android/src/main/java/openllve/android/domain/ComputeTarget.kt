package openllve.android.domain

/**
 * Compute backends the enhancement engine can run on.
 *
 * This is an OpenLLVE domain concept, intentionally **not** tied to any
 * LiteRT/TFLite delegate class. The UI and the eventual Rust core both
 * reason in terms of [ComputeTarget]; the concrete delegate wiring lives
 * behind [EnhancementEngine] (see `engine/AndroidLiteRtEngine`, which is a
 * temporary Android implementation slated to be replaced by the Rust engine
 * via the C FFI).
 */
enum class ComputeTarget {
    /** Plain CPU inference (always available). */
    CPU,

    /** CPU with the XNNPACK delegate (CPU optimization; broadly available). */
    XNNPACK,

    /** GPU delegate (OpenGL ES). Device/OpenGL dependent. */
    GPU,

    /** NPU via the NNAPI delegate. Requires a compatible device/NPU. */
    NPU;

    /** Human-readable label for the UI. */
    val label: String
        get() = when (this) {
            CPU -> "CPU"
            XNNPACK -> "XNNPACK (CPU)"
            GPU -> "GPU"
            NPU -> "NPU (NNAPI)"
        }
}
