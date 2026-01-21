//! Error Types
//!
//! Error handling for Tauri commands, converting service errors to serializable responses.

use mimir_core::services::ServiceError;
use serde::Serialize;

/// Error type returned from Tauri commands.
///
/// This wraps service errors and makes them serializable for the frontend.
#[derive(Debug, Serialize)]
pub struct CommandError {
    /// Human-readable error message.
    pub message: String,
    /// Error code for programmatic handling.
    pub code: String,
}

impl From<ServiceError> for CommandError {
    fn from(err: ServiceError) -> Self {
        let (code, message) = match &err {
            ServiceError::NotFound { entity_type, id } => {
                ("NOT_FOUND".to_string(), format!("{} with id '{}' not found", entity_type, id))
            }
            ServiceError::Validation(msg) => ("VALIDATION_ERROR".to_string(), msg.clone()),
            ServiceError::Database(e) => ("DATABASE_ERROR".to_string(), e.to_string()),
            ServiceError::Io(e) => ("IO_ERROR".to_string(), e.to_string()),
        };
        Self { message, code }
    }
}

impl From<diesel::result::Error> for CommandError {
    fn from(err: diesel::result::Error) -> Self {
        Self {
            message: err.to_string(),
            code: "DATABASE_ERROR".to_string(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: err.to_string(),
            code: "IO_ERROR".to_string(),
        }
    }
}

/// Result type for Tauri commands.
pub type CommandResult<T> = Result<T, CommandError>;
