// Fuzz harness (P3.8) for the C ABI in `core/src/ffi.rs`: adversarial handles,
// dimensions, strides and filter weights must be classified by the documented
// `OpenLlveError` codes, must never write outside the buffers the caller owns,
// and must never free a handle twice.
//
// Fuzz envelope: `stride * height` stays inside `usize` for the frame properties
// below, so the ABI's length math is exact and every call there is defined
// behaviour. Pairs whose product leaves the address space are asserted rejected
// instead (P1.4): the ABI used to wrap them into a short slice, which then
// validated as a legal frame over memory the caller does not own.

use openllve_core::ffi::{
    OpenLlveError, openllve_blend_filter_apply, openllve_blend_filter_free, openllve_blend_filter_new,
};
use openllve_core::ffi::{openllve_ewma_filter_apply, openllve_ewma_filter_free, openllve_ewma_filter_new};
use openllve_core::ffi::{openllve_pipeline_free, openllve_pipeline_new_temporal, openllve_process_frame};
use proptest::collection::vec;
use proptest::prop_assume;
use proptest::proptest;
use proptest::sample::select;

const MAX_ELEMENTS: usize = 512;
const TOLERANCE: f32 = 1e-5;

/// The interval the filter constructors accept; NaN fails both comparisons,
/// so it is rejected like the infinities are.
fn valid_weight(value: f32) -> bool {
    value >= 0.0 && value <= 1.0
}

/// The element count the ABI derives for one buffer from these claims. The
/// harness allocates exactly this many elements, so every byte the ABI touches
/// belongs to the caller.
fn claimed_length(stride: usize, height: u32) -> usize {
    stride * height as usize / 4
}

/// Trim/pad fuzz payload to the length the ABI will claim, so the buffers handed
/// over are always exactly sized.
fn exact_size(data: Vec<f32>, elements: usize) -> Vec<f32> {
    let mut sized = data;
    sized.truncate(elements);
    sized.resize(elements, 0.5);
    sized
}

/// Element indices the ABI may read/write for these dimensions: rows start
/// `stride / 4` elements apart and only the first `width * channels` of each row
/// are pixels, the rest is stride padding the filters must leave untouched.
fn pixel_indices(width: u32, height: u32, channels: u32, stride: usize) -> Vec<usize> {
    let row = stride / 4;
    let per_row = (width as usize) * (channels as usize);
    (0..(height as usize) * per_row)
        .map(|index| index / per_row * row + index % per_row)
        .collect()
}

// Legal frames: `stride * height / 4` stays inside `MAX_ELEMENTS` for every pair.
const WIDTH: [u32; 5] = [1, 2, 3, 4, 8];
const HEIGHT: [u32; 4] = [1, 2, 4, 8];
const CHANNELS: [u32; 3] = [1, 3, 4];
const STRIDE: [usize; 6] = [4, 8, 12, 16, 48, 256];

// Hostile claims: zero dimensions, an unaligned stride, a stride below the row
// width, and a row width beyond any JVM `Int` (the `as u32` reinterpretation of
// a negative `Int`).
const BAD_WIDTH: [u32; 5] = [0, 1, 5, u32::MAX, 0xFFFF_FFFC];
const BAD_HEIGHT: [u32; 4] = [0, 1, 2, 8];
const BAD_CHANNELS: [u32; 5] = [0, 1, 3, 4, 1024];
const BAD_STRIDE: [usize; 5] = [0, 3, 4, 16, 1024];

// Stride/height pairs whose product leaves the address space, so no buffer can
// ever satisfy them; the stride itself is aligned and wide enough, so the only
// reason to reject these is that the frame cannot be sized.
const IMPOSSIBLE_STRIDE: [usize; 3] = [1usize << 62, (1usize << 62) + 4, usize::MAX - 3];
const IMPOSSIBLE_HEIGHT: u32 = 8;

// Weights outside the unit interval, including the non-ordered float values.
const WEIGHTS: [f32; 9] = [
    -1.0, //
    -0.5,
    0.0,
    0.5,
    1.0,
    1.5,
    0.0 / 0.0,  // NaN
    1.0 / 0.0,  // +infinity
    -1.0 / 0.0, // -infinity
];

// Weights the constructors accept, so the reference properties can run without
// rejecting most of their cases.
const VALID_WEIGHTS: [f32; 5] = [0.0, 0.25, 0.5, 0.7, 1.0];

