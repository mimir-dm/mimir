//! Shared type definitions

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// API-level errors that can be sent to the frontend
///
/// All variants are serializable to JSON with a "type" and "message" field.
/// This enables structured error handling on the frontend.
#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum ApiError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(String),

    /// IO operation failed
    #[error("IO error: {0}")]
    Io(String),

    /// Serialization/deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Requested resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Request validation failed
    #[error("Validation error: {0}")]
    Validation(String),

    /// Request is invalid or malformed
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// User lacks permission for this operation
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<mimir_dm_core::error::DbError> for ApiError {
    fn from(err: mimir_dm_core::error::DbError) -> Self {
        match err {
            mimir_dm_core::error::DbError::NotFound { entity_type, id } => {
                ApiError::NotFound(format!("{} with id '{}' not found", entity_type, id))
            }
            mimir_dm_core::error::DbError::ConstraintViolation { field, message } => {
                ApiError::Validation(format!("{}: {}", field, message))
            }
            mimir_dm_core::error::DbError::InvalidData(msg) => ApiError::Validation(msg),
            _ => ApiError::Database(err.to_string()),
        }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::Serialization(err.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl From<String> for ApiError {
    fn from(err: String) -> Self {
        ApiError::Internal(err)
    }
}
