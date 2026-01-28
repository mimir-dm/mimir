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
