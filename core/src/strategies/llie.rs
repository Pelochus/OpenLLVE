use super::InferenceStrategy;
use crate::error::{CoreError, Result};
use crate::filters::{EwmaFilter, FrameBlendFilter};

/// Strategy A: LLIE (static frame enhancer) with optional temporal toppings.
/// This is the place where EWMA smoothing can be attached, but not required.
pub struct LlieStrategy {
    ewma: Option<EwmaFilter>,
    blend: Option<FrameBlendFilter>,
    previous_frame: Option<Vec<f32>>,
}

impl LlieStrategy {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ewma: None,
            blend: None,
            previous_frame: None,
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

impl Default for LlieStrategy {
    fn default() -> Self {
        Self::new().unwrap()
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

        if let Some(blend) = &self.blend {
            let raw = self.previous_frame.clone().unwrap_or_else(|| input.to_vec());
            processed = blend.apply(&raw, &processed)?;
        }

        self.previous_frame = Some(input.to_vec());
        Ok(processed)
    }

    fn reset(&mut self) {
        self.previous_frame = None;
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
}
