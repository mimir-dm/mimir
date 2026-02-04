//! Application State
//!
//! Manages shared state for the Tauri application including database connections,
//! application paths, and dev/production mode detection.
//!
//! Database connections are created on-demand rather than held in a mutex.
//! This allows concurrent read operations with SQLite WAL mode.

use diesel::SqliteConnection;
use mimir_core::db::create_connection;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// Check if running in development mode.
///
/// Returns true if either:
/// - Compiled with debug assertions (`cargo build` without `--release`)
/// - The `MIMIR_DEV` environment variable is set
pub fn is_dev_mode() -> bool {
    cfg!(debug_assertions) || std::env::var("MIMIR_DEV").is_ok()
}

/// Application paths configuration.
///
/// Paths differ based on dev vs production mode to prevent dev testing
/// from affecting production data.
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Root application data directory.
    pub app_dir: PathBuf,
    /// Directory for configuration files.
    pub config_dir: PathBuf,
    /// Directory for data files (database, etc.).
    pub data_dir: PathBuf,
    /// Directory for log files.
    pub logs_dir: PathBuf,
    /// Directory for campaign assets (maps, images).
    pub assets_dir: PathBuf,
    /// Path to the SQLite database file.
    pub database_path: PathBuf,
    /// Whether running in development mode.
    pub is_dev: bool,
}

impl AppPaths {
    /// Initialize application paths from Tauri's app data directory.
    ///
    /// In dev mode, uses a separate directory to avoid affecting production data.
    pub fn from_tauri_path(tauri_app_data_dir: PathBuf) -> std::io::Result<Self> {
        let is_dev = is_dev_mode();

        // In dev mode, use a subdirectory to keep dev data separate
        let app_dir = if is_dev {
            tauri_app_data_dir.join("dev")
        } else {
            tauri_app_data_dir
        };

        let config_dir = app_dir.join("config");
        let data_dir = app_dir.join("data");
        let logs_dir = app_dir.join("logs");
        let assets_dir = app_dir.join("assets");
        let database_path = data_dir.join("mimir.db");

        let paths = Self {
            app_dir,
            config_dir,
            data_dir,
            logs_dir,
            assets_dir,
            database_path,
            is_dev,
        };

        // Create directories
        paths.ensure_directories()?;

        // Log initialization
        eprintln!(
            "Mimir {} mode initialized:",
            if is_dev { "DEVELOPMENT" } else { "PRODUCTION" }
        );
        eprintln!("  App dir: {}", paths.app_dir.display());
        eprintln!("  Data dir: {}", paths.data_dir.display());
        eprintln!("  Database: {}", paths.database_path.display());

        Ok(paths)
    }

    /// Ensure all required directories exist.
    fn ensure_directories(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.app_dir)?;
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.data_dir)?;
        fs::create_dir_all(&self.logs_dir)?;
        fs::create_dir_all(&self.assets_dir)?;
        Ok(())
    }

    /// Check if this is a fresh database (for seeding/first-run logic).
    pub fn is_new_database(&self) -> bool {
        !self.database_path.exists()
    }

    /// Get the database path as a string for Diesel.
    pub fn database_url(&self) -> String {
        self.database_path.to_string_lossy().to_string()
    }
}

/// Application state shared across all Tauri commands.
///
/// This struct is managed by Tauri and can be accessed in command handlers
/// via the `State` extractor.
///
/// Database connections are created on-demand via `connect()` rather than
/// held in a mutex. This allows concurrent read operations with SQLite WAL mode.
pub struct AppState {
    /// Database URL for creating connections.
    db_url: String,
    /// Application paths configuration.
    pub paths: AppPaths,
    /// Active campaign ID (for commands that need it).
    pub active_campaign_id: Mutex<Option<String>>,
}

impl AppState {
    /// Create a new AppState with the given paths.
    ///
    /// The database should already be initialized (migrations run) before creating AppState.
    pub fn new(paths: AppPaths) -> Self {
        Self {
            db_url: paths.database_url(),
            paths,
            active_campaign_id: Mutex::new(None),
        }
    }

    /// Create a new database connection.
    ///
    /// Each connection is configured with WAL mode and foreign keys enabled.
    /// Returns an error if the connection cannot be established.
    pub fn connect(&self) -> Result<SqliteConnection, String> {
        create_connection(&self.db_url).map_err(|e| format!("Database connection error: {}", e))
    }

    /// Check if running in development mode.
    pub fn is_dev(&self) -> bool {
        self.paths.is_dev
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
}
