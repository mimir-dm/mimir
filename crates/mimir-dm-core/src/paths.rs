//! Application path resolution
//!
//! Provides standardized path resolution for Mimir application data and config.
//! This module is shared between the Tauri application and the MCP server to
//! ensure they discover the same directories.

use std::path::PathBuf;

/// Application paths for data and configuration
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Base application data directory
    pub app_dir: PathBuf,
    /// Configuration directory
    pub config_dir: PathBuf,
    /// Data subdirectory (contains database)
    pub data_dir: PathBuf,
    /// Logs directory
    pub logs_dir: PathBuf,
    /// Path to the SQLite database
    pub database_path: PathBuf,
}

impl AppPaths {
    /// Resolve application paths for the current environment.
    ///
    /// In development mode (debug builds or MIMIR_DEV env var set),
    /// uses "mimir-test" as the app name to avoid conflicts with production data.
    ///
    /// # Returns
    /// - `Some(AppPaths)` if paths could be resolved
    /// - `None` if the platform doesn't support standard directories
    pub fn resolve() -> Option<Self> {
        let is_dev = cfg!(debug_assertions) || std::env::var("MIMIR_DEV").is_ok();
        let app_name = if is_dev { "mimir-test" } else { "mimir" };
        Self::resolve_for_app(app_name)
    }

    /// Resolve application paths for a specific app name.
    ///
    /// # Arguments
    /// - `app_name` - The application name ("mimir" or "mimir-test")
    ///
    /// # Returns
    /// - `Some(AppPaths)` if paths could be resolved
    /// - `None` if the platform doesn't support standard directories
    pub fn resolve_for_app(app_name: &str) -> Option<Self> {
        let project_dirs = directories::ProjectDirs::from("com", "mimir", app_name)?;

        let app_dir = project_dirs.data_dir().to_path_buf();
        let config_dir = project_dirs.config_dir().to_path_buf();
        let data_dir = app_dir.join("data");
        let logs_dir = app_dir.join("logs");
        let database_path = data_dir.join("mimir.db");

        Some(Self {
            app_dir,
            config_dir,
            data_dir,
            logs_dir,
            database_path,
        })
    }

    /// Check if the database exists at the resolved path
    pub fn database_exists(&self) -> bool {
        self.database_path.exists()
    }

    /// Get the database path as a string for database connections
    pub fn database_url(&self) -> String {
        self.database_path.to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_paths() {
        let paths = AppPaths::resolve();
        assert!(paths.is_some());

        let paths = paths.unwrap();
        assert!(paths.database_path.ends_with("mimir.db"));
        assert!(paths.data_dir.ends_with("data"));
    }

    #[test]
    fn test_resolve_for_app() {
        let prod_paths = AppPaths::resolve_for_app("mimir");
        let dev_paths = AppPaths::resolve_for_app("mimir-test");

        assert!(prod_paths.is_some());
        assert!(dev_paths.is_some());

        // Paths should be different for different app names
        assert_ne!(
            prod_paths.unwrap().app_dir,
            dev_paths.unwrap().app_dir
        );
    }
}
