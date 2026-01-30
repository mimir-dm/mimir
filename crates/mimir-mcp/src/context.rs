//! MCP Context
//!
//! Manages database connections and shared state for the MCP server.

use diesel::SqliteConnection;
use mimir_core::db::init_database;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::McpError;

/// Shared context for the MCP server.
///
/// Contains database connection and state that persists across tool calls.
pub struct McpContext {
    /// Database connection (wrapped in Mutex for thread safety)
    pub db: Mutex<SqliteConnection>,
    /// Directory for asset files (images, etc.)
    pub assets_dir: PathBuf,
    /// Currently active campaign ID
    pub active_campaign_id: Mutex<Option<String>>,
}

impl McpContext {
    /// Create a new context from `MIMIR_DATABASE_PATH` environment variable.
    ///
    /// The env var must point to the SQLite database file
    /// (e.g. `~/Library/Application Support/com.mimir.app/data/mimir.db`).
    pub fn new() -> Result<Self, McpError> {
        let db_path = std::env::var("MIMIR_DATABASE_PATH")
            .ok()
            .filter(|p| !p.is_empty() && !p.starts_with("${"))
            .or_else(|| Self::default_db_path())
            .map(|p| Self::expand_path(&p))
            .ok_or_else(|| {
                McpError::Initialization(
                    "Could not determine database path. Set MIMIR_DATABASE_PATH \
                     to the path of your Mimir database file \
                     (e.g. ~/Library/Application Support/com.mimir.app/data/mimir.db)"
                        .to_string(),
                )
            })?;

        let assets_dir = db_path
            .parent()
            .map(|p| p.join("assets"))
            .unwrap_or_else(|| PathBuf::from("assets"));

        let db = init_database(db_path.to_str().unwrap_or("mimir.db")).map_err(|e| {
            McpError::Initialization(format!(
                "Failed to connect to database at {}: {}",
                db_path.display(),
                e
            ))
        })?;

        Ok(Self {
            db: Mutex::new(db),
            assets_dir,
            active_campaign_id: Mutex::new(None),
        })
    }

    /// Expand `~` and `$HOME` in a path string.
    fn expand_path(path: &str) -> PathBuf {
        let home = std::env::var("HOME").ok();
        if path.starts_with("~/") {
            if let Some(h) = &home {
                return PathBuf::from(h).join(&path[2..]);
            }
        } else if path.starts_with("$HOME/") {
            if let Some(h) = &home {
                return PathBuf::from(h).join(&path[6..]);
            }
        }
        PathBuf::from(path)
    }

    /// Return the default database path for the current platform.
    fn default_db_path() -> Option<String> {
        let home = std::env::var("HOME").ok()?;
        #[cfg(target_os = "macos")]
        {
            Some(format!("{}/Library/Application Support/com.mimir.app/data/mimir.db", home))
        }
        #[cfg(target_os = "linux")]
        {
            let data = std::env::var("XDG_DATA_HOME")
                .unwrap_or_else(|_| format!("{}/.local/share", home));
            Some(format!("{}/com.mimir.app/data/mimir.db", data))
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            None
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

    /// Get a mutable reference to the database connection.
    ///
    /// Returns an error if the mutex is poisoned.
    pub fn db(&self) -> Result<std::sync::MutexGuard<'_, SqliteConnection>, McpError> {
        self.db
            .lock()
            .map_err(|_| McpError::Internal("Database mutex poisoned".to_string()))
    }
}
