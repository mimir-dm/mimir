//! MCP Context
//!
//! Manages database connections and shared state for the MCP server.

use diesel::SqliteConnection;
use mimir_core::db::init_database;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::McpError;

/// Application paths for data storage.
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Directory containing the database
    pub data_dir: PathBuf,
    /// Directory for asset files (images, etc.)
    pub assets_dir: PathBuf,
}

impl AppPaths {
    /// Create paths using standard locations.
    pub fn standard() -> Self {
        let data_dir = directories::ProjectDirs::from("io", "colliery", "mimir")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            assets_dir: data_dir.join("assets"),
            data_dir,
        }
    }
}

/// Shared context for the MCP server.
///
/// Contains database connection and state that persists across tool calls.
pub struct McpContext {
    /// Database connection (wrapped in Mutex for thread safety)
    pub db: Mutex<SqliteConnection>,
    /// Application paths
    pub paths: AppPaths,
    /// Currently active campaign ID
    pub active_campaign_id: Mutex<Option<String>>,
}

impl McpContext {
    /// Create a new context with standard paths.
    pub fn new() -> Result<Self, McpError> {
        let paths = AppPaths::standard();

        // Ensure data directory exists
        std::fs::create_dir_all(&paths.data_dir).map_err(|e| {
            McpError::Initialization(format!("Failed to create data directory: {}", e))
        })?;

        let db_path = paths.data_dir.join("mimir.db");
        let db = init_database(db_path.to_str().unwrap_or("mimir.db")).map_err(|e| {
            McpError::Initialization(format!("Failed to connect to database: {}", e))
        })?;

        Ok(Self {
            db: Mutex::new(db),
            paths,
            active_campaign_id: Mutex::new(None),
        })
    }

    /// Get the active campaign ID, if set.
    pub fn get_active_campaign_id(&self) -> Option<String> {
        self.active_campaign_id
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }

    /// Set the active campaign ID.
    pub fn set_active_campaign_id(&self, campaign_id: Option<String>) {
        if let Ok(mut guard) = self.active_campaign_id.lock() {
            *guard = campaign_id;
        }
    }

    /// Get a mutable reference to the database connection.
    ///
    /// Returns an error if the mutex is poisoned.
    pub fn db(&self) -> Result<std::sync::MutexGuard<'_, SqliteConnection>, McpError> {
        self.db
            .lock()
            .map_err(|_| McpError::Internal("Database mutex poisoned".to_string()))
    }
}
