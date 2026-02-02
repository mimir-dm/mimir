//! MCP Error Types

use thiserror::Error;

/// Errors that can occur in the MCP server.
#[derive(Debug, Error)]
pub enum McpError {
    /// Server initialization failed
    #[error("Initialization error: {0}")]
    Initialization(String),

    /// Database operation failed
    #[error("Database error: {0}")]
    Database(String),

    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Invalid tool arguments
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// No active campaign set
    #[error("No active campaign. Use set_active_campaign first.")]
    NoActiveCampaign,

    /// Resource not found
    #[error("{0} not found: {1}")]
    NotFound(String, String),

    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<diesel::result::Error> for McpError {
    fn from(e: diesel::result::Error) -> Self {
        McpError::Database(e.to_string())
    }
}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        McpError::InvalidArguments(e.to_string())
    }
}

impl From<mimir_core::services::ServiceError> for McpError {
    fn from(e: mimir_core::services::ServiceError) -> Self {
        match e {
            mimir_core::services::ServiceError::NotFound { entity_type, id } => {
                McpError::NotFound(entity_type, id)
            }
            mimir_core::services::ServiceError::Validation(msg) => {
                McpError::InvalidArguments(msg)
            }
            mimir_core::services::ServiceError::Database(e) => {
                McpError::Database(e.to_string())
            }
            mimir_core::services::ServiceError::Io(e) => {
                McpError::Internal(e.to_string())
            }
        }
    }
}
