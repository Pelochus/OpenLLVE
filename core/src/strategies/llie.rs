use super::InferenceStrategy;
use crate::error::Result;
use crate::filters::{EwmaFilter, FrameBlendFilter};
use crate::frame::{Frame, FrameRef};

/// Strategy A: LLIE (static frame enhancer) with optional temporal toppings.
/// This is the place where EWMA smoothing can be attached, but not required.
#[derive(Default)]
pub struct LlieStrategy {
    ewma: Option<EwmaFilter>,
    blend: Option<FrameBlendFilter>,
}

impl LlieStrategy {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ewma: None,
            blend: None,
        })
    }

    pub fn with_ewma(mut self, alpha: f32) -> Result<Self> {
        self.ewma = Some(EwmaFilter::new(alpha)?);
        Ok(self)
    }

    pub fn with_blend(mut self, beta: f32) -> Result<Self> {
        self.blend = Some(FrameBlendFilter::new(beta)?);
        Ok(self)
    }
}

impl InferenceStrategy for LlieStrategy {
    fn process(&mut self, input: &FrameRef, output: &mut Frame) -> Result<()> {
        // Placeholder "model": enhanced == raw. P1.2 replaces this with the
        // ModelRunner from ADR-0001 (zero-dce-int8.tflite).
        if let Some(ewma) = &mut self.ewma {
            ewma.apply(input, output)?;
        } else {
            output.copy_from(input)?;
        }

        // Blend the *current* raw frame with the enhanced frame:
        // output = beta * enhanced + (1 - beta) * raw
        if let Some(blend) = &self.blend {
            blend.apply_in_place(input, output)?;
        }

        Ok(())
    }

    fn reset(&mut self) {
        if let Some(ewma) = &mut self.ewma {
            ewma.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;

    fn frame_ref(data: &[f32], w: u32, h: u32, c: u32) -> FrameRef<'_> {
        FrameRef::new(w, h, c, (w * c) as usize * 4, data).unwrap()
    }

    fn frame_mut(data: &mut [f32], w: u32, h: u32, c: u32) -> Frame<'_> {
        Frame::new(w, h, c, (w * c) as usize * 4, data).unwrap()
    }

    #[test]
    fn test_llie_strategy() {
        let mut strategy = LlieStrategy::new().unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        strategy
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_llie_strategy_with_ewma() {
        let mut strategy = LlieStrategy::new().unwrap().with_ewma(0.5).unwrap();
        let input = vec![0.0, 10.0];
        let mut out1 = vec![0.0; 2];
        strategy
            .process(
                &frame_ref(&input, 2, 1, 1),
                &mut frame_mut(&mut out1, 2, 1, 1),
            )
            .unwrap();
        assert_eq!(out1, input);

        let input2 = vec![10.0, 0.0];
        let mut out2 = vec![0.0; 2];
        strategy
            .process(
                &frame_ref(&input2, 2, 1, 1),
                &mut frame_mut(&mut out2, 2, 1, 1),
            )
            .unwrap();
        assert_eq!(out2, vec![5.0, 5.0]);
    }

    #[test]
    fn test_llie_strategy_with_blend_uses_current_frame() {
        // Without a model the enhanced frame equals the raw frame, so a blend
        // with the current frame leaves the output equal to the input.
        let mut strategy = LlieStrategy::new().unwrap().with_blend(0.5).unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        strategy
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output, input);
        // A second frame must not leak state from the first frame.
        let input2 = vec![4.0, 5.0, 6.0];
        let mut output2 = vec![0.0; 3];
        strategy
            .process(
                &frame_ref(&input2, 3, 1, 1),
                &mut frame_mut(&mut output2, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output2, input2);
    }

    #[test]
    fn test_llie_strategy_output_dim_mismatch() {
        let mut strategy = LlieStrategy::new().unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 6];
        let err = strategy
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 2, 1),
            )
            .unwrap_err();
        assert!(matches!(err, CoreError::BufferDimensionMismatch { .. }));
    }
}
