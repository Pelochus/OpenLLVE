package openllve.shared.media

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
            return "$minutes:${seconds.toString().padStart(2, '0')}"
        }
}
