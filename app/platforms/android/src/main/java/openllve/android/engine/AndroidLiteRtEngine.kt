package openllve.android.engine

import android.content.Context
import android.os.SystemClock
import openllve.android.domain.BackendProbeResult
import openllve.android.domain.BackendSelection
import openllve.android.domain.ComputeTarget
import openllve.android.domain.EnhancementEngine
import openllve.android.domain.EnhancementSettings
import org.tensorflow.lite.DataType
import org.tensorflow.lite.Interpreter
import org.tensorflow.lite.gpu.GpuDelegate
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Temporary Android implementation of [EnhancementEngine] that runs the
 * Zero-DCE model directly on the LiteRT (TFLite) runtime.
 *
 * WHY THIS EXISTS / WHY IT IS TEMPORARY
 * -------------------------------------
 * Per the architecture tenets (`docs/ARCHITECTURE.md` §2, §11), all business
 * logic and compute belongs in the Rust core, and the platform should call
 * the Rust core via the C FFI. That wiring (TODO.md P1.1: cross-compile the
 * cdylib, Kotlin `external fun`s, package the `.so`) is intentionally NOT done
 * in this Android-first slice.
 *
 * So that the app is a *functional* vertical slice (select media → LiteRT
 * inference → enhanced result), this class implements the model path in
 * Kotlin, faithfully mirroring the Rust `ModelRunner`
 * (`core/src/model.rs`): 256×256 patch tiling with 16px overlap, reflect
 * padding, linear-ramp reassembly, and the 8 learned curves.
 *
 * It is deliberately isolated behind [EnhancementEngine]: the UI and
 * ViewModels only see the domain types. When the Rust engine is wired, a
 * `RustEngine : EnhancementEngine` replaces this class and the UI does not
 * change. See TODO-app.md.
 */
