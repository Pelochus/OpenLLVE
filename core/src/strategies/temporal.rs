use super::InferenceStrategy;
use crate::error::{CoreError, Result};
use crate::filters::FrameBlendFilter;

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
    fn process(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        if input.is_empty() {
            return Err(CoreError::InvalidParameter(
                "Input buffer is empty".to_string(),
            ));
        }

        let mut processed = input.to_vec();

        // Blend the *current* raw frame with the enhanced frame:
        // output = beta * enhanced + (1 - beta) * raw
        if let Some(blend) = &self.blend {
            processed = blend.apply(input, &processed)?;
        }

        Ok(processed)
    }

    fn reset(&mut self) {
        // No temporal state yet; will reset model state when a real temporal
        // model is integrated.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_strategy() {
        let mut strategy = LlveTemporalStrategy::new();
        let input = vec![1.0, 2.0, 3.0];
        let res = strategy.process(&input).unwrap();
        assert_eq!(res, input);
        strategy.reset();
    }

    #[test]
    fn test_temporal_strategy_with_blend_uses_current_frame() {
        // Without a model the enhanced frame equals the raw frame, so a blend
        // with the current frame leaves the output equal to the input.
        let mut strategy = LlveTemporalStrategy::new().with_blend(0.5).unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let res = strategy.process(&input).unwrap();
        assert_eq!(res, input);
        let input2 = vec![4.0, 5.0, 6.0];
        let res2 = strategy.process(&input2).unwrap();
        assert_eq!(res2, input2);
    }
}
