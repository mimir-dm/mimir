//! Error types for Mimir
//!
//! This module provides a unified error handling system:
//! - `DbError` - Database-specific errors
//! - `MimirError` - Unified error type that wraps domain errors
//!
//! ## Usage
//!
//! ### In services that use database operations
//! ```ignore
//! use mimir_dm_core::{MimirError, MimirResult};
//!
//! fn get_item(id: i32) -> MimirResult<Item> {
//!     // DbError automatically converts to MimirError
//!     let item = repo.find_by_id(id)?;
//!     Ok(item)
//! }
//! ```
//!
//! ### Converting to String for Tauri commands
//! ```ignore
//! #[tauri::command]
//! async fn get_item(id: i32) -> Result<Item, String> {
//!     service.get_item(id).map_err(|e| e.to_string())
//! }
//! ```
//!
//! ### Wrapping other error types
//! ```ignore
//! // PrintError -> MimirError (preserves full error chain)
//! let result = print_service.render()
//!     .map_err(|e| MimirError::Print(Box::new(e)))?;
//!
//! // LlmError -> MimirError (preserves full error chain)
//! let response = llm_service.query()
//!     .map_err(|e| MimirError::Llm(Box::new(e)))?;
//! ```

use thiserror::Error;

/// Result type alias for database operations
pub type Result<T> = std::result::Result<T, DbError>;

/// Result type alias using the unified MimirError
pub type MimirResult<T> = std::result::Result<T, MimirError>;

/// Boxed error type for cross-crate error handling.
/// Preserves the full error chain while avoiding circular dependencies.
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Unified error type for the Mimir application.
///
/// This enum wraps domain-specific errors and provides a consistent
/// error handling interface across the application.
#[derive(Error, Debug)]
pub enum MimirError {
    /// Database operation error
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    /// Print/PDF generation error (boxed to avoid circular deps with mimir-dm-print)
    #[error("Print error: {0}")]
    Print(#[source] BoxedError),

    /// LLM operation error (boxed to avoid circular deps with mimir-dm-llm)
    #[error("LLM error: {0}")]
    Llm(#[source] BoxedError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Permission/authorization error
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// External service error
    #[error("External service error: {0}")]
    ExternalService(String),

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl MimirError {
    /// Create a print error from any error type
    pub fn print<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        MimirError::Print(Box::new(err))
    }

    /// Create an LLM error from any error type
    pub fn llm<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        MimirError::Llm(Box::new(err))
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        MimirError::Validation(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        MimirError::NotFound(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        MimirError::Internal(msg.into())
    }

    /// Check if this is a not found error
    pub fn is_not_found(&self) -> bool {
        match self {
            MimirError::NotFound(_) => true,
            MimirError::Database(db) => db.is_not_found(),
            _ => false,
        }
    }
}

/// Convert MimirError to String for Tauri command responses
impl From<MimirError> for String {
    fn from(err: MimirError) -> String {
        err.to_string()
    }
}

/// Database error types
#[derive(Error, Debug)]
pub enum DbError {
    /// Diesel connection error
    #[error("Database connection error: {0}")]
    Connection(#[from] diesel::ConnectionError),

    /// Diesel query/result error
    #[error("Database query error: {0}")]
    Query(#[from] diesel::result::Error),

    /// Entity not found
    #[error("Entity not found: {entity_type} with id '{id}'")]
    NotFound {
        /// Type of entity that was not found.
        entity_type: String,
        /// Identifier that was searched for.
        id: String,
    },

    /// Constraint violation
    #[error("Constraint violation: {field} - {message}")]
    ConstraintViolation {
        /// Field that violated the constraint.
        field: String,
        /// Description of the violation.
        message: String,
    },

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid data
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Async runtime error
    #[error("Async runtime error: {0}")]
    Runtime(#[from] tokio::task::JoinError),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl DbError {
    /// Check if error is a unique constraint violation
    pub fn is_unique_violation(&self) -> bool {
        matches!(
            self,
            DbError::Query(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _
            ))
        )
    }

    /// Check if error is a foreign key violation
    pub fn is_foreign_key_violation(&self) -> bool {
        matches!(
            self,
            DbError::Query(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _
            ))
        )
    }

    /// Check if error is not found
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            DbError::NotFound { .. } | DbError::Query(diesel::result::Error::NotFound)
        )
    }
}
