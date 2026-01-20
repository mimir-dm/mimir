//! Player management commands.
//!
//! Provides Tauri commands for CRUD operations on player records.
//! Players represent real people who participate in campaigns and own characters.

use crate::state::AppState;
use mimir_dm_core::models::player::Player;
use mimir_dm_core::services::PlayerService;
use tauri::State;
use tracing::error;

/// Create a new player record.
///
/// # Parameters
/// - `name` - Display name for the player
/// - `email` - Optional email address
/// - `notes` - Optional notes about the player
///
/// # Returns
/// The created `Player` record with assigned ID.
///
/// # Errors
/// Returns error string if database operation fails.
#[tauri::command]
pub async fn create_player(
    name: String,
    email: Option<String>,
    notes: Option<String>,
    state: State<'_, AppState>,
) -> Result<Player, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut player_service = PlayerService::new(&mut conn);
    player_service
        .create_player(&name, email, notes)
        .map_err(|e| format!("Failed to create player: {}", e))
}

/// Get a player by database ID.
///
/// # Parameters
/// - `player_id` - Database ID of the player
///
/// # Returns
/// The `Player` record.
///
/// # Errors
/// Returns error string if player not found or database fails.
#[tauri::command]
pub async fn get_player(player_id: i32, state: State<'_, AppState>) -> Result<Player, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut player_service = PlayerService::new(&mut conn);
    player_service
        .get_player(player_id)
        .map_err(|e| format!("Failed to get player: {}", e))
}

/// List all players.
///
/// # Returns
/// Vector of all `Player` records.
///
/// # Errors
/// Returns error string if database operation fails.
#[tauri::command]
pub async fn list_players(state: State<'_, AppState>) -> Result<Vec<Player>, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut player_service = PlayerService::new(&mut conn);
    player_service
        .list_players()
        .map_err(|e| format!("Failed to list players: {}", e))
}

/// Update an existing player record.
///
/// All fields are optional - only provided fields are updated.
/// Use `Some(None)` to explicitly clear a nullable field.
///
/// # Parameters
/// - `player_id` - Database ID of the player to update
/// - `name` - New name (if changing)
/// - `email` - New email (`Some(Some("email"))` to set, `Some(None)` to clear)
/// - `notes` - New notes (`Some(Some("notes"))` to set, `Some(None)` to clear)
///
/// # Returns
/// The updated `Player` record.
///
/// # Errors
/// Returns error string if player not found or database fails.
#[tauri::command]
pub async fn update_player(
    player_id: i32,
    name: Option<String>,
    email: Option<Option<String>>,
    notes: Option<Option<String>>,
    state: State<'_, AppState>,
) -> Result<Player, String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut player_service = PlayerService::new(&mut conn);
    player_service
        .update_player(player_id, name, email, notes)
        .map_err(|e| format!("Failed to update player: {}", e))
}

/// Delete a player record.
///
/// # Parameters
/// - `player_id` - Database ID of the player to delete
///
/// # Errors
/// Returns error string if player not found or has active characters.
#[tauri::command]
pub async fn delete_player(player_id: i32, state: State<'_, AppState>) -> Result<(), String> {
    let mut conn = state.db.get_connection().map_err(|e| {
        error!("Failed to get database connection: {}", e);
        format!("Database connection failed: {}", e)
    })?;

    let mut player_service = PlayerService::new(&mut conn);
    player_service
        .delete_player(player_id)
        .map_err(|e| format!("Failed to delete player: {}", e))
}