class AndroidLiteRtEngine(
    override val modelAssetPath: String = MODEL_ASSET_PATH,
    override val modelName: String = "zero-dce-int8"
) : EnhancementEngine {

    private var interpreter: Interpreter? = null
    private var inputBuffer: ByteBuffer? = null
    private var outputBuffer: ByteBuffer? = null
    private var outputIsInt8 = false
    private var outputScale = 1.0f
    private var outputZeroPoint = 0

    @Volatile
    private var lastInferenceMsValue = 0L

    override val lastInferenceMs: Long
        get() = lastInferenceMsValue

    override suspend fun probeBackends(context: Context): BackendProbeResult {
        val modelFile = loadModelFile(context)
        val supported = mutableSetOf<ComputeTarget>()
        val notes = mutableMapOf<ComputeTarget, String>()
        for (target in ComputeTarget.entries) {
            try {
                val options = buildOptions(target)
                Interpreter(modelFile, options).use { it.close() }
                supported += target
            } catch (e: Exception) {
                notes[target] = "${target.label} unavailable: ${e.message ?: "unknown error"}"
            }
        }
        return BackendProbeResult(supported, notes)
    }

    override suspend fun configure(context: Context, settings: EnhancementSettings): BackendSelection {
        val modelFile = loadModelFile(context)
        val requested = settings.computeTarget
        release()

        val threads = Runtime.getRuntime().availableProcessors().coerceAtLeast(1)
        val (interp, actual, reason) = createInterpreterWithFallback(modelFile, requested, threads)
        interpreter = interp

        // The classic `org.tensorflow.lite` API feeds `Interpreter.run` raw direct
        // `ByteBuffer`s (there is no `TensorBuffer` class in this artifact). The
        // input is FLOAT32 `(1, 256, 256, 4)`; the output is `(1, 256, 256, 24)`
        // and may be FLOAT32 or INT8 (dequantized below), mirroring the Rust
        // `ModelRunner.run_model` dtype handling.
        inputBuffer = ByteBuffer.allocateDirect(PATCH * PATCH * 4 * 4).order(ByteOrder.nativeOrder())
        val outTensor = interp.getOutputTensor(0)
        outputIsInt8 = outTensor.dataType() == DataType.INT8
        if (outputIsInt8) {
            val q = outTensor.quantizationParams()
            outputScale = q.getScale()
            outputZeroPoint = q.getZeroPoint()
            outputBuffer = ByteBuffer.allocateDirect(PATCH * PATCH * 24).order(ByteOrder.nativeOrder())
        } else {
            outputScale = 1.0f
            outputZeroPoint = 0
            outputBuffer = ByteBuffer.allocateDirect(PATCH * PATCH * 24 * 4).order(ByteOrder.nativeOrder())
        }
        return BackendSelection(requested, actual, reason)
    }

    override fun enhanceFrame(input: FloatArray, width: Int, height: Int): FloatArray {
        val interp = interpreter ?: throw IllegalStateException("engine not configured; call configure() first")
        val inBuf = inputBuffer!!
        val outBuf = outputBuffer!!

        val (stride, numW, numH, wPad, hPad) = patchInfo(width, height)
        val brightness = brightnessChannel(input)

        val patchIn = FloatArray(PATCH * PATCH * 4)
        val patchOut = FloatArray(PATCH * PATCH * 24)
        val accum = FloatArray(hPad * wPad * 24)

        val start = SystemClock.elapsedRealtime()
        for (i in 0 until numH) {
            for (j in 0 until numW) {
                preparePatch(input, width, height, j * stride, i * stride, brightness, patchIn)
                runModel(interp, inBuf, outBuf, patchIn, patchOut)
                accumulate(i, j, numH, numW, wPad, patchOut, accum)
            }
        }
        val output = FloatArray(width * height * 3)
        applyCurves(input, output, width, height, wPad, accum)
        this.lastInferenceMsValue = SystemClock.elapsedRealtime() - start
        return output
    }

    override fun release() {
        interpreter?.close()
        interpreter = null
        inputBuffer = null
        outputBuffer = null
    }

    // ---- Model path (mirrors core/src/model.rs) ----

    private fun createInterpreterWithFallback(
        modelFile: File,
        requested: ComputeTarget,
        threads: Int
    ): Triple<Interpreter, ComputeTarget, String?> {
        try {
            val options = buildOptions(requested)
            val interp = Interpreter(modelFile, options)
            return Triple(interp, requested, null)
        } catch (e: Exception) {
            // Never silently fall back: report why.
            val cpuOptions = Interpreter.Options()
            cpuOptions.setNumThreads(threads)
            cpuOptions.setUseXNNPACK(false)
            val interp = Interpreter(modelFile, cpuOptions)
            return Triple(interp, ComputeTarget.CPU, "${requested.label} delegate unavailable: ${e.message ?: "unknown error"}")
        }
    }

    private fun buildOptions(target: ComputeTarget): Interpreter.Options {
        val options = Interpreter.Options()
        options.setNumThreads(Runtime.getRuntime().availableProcessors().coerceAtLeast(1))
        when (target) {
            ComputeTarget.CPU -> options.setUseXNNPACK(false)
            ComputeTarget.XNNPACK -> options.setUseXNNPACK(true)
            ComputeTarget.NPU -> options.setUseNNAPI(true)
            ComputeTarget.GPU -> options.addDelegate(GpuDelegate())
        }
        return options
    }

    /** Patch grid for a (w, h) frame: (stride, numW, numH, wPad, hPad). */
    private fun patchInfo(w: Int, h: Int): IntArray {
        val patch = PATCH
        val overlap = OVERLAP
        val stride = patch - overlap
        val numW = if (w <= patch) 1 else (w - patch + stride - 1) / stride + 1
        val numH = if (h <= patch) 1 else (h - patch + stride - 1) / stride + 1
        val wPad = (numW - 1) * stride + patch
        val hPad = (numH - 1) * stride + patch
        return intArrayOf(stride, numW, numH, wPad, hPad)
    }

    /** Brightness guidance channel: global RGB mean clamped to 0.5. */
    private fun brightnessChannel(input: FloatArray): Float {
        var sum = 0.0f
        for (v in input) sum += v
        return (sum / input.size).coerceAtMost(0.5f)
    }

    /** Reflect coordinate beyond `dim` (numpy mode='reflect'; edge not repeated). */
    private fun reflectCoord(dim: Int, pos: Int): Int {
        if (dim <= 1) return 0
        val period = 2 * (dim - 1)
        val p = pos % period
        return if (p < dim) p else period - p
    }

    private fun preparePatch(
        input: FloatArray,
        w: Int,
        h: Int,
        x0: Int,
        y0: Int,
        brightness: Float,
        out: FloatArray
    ) {
        for (py in 0 until PATCH) {
            val sy = reflectCoord(h, y0 + py)
            for (px in 0 until PATCH) {
                val sx = reflectCoord(w, x0 + px)
                val base = (py * PATCH + px) * 4
                val src = (sy * w + sx) * 3
                out[base] = input[src]
                out[base + 1] = input[src + 1]
                out[base + 2] = input[src + 2]
                out[base + 3] = brightness
            }
        }
    }

    private fun runModel(
        interp: Interpreter,
        inBuf: ByteBuffer,
        outBuf: ByteBuffer,
        patchIn: FloatArray,
        patchOut: FloatArray
    ) {
        // Fill the input from position 0. The native input path memcpys from the
        // direct buffer's start, so the input position does not matter, but we
        // rewind for clarity.
        inBuf.rewind()
        inBuf.asFloatBuffer().put(patchIn)
        // `Interpreter.run` writes the output tensor into `outBuf` starting at its
        // current position, so rewind before running and again before reading.
        outBuf.rewind()
        interp.run(inBuf, outBuf)
        outBuf.rewind()
        if (outputIsInt8) {
            val raw = ByteArray(PATCH * PATCH * 24)
            outBuf.get(raw)
            for (i in raw.indices) {
                patchOut[i] = (raw[i].toInt() - outputZeroPoint) * outputScale
            }
        } else {
            val raw = FloatArray(PATCH * PATCH * 24)
            outBuf.asFloatBuffer().get(raw)
            System.arraycopy(raw, 0, patchOut, 0, raw.size)
        }
    }

    private fun accumulate(
        i: Int,
        j: Int,
        numH: Int,
        numW: Int,
        wPad: Int,
        patchOut: FloatArray,
        accum: FloatArray
    ) {
        val denom = (OVERLAP - 1).toFloat()
        val stride = PATCH - OVERLAP
        val y0 = i * stride
        val x0 = j * stride
        for (py in 0 until PATCH) {
            var wy = if (i != 0 && py < OVERLAP) py / denom else 1.0f
            wy = if (i != numH - 1 && py >= PATCH - OVERLAP) wy * ((PATCH - 1 - py) / denom) else wy
            val rowBase = (y0 + py) * wPad
            for (px in 0 until PATCH) {
                var wx = if (j != 0 && px < OVERLAP) px / denom else 1.0f
                wx = if (j != numW - 1 && px >= PATCH - OVERLAP) wx * ((PATCH - 1 - px) / denom) else wx
                val weight = wx * wy
                val dst = rowBase + x0 + px
                val src = (py * PATCH + px) * 24
                for (c in 0 until 24) {
                    accum[dst * 24 + c] += patchOut[src + c] * weight
                }
            }
        }
    }

    private fun applyCurves(
        input: FloatArray,
        output: FloatArray,
        w: Int,
        h: Int,
        wPad: Int,
        accum: FloatArray
    ) {
        for (y in 0 until h) {
            for (x in 0 until w) {
                val base = (y * wPad + x) * 24
                val idx = (y * w + x) * 3
                var r = input[idx]
                var g = input[idx + 1]
                var b = input[idx + 2]
                for (k in 0 until 8) {
                    r += accum[base + 3 * k] * (r * r - r)
                    g += accum[base + 3 * k + 1] * (g * g - g)
                    b += accum[base + 3 * k + 2] * (b * b - b)
                }
                output[idx] = r.coerceIn(0.0f, 1.0f)
                output[idx + 1] = g.coerceIn(0.0f, 1.0f)
                output[idx + 2] = b.coerceIn(0.0f, 1.0f)
            }
        }
    }

    private fun loadModelFile(context: Context): File {
        val file = File(context.cacheDir, modelAssetPath.substringAfterLast('/'))
        if (!file.exists()) {
            context.assets.open(modelAssetPath).use { input ->
                file.outputStream().use { output -> input.copyTo(output) }
            }
        }
        return file
    }

    companion object {
        const val MODEL_ASSET_PATH = "models/zero-dce-int8.tflite"
        const val PATCH = 256
        const val OVERLAP = 16
    }
}
