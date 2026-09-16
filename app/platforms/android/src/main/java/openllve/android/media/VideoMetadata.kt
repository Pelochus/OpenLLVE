package openllve.android.media

import android.content.Context
import android.media.MediaMetadataRetriever
import android.net.Uri

/** Basic, practical metadata for a selected video. */
data class VideoMetadata(
    val durationMs: Long,
    val width: Int,
    val height: Int,
    val mimeType: String,
    val fileSizeBytes: Long
) {
    val resolution: String
        get() = "${width}×${height}"

    val durationLabel: String
        get() {
        val totalSeconds = (durationMs + 500) / 1000
        val minutes = totalSeconds / 60
        val seconds = totalSeconds % 60
        return "%d:%02d".format(minutes, seconds)
    }
}

/**
 * Fetches lightweight metadata from a video via [MediaMetadataRetriever]
 * (an Android-native API). Used to display basic info and to compute the
 * realtime factor for the processing metrics.
 */
object VideoMetadataReader {
    fun read(context: Context, uri: Uri): VideoMetadata {
        val retriever = MediaMetadataRetriever()
        try {
            retriever.setDataSource(context, uri)
            val durationMs = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION)
                ?.toLongOrNull() ?: 0L
            val width = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_WIDTH)
                ?.toIntOrNull() ?: 0
            val height = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_HEIGHT)
                ?.toIntOrNull() ?: 0
            val mime = context.contentResolver.getType(uri) ?: "video/*"
            val size = context.contentResolver.openInputStream(uri)?.use { it.available().toLong() }
                ?: 0L
            return VideoMetadata(durationMs, width, height, mime, size)
        } finally {
            retriever.release()
        }
    }
}
