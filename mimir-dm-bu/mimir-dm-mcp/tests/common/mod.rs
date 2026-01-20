//! Common test utilities for MCP tool tests.
//!
//! Provides helpers to construct test McpContext with in-memory database
//! for verifying MCP tools work correctly.

use mimir_dm_core::services::CampaignService;
use mimir_dm_mcp::McpContext;
use std::sync::Arc;
use tempfile::TempDir;

/// Test environment containing McpContext and temp directories
pub struct TestMcpEnv {
    /// The McpContext for testing
    pub context: Arc<McpContext>,
    /// Temp directory - kept alive to prevent cleanup during test
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    /// Path to the temp database file
    pub db_path: String,
}

impl TestMcpEnv {
    /// Create a new test environment with in-memory database
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_string_lossy().to_string();

        // Initialize database with file-based connection (required for McpContext)
        // Note: We use file-based because McpContext creates new connections
        let mut conn = mimir_dm_core::connection::establish_connection(&db_path_str)?;
        mimir_dm_core::run_migrations(&mut conn)?;
        // Seed templates for module and document creation
        mimir_dm_core::seed::template_seeder::seed_templates(&mut conn)?;
        drop(conn);

        // Create McpContext with database path
        let context = McpContext::new(db_path_str.clone());

        Ok(Self {
            context,
            temp_dir,
            db_path: db_path_str,
        })
    }

    /// Create a test campaign and return its ID and directory path
    pub fn create_campaign(&self, name: &str) -> anyhow::Result<(i32, String)> {
        let mut conn = self.context.get_connection()?;
        let mut service = CampaignService::new(&mut conn);

        let campaign = service.create_campaign(
            name,
            None,
            self.temp_dir.path().to_str().unwrap(),
        )?;

        Ok((campaign.id, campaign.directory_path))
    }

    /// Set a campaign as active in the context
    pub async fn set_active_campaign(&self, campaign_id: i32) -> anyhow::Result<()> {
        let mut conn = self.context.get_connection()?;
        let mut service = CampaignService::new(&mut conn);

        let campaign = service
            .get_campaign(campaign_id)?
            .ok_or_else(|| anyhow::anyhow!("Campaign not found"))?;

        self.context.set_active_campaign(campaign).await;
        Ok(())
    }
}
