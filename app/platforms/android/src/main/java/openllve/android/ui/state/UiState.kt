package openllve.android.ui.state

import android.graphics.Bitmap
import openllve.android.domain.BackendProbeResult
import openllve.android.domain.BackendSelection
import openllve.android.domain.EnhancementSettings
import openllve.android.domain.MediaInput
import openllve.android.domain.ProcessingMetrics
import openllve.android.media.VideoMetadata

/**
 * Presentation state for the enhancement flow. A single flat state (rather
 * than a deep sealed hierarchy) keeps the screens simple: each screen reads
 * the fields it needs.
 */
data class UiState(
    val settings: EnhancementSettings,
    val backendProbe: BackendProbeResult? = null,
    val input: MediaInput? = null,

    // Image path
    val imageOriginal: Bitmap? = null,
    val imageEnhanced: Bitmap? = null,

    // Video path
    val videoMetadata: VideoMetadata? = null,
    val videoOriginal: Bitmap? = null,
    val videoEnhanced: Bitmap? = null,
    val videoPlaying: Boolean = false,
    val videoFrameCount: Int = 0,
    val videoInferenceMs: Long = 0L,
    val videoBackend: BackendSelection? = null,

    // Shared
    val metrics: ProcessingMetrics? = null,
    val backendSelection: BackendSelection? = null,
    val processing: Boolean = false,
    val error: String? = null
)