proptest! {
    #[test]
    fn fuzz_filter_handles_follow_the_unit_interval(
        weight in select(WEIGHTS.to_vec()),
        extra in -2.0f32..2.0f32,
    ) {
        for value in [weight, extra] {
            let ewma = openllve_ewma_filter_new(value);
            assert_eq!(ewma.is_null(), !valid_weight(value));
            let blend = openllve_blend_filter_new(value);
            assert_eq!(blend.is_null(), !valid_weight(value));
            if !ewma.is_null() {
                unsafe {
            openllve_ewma_filter_free(ewma)
        };
            }
            if !blend.is_null() {
                unsafe {
            openllve_blend_filter_free(blend)
        };
            }
        }
    }

    #[test]
    fn fuzz_ewma_matches_its_reference_on_legal_frames(
        alpha in select(VALID_WEIGHTS.to_vec()),
        w in select(WIDTH.to_vec()),
        h in select(HEIGHT.to_vec()),
        c in select(CHANNELS.to_vec()),
        stride in select(STRIDE.to_vec()),
        payload in vec(-1.0f32..1.0f32, 0usize..MAX_ELEMENTS),
        later in vec(-1.0f32..1.0f32, 0usize..MAX_ELEMENTS),
    ) {
        prop_assume!(stride >= (w as usize) * (c as usize) * 4);
        let elements = claimed_length(stride, h);
        let current = exact_size(payload, elements);
        let next = exact_size(later, elements);

        let handle = openllve_ewma_filter_new(alpha);
        assert!(!handle.is_null());
        let mut smoothed = vec![0.0f32; elements];
        let code = unsafe {
            openllve_ewma_filter_apply( handle, w, h, c, stride, current.as_ptr(), smoothed.as_mut_ptr(), )
        };
        assert_eq!(code, OpenLlveError::Ok.as_i32());
        // First frame of a sequence passes through unchanged.
        for index in pixel_indices(w, h, c, stride) {
            assert_eq!(smoothed[index], current[index]);
        }

        let mut again = vec![0.0f32; elements];
        let code = unsafe {
            openllve_ewma_filter_apply(handle, w, h, c, stride, next.as_ptr(), again.as_mut_ptr())
        };
        assert_eq!(code, OpenLlveError::Ok.as_i32());
        for index in pixel_indices(w, h, c, stride) {
            let blended = again[index];
            let expected = alpha * next[index] + (1.0 - alpha) * smoothed[index];
            assert!((blended - expected).abs() <= TOLERANCE);
            let low = next[index].min(smoothed[index]);
            let high = next[index].max(smoothed[index]);
            assert!(blended >= low - TOLERANCE && blended <= high + TOLERANCE);
        }
        unsafe {
            openllve_ewma_filter_free(handle)
        };
    }

    #[test]
    fn fuzz_blend_matches_its_reference_on_legal_frames(
        beta in select(VALID_WEIGHTS.to_vec()),
        w in select(WIDTH.to_vec()),
        h in select(HEIGHT.to_vec()),
        c in select(CHANNELS.to_vec()),
        stride in select(STRIDE.to_vec()),
        payload in vec(0.0f32..1.0f32, 0usize..MAX_ELEMENTS),
    ) {
        prop_assume!(stride >= (w as usize) * (c as usize) * 4);
        let elements = claimed_length(stride, h);
        let raw = exact_size(payload, elements);
        let enhanced: Vec<f32> = raw.iter().map(|it| 1.0 - it).collect();

        let handle = openllve_blend_filter_new(beta);
        assert!(!handle.is_null());
        let mut out = vec![0.0f32; elements];
        let code = unsafe {
            openllve_blend_filter_apply( handle, w, h, c, stride, raw.as_ptr(), enhanced.as_ptr(), out.as_mut_ptr(), )
        };
        assert_eq!(code, OpenLlveError::Ok.as_i32());
        for index in pixel_indices(w, h, c, stride) {
            let blended = out[index];
            let expected = beta * enhanced[index] + (1.0 - beta) * raw[index];
            assert!((blended - expected).abs() <= TOLERANCE);
            let low = raw[index].min(enhanced[index]);
            let high = raw[index].max(enhanced[index]);
            assert!(blended >= low - TOLERANCE && blended <= high + TOLERANCE);
        }
        unsafe {
            openllve_blend_filter_free(handle)
        };
    }

    #[test]
    fn fuzz_illegal_claims_are_rejected_without_touching_the_output(
        w in select(BAD_WIDTH.to_vec()),
        h in select(BAD_HEIGHT.to_vec()),
        c in select(BAD_CHANNELS.to_vec()),
        stride in select(BAD_STRIDE.to_vec()),
        payload in vec(0.0f32..1.0f32, 0usize..MAX_ELEMENTS),
    ) {
        let elements = claimed_length(stride, h);
        prop_assume!(elements <= MAX_ELEMENTS);
        prop_assume!(w == 0 || c == 0 || h == 0 || stride % 4 != 0 || stride < (w as usize) * (c as usize) * 4);

        let handle = openllve_blend_filter_new(0.5);
        assert!(!handle.is_null());
        let data = exact_size(payload, elements);
        let mut out = vec![0.0f32; elements];
        let code = unsafe {
            openllve_blend_filter_apply( handle, w, h, c, stride, data.as_ptr(), data.as_ptr(), out.as_mut_ptr(), )
        };
        assert_eq!(code, OpenLlveError::InvalidParameter.as_i32());
        // A rejected frame is never half-written.
        assert!(out.iter().all(|it| *it == 0.0));
        unsafe {
            openllve_blend_filter_free(handle)
        };
    }

    #[test]
    fn fuzz_frames_larger_than_the_address_space_are_rejected(
        w in select(WIDTH.to_vec()),
        c in select(CHANNELS.to_vec()),
        stride in select(IMPOSSIBLE_STRIDE.to_vec()),
    ) {
        let data = vec![0.5f32; 16];
        let mut out = vec![0.0f32; 16];
        let handle = openllve_ewma_filter_new(0.5);
        assert!(!handle.is_null());
        let code = unsafe {
            openllve_ewma_filter_apply( handle, w, IMPOSSIBLE_HEIGHT, c, stride, data.as_ptr(), out.as_mut_ptr(), )
        };
        assert_eq!(code, OpenLlveError::InvalidParameter.as_i32());
        // The frame was never sized, so nothing was read or written.
        assert!(out.iter().all(|it| *it == 0.0));
        unsafe {
            openllve_ewma_filter_free(handle)
        };
    }

    #[test]
    fn fuzz_null_pointers_are_reported_not_dereferenced(
        w in select(WIDTH.to_vec()),
        h in select(HEIGHT.to_vec()),
        c in select(CHANNELS.to_vec()),
        stride in select(STRIDE.to_vec()),
        null_at in 0usize..3usize,
    ) {
        let elements = claimed_length(stride, h);
        let handle = openllve_ewma_filter_new(0.8);
        let data = vec![0.5f32; elements];
        let mut out = vec![0.0f32; elements];
        let filter = if null_at == 0 { std::ptr::null_mut() } else { handle };
        let input = if null_at == 1 { std::ptr::null() } else { data.as_ptr() };
        let output = if null_at == 2 { std::ptr::null_mut() } else { out.as_mut_ptr() };
        assert_eq!(
            unsafe {
            openllve_ewma_filter_apply(filter, w, h, c, stride, input, output)
        },
            OpenLlveError::NullPointer.as_i32(),
        );
        // The live handle is still freed exactly once, never through the null.
        unsafe {
            openllve_ewma_filter_free(handle)
        };
    }

    #[test]
    fn fuzz_process_frame_classifies_legal_and_illegal_frames(
        w in select(WIDTH.to_vec()),
        h in select(HEIGHT.to_vec()),
        c in select(CHANNELS.to_vec()),
        stride in select(STRIDE.to_vec()),
        ostride in select(BAD_STRIDE.to_vec()),
        payload in vec(0.0f32..1.0f32, 0usize..MAX_ELEMENTS),
    ) {
        let in_elements = claimed_length(stride, h);
        let out_elements = claimed_length(ostride, h);
        prop_assume!(out_elements <= MAX_ELEMENTS);

        let pipeline = openllve_pipeline_new_temporal();
        assert!(!pipeline.is_null());
        let input = exact_size(payload, in_elements);
        let mut output = vec![0.0f32; out_elements];
        let code = unsafe {
            openllve_process_frame( pipeline, w, h, c, stride, input.as_ptr(), w, h, c, ostride, output.as_mut_ptr(), )
        };
        prop_assume!(stride >= (w as usize) * (c as usize) * 4);
        let legal_output = ostride % 4 == 0 && ostride >= (w as usize) * (c as usize) * 4;
        if legal_output {
            assert_eq!(code, OpenLlveError::Ok.as_i32());
            let in_rows = pixel_indices(w, h, c, stride);
            let out_rows = pixel_indices(w, h, c, ostride);
            for pair in 0..out_rows.len() {
                assert_eq!(output[out_rows[pair]], input[in_rows[pair]]);
            }
        } else {
            assert!(code != OpenLlveError::Ok.as_i32());
            assert!(output.iter().all(|it| *it == 0.0));
        }
        unsafe {
            openllve_pipeline_free(pipeline)
        };
    }
}

#[test]
fn null_handles_are_never_freed() {
    unsafe { openllve_ewma_filter_free(std::ptr::null_mut()) };
    unsafe { openllve_blend_filter_free(std::ptr::null_mut()) };
    unsafe { openllve_pipeline_free(std::ptr::null_mut()) };
}
