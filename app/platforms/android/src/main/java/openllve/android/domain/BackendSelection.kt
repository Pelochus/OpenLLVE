package openllve.android.domain

/**
 * The backend that was actually used for inference, compared to the one the
 * user requested.
 *
 * The app never silently falls back: when [actual] != [requested], [reason]
 * explains why (e.g. "NPU delegate unavailable on this device").
 */
data class BackendSelection(
    val requested: ComputeTarget,
    val actual: ComputeTarget,
    val reason: String? = null
) {
    val fellBack: Boolean
        get() = requested != actual

    /** One-line summary for the UI, e.g. "Requested: NPU — Actual: CPU". */
    val summary: String
        get() = if (fellBack) {
            "Requested: ${requested.label} — Actual: ${actual.label}"
        } else {
            "Backend: ${actual.label}"
        }
}
