//! Module monster management commands.
//!
//! Provides Tauri commands for managing monster associations within campaign modules.
//! Supports adding monsters to modules, grouping by encounter, and retrieving monster data.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use mimir_dm_core::{
    models::campaign::module_monsters::{EncounterGroup, ModuleMonster, ModuleMonsterWithData},
    services::ModuleMonsterService,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMonsterRequest {
    pub module_id: i32,
    pub monster_name: String,
    pub monster_source: String,
    pub quantity: i32,
    pub encounter_tag: Option<String>,
    /// Custom display name (e.g., "Frost Wight" when using goblin stats)
    pub display_name: Option<String>,
    /// DM notes about customizations or thematic changes
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMonsterRequest {
    pub quantity: Option<i32>,
    pub encounter_tag: Option<Option<String>>,
    /// Custom display name (e.g., "Frost Wight" when using goblin stats)
    pub display_name: Option<Option<String>>,
    /// DM notes about customizations or thematic changes
    pub notes: Option<Option<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMonstersRequest {
    pub module_id: i32,
}

/// Add a monster to a module.
///
/// Associates a monster from the catalog with a module, optionally tagged to an encounter.
/// If the same monster already exists with the same encounter tag, quantities are combined.
///
/// # Parameters
/// - `request` - Monster details including module ID, name, source, quantity, and optional tag
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing the created or updated `ModuleMonster`.
#[tauri::command]
pub async fn add_module_monster(
    request: AddMonsterRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<ModuleMonster>, ApiError> {
    info!(
        "Adding monster {} ({}) to module {} with quantity {}",
        request.monster_name, request.monster_source, request.module_id, request.quantity
    );

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleMonsterService::new(&mut conn);

    match service.add_monster(
        request.module_id,
        request.monster_name,
        request.monster_source,
        request.quantity,
        request.encounter_tag,
        request.display_name,
        request.notes,
    ) {
        Ok(monster) => {
            info!("Monster added successfully with ID: {}", monster.id);
            Ok(ApiResponse::success(monster))
        }
        Err(e) => {
            error!("Failed to add monster: {}", e);
            Ok(ApiResponse::error(format!("Failed to add monster: {}", e)))
        }
    }
}

/// Remove a monster from a module.
///
/// Deletes a monster entry from a module by its ID.
///
/// # Parameters
/// - `monster_id` - The database ID of the module monster entry
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` with success or error status.
#[tauri::command]
pub async fn remove_module_monster(
    monster_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!("Removing module monster with ID: {}", monster_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleMonsterService::new(&mut conn);

    match service.remove_monster(monster_id) {
        Ok(()) => {
            info!("Monster removed successfully");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to remove monster: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to remove monster: {}",
                e
            )))
        }
    }
}

/// Update a module monster entry.
///
/// Modifies the quantity or encounter tag of an existing monster entry.
///
/// # Parameters
/// - `monster_id` - The database ID of the module monster entry
/// - `request` - Fields to update (quantity and/or encounter tag)
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing the updated `ModuleMonster`.
#[tauri::command]
pub async fn update_module_monster(
    monster_id: i32,
    request: UpdateMonsterRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<ModuleMonster>, ApiError> {
    info!("Updating module monster with ID: {}", monster_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleMonsterService::new(&mut conn);

    match service.update_monster(monster_id, request.quantity, request.encounter_tag, request.display_name, request.notes) {
        Ok(monster) => {
            info!("Monster updated successfully");
            Ok(ApiResponse::success(monster))
        }
        Err(e) => {
            error!("Failed to update monster: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to update monster: {}",
                e
            )))
        }
    }
}

/// List all monsters for a module.
///
/// Returns basic monster entries without full catalog data.
///
/// # Parameters
/// - `request` - Request containing the module ID
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing a vector of `ModuleMonster` objects.
#[tauri::command]
pub async fn list_module_monsters(
    request: ListMonstersRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ModuleMonster>>, ApiError> {
    info!("Listing monsters for module: {}", request.module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleMonsterService::new(&mut conn);

    match service.get_monsters_for_module(request.module_id) {
        Ok(monsters) => {
            info!("Found {} monsters", monsters.len());
            Ok(ApiResponse::success(monsters))
        }
        Err(e) => {
            error!("Failed to list monsters: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list monsters: {}",
                e
            )))
        }
    }
}

/// List monsters for a module with full catalog data.
///
/// Returns monster entries enriched with full monster JSON from the catalog.
///
/// # Parameters
/// - `module_id` - The database ID of the module
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing a vector of `ModuleMonsterWithData` objects.
#[tauri::command]
pub async fn list_module_monsters_with_data(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ModuleMonsterWithData>>, ApiError> {
    info!("Listing monsters with data for module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleMonsterService::new(&mut conn);

    match service.get_monsters_with_data(module_id) {
        Ok(monsters) => {
            info!("Found {} monsters with data", monsters.len());
            Ok(ApiResponse::success(monsters))
        }
        Err(e) => {
            error!("Failed to list monsters with data: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list monsters with data: {}",
                e
            )))
        }
    }
}

/// List monsters grouped by encounter tag.
///
/// Returns monsters organized into encounter groups for the play mode view.
///
/// # Parameters
/// - `module_id` - The database ID of the module
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing a vector of `EncounterGroup` objects.
#[tauri::command]
pub async fn list_module_monsters_by_encounter(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<EncounterGroup>>, ApiError> {
    info!(
        "Listing monsters grouped by encounter for module: {}",
        module_id
    );

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleMonsterService::new(&mut conn);

    match service.get_monsters_grouped_by_encounter(module_id) {
        Ok(groups) => {
            info!("Found {} encounter groups", groups.len());
            Ok(ApiResponse::success(groups))
        }
        Err(e) => {
            error!("Failed to list monsters by encounter: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list monsters by encounter: {}",
                e
            )))
        }
    }
}

/// Get encounter tags for a module.
///
/// Returns all distinct encounter tags used in the module.
///
/// # Parameters
/// - `module_id` - The database ID of the module
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing a vector of optional tag strings.
#[tauri::command]
pub async fn get_module_encounter_tags(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Option<String>>>, ApiError> {
    info!("Getting encounter tags for module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleMonsterService::new(&mut conn);

    match service.get_encounter_tags(module_id) {
        Ok(tags) => {
            info!("Found {} distinct tags", tags.len());
            Ok(ApiResponse::success(tags))
        }
        Err(e) => {
            error!("Failed to get encounter tags: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to get encounter tags: {}",
                e
            )))
        }
    }
}

/// Clear all monsters from a module.
///
/// Removes all monster entries for a module.
///
/// # Parameters
/// - `module_id` - The database ID of the module
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing the number of deleted entries.
#[tauri::command]
pub async fn clear_module_monsters(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<usize>, ApiError> {
    info!("Clearing all monsters from module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleMonsterService::new(&mut conn);

    match service.clear_module_monsters(module_id) {
        Ok(count) => {
            info!("Cleared {} monsters", count);
            Ok(ApiResponse::success(count))
        }
        Err(e) => {
            error!("Failed to clear monsters: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to clear monsters: {}",
                e
            )))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncMonstersRequest {
    pub module_id: i32,
    pub campaign_directory: String,
    pub module_number: i32,
    pub module_name: String,
}

/// Sync module monsters to a markdown file on disk.
///
/// Creates or updates a `monsters.md` file in the module directory containing
/// full stat blocks for all monsters, grouped by encounter tag.
///
/// # Parameters
/// - `request` - Contains module_id, campaign_directory, module_number, and module_name
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` with success or error status.
#[tauri::command]
pub async fn sync_module_monsters_to_file(
    request: SyncMonstersRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!(
        "Syncing monsters to file for module {} in {}",
        request.module_id, request.campaign_directory
    );

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleMonsterService::new(&mut conn);

    match service.sync_monsters_to_file(
        request.module_id,
        &request.campaign_directory,
        request.module_number,
        &request.module_name,
    ) {
        Ok(()) => {
            info!("Successfully synced monsters to file");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to sync monsters to file: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to sync monsters to file: {}",
                e
            )))
        }
    }
}
