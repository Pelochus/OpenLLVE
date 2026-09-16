package openllve.android.domain

import android.net.Uri

/**
 * The kind of media the user selected. The UI must always know whether the
 * current input is an [ImageInput] or a [VideoInput] and the two processing
 * paths stay separate (image = single frame; video = decoded frame stream).
 */
sealed interface MediaInput {
    val uri: Uri
    val displayName: String
}

/** A still image selected via the Storage Access Framework. */
data class ImageInput(
    override val uri: Uri,
    override val displayName: String
) : MediaInput

/** An MP4 (or other decodable video) selected via the Storage Access Framework. */
data class VideoInput(
    override val uri: Uri,
    override val displayName: String
) : MediaInput
