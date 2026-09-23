package openllve.android.engine

import android.content.Context
import android.os.SystemClock
import com.google.ai.edge.litert.Accelerator
import com.google.ai.edge.litert.CompiledModel
import com.google.ai.edge.litert.Environment
import com.google.ai.edge.litert.TensorBuffer
import com.google.ai.edge.litert.TensorType
import openllve.shared.domain.BackendProbeResult
import openllve.shared.domain.BackendSelection
import openllve.shared.domain.ComputeTarget
import openllve.shared.domain.EnhancementEngine
import openllve.shared.domain.EnhancementSettings
import java.io.File

/**
 * Temporary Android [EnhancementEngine]: runs the Zero-DCE model on the
 * LiteRT 2.2.0 `CompiledModel` API, mirroring the Rust `ModelRunner`
 * (patch tiling + learned curves), until the Rust engine is wired via the
 * C FFI (TODO.md P1.1).
 *
 * Backend notes: `CompiledModel` has no separate XNNPACK accelerator, so
 * [ComputeTarget.XNNPACK] maps to CPU (a mapping, not a fallback). Output
 * dtype is verified up front (FLOAT32); INT8 outputs are rejected because
 * the runtime exposes no quantization parameters. `configure` never falls
 * back silently — it reports the reason.
 */
