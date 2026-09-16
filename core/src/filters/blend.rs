use crate::error::{CoreError, Result};
use crate::frame::{Frame, FrameRef};

/// Original-to-Processed Frame Blending filter (residual/alpha mixing topping).
/// Blends the raw input frame with the AI-enhanced frame:
/// output = beta * enhanced + (1 - beta) * raw_input
#[derive(Debug, Clone)]
pub struct FrameBlendFilter {
    beta: f32,
}

impl FrameBlendFilter {
    pub fn new(beta: f32) -> Result<Self> {
        if !(0.0..=1.0).contains(&beta) {
            return Err(CoreError::InvalidParameter(format!(
                "beta blend weight must be in [0, 1], got {}",
                beta
            )));
        }
        Ok(Self { beta })
    }

    /// Blends two frames into a caller-owned `output` frame (no per-frame
    /// allocation).
    ///
    /// # Errors
    /// Returns [`CoreError::BufferDimensionMismatch`] if the three frames do
    /// not share the same `(width, height, channels)`.
    pub fn apply(&self, raw: &FrameRef, enhanced: &FrameRef, output: &mut Frame) -> Result<()> {
        if raw.dims() != enhanced.dims() {
            return Err(CoreError::BufferDimensionMismatch {
                expected: raw.pixel_count(),
                actual: enhanced.pixel_count(),
            });
        }
        if output.dims() != raw.dims() {
            return Err(CoreError::BufferDimensionMismatch {
                expected: raw.pixel_count(),
                actual: output.pixel_count(),
            });
        }

        let beta = self.beta;
        for y in 0..raw.height() {
            for x in 0..raw.width() {
                for c in 0..raw.channels() {
                    let enh = enhanced.pixel(x, y, c);
                    let raw_v = raw.pixel(x, y, c);
                    *output.pixel_mut(x, y, c) = beta * enh + (1.0 - beta) * raw_v;
                }
            }
        }
        Ok(())
    }

    /// In-place variant: `output` already holds the enhanced frame and is
    /// blended with `raw` element-wise:
    /// `out = beta * out + (1 - beta) * raw`.
    ///
    /// # Errors
    /// Returns [`CoreError::BufferDimensionMismatch`] if `output` has
    /// different dimensions than `raw`.
    pub fn apply_in_place(&self, raw: &FrameRef, output: &mut Frame) -> Result<()> {
        if output.dims() != raw.dims() {
            return Err(CoreError::BufferDimensionMismatch {
                expected: raw.pixel_count(),
                actual: output.pixel_count(),
            });
        }

        let beta = self.beta;
        for y in 0..raw.height() {
            for x in 0..raw.width() {
                for c in 0..raw.channels() {
                    let enh = output.pixel(x, y, c);
                    let raw_v = raw.pixel(x, y, c);
                    *output.pixel_mut(x, y, c) = beta * enh + (1.0 - beta) * raw_v;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_ref(data: &[f32], w: u32, h: u32, c: u32) -> FrameRef<'_> {
        FrameRef::new(w, h, c, (w * c) as usize * 4, data).unwrap()
    }

    fn frame_mut(data: &mut [f32], w: u32, h: u32, c: u32) -> Frame<'_> {
        Frame::new(w, h, c, (w * c) as usize * 4, data).unwrap()
    }

    #[test]
    fn test_frame_blend() {
        let blender = FrameBlendFilter::new(0.8).unwrap();
        let raw = vec![0.0, 10.0];
        let enh = vec![10.0, 0.0];
        let mut out = vec![0.0; 2];

        blender
            .apply(
                &frame_ref(&raw, 2, 1, 1),
                &frame_ref(&enh, 2, 1, 1),
                &mut frame_mut(&mut out, 2, 1, 1),
            )
            .unwrap();

        // 0.8 * 10.0 + 0.2 * 0.0 = 8.0
        // 0.8 * 0.0 + 0.2 * 10.0 = 2.0
        // f32 arithmetic is not exact, so compare with a tolerance.
        assert!((out[0] - 8.0).abs() < 1e-4);
        assert!((out[1] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_frame_blend_in_place() {
        let blender = FrameBlendFilter::new(0.8).unwrap();
        let raw = vec![0.0, 10.0];
        let mut out = vec![10.0, 0.0]; // holds the enhanced frame

        blender
            .apply_in_place(&frame_ref(&raw, 2, 1, 1), &mut frame_mut(&mut out, 2, 1, 1))
            .unwrap();

        assert!((out[0] - 8.0).abs() < 1e-4);
        assert!((out[1] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_frame_blend_rgb() {
        // 2x2 RGB: per-channel blending.
        let blender = FrameBlendFilter::new(0.5).unwrap();
        let raw = vec![0.0; 12];
        let enh = vec![1.0; 12];
        let mut out = vec![0.0; 12];

        blender
            .apply(
                &frame_ref(&raw, 2, 2, 3),
                &frame_ref(&enh, 2, 2, 3),
                &mut frame_mut(&mut out, 2, 2, 3),
            )
            .unwrap();
        assert!(out.iter().all(|v| (v - 0.5).abs() < 1e-4));
    }

    #[test]
    fn test_frame_blend_dim_mismatch() {
        let blender = FrameBlendFilter::new(0.5).unwrap();
        let raw = vec![0.0; 4];
        let enh = vec![1.0; 6];
        let mut out = vec![0.0; 4];

        let err = blender
            .apply(
                &frame_ref(&raw, 2, 2, 1),
                &frame_ref(&enh, 3, 2, 1),
                &mut frame_mut(&mut out, 2, 2, 1),
            )
            .unwrap_err();
        assert!(matches!(err, CoreError::BufferDimensionMismatch { .. }));
    }

    #[test]
    fn test_frame_blend_output_dim_mismatch() {
        let blender = FrameBlendFilter::new(0.5).unwrap();
        let raw = vec![0.0; 4];
        let enh = vec![1.0; 4];
        let mut out = vec![0.0; 6];

        let err = blender
            .apply(
                &frame_ref(&raw, 2, 2, 1),
                &frame_ref(&enh, 2, 2, 1),
                &mut frame_mut(&mut out, 3, 2, 1),
            )
            .unwrap_err();
        assert!(matches!(err, CoreError::BufferDimensionMismatch { .. }));
    }
}
