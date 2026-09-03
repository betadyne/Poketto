use thiserror::Error;

#[derive(Debug, Error)]
pub enum VndbError {
    #[error("http error: {0}")]
    Http(String),

    #[error("vndb api error: {0}")]
    Api(String),

    #[error("authentication required: {0}")]
    AuthRequired(String),

    #[error("vn not found: {0}")]
    NotFound(String),
}

pub type VndbResult<T> = Result<T, VndbError>;

impl From<reqwest::Error> for VndbError {
    fn from(e: reqwest::Error) -> Self {
        VndbError::Http(e.to_string())
    }
}
