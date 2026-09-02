pub mod error;
pub mod frame;
pub mod filters;
pub mod metrics;
pub mod strategies;
pub mod ffi;

pub use error::{CoreError, Result};
pub use filters::{EwmaFilter, FrameBlendFilter};
pub use frame::NativeFrameHandle;
pub use metrics::BenchmarkMetrics;
pub use strategies::{InferenceStrategy, LlieStrategy, LlveTemporalStrategy};

