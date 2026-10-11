//! Frame-buffer validation at the JVM boundary.
//!
//! `core/src/ffi.rs` derives each buffer's length from the dimensions it is handed and
//! trusts the caller's pointer, so a JVM array shorter than those dimensions would be
//! read past its allocation. Every check here runs before any pointer crosses the
//! boundary; the core re-validates the same invariants on its side.

use jni::sys::jint;
use openllve_core::ffi::OpenLlveError;

/// JVM `Int`s are signed, so non-positive dimensions/stride are caller errors.
fn non_positive_dimensions(width: jint, height: jint, channels: jint, stride: jint) -> bool {
    width <= 0 || height <= 0 || channels <= 0 || stride <= 0
}

/// Elements a JVM buffer must hold to cover `stride` bytes per row over `height`
/// rows of `f32`. Assumes the dimensions are positive.
fn required_elements(height: jint, stride: jint) -> usize {
    (stride as usize).saturating_mul(height as usize) / 4
}

/// The [`OpenLlveError`] code for these dimensions and buffer element counts,
/// [`Ok`] when every buffer is large enough. `buffers` holds the element count of
/// each JVM frame buffer the call touches.
pub fn frame_violation(width: jint, height: jint, channels: jint, stride: jint, buffers: &[usize]) -> jint {
    if non_positive_dimensions(width, height, channels, stride) {
        return OpenLlveError::InvalidParameter.as_i32();
    }
    // Saturating math: JVM `Int` is unbounded relative to these products, so an
    // absurd dimension must classify as invalid rather than wrap or trap.
    let row_bytes: usize = (width as usize).saturating_mul(channels as usize).saturating_mul(4);
    if (stride as usize) < row_bytes {
        return OpenLlveError::InvalidParameter.as_i32();
    }
    let required: usize = required_elements(height, stride);
    if buffers.iter().any(|it| *it < required) {
        return OpenLlveError::BufferDimensionMismatch.as_i32();
    }
    OpenLlveError::Ok.as_i32()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: jint = 4;
    const HEIGHT: jint = 2;
    const CHANNELS: jint = 3;
    const TIGHT_STRIDE: jint = WIDTH * CHANNELS * 4; // 48 bytes = 12 elements per row
    const TIGHT_ELEMENTS: usize = TIGHT_STRIDE as usize / 4 * HEIGHT as usize; // 24

    #[test]
    fn accepts_tightly_packed_buffers() {
        assert_eq!(
            frame_violation(
                WIDTH,
                HEIGHT,
                CHANNELS,
                TIGHT_STRIDE,
                &[TIGHT_ELEMENTS, TIGHT_ELEMENTS]
            ),
            OpenLlveError::Ok.as_i32(),
        );
    }

    #[test]
    fn rejects_buffer_one_element_short() {
        assert_eq!(
            frame_violation(
                WIDTH,
                HEIGHT,
                CHANNELS,
                TIGHT_STRIDE,
                &[TIGHT_ELEMENTS, TIGHT_ELEMENTS - 1]
            ),
            OpenLlveError::BufferDimensionMismatch.as_i32(),
        );
    }

    #[test]
    fn rejects_stride_below_row_width() {
        assert_eq!(
            frame_violation(
                WIDTH,
                HEIGHT,
                CHANNELS,
                TIGHT_STRIDE - 4,
                &[TIGHT_ELEMENTS, TIGHT_ELEMENTS]
            ),
            OpenLlveError::InvalidParameter.as_i32(),
        );
    }

    #[test]
    fn accepts_padded_stride() {
        // A stride wider than the row leaves gaps the core skips, and the buffer
        // must be long enough to cover the padding.
        let padded: jint = TIGHT_STRIDE + 16;
        let padded_elements: usize = padded as usize / 4 * HEIGHT as usize;
        assert_eq!(
            frame_violation(
                WIDTH,
                HEIGHT,
                CHANNELS,
                padded,
                &[padded_elements, padded_elements]
            ),
            OpenLlveError::Ok.as_i32(),
        );
    }

    #[test]
    fn rejects_non_positive_dimensions() {
        assert_eq!(
            frame_violation(
                -WIDTH,
                HEIGHT,
                CHANNELS,
                TIGHT_STRIDE,
                &[TIGHT_ELEMENTS, TIGHT_ELEMENTS]
            ),
            OpenLlveError::InvalidParameter.as_i32(),
        );
    }
}
