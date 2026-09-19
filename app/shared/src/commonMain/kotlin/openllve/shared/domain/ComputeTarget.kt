package openllve.shared.domain

/**
 * Compute backends the enhancement engine can run on.
 *
 * Shared OpenLLVE domain concept, intentionally **not** tied to any
 * LiteRT/TFLite or CoreML delegate class. The UI and the eventual Rust core
 * both reason in terms of [ComputeTarget]; the concrete delegate wiring lives
 * behind [EnhancementEngine] (on Android today:
 * `openllve.android.engine.AndroidLiteRtEngine`, a temporary implementation
 * slated to be replaced by the Rust engine via the C FFI).
 */
enum class ComputeTarget {
    /** Plain CPU inference (always available). */
    CPU,

    /**
     * CPU with the XNNPACK delegate (CPU optimization; broadly available).
     *
     * Note: the LiteRT 2.2.0 `CompiledModel` API has no separate XNNPACK
     * accelerator (the experimental YNNPACK CPU accelerator is a build/runtime
     * flag, not a delegate), so this target maps to [CPU] in
     * `AndroidLiteRtEngine` — a documented mapping, not a fallback.
     */
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
