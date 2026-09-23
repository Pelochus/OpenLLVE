package openllve.shared.ui

import openllve.shared.domain.BackendProbeResult
import openllve.shared.domain.BackendSelection
import openllve.shared.domain.EnhancementSettings
import openllve.shared.domain.MediaInput
import openllve.shared.domain.ProcessingMetrics
import openllve.shared.media.FrameImage
import openllve.shared.media.VideoMetadata

/**
 * Presentation state for the enhancement flow: one flat state, each screen
 * reads the fields it needs. Frames are the platform-neutral [FrameImage];
 * each host converts to its native pixel type only at render time.
 */
data class UiState(
    val settings: EnhancementSettings,
    val backendProbe: BackendProbeResult? = null,
    val input: MediaInput? = null,
    // Image path
    val imageOriginal: FrameImage? = null,
    val imageEnhanced: FrameImage? = null,
    // Video path
    val videoMetadata: VideoMetadata? = null,
    val videoOriginal: FrameImage? = null,
    val videoEnhanced: FrameImage? = null,
    val videoPlaying: Boolean = false,
    val videoFrameCount: Int = 0,
    val videoInferenceMs: Long = 0L,
    val videoBackend: BackendSelection? = null,
    // Shared
    val metrics: ProcessingMetrics? = null,
    val backendSelection: BackendSelection? = null,
    val processing: Boolean = false,
    val error: String? = null,
)
