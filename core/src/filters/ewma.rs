use crate::error::{CoreError, Result};

/// Exponentially Weighted Moving Average filter for temporal anti-flickering.
#[derive(Debug, Clone)]
pub struct EwmaFilter {
    alpha: f32,
    previous_frame: Option<Vec<f32>>,
}

impl EwmaFilter {
    pub fn new(alpha: f32) -> Result<Self> {
        if !(0.0..=1.0).contains(&alpha) {
            return Err(CoreError::InvalidParameter(format!(
                "alpha must be in [0, 1], got {}",
                alpha
            )));
        }
        Ok(Self {
            alpha,
            previous_frame: None,
        })
    }

    pub fn apply(&mut self, current: &[f32]) -> Vec<f32> {
        match &self.previous_frame {
            None => {
                self.previous_frame = Some(current.to_vec());
                current.to_vec()
            }
            Some(prev) => {
                let filtered: Vec<f32> = current
                    .iter()
                    .zip(prev.iter())
                    .map(|(&c, &p)| self.alpha * c + (1.0 - self.alpha) * p)
                    .collect();
                self.previous_frame = Some(filtered.clone());
                filtered
            }
        }
    }

    pub fn reset(&mut self) {
        self.previous_frame = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ewma_filter() {
        let mut filter = EwmaFilter::new(0.5).unwrap();
        let f1 = vec![0.0, 10.0];
        let f2 = vec![10.0, 0.0];

        assert_eq!(filter.apply(&f1), vec![0.0, 10.0]);
        assert_eq!(filter.apply(&f2), vec![5.0, 5.0]);
    }
}
