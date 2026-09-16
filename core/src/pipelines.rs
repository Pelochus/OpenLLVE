mod llie;
mod temporal;

pub use llie::LliePipeline;
pub use temporal::LlveTemporalPipeline;

use crate::error::Result;
use crate::frame::{Frame, FrameRef};

/// Whether a pipeline carries state across frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalMode {
    /// Stateless: each frame is processed independently (e.g. LLIE).
    Stateless,
    /// Recurrent: the pipeline keeps state across frames (e.g. LLVE temporal).
    Recurrent,
}

/// A frame-processing pipeline.
pub trait Pipeline {
    /// Processes `input`, writing the result into the caller-owned `output`
    /// frame (no per-frame allocation).
    fn process(&mut self, input: &FrameRef, output: &mut Frame) -> Result<()>;

    /// Resets any inter-frame state (no-op for stateless pipelines).
    fn reset(&mut self);

    /// Whether this pipeline carries state across frames.
    fn mode(&self) -> TemporalMode;
}
