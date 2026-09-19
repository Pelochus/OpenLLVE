package openllve.shared.media

/**
 * Platform-neutral decoded frame: row-major ARGB_8888 pixels
 * (each pixel is a `0xAARRGGBB` int), size `width * height`.
 *
 * Host platforms convert their native pixel type (Android `Bitmap`,
 * iOS `CVPixelBuffer`/`CGImage`) to/from [FrameImage] at the boundary, so
 * the UI state and the pixel math can be shared across platforms.
 */
data class FrameImage(
    val width: Int,
    val height: Int,
    val pixels: IntArray
)
