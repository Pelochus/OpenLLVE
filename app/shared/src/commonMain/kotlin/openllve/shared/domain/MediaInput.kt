package openllve.shared.domain

/**
 * The kind of media the user selected; the two processing paths stay separate
 * (image = single frame, video = decoded frame stream). [source] is a
 * platform-neutral URI string; each host converts it to its own URI type
 * (Android `Uri`, iOS `NSURL`).
 */
sealed interface MediaInput {
    val source: String
    val displayName: String
}

/** A still image selected via the platform's file picker. */
data class ImageInput(
    override val source: String,
    override val displayName: String
) : MediaInput

/** An MP4 (or other decodable video) selected via the platform's file picker. */
data class VideoInput(
    override val source: String,
    override val displayName: String
) : MediaInput
