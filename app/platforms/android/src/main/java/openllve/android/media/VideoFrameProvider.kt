package openllve.android.media

import android.content.Context
import android.graphics.Bitmap
import android.media.MediaCodec
import android.media.MediaExtractor
import android.media.MediaFormat
import android.net.Uri
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.runBlocking
import openllve.android.domain.BackendSelection
import openllve.android.domain.EnhancementEngine
import openllve.android.domain.EnhancementSettings
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Decodes an MP4 with the platform's hardware [MediaCodec] (via
 * [MediaExtractor] for demuxing) and runs each decoded frame through the
 * [EnhancementEngine].
 *
 * This is Android-native media handling (no custom decoder). It produces, per
 * decoded frame, a paired (original, enhanced) bitmap so the UI can compare
 * them in sync. Audio is intentionally dropped in this prototype (video-only
 * decode); see TODO-app.md.
 *
 * The whole decode/process loop runs on a single dedicated thread because the
 * LiteRT interpreter is not thread-safe and the engine must be used from one
 * thread.
 */
class VideoFrameProvider(
    private val context: Context,
    private val engine: EnhancementEngine
) {
    private var thread: Thread? = null
    private val running = AtomicBoolean(false)

    val latestOriginal = MutableStateFlow<Bitmap?>(null)
    val latestEnhanced = MutableStateFlow<Bitmap?>(null)
    val isPlaying = MutableStateFlow(false)
    val errorMessage = MutableStateFlow<String?>(null)
    val frameCount = MutableStateFlow(0)
    val inferenceMsTotal = MutableStateFlow(0L)
    val backendSelection = MutableStateFlow<BackendSelection?>(null)

    /** Starts decoding and enhancing [uri] with [settings]. */
    fun start(uri: Uri, settings: EnhancementSettings) {
        stop()
        latestOriginal.value = null
        latestEnhanced.value = null
        errorMessage.value = null
        frameCount.value = 0
        inferenceMsTotal.value = 0L
        backendSelection.value = null

        val worker = Thread {
            if (!running.compareAndSet(false, true)) return
            try {
                val selection = runBlocking { engine.configure(context, settings) }
                backendSelection.value = selection
                isPlaying.value = true
                decodeAndEnhance(uri, selection)
            } catch (e: Exception) {
                errorMessage.value = "Video processing failed: ${e.message ?: "unknown error"}"
            } finally {
                isPlaying.value = false
                running.set(false)
            }
        }
        worker.name = "openllve-video-decode"
        this.thread = worker
        worker.start()
    }

    fun stop() {
        running.set(false)
        thread?.interrupt()
        thread = null
        isPlaying.value = false
    }

    private fun decodeAndEnhance(uri: Uri, selection: BackendSelection) {
        val extractor = MediaExtractor()
        extractor.setDataSource(uri)
        extractor.prepare()

        val videoTrack = findVideoTrack(extractor)
            ?: run {
                extractor.release()
                throw IllegalArgumentException("No decodable video track found in this file")
            }
        val videoFormat = extractor.getTrackFormat(videoTrack)
        extractor.selectTrack(videoTrack)
        val mime = videoFormat.getString(MediaFormat.KEY_MIME)
            ?: run {
                extractor.release()
                throw IllegalArgumentException("Unknown video mime type")
            }

        val codec = MediaCodec.createDecoderByType(mime)
        codec.configure(videoFormat, null, null, 0)
        codec.start()

        val inputSample = ByteArray(64 * 1024)
        val outputInfo = MediaCodec.BufferInfo()
        var inputBufferIndex = codec.dequeueInputBuffer(INPUT_TIMEOUT_US)
        var eosFed = false
        var eos = false

        try {
            while (!eos && running.get()) {
                if (!eosFed && inputBufferIndex >= 0) {
                    val inputBuffer = codec.getInputBuffer(inputBufferIndex)
                    if (inputBuffer != null) {
                        val sampleSize = extractor.readSampleData(inputSample, 0)
                        if (sampleSize < 0) {
                            inputBuffer.setEncodingFlags(MediaCodec.BUFFER_FLAG_END_OF_STREAM)
                            inputBuffer.setPresentationTime(0)
                            eosFed = true
                        } else {
                            inputBuffer.put(inputSample, 0, sampleSize)
                            inputBuffer.setEncodingFlags(
                                if (extractor.sampleFlags and MediaExtractor.BUFFER_FLAG_SYNC_FRAME != 0) {
                                    MediaCodec.BUFFER_FLAG_SYNC_FRAME
                                } else {
                                    0
                                }
                            )
                            inputBuffer.setPresentationTime(extractor.sampleTime)
                        }
                        codec.queueInputBuffer(
                            inputBufferIndex,
                            0,
                            if (sampleSize < 0) 0 else sampleSize,
                            if (sampleSize < 0) 0 else extractor.sampleTime
                        )
                    }
                    inputBufferIndex = codec.dequeueInputBuffer(INPUT_TIMEOUT_US)
                }

                while (!eos) {
                    val idx = codec.dequeueOutputBuffer(outputInfo, 0)
                    when {
                        idx == MediaCodec.INFO_OUTPUT_BUFFERS_CHANGED -> continue
                        idx == MediaCodec.INFO_OUTPUT_EOS -> {
                            eos = true
                            break
                        }
                        idx < 0 -> break
                        else -> {
                            if (outputInfo.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG != 0) {
                                codec.releaseOutputBuffer(idx, false)
                            } else {
                                val hardwareBuffer: android.hardware.HardwareBuffer? =
                                    codec.getOutputBuffer(idx)?.getHardwareBuffer()
                                if (hardwareBuffer != null) {
                                    val original = Bitmap.createBitmap(Bitmap.wrapHardwareBuffer(hardwareBuffer, null))
                                    val enhanced = enhanceBitmap(original, selection)
                                    latestOriginal.value = original
                                    latestEnhanced.value = enhanced
                                    frameCount.value += 1
                                    inferenceMsTotal.value += engine.lastInferenceMs
                                }
                                codec.releaseOutputBuffer(idx, true)
                            }
                        }
                    }
                }
            }
        } finally {
            try {
                codec.stop()
            } catch (_: Exception) {
                // ignore stop errors during teardown
            }
            codec.release()
            extractor.release()
        }
    }

    private fun enhanceBitmap(original: Bitmap, selection: BackendSelection): Bitmap {
        val w = original.width
        val h = original.height
        val floats = FramePixels.bitmapToFloatRgb(original)
        val enhanced = engine.enhanceFrame(floats, w, h)
        return FramePixels.floatRgbToBitmap(enhanced, w, h)
    }

    private fun findVideoTrack(extractor: MediaExtractor): Int? {
        for (i in 0 until extractor.trackCount) {
            val mime = extractor.getTrackFormat(i).getString(MediaFormat.KEY_MIME)
            if (mime != null && mime.startsWith("video/")) return i
        }
        return null
    }

    companion object {
        private const val INPUT_TIMEOUT_US = 10_000L
    }
}
