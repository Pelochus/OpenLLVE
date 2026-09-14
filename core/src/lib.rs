pub mod error;
pub mod ffi;
pub mod filters;
pub mod frame;
pub mod metrics;
pub mod strategies;

pub use error::{CoreError, Result};
pub use filters::{EwmaFilter, FrameBlendFilter};
pub use frame::NativeFrameHandle;
pub use metrics::BenchmarkMetrics;
pub use strategies::{InferenceStrategy, LlieStrategy, LlveTemporalStrategy};
