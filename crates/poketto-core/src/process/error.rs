use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("game executable not found: {0}")]
    NotFound(String),

    #[error("path is not a file: {0}")]
    NotAFile(String),

    #[error("permission denied: cannot execute {0}")]
    PermissionDenied(String),

    #[error("no wine installation found; install Wine or configure Wine settings")]
    NoWine,

    #[error("process launch failed: {0}")]
    LaunchFailed(String),
}

pub type ProcessResult<T> = Result<T, ProcessError>;
