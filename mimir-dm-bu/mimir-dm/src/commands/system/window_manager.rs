//! Window management commands.
//!
//! Provides Tauri commands for opening and managing auxiliary windows
//! such as debug panels, chat windows, and log viewers.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow};
use tracing::info;

/// Open the context debug window.
///
/// Creates or focuses the context debug panel showing current context state.
/// If the window already exists, it will be focused instead of creating a new one.
///
/// # Parameters
/// - `app` - Tauri application handle for window management
///
/// # Errors
/// Returns an error string if window creation or focus fails.
#[tauri::command]
pub async fn open_context_debug_window(app: AppHandle) -> Result<(), String> {
    info!("Opening context debug window");

    // Check if window already exists
    if let Some(window) = app.get_webview_window("context-debug") {
        // Focus existing window
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus context debug window: {}", e))?;
        return Ok(());
    }

    // Create new window
    let _window = WebviewWindow::builder(
        &app,
        "context-debug",
        WebviewUrl::App("context-debug.html".into()),
    )
    .title("Context Debug - Mimir")
    .inner_size(800.0, 600.0)
    .min_inner_size(600.0, 400.0)
    .position(100.0, 100.0)
    .resizable(true)
    .build()
    .map_err(|e| format!("Failed to create context debug window: {}", e))?;

    info!("Context debug window created successfully");
    Ok(())
}

/// Open the chat window.
///
/// Creates or focuses the LLM chat interface window.
/// If the window already exists, it will be focused instead of creating a new one.
///
/// # Parameters
/// - `app` - Tauri application handle for window management
///
/// # Errors
/// Returns an error string if window creation or focus fails.
#[tauri::command]
pub async fn open_chat_window(app: AppHandle) -> Result<(), String> {
    info!("Opening chat window");

    // Check if window already exists
    if let Some(window) = app.get_webview_window("chat") {
        // Focus existing window
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus chat window: {}", e))?;
        return Ok(());
    }

    // Create new window
    let _window = WebviewWindow::builder(&app, "chat", WebviewUrl::App("chat.html".into()))
        .title("Mimir Chat")
        .inner_size(800.0, 700.0)
        .min_inner_size(600.0, 500.0)
        .position(100.0, 100.0)
        .resizable(true)
        .build()
        .map_err(|e| format!("Failed to create chat window: {}", e))?;

    info!("Chat window created successfully");
    Ok(())
}

/// Open a log viewer window for a specific log file.
///
/// Creates or focuses a log viewer window for the specified file.
/// Each log file gets its own window instance.
///
/// # Parameters
/// - `app` - Tauri application handle for window management
/// - `file_name` - Name of the log file to view
///
/// # Errors
/// Returns an error string if window creation or focus fails.
#[tauri::command]
pub async fn open_log_viewer_window(app: AppHandle, file_name: String) -> Result<(), String> {
    info!("Opening log viewer window for file: {}", file_name);

    // Create unique window label for each log file (sanitize filename for window label)
    let sanitized_name = file_name.replace(".", "-").replace(" ", "_");
    let window_label = format!("log-viewer-{}", sanitized_name);

    // Check if window for this file already exists
    if let Some(window) = app.get_webview_window(&window_label) {
        // Focus existing window
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus log viewer window: {}", e))?;
        return Ok(());
    }

    // Create new window with filename as query parameter (URL encode)
    let encoded_filename = file_name.replace(" ", "%20").replace(".", "%2E");
    let url = format!("log-viewer.html?file={}", encoded_filename);
    let _window = WebviewWindow::builder(&app, &window_label, WebviewUrl::App(url.into()))
        .title(format!("Log Viewer - {}", file_name))
        .inner_size(1000.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .position(150.0, 150.0)
        .resizable(true)
        .build()
        .map_err(|e| format!("Failed to create log viewer window: {}", e))?;

    info!(
        "Log viewer window created successfully for file: {}",
        file_name
    );
    Ok(())
}

/// Open the player display window.
///
/// Creates or focuses the player display window for showing maps on a TV/projector.
/// This window is designed to be full-screen for player viewing during in-person sessions.
/// If the window already exists, it will be focused instead of creating a new one.
///
/// # Parameters
/// - `app` - Tauri application handle for window management
///
/// # Errors
/// Returns an error string if window creation or focus fails.
#[tauri::command]
pub async fn open_player_display_window(app: AppHandle) -> Result<(), String> {
    info!("Opening player display window");

    // Check if window already exists
    if let Some(window) = app.get_webview_window("player-display") {
        // Focus existing window
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus player display window: {}", e))?;
        return Ok(());
    }

    // Create new window - designed for TV/projector display
    let _window = WebviewWindow::builder(
        &app,
        "player-display",
        WebviewUrl::App("player-display.html".into()),
    )
    .title("Player Display - Mimir")
    .inner_size(1920.0, 1080.0)  // Default to 1080p, user can resize/fullscreen
    .min_inner_size(800.0, 600.0)
    .position(100.0, 100.0)
    .resizable(true)
    .fullscreen(false)  // User can toggle fullscreen with F11 or menu
    .build()
    .map_err(|e| format!("Failed to create player display window: {}", e))?;

    info!("Player display window created successfully");
    Ok(())
}

/// Close the player display window.
///
/// Closes the player display window if it exists.
///
/// # Parameters
/// - `app` - Tauri application handle for window management
///
/// # Errors
/// Returns an error string if closing fails.
#[tauri::command]
pub async fn close_player_display_window(app: AppHandle) -> Result<(), String> {
    info!("Closing player display window");

    if let Some(window) = app.get_webview_window("player-display") {
        window
            .close()
            .map_err(|e| format!("Failed to close player display window: {}", e))?;
        info!("Player display window closed");
    }

    Ok(())
}

/// Toggle fullscreen mode for the player display window.
///
/// # Parameters
/// - `app` - Tauri application handle for window management
///
/// # Errors
/// Returns an error string if toggle fails or window doesn't exist.
#[tauri::command]
pub async fn toggle_player_display_fullscreen(app: AppHandle) -> Result<bool, String> {
    info!("Toggling player display fullscreen");

    if let Some(window) = app.get_webview_window("player-display") {
        let is_fullscreen = window
            .is_fullscreen()
            .map_err(|e| format!("Failed to check fullscreen state: {}", e))?;

        window
            .set_fullscreen(!is_fullscreen)
            .map_err(|e| format!("Failed to toggle fullscreen: {}", e))?;

        info!("Player display fullscreen: {}", !is_fullscreen);
        Ok(!is_fullscreen)
    } else {
        Err("Player display window not found".to_string())
    }
}
