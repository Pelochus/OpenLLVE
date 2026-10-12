//! Frame-buffer validation at the JVM boundary.
//!
//! `core/src/ffi.rs` derives each buffer's length from the dimensions it is handed and
//! trusts the caller's pointer, so a JVM array shorter than those dimensions would be
//! read past its allocation. Every check here runs before any pointer crosses the
//! boundary; the frame contract itself is the core's (`crate::frame`) and is called
//! through, never duplicated here — see docs/dev/architecture/FFI-WIRING.md.

use jni::sys::jint;
use openllve_core::ffi::OpenLlveError;
use openllve_core::frame::{buffer_elements, validate_dimensions};

/// The [`OpenLlveError`] code for these dimensions and buffer element counts,
/// [`Ok`] when every buffer is large enough. `buffers` holds the element count of
/// each JVM frame buffer the call touches.
///
/// Only host facts live here: JVM `Int` is signed, so a non-positive dimension is
/// a caller error the core never sees, and `buffers` is how a JVM array reports
/// its length. Dimensions, stride alignment, row width and the buffer requirement
/// are the core's checks.
pub fn frame_violation(width: jint, height: jint, channels: jint, stride: jint, buffers: &[usize]) -> jint {
    if [width, height, channels, stride].iter().any(|it| *it <= 0) {
        return OpenLlveError::InvalidParameter.as_i32();
    }
    let dimensions = validate_dimensions(
        width as u32,
        height as u32,
        channels as u32,
        stride as usize,
    );
    if dimensions.is_err() {
        return OpenLlveError::InvalidParameter.as_i32();
    }
    // JVM array lengths are `Int`, so a frame whose requirement overflows u64
    // cannot exist here; the core rejects those dimensions outright.
    match buffer_elements(height as u32, stride as usize) {
        None => OpenLlveError::InvalidParameter.as_i32(),
        Some(required) => {
            if buffers.iter().any(|it| (*it as u64) < required) {
                OpenLlveError::BufferDimensionMismatch.as_i32()
            } else {
                OpenLlveError::Ok.as_i32()
            }
        }
    }
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
    fn rejects_stride_that_is_not_a_whole_element() {
        // 46 bytes is not a whole number of f32: 46*2/4 would round down to 23
        // elements and let a 23-element buffer cross the boundary one byte short.
        assert_eq!(
            frame_violation(
                WIDTH,
                HEIGHT,
                CHANNELS,
                TIGHT_STRIDE - 2,
                &[TIGHT_ELEMENTS, TIGHT_ELEMENTS]
            ),
            OpenLlveError::InvalidParameter.as_i32(),
        )
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
    fn rejects_unusable_dimensions() {
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
// Fuzz harness (P3.8) for the JVM frame guards: adversarial JVM `Int` tuples
// must be classified by an independent oracle, and a guard that waves a frame
// through must never let a pointer cross past the JVM allocation.
#[cfg(test)]
mod ffi_fuzz {
    use super::*;
    use openllve_core::ffi::OpenLlveError;
    use openllve_core::frame::{Frame, FrameRef};
    use proptest::collection::vec;
    use proptest::prop_assume;
    use proptest::proptest;
    use proptest::sample::select;

    // JVM `Int` extremes: bridge.rs reinterprets these with `as u32` and `as usize`,
    // so the sign bit and the top bits are the attack surface.
    const JINT_BOUNDS: [jint; 14] = [
        i32::MIN, //
        -2,
        -1,
        0,
        1,
        2,
        3,
        4,
        5,
        12,
        1 << 29,
        1_073_741_824, // 2^30: jint max / 4, the stride/height crossover
        2_147_483_646,
        i32::MAX,
    ];

    // Buffer sizes relative to the exact requirement, so the accept/reject
    // boundary itself is always hit (off-by-one is the interesting case).
    const DELTAS: [i64; 7] = [-2, -1, 0, 1, 2, 4096, -4096];

    // Dimensions that form legal frames, so the buffer checks are reachable;
    // `LEGAL_STRIDE` is always a multiple of the element size.
    const LEGAL_WIDTH: [jint; 5] = [1, 2, 3, 4, 5];
    const LEGAL_HEIGHT: [jint; 5] = [1, 2, 4, 8, 64];
    const LEGAL_CHANNELS: [jint; 3] = [1, 3, 4];
    const LEGAL_STRIDE: [jint; 7] = [4, 8, 12, 16, 48, 256, 1024];

    // Frames the harness actually allocates, so the core can be consulted without
    // inventing memory.
    const MAX_ELEMENTS: usize = 4096;

    /// The element count a JVM buffer must hold for these dimensions, computed in
    /// u64 so no product can wrap. [`None`] means the dimensions are not a legal
    /// frame at all.
    fn exact_requirement(width: jint, height: jint, channels: jint, stride: jint) -> Option<u64> {
        if [width, height, channels, stride].iter().any(|it| *it <= 0) || stride % 4 != 0 {
            return None;
        }
        let row_bytes = (channels as u64) * 4 * (width as u64);
        if (stride as u64) < row_bytes {
            return None;
        }
        Some((stride as u64) * (height as u64) / 4)
    }

    /// The code the guard must return, derived from the frame contract rather than
    /// from the guard's own arithmetic.
    fn oracle(width: jint, height: jint, channels: jint, stride: jint, buffers: &[usize]) -> jint {
        match exact_requirement(width, height, channels, stride) {
            None => OpenLlveError::InvalidParameter.as_i32(),
            Some(required) => {
                if buffers.iter().any(|it| (*it as u64) < required) {
                    OpenLlveError::BufferDimensionMismatch.as_i32()
                } else {
                    OpenLlveError::Ok.as_i32()
                }
            }
        }
    }

    /// Buffer element counts for one fuzz case: the exact requirement nudged by
    /// each delta, so the accept/reject boundary is always exercised.
    fn fuzz_buffers(deltas: Vec<i64>, width: jint, height: jint, channels: jint, stride: jint) -> Vec<usize> {
        let required = exact_requirement(width, height, channels, stride).unwrap_or(0);
        deltas
            .iter()
            .map(|delta| required.saturating_add_signed(*delta).min(usize::MAX as u64) as usize)
            .collect()
    }

    proptest! {
        #[test]
        fn fuzz_guard_classifies_boundary_tuples_exactly(
            width in select(JINT_BOUNDS.to_vec()),
            height in select(JINT_BOUNDS.to_vec()),
            channels in select(JINT_BOUNDS.to_vec()),
            stride in select(JINT_BOUNDS.to_vec()),
            deltas in vec(select(DELTAS.to_vec()), 1usize..3usize),
        ) {
            let buffers = fuzz_buffers(deltas, width, height, channels, stride);
            let code = frame_violation(width, height, channels, stride, buffers.as_slice());
            assert_eq!(code, oracle(width, height, channels, stride, buffers.as_slice()));
            // A caller error is reported as a caller error, never as a crash code.
            assert!(
                code == OpenLlveError::Ok.as_i32()
                    || code == OpenLlveError::InvalidParameter.as_i32()
                    || code == OpenLlveError::BufferDimensionMismatch.as_i32()
            );
        }

        #[test]
        fn fuzz_guard_ok_never_lets_a_pointer_cross(
            width in select(JINT_BOUNDS.to_vec()),
            height in select(JINT_BOUNDS.to_vec()),
            channels in select(JINT_BOUNDS.to_vec()),
            stride in select(JINT_BOUNDS.to_vec()),
            deltas in vec(select(DELTAS.to_vec()), 1usize..3usize),
        ) {
            let buffers = fuzz_buffers(deltas, width, height, channels, stride);
            let code = frame_violation(width, height, channels, stride, buffers.as_slice());
            if code == OpenLlveError::Ok.as_i32() {
                // The invariant that makes the crossing sound: every JVM buffer covers
                // every byte the core will read for these dimensions.
                for elements in buffers {
                    assert!((elements as u64) * 4 >= (stride as u64) * (height as u64));
                }
            }
        }

        #[test]
        fn fuzz_buffer_verdict_is_monotone(
            width in select(LEGAL_WIDTH.to_vec()),
            height in select(LEGAL_HEIGHT.to_vec()),
            channels in select(LEGAL_CHANNELS.to_vec()),
            stride in select(LEGAL_STRIDE.to_vec()),
            deltas in vec(select(DELTAS.to_vec()), 1usize..3usize),
            nudge in 1usize..64usize,
        ) {
            let buffers = fuzz_buffers(deltas, width, height, channels, stride);
            let code = frame_violation(width, height, channels, stride, buffers.as_slice());
            // Growing a buffer the guard already accepted stays accepted ...
            let grown: Vec<usize> = buffers.iter().map(|it| it.saturating_add(nudge)).collect();
            if code == OpenLlveError::Ok.as_i32() {
                assert_eq!(frame_violation(width, height, channels, stride, grown.as_slice()),
                    OpenLlveError::Ok.as_i32());
            }
            // ... and shrinking a buffer the guard rejected stays rejected.
            let shorter: Vec<usize> = buffers.iter().map(|it| it.saturating_sub(nudge)).collect();
            if code == OpenLlveError::BufferDimensionMismatch.as_i32() {
                assert_eq!(frame_violation(width, height, channels, stride, shorter.as_slice()),
                    OpenLlveError::BufferDimensionMismatch.as_i32());
            }
        }

        #[test]
        fn fuzz_guard_agrees_with_the_core_on_aligned_strides(
            width in select(LEGAL_WIDTH.to_vec()),
            height in select(LEGAL_HEIGHT.to_vec()),
            channels in select(LEGAL_CHANNELS.to_vec()),
            stride in select(LEGAL_STRIDE.to_vec()),
            deltas in vec(select(DELTAS.to_vec()), 1usize..2usize),
        ) {
            assert!(stride % 4 == 0);
            let buffers = fuzz_buffers(deltas, width, height, channels, stride);
            prop_assume!(buffers.iter().all(|it| *it <= MAX_ELEMENTS));
            let code = frame_violation(width, height, channels, stride, buffers.as_slice());

            // Same tuple through the core's own validation: on aligned strides the
            // guard and the core must agree, in both directions.
            let w = width as u32;
            let h = height as u32;
            let c = channels as u32;
            let s = stride as usize;
            let data = vec![0.0f32; buffers[0]];
            let mut out = vec![0.0f32; buffers[0]];
            let core_ok = FrameRef::new(w, h, c, s, &data).is_ok() && Frame::new(w, h, c, s, &mut out).is_ok();
            assert_eq!(code == OpenLlveError::Ok.as_i32(), core_ok);
        }
    }
}
