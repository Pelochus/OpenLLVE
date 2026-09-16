use super::{Pipeline, TemporalMode};
use crate::error::Result;
use crate::filters::FrameBlendFilter;
use crate::frame::{Frame, FrameRef};

/// Pipeline B: LLVE Temporal native (stateful sequence model).
/// It owns temporal state and may optionally accept a raw-vs-output blend
/// topping, but it deliberately does not use EWMA because the model is
/// already temporal.
///
/// Note: temporal model state is not implemented yet — it will arrive with a
/// real temporal model. Until then this pipeline is stateless in practice and
/// [`Pipeline::reset`] is a no-op, but its declared [`TemporalMode`] is
/// `Recurrent` because it is designed to carry inter-frame state.
pub struct LlveTemporalPipeline {
    blend: Option<FrameBlendFilter>,
}

impl LlveTemporalPipeline {
    pub fn new() -> Self {
        Self { blend: None }
    }

    pub fn with_blend(mut self, beta: f32) -> Result<Self> {
        self.blend = Some(FrameBlendFilter::new(beta)?);
        Ok(self)
    }
}

impl Default for LlveTemporalPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline for LlveTemporalPipeline {
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

    fn mode(&self) -> TemporalMode {
        TemporalMode::Recurrent
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
    fn test_temporal_pipeline() {
        let mut pipeline = LlveTemporalPipeline::new();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        pipeline
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output, input);
        assert_eq!(pipeline.mode(), TemporalMode::Recurrent);
        pipeline.reset();
    }

    #[test]
    fn test_temporal_pipeline_with_blend_uses_current_frame() {
        // Without a model the enhanced frame equals the raw frame, so a blend
        // with the current frame leaves the output equal to the input.
        let mut pipeline = LlveTemporalPipeline::new().with_blend(0.5).unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        pipeline
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output, input);
        let input2 = vec![4.0, 5.0, 6.0];
        let mut output2 = vec![0.0; 3];
        pipeline
            .process(
                &frame_ref(&input2, 3, 1, 1),
                &mut frame_mut(&mut output2, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output2, input2);
    }

    #[test]
    fn test_temporal_pipeline_output_dim_mismatch() {
        let mut pipeline = LlveTemporalPipeline::new();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 6];
        let err = pipeline
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 2, 1),
            )
            .unwrap_err();
        assert!(matches!(err, CoreError::BufferDimensionMismatch { .. }));
    }
}
