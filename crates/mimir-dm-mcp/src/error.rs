//! MCP server error types
//!
//! Provides error types for the MCP server that integrate with both
//! mimir-dm-core errors and MCP protocol error responses.

use thiserror::Error;

/// Errors that can occur in the MCP server
#[derive(Error, Debug)]
pub enum McpError {
    /// No active campaign has been set
    #[error("No active campaign selected. Use set_active_campaign first.")]
    NoCampaignSelected,

    /// Campaign not found
    #[error("Campaign not found: {0}")]
    CampaignNotFound(String),

    /// Database connection error
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    /// Database connection pool error
    #[error("Database connection error: {0}")]
    ConnectionPool(String),

    /// Service error from mimir-dm-core
    #[error("Service error: {0}")]
    Service(String),

    /// Invalid input parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Document not found
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Character not found
    #[error("Character not found: {0}")]
    CharacterNotFound(String),

    /// Module not found
    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    /// File I/O error
    #[error("File error: {0}")]
    FileIo(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl McpError {
    /// Convert to an MCP-friendly error message
    pub fn to_mcp_error(&self) -> String {
        self.to_string()
    }
}
