package openllve.android.media

import android.graphics.Bitmap

/**
 * Converts between Android [Bitmap] (ARGB_8888) and the engine's working
 * format: row-major RGB floats in `[0, 1]` (size `width * height * 3`).
 *
 * Shared by both the image and video paths so the pixel format is consistent
 * at the [openllve.android.domain.EnhancementEngine] boundary.
 */
object FramePixels {

    fun bitmapToFloatRgb(bitmap: Bitmap): FloatArray {
        val w = bitmap.width
        val h = bitmap.height
        val pixels = IntArray(w * h)
        bitmap.getPixels(pixels, 0, w, 0, 0, w, h)
        val out = FloatArray(w * h * 3)
        for (i in pixels.indices) {
            val argb = pixels[i]
            val idx = i * 3
            out[idx] = ((argb shr 16) and 0xFF) / 255.0f
            out[idx + 1] = ((argb shr 8) and 0xFF) / 255.0f
            out[idx + 2] = (argb and 0xFF) / 255.0f
        }
        return out
    }

    fun floatRgbToBitmap(floats: FloatArray, width: Int, height: Int): Bitmap {
        val pixels = IntArray(width * height)
        for (i in pixels.indices) {
            val idx = i * 3
            val r = (floats[idx] * 255f).toInt().coerceIn(0, 255)
            val g = (floats[idx + 1] * 255f).toInt().coerceIn(0, 255)
            val b = (floats[idx + 2] * 255f).toInt().coerceIn(0, 255)
            pixels[i] = (0xFF shl 24) or (r shl 16) or (g shl 8) or b
        }
        val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        bitmap.setPixels(pixels, 0, width, 0, 0, width, height)
        return bitmap
    }
}
