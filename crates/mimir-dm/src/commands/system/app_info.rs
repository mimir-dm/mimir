//! Application info commands.
//!
//! Provides Tauri commands for retrieving application configuration
//! and path information.

use crate::state::AppState;
use crate::types::ApiResponse;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

/// Application configuration and path information.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    /// Path to the SQLite database file.
    pub database_path: String,
    /// Root application directory.
    pub app_dir: String,
    /// Configuration directory for settings.
    pub config_dir: String,
    /// Data directory for user content.
    pub data_dir: String,
    /// Path to bundled resources (skills, plugins, etc.).
    pub resources_dir: Option<String>,
}

/// Get application path and configuration information.
///
/// Returns paths to important application directories and files.
///
/// # Returns
/// `ApiResponse` containing `AppInfo` with all relevant paths.
#[tauri::command]
pub async fn get_app_info(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApiResponse<AppInfo>, String> {
    // Get the bundled resources directory (for skills, etc.)
    let resources_dir = app
        .path()
        .resource_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let app_info = AppInfo {
        database_path: state.paths.database_path_str(),
        app_dir: state.paths.app_dir.to_string_lossy().to_string(),
        config_dir: state.paths.config_dir.to_string_lossy().to_string(),
        data_dir: state.paths.data_dir.to_string_lossy().to_string(),
        resources_dir,
    };
    Ok(ApiResponse::success(app_info))
}

/// Simple greeting command for testing.
///
/// # Parameters
/// - `name` - Name to include in greeting
///
/// # Returns
/// Greeting message string.
#[tauri::command]
pub async fn greet(name: String) -> String {
    format!("Hello, {}! Welcome to Mimir.", name)
}

/// Get the default directory for campaign storage.
///
/// Returns the user's Documents folder with "Mimir Campaigns" appended.
///
/// # Returns
/// `ApiResponse` containing the default campaigns directory path.
///
/// # Errors
/// Returns error response if user directories cannot be determined.
#[tauri::command]
pub async fn get_default_campaigns_directory() -> Result<ApiResponse<String>, String> {
    use directories::UserDirs;

    match UserDirs::new() {
        Some(user_dirs) => {
            let documents_dir = user_dirs
                .document_dir()
                .unwrap_or_else(|| user_dirs.home_dir())
                .join(crate::app_init::AppPaths::campaigns_folder_name());

            Ok(ApiResponse::success(
                documents_dir.to_string_lossy().to_string(),
            ))
        }
        None => Ok(ApiResponse::error(
            "Could not determine user directories".to_string(),
        )),
    }
}
