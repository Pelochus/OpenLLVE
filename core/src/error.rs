use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CoreError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Buffer dimension mismatch: expected {expected}, got {actual}")]
    BufferDimensionMismatch { expected: usize, actual: usize },

    #[error("Inference execution failed: {0}")]
    InferenceFailure(String),

    #[error("Hardware acceleration error: {0}")]
    HardwareError(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