class AndroidLiteRtEngine(
    private val context: Context,
    override val modelAssetPath: String = MODEL_ASSET_PATH,
    override val modelName: String = "zero-dce-int8",
) : EnhancementEngine {
    private var compiledModel: CompiledModel? = null
    private var inputBuffer: TensorBuffer? = null
    private var outputBuffer: TensorBuffer? = null

    @Volatile
    private var lastInferenceMsValue = 0L

    override val lastInferenceMs: Long
        get() = lastInferenceMsValue

    override suspend fun probeBackends(): BackendProbeResult {
        val supported = mutableSetOf<ComputeTarget>()
        val notes = mutableMapOf<ComputeTarget, String>()
        val available = Environment.create(context).use { it.getAvailableAccelerators() }
        for (target in ComputeTarget.entries) {
            val ok =
                when (target) {
                    // CPU is always available. XNNPACK maps to CPU: LiteRT
                    // `CompiledModel` has no separate XNNPACK accelerator (the
                    // experimental YNNPACK CPU accelerator is a build flag, not a
                    // delegate).
                    ComputeTarget.CPU, ComputeTarget.XNNPACK -> true

                    ComputeTarget.GPU -> Accelerator.GPU in available

                    ComputeTarget.NPU -> Accelerator.NPU in available
                }
            if (ok) {
                supported += target
            } else {
                notes[target] = "${target.label} unavailable on this device"
            }
        }
        return BackendProbeResult(supported, notes)
    }

    override suspend fun configure(settings: EnhancementSettings): BackendSelection {
        val modelFile = loadModelFile(context)
        val requested = settings.computeTarget
        release()

        val (model, actual, reason) = createModelWithFallback(modelFile, requested, defaultThreads())
        compiledModel = model

        // Verify the output dtype before reading: reinterpreting an INT8
        // buffer as floats would silently produce garbage.
        val outType = model.getOutputTensorType(OUTPUT_TENSOR_NAME)
        if (outType.elementType != TensorType.ElementType.FLOAT) {
            model.close()
            compiledModel = null
            throw IllegalStateException(
                "unsupported model output dtype ${outType.elementType} (expected FLOAT32; " +
                    "LiteRT 2.2.0 does not expose INT8 quantization parameters for dequantization)",
            )
        }

        inputBuffer = model.createInputBuffers().single()
        outputBuffer = model.createOutputBuffers().single()
        return BackendSelection(requested, actual, reason)
    }

    override fun enhanceFrame(
        input: FloatArray,
        width: Int,
        height: Int,
    ): FloatArray {
        val model = compiledModel ?: throw IllegalStateException("engine not configured; call configure() first")
        val inBuf = inputBuffer!!
        val outBuf = outputBuffer!!

        val (stride, numW, numH, wPad, hPad) = patchInfo(width, height)
        val brightness = brightnessChannel(input)

        val patchIn = FloatArray(PATCH * PATCH * 4)
        var patchOut = FloatArray(PATCH * PATCH * 24)
        val accum = FloatArray(hPad * wPad * 24)

        val start = SystemClock.elapsedRealtime()
        for (i in 0 until numH) {
            for (j in 0 until numW) {
                preparePatch(input, width, height, j * stride, i * stride, brightness, patchIn)
                patchOut = runModel(model, inBuf, outBuf, patchIn)
                accumulate(i, j, numH, numW, wPad, patchOut, accum)
            }
        }
        val output = FloatArray(width * height * 3)
        applyCurves(input, output, width, height, wPad, accum)
        this.lastInferenceMsValue = SystemClock.elapsedRealtime() - start
        return output
    }

    override fun release() {
        inputBuffer?.close()
        inputBuffer = null
        outputBuffer?.close()
        outputBuffer = null
        compiledModel?.close()
        compiledModel = null
    }

    // ---- Model path (mirrors core/src/model.rs) ----

    private fun createModelWithFallback(
        modelFile: File,
        requested: ComputeTarget,
        threads: Int,
    ): Triple<CompiledModel, ComputeTarget, String?> {
        if (requested == ComputeTarget.XNNPACK) {
            // Maps to CPU (no XNNPACK accelerator in `CompiledModel`).
            val model = CompiledModel.create(modelFile.absolutePath, buildOptions(ComputeTarget.CPU, threads))
            return Triple(model, ComputeTarget.XNNPACK, null)
        }
        try {
            val model = CompiledModel.create(modelFile.absolutePath, buildOptions(requested, threads))
            return Triple(model, requested, null)
        } catch (e: Exception) {
            // Never silently fall back: report why.
            val model = CompiledModel.create(modelFile.absolutePath, buildOptions(ComputeTarget.CPU, threads))
            return Triple(model, ComputeTarget.CPU, "${requested.label} unavailable: ${e.message ?: "unknown error"}")
        }
    }

    private fun buildOptions(
        target: ComputeTarget,
        threads: Int,
    ): CompiledModel.Options {
        val options =
            when (target) {
                ComputeTarget.CPU, ComputeTarget.XNNPACK -> CompiledModel.Options(Accelerator.CPU)
                ComputeTarget.GPU -> CompiledModel.Options(Accelerator.GPU)
                ComputeTarget.NPU -> CompiledModel.Options(Accelerator.NPU)
            }
        options.cpuOptions = CompiledModel.CpuOptions(numThreads = threads)
        return options
    }

    private fun defaultThreads(): Int = Runtime.getRuntime().availableProcessors().coerceAtLeast(1)

    /** Patch grid for a (w, h) frame: (stride, numW, numH, wPad, hPad). */
    private fun patchInfo(
        w: Int,
        h: Int,
    ): IntArray {
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
    private fun reflectCoord(
        dim: Int,
        pos: Int,
    ): Int {
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
        out: FloatArray,
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

    /** Runs one patch; `TensorBuffer.readFloat()` allocates a fresh array (no in-place read). */
    private fun runModel(
        model: CompiledModel,
        inBuf: TensorBuffer,
        outBuf: TensorBuffer,
        patchIn: FloatArray,
    ): FloatArray {
        inBuf.writeFloat(patchIn)
        model.run(listOf(inBuf), listOf(outBuf))
        return outBuf.readFloat()
    }

    private fun accumulate(
        i: Int,
        j: Int,
        numH: Int,
        numW: Int,
        wPad: Int,
        patchOut: FloatArray,
        accum: FloatArray,
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
        accum: FloatArray,
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

        /** Output tensor name in the model asset; the dtype query is name-based. */
        const val OUTPUT_TENSOR_NAME = "StatefulPartitionedCall_1:0"
    }
}
