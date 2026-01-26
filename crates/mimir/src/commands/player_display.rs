//! Player Display Window Commands
//!
//! Tauri commands for managing the player display window.
//! These commands handle window creation, destruction, fullscreen, and IPC events.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

/// Payload for map updates sent to the player display
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapUpdatePayload {
    pub map_id: String,
    pub grid_type: String,
    pub grid_size_px: Option<i32>,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub ambient_light: Option<String>,
    pub map_width: Option<i32>,
    pub map_height: Option<i32>,
}

/// Payload for viewport updates sent to the player display
#[derive(Debug, Clone, Serialize)]
pub struct ViewportPayload {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

/// Payload for blackout state updates
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlackoutPayload {
    pub is_blackout: bool,
}

const PLAYER_DISPLAY_LABEL: &str = "player-display";

/// Check if the player display window is currently open.
#[tauri::command]
pub fn is_player_display_open(app: AppHandle) -> bool {
    app.get_webview_window(PLAYER_DISPLAY_LABEL).is_some()
}

/// Open the player display window.
#[tauri::command]
pub fn open_player_display_window(app: AppHandle) -> Result<(), String> {
    // Check if window already exists
    if app.get_webview_window(PLAYER_DISPLAY_LABEL).is_some() {
        return Ok(());
    }

    // Create the window
    WebviewWindowBuilder::new(&app, PLAYER_DISPLAY_LABEL, WebviewUrl::App("/player-display".into()))
        .title("Player Display")
        .inner_size(1280.0, 720.0)
        .resizable(true)
        .visible(true)
        .build()
        .map_err(|e| format!("Failed to create player display window: {}", e))?;

    Ok(())
}

/// Close the player display window.
#[tauri::command]
pub fn close_player_display_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(PLAYER_DISPLAY_LABEL) {
        window
            .close()
            .map_err(|e| format!("Failed to close player display window: {}", e))?;
    }
    Ok(())
}

/// Toggle fullscreen mode on the player display window.
/// Returns the new fullscreen state.
#[tauri::command]
pub fn toggle_player_display_fullscreen(app: AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window(PLAYER_DISPLAY_LABEL)
        .ok_or_else(|| "Player display window not open".to_string())?;

    let is_fullscreen = window
        .is_fullscreen()
        .map_err(|e| format!("Failed to check fullscreen state: {}", e))?;

    window
        .set_fullscreen(!is_fullscreen)
        .map_err(|e| format!("Failed to toggle fullscreen: {}", e))?;

    Ok(!is_fullscreen)
}

/// Send a map to the player display window.
#[tauri::command]
pub fn send_map_to_display(
    app: AppHandle,
    map_id: String,
    grid_type: String,
    grid_size_px: Option<i32>,
    grid_offset_x: i32,
    grid_offset_y: i32,
    ambient_light: Option<String>,
    map_width: Option<i32>,
    map_height: Option<i32>,
) -> Result<(), String> {
    let window = app
        .get_webview_window(PLAYER_DISPLAY_LABEL)
        .ok_or_else(|| "Player display window not open".to_string())?;

    let payload = MapUpdatePayload {
        map_id,
        grid_type,
        grid_size_px,
        grid_offset_x,
        grid_offset_y,
        ambient_light,
        map_width,
        map_height,
    };

    window
        .emit("player-display:map-update", payload)
        .map_err(|e| format!("Failed to emit map update: {}", e))?;

    Ok(())
}

/// Update the viewport on the player display window (pan/zoom).
#[tauri::command]
pub fn update_display_viewport(app: AppHandle, x: f64, y: f64, zoom: f64) -> Result<(), String> {
    let window = app
        .get_webview_window(PLAYER_DISPLAY_LABEL)
        .ok_or_else(|| "Player display window not open".to_string())?;

    let payload = ViewportPayload { x, y, zoom };

    window
        .emit("player-display:viewport-update", payload)
        .map_err(|e| format!("Failed to emit viewport update: {}", e))?;

    Ok(())
}

/// Toggle or set blackout mode on the player display.
#[tauri::command]
pub fn toggle_display_blackout(app: AppHandle, is_blackout: bool) -> Result<(), String> {
    let window = app
        .get_webview_window(PLAYER_DISPLAY_LABEL)
        .ok_or_else(|| "Player display window not open".to_string())?;

    let payload = BlackoutPayload { is_blackout };

    window
        .emit("player-display:blackout", payload)
        .map_err(|e| format!("Failed to emit blackout update: {}", e))?;

    Ok(())
}
