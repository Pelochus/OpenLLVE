package openllve.android.ui.viewmodel

import android.content.Context
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import openllve.android.data.SettingsRepository
import openllve.android.domain.BackendSelection
import openllve.android.domain.EnhancementEngine
import openllve.android.domain.EnhancementSettings
import openllve.android.domain.ImageInput
import openllve.android.domain.MediaInput
import openllve.android.domain.ProcessingMetrics
import openllve.android.domain.VideoInput
import openllve.android.media.FramePixels
import openllve.android.media.ImageFrameProvider
import openllve.android.media.VideoFrameProvider
import openllve.android.media.VideoMetadataReader
import openllve.android.ui.state.UiState

/**
 * Presentation logic for the enhancement flow. Owns the application state and
 * orchestrates the media layer and the [EnhancementEngine]. It depends only on
 * domain types — the UI never sees LiteRT classes.
 *
 * The engine is shared between the image and video paths; only one is active
 * at a time (selecting media resets the other).
 */
class EnhancementViewModel(
    private val context: Context,
    private val engine: EnhancementEngine,
    private val settingsRepository: SettingsRepository
) : ViewModel() {

    private val imageProvider = ImageFrameProvider(context)
    private val videoProvider = VideoFrameProvider(context, engine)

    private val _uiState = MutableStateFlow(UiState(settings = EnhancementSettings()))
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    /** Model display name (domain value; the engine stays hidden from the UI). */
    val modelName: String
        get() = engine.modelName

    init {
        viewModelScope.launch {
            settingsRepository.observeSettings().collect { settings ->
                _uiState.update { it.copy(settings = settings) }
            }
        }
        viewModelScope.launch {
            val probe = withContext(Dispatchers.Default) { engine.probeBackends(context) }
            _uiState.update { it.copy(backendProbe = probe) }
        }
    }

    // ---- Settings ----

    fun updateSettings(settings: EnhancementSettings) {
        viewModelScope.launch { settingsRepository.updateSettings(settings) }
    }

    // ---- Image path ----

    fun selectImage(uri: Uri, name: String) {
        resetResult()
        _uiState.update { it.copy(input = ImageInput(uri, name)) }
        viewModelScope.launch {
            _uiState.update { it.copy(processing = true, error = null) }
            try {
                val result = withContext(Dispatchers.Default) {
                    val bitmap = imageProvider.loadBitmap(uri)
                    val floats = FramePixels.bitmapToFloatRgb(bitmap)
                    val selection = engine.configure(context, _uiState.value.settings)
                    val enhanced = engine.enhanceFrame(floats, bitmap.width, bitmap.height)
                    val enhancedBitmap = FramePixels.floatRgbToBitmap(enhanced, bitmap.width, bitmap.height)
                    EnhancedImageResult(bitmap, enhancedBitmap, selection, engine.lastInferenceMs)
                }
                val metrics = ProcessingMetrics(
                    frameCount = 1,
                    inferenceMs = result.inferenceMs,
                    averageFrameMs = result.inferenceMs.toDouble(),
                    fps = if (result.inferenceMs > 0) 1000.0 / result.inferenceMs else 0.0,
                    inputResolution = "${result.original.width}×${result.original.height}",
                    backend = result.selection.actual,
                    modelName = engine.modelName
                )
                _uiState.update {
                    it.copy(
                        imageOriginal = result.original,
                        imageEnhanced = result.enhanced,
                        metrics = metrics,
                        backendSelection = result.selection,
                        processing = false
                    )
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(error = "Image processing failed: ${e.message ?: "unknown error"}", processing = false) }
            }
        }
    }

    fun rerunImage() {
        val input = _uiState.value.input as? ImageInput ?: return
        selectImage(input.uri, input.displayName)
    }

    // ---- Video path ----

    fun selectVideo(uri: Uri, name: String) {
        stopVideo()
        resetResult()
        _uiState.update { it.copy(input = VideoInput(uri, name)) }
        viewModelScope.launch {
            val metadata = withContext(Dispatchers.Default) { VideoMetadataReader.read(context, uri) }
            _uiState.update { it.copy(videoMetadata = metadata) }
        }
    }

    fun startVideo() {
        val input = _uiState.value.input as? VideoInput ?: return
        stopVideo()
        _uiState.update {
            it.copy(
                videoOriginal = null,
                videoEnhanced = null,
                videoFrameCount = 0,
                videoInferenceMs = 0L,
                videoBackend = null,
                error = null
            )
        }
        videoProvider.start(input.uri, _uiState.value.settings)
        collectVideoState()
    }

    fun stopVideo() {
        videoProvider.stop()
        _uiState.update { it.copy(videoPlaying = false) }
    }

    private fun collectVideoState() {
        viewModelScope.launch {
            videoProvider.backendSelection.collect { sel -> _uiState.update { it.copy(videoBackend = sel) } }
        }
        viewModelScope.launch {
            videoProvider.isPlaying.collect { playing -> _uiState.update { it.copy(videoPlaying = playing) } }
        }
        viewModelScope.launch {
            videoProvider.latestOriginal.collect { b -> _uiState.update { it.copy(videoOriginal = b) } }
        }
        viewModelScope.launch {
            videoProvider.latestEnhanced.collect { b -> _uiState.update { it.copy(videoEnhanced = b) } }
        }
        viewModelScope.launch {
            videoProvider.frameCount.collect { c -> _uiState.update { it.copy(videoFrameCount = c) } }
        }
        viewModelScope.launch {
            videoProvider.inferenceMsTotal.collect { ms -> _uiState.update { it.copy(videoInferenceMs = ms) } }
        }
        viewModelScope.launch {
            videoProvider.errorMessage.collect { e -> _uiState.update { it.copy(error = e) } }
        }
    }

    private fun resetResult() {
        _uiState.update {
            it.copy(
                imageOriginal = null,
                imageEnhanced = null,
                videoOriginal = null,
                videoEnhanced = null,
                videoMetadata = null,
                videoPlaying = false,
                videoFrameCount = 0,
                videoInferenceMs = 0L,
                videoBackend = null,
                metrics = null,
                backendSelection = null,
                error = null,
                processing = false
            )
        }
    }

    override fun onCleared() {
        videoProvider.stop()
        engine.release()
    }

    /** Result of a single image enhancement. */
    private data class EnhancedImageResult(
        val original: android.graphics.Bitmap,
        val enhanced: android.graphics.Bitmap,
        val selection: BackendSelection,
        val inferenceMs: Long
    )

    companion object {
        fun factory(
            context: Context,
            engine: EnhancementEngine,
            settingsRepository: SettingsRepository
        ): ViewModelProvider.Factory {
            return object : ViewModelProvider.Factory {
                @Suppress("UNCHECKED_CAST")
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    return EnhancementViewModel(context, engine, settingsRepository) as T
                }
            }
        }
    }
}
