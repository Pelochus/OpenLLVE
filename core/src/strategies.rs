mod llie;
mod temporal;

pub use llie::LlieStrategy;
pub use temporal::LlveTemporalStrategy;

use crate::error::Result;

pub trait InferenceStrategy {
    fn process(&mut self, input: &[f32]) -> Result<Vec<f32>>;
    fn reset(&mut self);
}
