package openllve.shared.media

/**
 * Platform-neutral decoded frame: row-major ARGB_8888 pixels
 * (`0xAARRGGBB` ints), size `width * height`. Hosts convert their native
 * pixel type to/from [FrameImage] at the boundary.
 */
data class FrameImage(
    val width: Int,
    val height: Int,
    val pixels: IntArray
)
