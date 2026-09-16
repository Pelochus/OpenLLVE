use super::InferenceStrategy;
use crate::error::Result;
use crate::filters::FrameBlendFilter;
use crate::frame::{Frame, FrameRef};

/// Strategy B: LLVE Temporal native (stateful sequence model).
/// It owns temporal state and may optionally accept a raw-vs-output blend
/// topping, but it deliberately does not use EWMA because the model is
/// already temporal.
///
/// Note: temporal model state is not implemented yet — it will arrive with a
/// real temporal model. Until then this strategy is stateless and
/// [`InferenceStrategy::reset`] is a no-op.
pub struct LlveTemporalStrategy {
    blend: Option<FrameBlendFilter>,
}

impl LlveTemporalStrategy {
    pub fn new() -> Self {
        Self { blend: None }
    }

    pub fn with_blend(mut self, beta: f32) -> Result<Self> {
        self.blend = Some(FrameBlendFilter::new(beta)?);
        Ok(self)
    }
}

impl Default for LlveTemporalStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceStrategy for LlveTemporalStrategy {
    fn process(&mut self, input: &FrameRef, output: &mut Frame) -> Result<()> {
        // Placeholder temporal model: enhanced == raw. P1.2 replaces this
        // with the real temporal model.
        output.copy_from(input)?;

        // Blend the *current* raw frame with the enhanced frame:
        // output = beta * enhanced + (1 - beta) * raw
        if let Some(blend) = &self.blend {
            blend.apply_in_place(input, output)?;
        }

        Ok(())
    }

    fn reset(&mut self) {
        // No temporal state yet; will reset model state when a real temporal
        // model is integrated.
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
    fn test_temporal_strategy() {
        let mut strategy = LlveTemporalStrategy::new();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        strategy
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output, input);
        strategy.reset();
    }

    #[test]
    fn test_temporal_strategy_with_blend_uses_current_frame() {
        // Without a model the enhanced frame equals the raw frame, so a blend
        // with the current frame leaves the output equal to the input.
        let mut strategy = LlveTemporalStrategy::new().with_blend(0.5).unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        strategy
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output, input);
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
    fn test_temporal_strategy_output_dim_mismatch() {
        let mut strategy = LlveTemporalStrategy::new();
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
