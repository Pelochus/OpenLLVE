package openllve.android.media

import android.content.Context
import android.media.MediaMetadataRetriever
import android.net.Uri
import openllve.shared.media.VideoMetadata

/** Fetches video metadata via [MediaMetadataRetriever]. */
object VideoMetadataReader {
    fun read(
        context: Context,
        uri: Uri,
    ): VideoMetadata {
        val retriever = MediaMetadataRetriever()
        try {
            retriever.setDataSource(context, uri)
            val durationMs =
                retriever
                    .extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION)
                    ?.toLongOrNull() ?: 0L
            val width =
                retriever
                    .extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_WIDTH)
                    ?.toIntOrNull() ?: 0
            val height =
                retriever
                    .extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_HEIGHT)
                    ?.toIntOrNull() ?: 0
            val mime = context.contentResolver.getType(uri) ?: "video/*"
            val size =
                context.contentResolver.openInputStream(uri)?.use { it.available().toLong() }
                    ?: 0L
            return VideoMetadata(durationMs, width, height, mime, size)
        } finally {
            retriever.release()
        }
    }
}
