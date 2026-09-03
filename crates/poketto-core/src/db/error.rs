use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("database configuration failed: {0}")]
    Config(String),

    #[error("integer out of range: {0}")]
    OutOfRange(u64),
    #[error("game not found: {0}")]
    GameNotFound(String),
}

pub type DbResult<T> = Result<T, DbError>;

impl From<serde_json::Error> for DbError {
    fn from(e: serde_json::Error) -> Self {
        DbError::Serialization(e.to_string())
    }
}
