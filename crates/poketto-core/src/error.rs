use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("{0}")]
    NotFound(String),

    #[error("VNDB API error: {0}")]
    VndbApi(String),

    #[error("Authentication required: {0}")]
    AuthRequired(String),

    #[error("Process launch failed: {0}")]
    ProcessLaunch(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_match_legacy_wording() {
        assert_eq!(
            AppError::NotFound("game-1".to_string()).to_string(),
            "game-1"
        );
        assert_eq!(
            AppError::VndbApi("timeout".to_string()).to_string(),
            "VNDB API error: timeout"
        );
        assert_eq!(
            AppError::Validation("bad id".to_string()).to_string(),
            "Validation error: bad id"
        );
    }

    #[test]
    fn io_error_converts() {
        let err: AppError = std::io::Error::new(std::io::ErrorKind::NotFound, "missing").into();
        assert!(matches!(err, AppError::Io(_)));
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn result_alias_propagates_with_question_mark() {
        fn failing() -> AppResult<u32> {
            Err(AppError::Database("locked".to_string()))
        }
        fn caller() -> AppResult<u32> {
            failing()?;
            Ok(0)
        }
        assert!(matches!(caller(), Err(AppError::Database(_))));
    }
}
