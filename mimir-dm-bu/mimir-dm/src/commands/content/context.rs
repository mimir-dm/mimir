//! Context management commands.
//!
//! Provides Tauri commands for managing shared context between windows
//! and preparing context data for LLM consumption.

use crate::state::AppState;
use tauri::State;
use tracing::debug;

/// Update context data for a specific window and context type.
///
/// Stores context information that can be shared across windows or used by the LLM.
///
/// # Parameters
/// - `window_id` - Unique identifier of the window updating context
/// - `context_type` - Category of context (e.g., "session", "character")
/// - `data` - JSON string containing the context data
///
/// # Errors
/// Returns an error string if the context update fails.
#[tauri::command]
pub async fn update_context(
    window_id: String,
    context_type: String,
    data: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    debug!(
        "Updating context for window {}: type={}",
        window_id, context_type
    );
    state.context.0.update_context(&context_type, &data)
}

/// Get the complete shared context as a JSON string.
///
/// Returns all context data from all registered windows.
///
/// # Returns
/// JSON string containing all shared context data.
///
/// # Errors
/// Returns an error string if context retrieval fails.
#[tauri::command]
pub async fn get_full_context(state: State<'_, AppState>) -> Result<String, String> {
    debug!("Getting full context");
    state.context.0.get_full_context()
}

/// Register a new window in the context system.
///
/// Windows must be registered before they can contribute context data.
///
/// # Parameters
/// - `window_id` - Unique identifier for the window
/// - `window_type` - Type of window (e.g., "main", "session", "character")
/// - `title` - Human-readable window title
///
/// # Errors
/// Returns an error string if registration fails.
#[tauri::command]
pub async fn register_window(
    window_id: String,
    window_type: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    debug!("Registering window: {} ({})", window_id, window_type);
    state.context.0.register_window(&window_id, &window_type, &title)
}

/// Unregister a window from the context system.
///
/// Removes the window and clears its associated context data.
///
/// # Parameters
/// - `window_id` - Unique identifier of the window to unregister
///
/// # Errors
/// Returns an error string if unregistration fails.
#[tauri::command]
pub async fn unregister_window(
    window_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    debug!("Unregistering window: {}", window_id);
    state.context.0.unregister_window(&window_id)
}

/// Clear all shared context data.
///
/// Removes all context from all windows. Windows remain registered.
///
/// # Errors
/// Returns an error string if clearing fails.
#[tauri::command]
pub async fn clear_shared_context(state: State<'_, AppState>) -> Result<(), String> {
    debug!("Clearing shared context");
    state.context.0.clear_context()
}

/// Get context formatted for LLM consumption.
///
/// Returns a formatted string suitable for inclusion in LLM prompts.
/// May include summarization or filtering for relevance.
///
/// # Returns
/// Formatted context string optimized for LLM input.
///
/// # Errors
/// Returns an error string if context retrieval fails.
#[tauri::command]
pub async fn get_context_for_llm(state: State<'_, AppState>) -> Result<String, String> {
    debug!("Getting context for LLM");
    state.context.0.get_context_for_llm()
}

/// Update the context token usage statistics.
///
/// Tracks how many tokens the context is consuming for monitoring.
///
/// # Parameters
/// - `usage` - Number of tokens used by current context
///
/// # Errors
/// Returns an error string if update fails.
#[tauri::command]
pub async fn update_context_usage(
    usage: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    debug!("Updating context usage: {} tokens", usage);
    state.context.0.update_context_usage(usage)
}
