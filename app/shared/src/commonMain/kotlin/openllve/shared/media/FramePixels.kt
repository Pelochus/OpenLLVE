package openllve.shared.media

/**
 * Converts between [FrameImage] (ARGB_8888 ints) and the engine's working
 * format: row-major RGB floats in `[0, 1]`.
 */
object FramePixels {

    /** ARGB_8888 ints -> row-major RGB floats in `[0, 1]`. */
    fun argbToFloatRgb(pixels: IntArray): FloatArray {
        val out = FloatArray(pixels.size * 3)
        for (i in pixels.indices) {
            val argb = pixels[i]
            val idx = i * 3
            out[idx] = ((argb shr 16) and 0xFF) / 255.0f
            out[idx + 1] = ((argb shr 8) and 0xFF) / 255.0f
            out[idx + 2] = (argb and 0xFF) / 255.0f
        }
        return out
    }

    /** Row-major RGB floats in `[0, 1]` -> ARGB_8888 ints. */
    fun floatRgbToArgb(floats: FloatArray, width: Int, height: Int): IntArray {
        val pixels = IntArray(width * height)
        for (i in pixels.indices) {
            val idx = i * 3
            val r = (floats[idx] * 255f).toInt().coerceIn(0, 255)
            val g = (floats[idx + 1] * 255f).toInt().coerceIn(0, 255)
            val b = (floats[idx + 2] * 255f).toInt().coerceIn(0, 255)
            pixels[i] = (0xFF shl 24) or (r shl 16) or (g shl 8) or b
        }
        return pixels
    }
}
