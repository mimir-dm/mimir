//! DM Map Window Commands
//!
//! Tauri commands for managing the DM map window.
//! This is a separate window from the main app that shows the battle map
//! while the main window stays on the module dashboard.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const DM_MAP_LABEL: &str = "dm-map";

/// Check if the DM map window is currently open.
#[tauri::command]
pub fn is_dm_map_open(app: AppHandle) -> bool {
    app.get_webview_window(DM_MAP_LABEL).is_some()
}

/// Open the DM map window for a specific module.
///
/// The window loads with query parameters for moduleId and campaignId
/// so it can fetch the module's maps.
#[tauri::command]
pub fn open_dm_map_window(
    app: AppHandle,
    module_id: String,
    campaign_id: String,
) -> Result<(), String> {
    // Check if window already exists
    if let Some(window) = app.get_webview_window(DM_MAP_LABEL) {
        // Focus existing window
        window.set_focus().ok();
        return Ok(());
    }

    // Build URL with query parameters
    let url = format!("/dm-map?moduleId={}&campaignId={}", module_id, campaign_id);

    // Create the window
    WebviewWindowBuilder::new(&app, DM_MAP_LABEL, WebviewUrl::App(url.into()))
        .title("DM Map")
        .inner_size(1280.0, 900.0)
        .resizable(true)
        .visible(true)
        .build()
        .map_err(|e| format!("Failed to create DM map window: {}", e))?;

    Ok(())
}

/// Close the DM map window.
#[tauri::command]
pub fn close_dm_map_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(DM_MAP_LABEL) {
        window
            .close()
            .map_err(|e| format!("Failed to close DM map window: {}", e))?;
    }
    Ok(())
}

/// Toggle fullscreen mode on the DM map window.
/// Returns the new fullscreen state.
#[tauri::command]
pub fn toggle_dm_map_fullscreen(app: AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window(DM_MAP_LABEL)
        .ok_or_else(|| "DM map window not open".to_string())?;

    let is_fullscreen = window
        .is_fullscreen()
        .map_err(|e| format!("Failed to check fullscreen state: {}", e))?;

    window
        .set_fullscreen(!is_fullscreen)
        .map_err(|e| format!("Failed to toggle fullscreen: {}", e))?;

    Ok(!is_fullscreen)
}
