// Property tests (P3.8) for the temporal toppings: invariants that must hold
// for EVERY input, not just the unit-test samples. Filter contracts live in
// core/src/filters/{ewma,blend}.rs.

use openllve_core::filters::{EwmaFilter, FrameBlendFilter};
use openllve_core::frame::{Frame, FrameRef};
use proptest::collection::vec;
use proptest::proptest;

const MAX_ELEMS: usize = 8 * 4 * 3;

fn make_ref(data: &[f32], w: u32, h: u32, c: u32) -> FrameRef<'_> {
    FrameRef::new(w, h, c, (w * c) as usize * 4, data).unwrap()
}

fn make_frame(data: &mut [f32], w: u32, h: u32, c: u32) -> Frame<'_> {
    Frame::new(w, h, c, (w * c) as usize * 4, data).unwrap()
}

// Compact (no stride padding) element index, matching make_ref/make_frame.
fn elem(x: u32, y: u32, ch: u32, w: u32, c: u32) -> usize {
    ((y * w + x) as usize) * (c as usize) + ch as usize
}

fn pix_min(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

fn pix_max(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

proptest! {
    #[test]
    fn ewma_output_stays_within_pixel_extremes(
        alpha in 0.0f32..1.0f32,
        w in 1..8,
        h in 1..4,
        c in 1..3,
        first in vec(0.0f32..1.0f32, MAX_ELEMS),
        second in vec(0.0f32..1.0f32, MAX_ELEMS),
    ) {
        let w = w as u32;
        let h = h as u32;
        let c = c as u32;
        let mut filter = EwmaFilter::new(alpha).unwrap();
        let mut out = vec![0.0f32; MAX_ELEMS];

        filter.apply(&make_ref(&first, w, h, c), &mut make_frame(out.as_mut_slice(), w, h, c)).unwrap();
        filter.apply(&make_ref(&second, w, h, c), &mut make_frame(out.as_mut_slice(), w, h, c)).unwrap();

        for y in 0..h {
            for x in 0..w {
                for ch in 0..c {
                    let i = elem(x, y, ch, w, c);
                    let lo = pix_min(first[i], second[i]);
                    let hi = pix_max(first[i], second[i]);
                    // A convex combination of two floats cannot leave their
                    // [lo, hi] span beyond f32 rounding.
                    assert!(out[i] >= lo - 1e-6 && out[i] <= hi + 1e-6);
                }
            }
        }
    }

    #[test]
    fn blend_output_is_convex_combination(
        beta in 0.0f32..1.0f32,
        w in 1..8,
        h in 1..4,
        c in 1..3,
        raw in vec(0.0f32..1.0f32, MAX_ELEMS),
        enhanced in vec(0.0f32..1.0f32, MAX_ELEMS),
    ) {
        let w = w as u32;
        let h = h as u32;
        let c = c as u32;
        let blender = FrameBlendFilter::new(beta).unwrap();
        let mut out = vec![0.0f32; MAX_ELEMS];

        blender.apply(
            &make_ref(&raw, w, h, c),
            &make_ref(&enhanced, w, h, c),
            &mut make_frame(out.as_mut_slice(), w, h, c),
        ).unwrap();

        for y in 0..h {
            for x in 0..w {
                for ch in 0..c {
                    let i = elem(x, y, ch, w, c);
                    let lo = pix_min(raw[i], enhanced[i]);
                    let hi = pix_max(raw[i], enhanced[i]);
                    assert!(out[i] >= lo - 1e-6 && out[i] <= hi + 1e-6);
                    // Convex combinations of [0,1] inputs stay in [0,1].
                    assert!(out[i] >= -1e-6 && out[i] <= 1.0 + 1e-6);
                }
            }
        }
    }

    #[test]
    fn blend_endpoints_are_pure_inputs(
        w in 1..8,
        h in 1..4,
        c in 1..3,
        raw in vec(0.0f32..1.0f32, MAX_ELEMS),
        enhanced in vec(0.0f32..1.0f32, MAX_ELEMS),
    ) {
        let w = w as u32;
        let h = h as u32;
        let c = c as u32;
        for beta in [0.0f32, 1.0f32] {
            let blender = FrameBlendFilter::new(beta).unwrap();
            let mut out = vec![0.0f32; MAX_ELEMS];
            blender.apply(
                &make_ref(&raw, w, h, c),
                &make_ref(&enhanced, w, h, c),
                &mut make_frame(out.as_mut_slice(), w, h, c),
            ).unwrap();
            let source: &[f32] = if beta == 0.0f32 { &raw } else { &enhanced };
            for i in 0..elem(w - 1, h - 1, c - 1, w, c) + 1 {
                assert!((out[i] - source[i]).abs() <= 1e-6);
            }
        }
    }
}

/// Deterministic pseudo-random frame values in [0, 1) (LCG), so the golden
/// hash never drifts with the RNG.
fn golden_frame(seed: u32, count: usize) -> Vec<f32> {
    let mut state = seed;
    let mut values = vec![];
    for _ in 0..count {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_232);
        values.push(((state >> 8) as f32) / 0xFFFFFF as f32);
    }
    values
}

/// FNV-1a over the raw bit patterns of a frame buffer.
fn frame_hash(data: &[f32]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for value in data {
        let bits = value.to_bits();
        hash = (hash ^ (bits as u64)).wrapping_mul(0x100000001b3);
    }
    hash
}

#[test]
fn golden_blend_then_ewma_chain() {
    // Pins the app defaults (CoreToppings EWMA_ALPHA = 0.8f, BLEND_BETA = 0.9f):
    // blend(raw, enhanced) -> EWMA over three frames -> stable hash. If a filter
    // formula or these defaults change, this hash changes and the test fails.
    let w: u32 = 4;
    let h: u32 = 3;
    let c: u32 = 3;
    let count = (w * h * c) as usize;
    let blend = FrameBlendFilter::new(0.9).unwrap();
    let mut ewma = EwmaFilter::new(0.8).unwrap();

    let mut final_data = vec![0.0f32; count];
    for frame_no in 1..=3 {
        let raw = golden_frame(0x5EED + frame_no as u32, count);
        let enhanced = golden_frame(0xBEEF + frame_no as u32, count);
        let mut blended = vec![0.0f32; count];
        blend
            .apply(
                &make_ref(&raw, w, h, c),
                &make_ref(&enhanced, w, h, c),
                &mut make_frame(blended.as_mut_slice(), w, h, c),
            )
            .unwrap();
        ewma.apply(
            &make_ref(&blended, w, h, c),
            &mut make_frame(final_data.as_mut_slice(), w, h, c),
        )
        .unwrap();
    }

    assert_eq!(frame_hash(&final_data), 5_135_961_771_813_526_578);
}
