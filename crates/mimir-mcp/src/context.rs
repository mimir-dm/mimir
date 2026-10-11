//! MCP Context
//!
//! Manages database connections and shared state for the MCP server.
//!
//! Database connections are created on-demand rather than held in a mutex.
//! This allows concurrent read operations with SQLite WAL mode.

use diesel::SqliteConnection;
use mimir_core::db::create_connection;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::McpError;

/// Shared context for the MCP server.
///
/// Contains database path and state that persists across tool calls.
/// Connections are created on-demand via `connect()`.
pub struct McpContext {
    /// Database URL for creating connections.
    db_url: String,
    /// The app folder: it holds `data/` (the database) and `assets/`; the
    /// assets' paths are relative to it ("assets/<id>.<ext>").
    pub app_dir: PathBuf,
    /// Its `assets` folder.
    pub assets_dir: PathBuf,
    /// Currently active campaign ID
    pub active_campaign_id: Mutex<Option<String>>,
}

impl McpContext {
    /// A context over a database already set up (the server's `/mcp`, one
    /// per MCP session: the active campaign is the session's).
    pub fn from_parts(db_url: impl Into<String>, app_dir: impl Into<PathBuf>) -> Self {
        let app_dir = app_dir.into();
        Self {
            db_url: db_url.into(),
            assets_dir: app_dir.join("assets"),
            app_dir,
            active_campaign_id: Mutex::new(None),
        }
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

    /// Create a new database connection.
    ///
    /// Each connection is configured with WAL mode and foreign keys enabled.
    /// Returns an error if the connection cannot be established.
    pub fn connect(&self) -> Result<SqliteConnection, McpError> {
        create_connection(&self.db_url)
            .map_err(|e| McpError::Internal(format!("Database connection error: {}", e)))
    }

    /// Create a context for testing with a temporary file-based database.
    ///
    /// Uses a unique temp file so multiple connections share the same DB.
    /// The database is initialized with migrations run.
    #[cfg(test)]
    pub fn for_testing() -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let db_url = format!("/tmp/mimir-test-{}-{}.db", std::process::id(), id);

        // Initialize with migrations
        mimir_core::db::init_database(&db_url).expect("Failed to initialize test database");

        Self {
            db_url,
            app_dir: PathBuf::from("/tmp/mimir-test-app"),
            assets_dir: PathBuf::from("/tmp/mimir-test-app/assets"),
            active_campaign_id: Mutex::new(None),
        }
    }
}
