pub mod error;
pub mod ffi;
pub mod filters;
pub mod frame;
pub mod metrics;
#[cfg(feature = "model")]
pub mod model;
pub mod strategies;

pub use error::{CoreError, Result};
pub use filters::{EwmaFilter, FrameBlendFilter};
pub use frame::{Frame, FrameFormat, FrameRef, NativeFrameHandle, OwnedFrame};
pub use metrics::BenchmarkMetrics;
#[cfg(feature = "model")]
pub use model::ModelRunner;
pub use strategies::{InferenceStrategy, LlieStrategy, LlveTemporalStrategy};
