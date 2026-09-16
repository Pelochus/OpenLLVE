mod llie;
mod temporal;

pub use llie::LlieStrategy;
pub use temporal::LlveTemporalStrategy;

use crate::error::Result;
use crate::frame::{Frame, FrameRef};

pub trait InferenceStrategy {
    /// Processes `input`, writing the result into the caller-owned `output`
    /// frame (no per-frame allocation).
    fn process(&mut self, input: &FrameRef, output: &mut Frame) -> Result<()>;
    fn reset(&mut self);
}
