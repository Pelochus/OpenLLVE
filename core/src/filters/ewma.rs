use crate::error::{CoreError, Result};
use crate::frame::{Frame, FrameRef, OwnedFrame};

/// Exponentially Weighted Moving Average filter for temporal anti-flickering.
///
/// `out = alpha * current + (1 - alpha) * previous`, where `previous` is the
/// last *filtered* frame. The first frame of a sequence is passed through
/// unchanged. A frame whose dimensions differ from the stored state starts
/// a new sequence (the previous state is dropped).
#[derive(Debug, Clone)]
pub struct EwmaFilter {
    alpha: f32,
    previous: Option<OwnedFrame>,
}

impl EwmaFilter {
    pub fn new(alpha: f32) -> Result<Self> {
        if !(0.0..=1.0).contains(&alpha) {
            return Err(CoreError::InvalidParameter(format!(
                "alpha must be in [0, 1], got {}",
                alpha
            )));
        }
        Ok(Self { alpha, previous: None })
    }

    /// Applies EWMA smoothing, writing into the caller-owned `output` frame
    /// (no per-frame allocation).
    ///
    /// # Errors
    /// Returns [`CoreError::BufferDimensionMismatch`] if `output` has
    /// different dimensions than `current`.
    pub fn apply(&mut self, current: &FrameRef, output: &mut Frame) -> Result<()> {
        output.copy_from(current)?;

        let is_new_sequence = match &self.previous {
            None => true,
            Some(prev) => prev.dims() != current.dims(),
        };

        let prev = if is_new_sequence { None } else { self.previous.as_ref() };

        if let Some(prev) = prev {
            let alpha = self.alpha;
            for y in 0..current.height() {
                for x in 0..current.width() {
                    for c in 0..current.channels() {
                        let cur = current.pixel(x, y, c);
                        let prev = prev.pixel(x, y, c);
                        *output.pixel_mut(x, y, c) = alpha * cur + (1.0 - alpha) * prev;
                    }
                }
            }
        }

        self.previous = Some(OwnedFrame::from_frame(&output.as_ref()));
        Ok(())
    }

    pub fn reset(&mut self) {
        self.previous = None;
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
    fn test_ewma_filter() {
        let mut filter = EwmaFilter::new(0.5).unwrap();
        let f1 = vec![0.0, 10.0];
        let mut out1 = vec![0.0; 2];
        filter
            .apply(&frame_ref(&f1, 2, 1, 1), &mut frame_mut(&mut out1, 2, 1, 1))
            .unwrap();
        assert_eq!(out1, vec![0.0, 10.0]);

        let f2 = vec![10.0, 0.0];
        let mut out2 = vec![0.0; 2];
        filter
            .apply(&frame_ref(&f2, 2, 1, 1), &mut frame_mut(&mut out2, 2, 1, 1))
            .unwrap();
        assert_eq!(out2, vec![5.0, 5.0]);
    }

    #[test]
    fn test_ewma_first_frame_passthrough_and_stride() {
        // Frame with stride padding: only valid pixels are processed.
        // 2x1x2 frame with stride 16 bytes (4 elems/row); last 2 elems are padding.
        let data = vec![1.0, 2.0, 99.0, 99.0];
        let mut out = vec![0.0f32; 4];
        let mut filter = EwmaFilter::new(0.5).unwrap();
        filter
            .apply(
                &FrameRef::new(2, 1, 2, 16, &data).unwrap(),
                &mut Frame::new(2, 1, 2, 16, &mut out).unwrap(),
            )
            .unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn test_ewma_dim_change_starts_new_sequence() {
        let mut filter = EwmaFilter::new(0.5).unwrap();
        let f1 = vec![0.0, 10.0];
        let mut out1 = vec![0.0; 2];
        filter
            .apply(&frame_ref(&f1, 2, 1, 1), &mut frame_mut(&mut out1, 2, 1, 1))
            .unwrap();

        // Different dimensions: state is dropped, frame passes through.
        let f2 = vec![1.0, 2.0, 3.0, 4.0];
        let mut out2 = vec![0.0; 4];
        filter
            .apply(&frame_ref(&f2, 2, 2, 1), &mut frame_mut(&mut out2, 2, 2, 1))
            .unwrap();
        assert_eq!(out2, f2);
    }

    #[test]
    fn test_ewma_output_dim_mismatch() {
        let mut filter = EwmaFilter::new(0.5).unwrap();
        let f1 = vec![0.0, 10.0];
        let mut out = vec![0.0; 4];
        let err = filter
            .apply(&frame_ref(&f1, 2, 1, 1), &mut frame_mut(&mut out, 2, 2, 1))
            .unwrap_err();
        assert!(matches!(err, CoreError::BufferDimensionMismatch { .. }));
    }

    #[test]
    fn test_ewma_reset() {
        let mut filter = EwmaFilter::new(0.5).unwrap();
        let f1 = vec![0.0, 10.0];
        let mut out1 = vec![0.0; 2];
        filter
            .apply(&frame_ref(&f1, 2, 1, 1), &mut frame_mut(&mut out1, 2, 1, 1))
            .unwrap();
        filter.reset();

        // After reset the next frame is a first frame again.
        let f2 = vec![10.0, 0.0];
        let mut out2 = vec![0.0; 2];
        filter
            .apply(&frame_ref(&f2, 2, 1, 1), &mut frame_mut(&mut out2, 2, 1, 1))
            .unwrap();
        assert_eq!(out2, f2);
    }
}
