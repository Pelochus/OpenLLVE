package openllve.android.media

import android.graphics.Bitmap
import android.net.Uri
import openllve.shared.domain.MediaInput
import openllve.shared.media.FrameImage

/** Android boundary between [Bitmap] and the shared [FrameImage]. */
object BitmapFrameAdapter {

    fun toFrame(bitmap: Bitmap): FrameImage {
        val pixels = IntArray(bitmap.width * bitmap.height)
        bitmap.getPixels(pixels, 0, bitmap.width, 0, 0, bitmap.width, bitmap.height)
        return FrameImage(bitmap.width, bitmap.height, pixels)
    }

    fun toBitmap(frame: FrameImage): Bitmap {
        val bitmap = Bitmap.createBitmap(frame.width, frame.height, Bitmap.Config.ARGB_8888)
        bitmap.setPixels(frame.pixels, 0, frame.width, 0, 0, frame.width, frame.height)
        return bitmap
    }
}

/** Converts a shared [MediaInput] source string to an Android [Uri]. */
fun MediaInput.sourceUri(): Uri = Uri.parse(source)

/** Converts an Android [Uri] to the shared [MediaInput] source string. */
fun Uri.mediaSource(): String = toString()

/** Converts a shared [FrameImage] to an Android [Bitmap] (for rendering). */
fun FrameImage.toBitmap(): Bitmap = BitmapFrameAdapter.toBitmap(this)
