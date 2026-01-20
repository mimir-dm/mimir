//! Session todo query and configuration commands.
//!
//! Provides Tauri commands for retrieving and configuring session-based
//! todo items managed by the LLM service.

use crate::{state::AppState, types::ApiResponse};
use mimir_dm_llm::TodoItem;
use std::path::PathBuf;
use tauri::State;
use tracing::{debug, info, warn};

/// Get todos for a specific session from the LLM service's ephemeral state.
///
/// Retrieves the list of todo items associated with a chat session.
///
/// # Parameters
/// - `session_id` - Unique identifier of the chat session
///
/// # Returns
/// `ApiResponse` containing a vector of `TodoItem` objects.
///
/// # Notes
/// Returns an empty list if the LLM service is not initialized.
#[tauri::command]
pub async fn get_session_todos(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ApiResponse<Vec<TodoItem>>, String> {
    info!("Getting todos for session: {}", session_id);

    let service = state.llm.lock().await;

    if let Some(llm) = service.as_ref() {
        let todos = llm.get_session_todos(&session_id);
        debug!("Found {} todos for session {}", todos.len(), session_id);
        Ok(ApiResponse::success(todos))
    } else {
        debug!("LLM service not initialized, returning empty todos");
        Ok(ApiResponse::success(Vec::new()))
    }
}

/// Configure where todos should be stored on disk.
///
/// Sets the filesystem path where todo items will be persisted.
///
/// # Parameters
/// - `storage_path` - Filesystem path for todo storage
///
/// # Returns
/// `ApiResponse` indicating success or failure.
///
/// # Errors
/// Returns error response if the LLM service is not initialized
/// or if the storage path configuration fails.
#[tauri::command]
pub async fn configure_todo_storage(
    state: State<'_, AppState>,
    storage_path: String,
) -> Result<ApiResponse<()>, String> {
    info!("Configuring todo storage path: {}", storage_path);

    let service = state.llm.lock().await;

    if let Some(llm) = service.as_ref() {
        let path = PathBuf::from(storage_path);
        match llm.configure_todo_storage(path.clone()) {
            Ok(()) => {
                info!("Todo storage configured successfully to: {:?}", path);
                Ok(ApiResponse::success(()))
            }
            Err(e) => {
                warn!("Failed to configure todo storage: {}", e);
                Ok(ApiResponse::error(format!(
                    "Failed to configure todo storage: {}",
                    e
                )))
            }
        }
    } else {
        warn!("LLM service not initialized, cannot configure todo storage");
        Ok(ApiResponse::error(
            "LLM service not initialized".to_string(),
        ))
    }
}
