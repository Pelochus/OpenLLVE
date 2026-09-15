use super::InferenceStrategy;
use crate::error::{CoreError, Result};
use crate::filters::{EwmaFilter, FrameBlendFilter};

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
    fn process(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        if input.is_empty() {
            return Err(CoreError::InvalidParameter(
                "Input buffer is empty".to_string(),
            ));
        }

        let mut processed = input.to_vec();

        if let Some(ewma) = &mut self.ewma {
            processed = ewma.apply(&processed);
        }

        // Blend the *current* raw frame with the enhanced frame:
        // output = beta * enhanced + (1 - beta) * raw
        if let Some(blend) = &self.blend {
            processed = blend.apply(input, &processed)?;
        }

        Ok(processed)
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

    #[test]
    fn test_llie_strategy() {
        let mut strategy = LlieStrategy::new().unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let res = strategy.process(&input).unwrap();
        assert_eq!(res, input);
    }

    #[test]
    fn test_llie_strategy_with_ewma() {
        let mut strategy = LlieStrategy::new().unwrap().with_ewma(0.5).unwrap();
        let input = vec![0.0, 10.0];
        let out1 = strategy.process(&input).unwrap();
        assert_eq!(out1, input);
        let input2 = vec![10.0, 0.0];
        let out2 = strategy.process(&input2).unwrap();
        assert_eq!(out2, vec![5.0, 5.0]);
    }

    #[test]
    fn test_llie_strategy_with_blend_uses_current_frame() {
        // Without a model the enhanced frame equals the raw frame, so a blend
        // with the current frame leaves the output equal to the input.
        let mut strategy = LlieStrategy::new().unwrap().with_blend(0.5).unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let res = strategy.process(&input).unwrap();
        assert_eq!(res, input);
        // A second frame must not leak state from the first frame.
        let input2 = vec![4.0, 5.0, 6.0];
        let res2 = strategy.process(&input2).unwrap();
        assert_eq!(res2, input2);
    }
}
