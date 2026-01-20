//! Display control commands for Visual Display System.
//!
//! Provides Tauri commands for controlling the player display window from
//! the main DM interface. Uses Tauri events for inter-window communication.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tracing::info;

/// Payload for map update events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapUpdatePayload {
    pub map_id: i32,
    pub grid_type: String,
    pub grid_size_px: Option<i32>,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub ambient_light: Option<String>,
    pub map_width: Option<i32>,
    pub map_height: Option<i32>,
}

/// Payload for viewport update events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportUpdatePayload {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

/// Payload for blackout toggle events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlackoutPayload {
    pub is_blackout: bool,
}

/// Send a map to the player display.
///
/// Emits an event to the player display window to load and display a map.
///
/// # Parameters
/// - `app` - Tauri application handle
/// - `map_id` - Database ID of the map to display
/// - `grid_type` - Grid type ("square", "hex", or "none")
/// - `grid_size_px` - Grid cell size in pixels (None for no grid)
/// - `grid_offset_x` - Grid X offset for alignment
/// - `grid_offset_y` - Grid Y offset for alignment
/// - `ambient_light` - Ambient light level ("bright", "dim", or "darkness")
/// - `map_width` - Map width in pixels
/// - `map_height` - Map height in pixels
///
/// # Errors
/// Returns an error string if the player display window doesn't exist.
#[tauri::command]
pub async fn send_map_to_display(
    app: AppHandle,
    map_id: i32,
    grid_type: String,
    grid_size_px: Option<i32>,
    grid_offset_x: i32,
    grid_offset_y: i32,
    ambient_light: Option<String>,
    map_width: Option<i32>,
    map_height: Option<i32>,
) -> Result<(), String> {
    info!(
        "Sending map {} to player display (grid: {}, size: {:?}, ambient: {:?})",
        map_id, grid_type, grid_size_px, ambient_light
    );

    // Check if player display window exists
    if app.get_webview_window("player-display").is_none() {
        return Err("Player display window is not open".to_string());
    }

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

    app.emit_to("player-display", "player-display:map-update", payload)
        .map_err(|e| format!("Failed to send map update: {}", e))?;

    info!("Map update sent to player display");
    Ok(())
}

/// Update the viewport on the player display.
///
/// Emits an event to update pan/zoom on the player display.
///
/// # Parameters
/// - `app` - Tauri application handle
/// - `x` - Viewport X translation
/// - `y` - Viewport Y translation
/// - `zoom` - Zoom level (1.0 = 100%)
///
/// # Errors
/// Returns an error string if the player display window doesn't exist.
#[tauri::command]
pub async fn update_display_viewport(
    app: AppHandle,
    x: f64,
    y: f64,
    zoom: f64,
) -> Result<(), String> {
    info!("Updating display viewport: x={}, y={}, zoom={}", x, y, zoom);

    // Check if player display window exists
    if app.get_webview_window("player-display").is_none() {
        return Err("Player display window is not open".to_string());
    }

    let payload = ViewportUpdatePayload { x, y, zoom };

    app.emit_to("player-display", "player-display:viewport-update", payload)
        .map_err(|e| format!("Failed to send viewport update: {}", e))?;

    Ok(())
}

/// Toggle blackout mode on the player display.
///
/// Emits an event to toggle the blackout overlay on the player display.
/// When in blackout mode, the display shows a black screen.
///
/// # Parameters
/// - `app` - Tauri application handle
/// - `is_blackout` - Whether to enable blackout mode
///
/// # Errors
/// Returns an error string if the player display window doesn't exist.
#[tauri::command]
pub async fn toggle_display_blackout(
    app: AppHandle,
    is_blackout: bool,
) -> Result<(), String> {
    info!("Setting display blackout: {}", is_blackout);

    // Check if player display window exists
    if app.get_webview_window("player-display").is_none() {
        return Err("Player display window is not open".to_string());
    }

    let payload = BlackoutPayload { is_blackout };

    app.emit_to("player-display", "player-display:blackout", payload)
        .map_err(|e| format!("Failed to send blackout toggle: {}", e))?;

    Ok(())
}

/// Check if the player display window is open.
///
/// # Parameters
/// - `app` - Tauri application handle
///
/// # Returns
/// `true` if the player display window exists, `false` otherwise.
#[tauri::command]
pub async fn is_player_display_open(app: AppHandle) -> bool {
    app.get_webview_window("player-display").is_some()
}
