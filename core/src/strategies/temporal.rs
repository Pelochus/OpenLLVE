use crate::error::{CoreError, Result};
use crate::filters::FrameBlendFilter;
use super::InferenceStrategy;

/// Strategy B: LLVE Temporal native (stateful sequence model).
/// It owns temporal state and may optionally accept a raw-vs-output blend topping,
/// but it deliberately does not use EWMA because the model is already temporal.
pub struct LlveTemporalStrategy {
    state: Option<Vec<f32>>,
    blend: Option<FrameBlendFilter>,
}

impl LlveTemporalStrategy {
    pub fn new() -> Self {
        Self {
            state: None,
            blend: None,
        }
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
            return Err(CoreError::InvalidParameter("Input buffer is empty".to_string()));
        }

        let mut processed = input.to_vec();

        if let Some(blend) = &self.blend {
            let raw = self.state.clone().unwrap_or_else(|| input.to_vec());
            processed = blend.apply(&raw, &processed)?;
        }

        self.state = Some(input.to_vec());
        Ok(processed)
    }

    fn reset(&mut self) {
        self.state = None;
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
        assert!(strategy.state.is_some());
        strategy.reset();
        assert!(strategy.state.is_none());
    }
}
