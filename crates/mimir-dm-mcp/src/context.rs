//! MCP context management
//!
//! Provides context state for the MCP server including the active campaign
//! and database connection management.

use crate::error::McpError;
use mimir_dm_core::connection::{establish_connection, DbConnection};
use mimir_dm_core::models::campaign::campaigns::Campaign;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// MCP server context containing shared state
pub struct McpContext {
    /// The currently selected campaign
    active_campaign: Mutex<Option<Campaign>>,
    /// Path to the SQLite database
    database_path: String,
}

impl McpContext {
    /// Create a new MCP context
    pub fn new(database_path: String) -> Arc<Self> {
        Arc::new(Self {
            active_campaign: Mutex::new(None),
            database_path,
        })
    }

    /// Get a new database connection
    ///
    /// Creates a fresh connection for each operation to ensure thread safety
    /// and proper cleanup.
    pub fn get_connection(&self) -> Result<DbConnection, McpError> {
        establish_connection(&self.database_path)
            .map_err(|e| McpError::ConnectionPool(e.to_string()))
    }

    /// Get the database path
    pub fn database_path(&self) -> &str {
        &self.database_path
    }

    /// Set the active campaign
    pub async fn set_active_campaign(&self, campaign: Campaign) {
        info!(campaign_id = campaign.id, name = %campaign.name, "Setting active campaign");
        let mut active = self.active_campaign.lock().await;
        *active = Some(campaign);
    }

    /// Get the active campaign
    pub async fn get_active_campaign(&self) -> Option<Campaign> {
        let active = self.active_campaign.lock().await;
        active.clone()
    }

    /// Get the active campaign or return an error
    pub async fn require_active_campaign(&self) -> Result<Campaign, McpError> {
        self.get_active_campaign()
            .await
            .ok_or(McpError::NoCampaignSelected)
    }

    /// Clear the active campaign
    pub async fn clear_active_campaign(&self) {
        let mut active = self.active_campaign.lock().await;
        *active = None;
    }
}
