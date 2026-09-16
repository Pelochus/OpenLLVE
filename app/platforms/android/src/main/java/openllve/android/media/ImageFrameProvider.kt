package openllve.android.media

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri

/**
 * Loads a still image selected via the Storage Access Framework into a
 * [Bitmap]. Kept separate from the video path: an image is a single frame and
 * is processed once, with no decoder involved.
 */
class ImageFrameProvider(private val context: Context) {

    fun loadBitmap(uri: Uri): Bitmap {
        context.contentResolver.openInputStream(uri)?.use { input ->
            val bitmap = BitmapFactory.decodeStream(input)
                ?: throw IllegalArgumentException("Malformed or unsupported image: ${uri.lastPathSegment}")
            return bitmap
        } ?: throw IllegalArgumentException("Could not open image: ${uri.lastPathSegment}")
    }
}
