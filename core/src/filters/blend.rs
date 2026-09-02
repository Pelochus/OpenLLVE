use crate::error::{CoreError, Result};

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

    pub fn apply(&self, raw_input: &[f32], enhanced: &[f32]) -> Result<Vec<f32>> {
        if raw_input.len() != enhanced.len() {
            return Err(CoreError::BufferDimensionMismatch {
                expected: raw_input.len(),
                actual: enhanced.len(),
            });
        }

        let blended: Vec<f32> = raw_input
            .iter()
            .zip(enhanced.iter())
            .map(|(&raw, &enh)| self.beta * enh + (1.0 - self.beta) * raw)
            .collect();

        Ok(blended)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_blend() {
        let blender = FrameBlendFilter::new(0.8).unwrap();
        let raw = vec![0.0, 10.0];
        let enh = vec![10.0, 0.0];

        let result = blender.apply(&raw, &enh).unwrap();
        // 0.8 * 10.0 + 0.2 * 0.0 = 8.0
        // 0.8 * 0.0 + 0.2 * 10.0 = 2.0
        assert_eq!(result, vec![8.0, 2.0]);
    }
}
