use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, specta::Type)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Bincode(String),

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

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Http(e.to_string())
    }
}

impl From<redb::Error> for AppError {
    fn from(e: redb::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<redb::TransactionError> for AppError {
    fn from(e: redb::TransactionError) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<redb::TableError> for AppError {
    fn from(e: redb::TableError) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<redb::StorageError> for AppError {
    fn from(e: redb::StorageError) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<redb::CommitError> for AppError {
    fn from(e: redb::CommitError) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<bincode::Error> for AppError {
    fn from(e: bincode::Error) -> Self {
        AppError::Bincode(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    mod from_io_error_tests {
        use super::*;
        use std::io::{Error as IoError, ErrorKind};

        #[test]
        fn test_converts_not_found_error() {
            let io_err = IoError::new(ErrorKind::NotFound, "file not found");
            let app_err: AppError = io_err.into();

            match app_err {
                AppError::Io(msg) => {
                    assert!(msg.contains("file not found"));
                }
                _ => panic!("Expected Io error"),
            }
        }

        #[test]
        fn test_converts_permission_denied() {
            let io_err = IoError::new(ErrorKind::PermissionDenied, "access denied");
            let app_err: AppError = io_err.into();

            match app_err {
                AppError::Io(msg) => {
                    assert!(msg.contains("access denied"));
                }
                _ => panic!("Expected Io error"),
            }
        }

        #[test]
        fn test_preserves_error_message() {
            let io_err = IoError::new(ErrorKind::Other, "custom error message");
            let app_err: AppError = io_err.into();

            assert!(app_err.to_string().contains("custom error message"));
        }
    }

    mod from_json_error_tests {
        use super::*;

        #[test]
        fn test_converts_json_syntax_error() {
            let result: Result<serde_json::Value, _> = serde_json::from_str("invalid json");
            let json_err = result.unwrap_err();
            let app_err: AppError = json_err.into();

            match app_err {
                AppError::Json(msg) => {
                    assert!(!msg.is_empty());
                }
                _ => panic!("Expected Json error"),
            }
        }

        #[test]
        fn test_error_display_includes_json_prefix() {
            let result: Result<serde_json::Value, _> = serde_json::from_str("{invalid}");
            let json_err = result.unwrap_err();
            let app_err: AppError = json_err.into();

            assert!(app_err.to_string().contains("JSON error"));
        }
    }

    mod error_display_tests {
        use super::*;

        #[test]
        fn test_io_error_display() {
            let err = AppError::Io("test message".to_string());
            assert_eq!(err.to_string(), "IO error: test message");
        }

        #[test]
        fn test_json_error_display() {
            let err = AppError::Json("parse failed".to_string());
            assert_eq!(err.to_string(), "JSON error: parse failed");
        }

        #[test]
        fn test_http_error_display() {
            let err = AppError::Http("connection failed".to_string());
            assert_eq!(err.to_string(), "HTTP error: connection failed");
        }

        #[test]
        fn test_database_error_display() {
            let err = AppError::Database("db error".to_string());
            assert_eq!(err.to_string(), "Database error: db error");
        }

        #[test]
        fn test_bincode_error_display() {
            let err = AppError::Bincode("serialize failed".to_string());
            assert_eq!(err.to_string(), "Serialization error: serialize failed");
        }

        #[test]
        fn test_not_found_error_display() {
            let err = AppError::NotFound("game not found".to_string());
            assert_eq!(err.to_string(), "game not found");
        }

        #[test]
        fn test_vndb_api_error_display() {
            let err = AppError::VndbApi("rate limited".to_string());
            assert_eq!(err.to_string(), "VNDB API error: rate limited");
        }

        #[test]
        fn test_auth_required_error_display() {
            let err = AppError::AuthRequired("token missing".to_string());
            assert_eq!(err.to_string(), "Authentication required: token missing");
        }

        #[test]
        fn test_process_launch_error_display() {
            let err = AppError::ProcessLaunch("exe not found".to_string());
            assert_eq!(err.to_string(), "Process launch failed: exe not found");
        }

        #[test]
        fn test_validation_error_display() {
            let err = AppError::Validation("invalid path".to_string());
            assert_eq!(err.to_string(), "Validation error: invalid path");
        }
    }

    mod serialize_tests {
        use super::*;

        #[test]
        fn test_serializes_to_string() {
            let err = AppError::Io("test error".to_string());
            let serialized = serde_json::to_string(&err).unwrap();

            assert_eq!(serialized, "\"IO error: test error\"");
        }

        #[test]
        fn test_all_variants_serialize() {
            let variants = vec![
                AppError::Io("msg".to_string()),
                AppError::Json("msg".to_string()),
                AppError::Http("msg".to_string()),
                AppError::Database("msg".to_string()),
                AppError::Bincode("msg".to_string()),
                AppError::NotFound("msg".to_string()),
                AppError::VndbApi("msg".to_string()),
                AppError::AuthRequired("msg".to_string()),
                AppError::ProcessLaunch("msg".to_string()),
                AppError::Validation("msg".to_string()),
            ];

            for err in variants {
                let result = serde_json::to_string(&err);
                assert!(result.is_ok(), "Failed to serialize: {:?}", err);
            }
        }
    }
}
