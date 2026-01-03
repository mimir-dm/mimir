//! Module NPC management commands.
//!
//! Provides Tauri commands for managing NPC associations within campaign modules.
//! NPCs are characters with is_npc=true, linked by character_id.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use mimir_dm_core::{
    models::campaign::module_npcs::{ModuleNpc, ModuleNpcWithCharacter, RoleGroup},
    services::ModuleNpcService,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddNpcRequest {
    pub module_id: i32,
    pub character_id: i32,
    pub role: Option<String>,
    pub encounter_tag: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddNpcByNameRequest {
    pub module_id: i32,
    pub campaign_id: i32,
    pub character_name: String,
    pub role: Option<String>,
    pub encounter_tag: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNpcRequest {
    pub role: Option<Option<String>>,
    pub encounter_tag: Option<Option<String>>,
    pub notes: Option<Option<String>>,
}

/// Add an NPC to a module by character ID.
#[tauri::command]
pub async fn add_module_npc(
    request: AddNpcRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<ModuleNpc>, ApiError> {
    info!(
        "Adding NPC (character_id {}) to module {}",
        request.character_id, request.module_id
    );

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleNpcService::new(&mut conn);

    match service.add_npc(
        request.module_id,
        request.character_id,
        request.role,
        request.encounter_tag,
        request.notes,
    ) {
        Ok(npc) => {
            info!("NPC added successfully with ID: {}", npc.id);
            Ok(ApiResponse::success(npc))
        }
        Err(e) => {
            error!("Failed to add NPC: {}", e);
            Ok(ApiResponse::error(format!("Failed to add NPC: {}", e)))
        }
    }
}

/// Add an NPC to a module by character name.
///
/// Looks up the character by name in the campaign and links it.
#[tauri::command]
pub async fn add_module_npc_by_name(
    request: AddNpcByNameRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<ModuleNpc>, ApiError> {
    info!(
        "Adding NPC '{}' to module {} in campaign {}",
        request.character_name, request.module_id, request.campaign_id
    );

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleNpcService::new(&mut conn);

    match service.add_npc_by_name(
        request.module_id,
        request.campaign_id,
        &request.character_name,
        request.role,
        request.encounter_tag,
        request.notes,
    ) {
        Ok(npc) => {
            info!("NPC added successfully with ID: {}", npc.id);
            Ok(ApiResponse::success(npc))
        }
        Err(e) => {
            error!("Failed to add NPC by name: {}", e);
            Ok(ApiResponse::error(format!("Failed to add NPC: {}", e)))
        }
    }
}

/// Remove an NPC from a module.
#[tauri::command]
pub async fn remove_module_npc(
    npc_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!("Removing module NPC with ID: {}", npc_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleNpcService::new(&mut conn);

    match service.remove_npc(npc_id) {
        Ok(()) => {
            info!("NPC removed successfully");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to remove NPC: {}", e);
            Ok(ApiResponse::error(format!("Failed to remove NPC: {}", e)))
        }
    }
}

/// Update a module NPC entry.
#[tauri::command]
pub async fn update_module_npc(
    npc_id: i32,
    request: UpdateNpcRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<ModuleNpc>, ApiError> {
    info!("Updating module NPC with ID: {}", npc_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleNpcService::new(&mut conn);

    match service.update_npc(npc_id, request.role, request.encounter_tag, request.notes) {
        Ok(npc) => {
            info!("NPC updated successfully");
            Ok(ApiResponse::success(npc))
        }
        Err(e) => {
            error!("Failed to update NPC: {}", e);
            Ok(ApiResponse::error(format!("Failed to update NPC: {}", e)))
        }
    }
}

/// List all NPCs for a module.
#[tauri::command]
pub async fn list_module_npcs(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ModuleNpc>>, ApiError> {
    info!("Listing NPCs for module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleNpcService::new(&mut conn);

    match service.get_npcs_for_module(module_id) {
        Ok(npcs) => {
            info!("Found {} NPCs", npcs.len());
            Ok(ApiResponse::success(npcs))
        }
        Err(e) => {
            error!("Failed to list NPCs: {}", e);
            Ok(ApiResponse::error(format!("Failed to list NPCs: {}", e)))
        }
    }
}

/// List NPCs for a module with character data.
#[tauri::command]
pub async fn list_module_npcs_with_data(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ModuleNpcWithCharacter>>, ApiError> {
    info!("Listing NPCs with character data for module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleNpcService::new(&mut conn);

    match service.get_npcs_with_character_data(module_id) {
        Ok(npcs) => {
            info!("Found {} NPCs with data", npcs.len());
            Ok(ApiResponse::success(npcs))
        }
        Err(e) => {
            error!("Failed to list NPCs with data: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list NPCs with data: {}",
                e
            )))
        }
    }
}

/// List NPCs grouped by role.
#[tauri::command]
pub async fn list_module_npcs_by_role(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<RoleGroup>>, ApiError> {
    info!("Listing NPCs grouped by role for module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleNpcService::new(&mut conn);

    match service.get_npcs_grouped_by_role(module_id) {
        Ok(groups) => {
            info!("Found {} role groups", groups.len());
            Ok(ApiResponse::success(groups))
        }
        Err(e) => {
            error!("Failed to list NPCs by role: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list NPCs by role: {}",
                e
            )))
        }
    }
}

/// Get NPC roles for a module.
#[tauri::command]
pub async fn get_module_npc_roles(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Option<String>>>, ApiError> {
    info!("Getting NPC roles for module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleNpcService::new(&mut conn);

    match service.get_roles(module_id) {
        Ok(roles) => {
            info!("Found {} distinct roles", roles.len());
            Ok(ApiResponse::success(roles))
        }
        Err(e) => {
            error!("Failed to get NPC roles: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to get NPC roles: {}",
                e
            )))
        }
    }
}

/// Clear all NPCs from a module.
#[tauri::command]
pub async fn clear_module_npcs(
    module_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<usize>, ApiError> {
    info!("Clearing all NPCs from module: {}", module_id);

    let mut conn = state.db.get_connection()?;
    let mut service = ModuleNpcService::new(&mut conn);

    match service.clear_module_npcs(module_id) {
        Ok(count) => {
            info!("Cleared {} NPCs", count);
            Ok(ApiResponse::success(count))
        }
        Err(e) => {
            error!("Failed to clear NPCs: {}", e);
            Ok(ApiResponse::error(format!("Failed to clear NPCs: {}", e)))
        }
    }
}
