use thiserror::Error;

#[derive(Debug, Error)]
pub enum WineError {
    #[error("wine binary not found: {0}")]
    NotFound(String),

    #[error("path is not a file: {0}")]
    NotAFile(String),

    #[error("failed to execute wine binary: {0}")]
    ExecutionFailed(String),

    #[error("io error: {0}")]
    Io(String),
}

pub type WineResult<T> = Result<T, WineError>;

impl From<std::io::Error> for WineError {
    fn from(e: std::io::Error) -> Self {
        WineError::Io(e.to_string())
    }
}
