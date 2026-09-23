package openllve.android.media

import android.content.Context
import android.graphics.Bitmap
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaExtractor
import android.media.MediaFormat
import android.net.Uri
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.runBlocking
import openllve.shared.domain.BackendSelection
import openllve.shared.domain.EnhancementEngine
import openllve.shared.domain.EnhancementSettings
import openllve.shared.media.FrameImage
import openllve.shared.media.FramePixels
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Decodes an MP4 with the platform's hardware [MediaCodec] (demuxed via
 * [MediaExtractor]) and runs each decoded frame through the
 * [EnhancementEngine], producing paired (original, enhanced) bitmaps. Audio
 * is dropped (video-only decode). Runs on a single dedicated thread: the
 * LiteRT `CompiledModel` is not thread-safe.
 */
class VideoFrameProvider(
    private val context: Context,
    private val engine: EnhancementEngine,
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
    fun start(
        uri: Uri,
        settings: EnhancementSettings,
    ) {
        stop()
        latestOriginal.value = null
        latestEnhanced.value = null
        errorMessage.value = null
        frameCount.value = 0
        inferenceMsTotal.value = 0L
        backendSelection.value = null

        val worker =
            Thread {
                if (running.compareAndSet(false, true)) {
                    try {
                        val selection = runBlocking { engine.configure(settings) }
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

    private fun decodeAndEnhance(
        uri: Uri,
        selection: BackendSelection,
    ) {
        // MediaExtractor has no Uri overload; open the SAF Uri via the
        // ContentResolver and keep the descriptor open for the whole decode.
        val fd =
            context.contentResolver.openFileDescriptor(uri, "r")
                ?: throw IllegalStateException("Could not open a file descriptor for $uri")

        val extractor = MediaExtractor()
        extractor.setDataSource(fd.fileDescriptor, 0, fd.statSize)

        val videoTrack =
            findVideoTrack(extractor)
                ?: run {
                    extractor.release()
                    fd.close()
                    throw IllegalArgumentException("No decodable video track found in this file")
                }
        val videoFormat = extractor.getTrackFormat(videoTrack)
        extractor.selectTrack(videoTrack)
        val mime =
            videoFormat.getString(MediaFormat.KEY_MIME)
                ?: run {
                    extractor.release()
                    fd.close()
                    throw IllegalArgumentException("Unknown video mime type")
                }

        val codec = MediaCodec.createDecoderByType(mime)
        // Software output (null surface): decoded frames come back as a ByteBuffer
        // in the codec's YUV format, which we convert to an ARGB_8888 Bitmap.
        codec.configure(videoFormat, null, null, 0)
        codec.start()
        val outputFormat = codec.getOutputFormat()
        val colorFormat = outputFormat.getInteger(MediaFormat.KEY_COLOR_FORMAT)
        val frameWidth = outputFormat.getInteger(MediaFormat.KEY_WIDTH)
        val frameHeight = outputFormat.getInteger(MediaFormat.KEY_HEIGHT)

        val inputSample = ByteArray(64 * 1024)
        val inputSampleBuffer = ByteBuffer.wrap(inputSample)
        val outputInfo = MediaCodec.BufferInfo()
        var inputBufferIndex = codec.dequeueInputBuffer(INPUT_TIMEOUT_US)
        var eosFed = false
        var finished = false

        try {
            while (!finished && running.get()) {
                // Feed the next input sample (or the EOS flag when the stream is
                // exhausted).
                if (!eosFed && inputBufferIndex >= 0) {
                    val inputBuffer = codec.getInputBuffer(inputBufferIndex)
                    if (inputBuffer != null) {
                        val sampleSize = extractor.readSampleData(inputSampleBuffer, 0)
                        if (sampleSize < 0) {
                            inputBuffer.clear()
                            codec.queueInputBuffer(
                                inputBufferIndex,
                                0,
                                0,
                                0,
                                MediaCodec.BUFFER_FLAG_END_OF_STREAM,
                            )
                            eosFed = true
                        } else {
                            inputBuffer.clear()
                            inputBuffer.put(inputSample, 0, sampleSize)
                            inputBuffer.position(0)
                            val flags =
                                if (extractor.sampleFlags and MediaExtractor.SAMPLE_FLAG_SYNC != 0) {
                                    MediaCodec.BUFFER_FLAG_SYNC_FRAME
                                } else {
                                    0
                                }
                            codec.queueInputBuffer(
                                inputBufferIndex,
                                0,
                                sampleSize,
                                extractor.sampleTime,
                                flags,
                            )
                        }
                    }
                    inputBufferIndex = codec.dequeueInputBuffer(INPUT_TIMEOUT_US)
                }

                // Drain decoded output frames until none are available.
                while (!finished) {
                    val idx = codec.dequeueOutputBuffer(outputInfo, OUTPUT_TIMEOUT_US)
                    when (idx) {
                        MediaCodec.INFO_OUTPUT_BUFFERS_CHANGED,
                        MediaCodec.INFO_OUTPUT_FORMAT_CHANGED,
                        -> {
                            Unit
                        }

                        MediaCodec.INFO_TRY_AGAIN_LATER -> {
                            // After the EOS flag is fed, TRY_AGAIN_LATER means the
                            // decoder has drained every frame. Stop.
                            if (eosFed) finished = true
                            break
                        }

                        else -> {
                            if (idx < 0) break
                            if (outputInfo.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG != 0) {
                                codec.releaseOutputBuffer(idx, false)
                            } else {
                                val outBuffer = codec.getOutputBuffer(idx)
                                if (outBuffer != null) {
                                    val original = yuvToBitmap(outBuffer, frameWidth, frameHeight, colorFormat)
                                    val enhanced = enhanceBitmap(original)
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
            fd.close()
        }
    }

    /**
     * Converts a software-decoded YUV frame (NV12 or YV12) into an ARGB_8888
     * [Bitmap]; row stride equals the frame width (no padding).
     */
    private fun yuvToBitmap(
        buffer: ByteBuffer,
        width: Int,
        height: Int,
        colorFormat: Int,
    ): Bitmap {
        val yPlane = ByteArray(width * height)
        buffer.rewind()
        buffer.get(yPlane)
        val pixels = IntArray(width * height)
        if (colorFormat == MediaCodecInfo.CodecCapabilities.COLOR_FormatYUV420PackedPlanar) {
            // YV12: Y plane, then U plane, then V plane (w*h/4 bytes each).
            val uPlane = ByteArray(width * height / 4)
            val vPlane = ByteArray(width * height / 4)
            buffer.get(uPlane)
            buffer.get(vPlane)
            for (py in 0 until height) {
                for (px in 0 until width) {
                    val y = (yPlane[py * width + px].toInt() and 0xFF) - 16
                    val uvIdx = (py / 2) * (width / 2) + (px / 2)
                    val u = (uPlane[uvIdx].toInt() and 0xFF) - 128
                    val v = (vPlane[uvIdx].toInt() and 0xFF) - 128
                    pixels[py * width + px] = yuvToArgb(y, u, v)
                }
            }
        } else {
            // NV12 (default): Y plane, then interleaved UV (w*h/2 bytes).
            val uv = ByteArray(width * height / 2)
            buffer.get(uv)
            for (py in 0 until height) {
                for (px in 0 until width) {
                    val y = (yPlane[py * width + px].toInt() and 0xFF) - 16
                    val uvIdx = (py / 2) * width + px * 2
                    val u = (uv[uvIdx].toInt() and 0xFF) - 128
                    val v = (uv[uvIdx + 1].toInt() and 0xFF) - 128
                    pixels[py * width + px] = yuvToArgb(y, u, v)
                }
            }
        }
        val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        bitmap.setPixels(pixels, 0, width, 0, 0, width, height)
        return bitmap
    }

    private fun yuvToArgb(
        y: Int,
        u: Int,
        v: Int,
    ): Int {
        val r = (1.164f * y + 1.596f * v).toInt().coerceIn(0, 255)
        val g = (1.164f * y - 0.391f * u - 0.813f * v).toInt().coerceIn(0, 255)
        val b = (1.164f * y + 2.018f * u).toInt().coerceIn(0, 255)
        return (0xFF shl 24) or (r shl 16) or (g shl 8) or b
    }

    private fun enhanceBitmap(original: Bitmap): Bitmap {
        val frame = BitmapFrameAdapter.toFrame(original)
        val w = frame.width
        val h = frame.height
        val floats = FramePixels.argbToFloatRgb(frame.pixels)
        val enhanced = engine.enhanceFrame(floats, w, h)
        val argb = FramePixels.floatRgbToArgb(enhanced, w, h)
        return BitmapFrameAdapter.toBitmap(FrameImage(w, h, argb))
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
        private const val OUTPUT_TIMEOUT_US = 10_000L
    }
}
