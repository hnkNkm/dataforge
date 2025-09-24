use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application-wide error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] crate::database::DatabaseError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

/// Error response structure for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_type: String,
    pub message: String,
    pub details: Option<String>,
    pub code: Option<String>,
}

impl From<&AppError> for ErrorResponse {
    fn from(err: &AppError) -> Self {
        let error_type = match err {
            AppError::Database(_) => "database",
            AppError::Config(_) => "config",
            AppError::Io(_) => "io",
            AppError::Serialization(_) => "serialization",
            AppError::Tauri(_) => "tauri",
            AppError::Network(_) => "network",
            AppError::Auth(_) => "auth",
            AppError::Validation(_) => "validation",
            AppError::NotFound(_) => "not_found",
            AppError::PermissionDenied(_) => "permission_denied",
            AppError::Cancelled => "cancelled",
            AppError::Unknown(_) => "unknown",
        };

        ErrorResponse {
            error_type: error_type.to_string(),
            message: err.to_string(),
            details: None,
            code: None,
        }
    }
}

impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        ErrorResponse::from(&err)
    }
}

/// Convert AppError to a format suitable for Tauri commands
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        serde_json::to_string(&ErrorResponse::from(&err))
            .unwrap_or_else(|_| err.to_string())
    }
}

/// Macro for creating validation errors with context
#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        $crate::error::AppError::Validation($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::AppError::Validation(format!($fmt, $($arg)*))
    };
}

/// Macro for creating not found errors with context
#[macro_export]
macro_rules! not_found_error {
    ($msg:expr) => {
        $crate::error::AppError::NotFound($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::error::AppError::NotFound(format!($fmt, $($arg)*))
    };
}

/// Extension trait for Result types to add context
pub trait ErrorContext<T> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<AppError>,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let base_error = e.into();
            AppError::Unknown(format!("{}: {}", context, base_error))
        })
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let base_error = e.into();
            AppError::Unknown(format!("{}: {}", f(), base_error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let err = AppError::Database(crate::database::DatabaseError::ConnectionFailed(
            "test".to_string(),
        ));
        let response = ErrorResponse::from(err);
        assert_eq!(response.error_type, "database");
    }

    #[test]
    fn test_validation_error_macro() {
        let err = validation_error!("Invalid input");
        match err {
            AppError::Validation(msg) => assert_eq!(msg, "Invalid input"),
            _ => panic!("Expected validation error"),
        }
    }

    #[test]
    fn test_error_context() {
        let result: std::result::Result<i32, std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));

        let with_context = result.context("Failed to read configuration");
        assert!(with_context.is_err());

        if let Err(err) = with_context {
            let msg = err.to_string();
            assert!(msg.contains("Failed to read configuration"));
        }
    }
}