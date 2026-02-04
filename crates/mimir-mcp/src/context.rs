//! MCP Context
//!
//! Manages database connections and shared state for the MCP server.
//!
//! Database connections are created on-demand rather than held in a mutex.
//! This allows concurrent read operations with SQLite WAL mode.

use diesel::SqliteConnection;
use mimir_core::db::{create_connection, init_database};
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

        let db_url = db_path.to_string_lossy().to_string();

        // Initialize database (runs migrations) - connection is dropped after
        let _db = init_database(&db_url).map_err(|e| {
            McpError::Initialization(format!(
                "Failed to initialize database at {}: {}",
                db_path.display(),
                e
            ))
        })?;

        Ok(Self {
            db_url,
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

    /// Create a new database connection.
    ///
    /// Each connection is configured with WAL mode and foreign keys enabled.
    /// Returns an error if the connection cannot be established.
    pub fn connect(&self) -> Result<SqliteConnection, McpError> {
        create_connection(&self.db_url).map_err(|e| {
            McpError::Internal(format!("Database connection error: {}", e))
        })
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
        init_database(&db_url).expect("Failed to initialize test database");

        Self {
            db_url,
            assets_dir: PathBuf::from("/tmp/mimir-test-assets"),
            active_campaign_id: Mutex::new(None),
        }
    }
}
