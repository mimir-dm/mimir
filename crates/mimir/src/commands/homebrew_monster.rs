//! Homebrew Monster Commands
//!
//! Tauri commands for managing campaign homebrew monsters.

use mimir_core::models::campaign::CampaignHomebrewMonster;
use mimir_core::services::{CreateHomebrewMonsterInput, HomebrewService, UpdateHomebrewMonsterInput};
use serde::Deserialize;
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Input for creating a homebrew monster (Tauri-facing, Deserialize).
#[derive(Debug, Deserialize)]
pub struct TauriCreateHomebrewMonsterInput {
    pub campaign_id: String,
    pub name: String,
    pub cr: Option<String>,
    pub creature_type: Option<String>,
    pub size: Option<String>,
    pub data: String,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew monster (Tauri-facing, Deserialize).
#[derive(Debug, Deserialize)]
pub struct TauriUpdateHomebrewMonsterInput {
    pub name: Option<String>,
    pub cr: Option<Option<String>>,
    pub creature_type: Option<Option<String>>,
    pub size: Option<Option<String>>,
    pub data: Option<String>,
}

/// List all homebrew monsters for a campaign.
#[tauri::command]
pub fn list_homebrew_monsters(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<CampaignHomebrewMonster>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(HomebrewService::new(&mut db).list_monsters(&campaign_id))
}

/// Get a homebrew monster by ID.
#[tauri::command]
pub fn get_homebrew_monster(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<CampaignHomebrewMonster> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(HomebrewService::new(&mut db).get_monster(&id))
}

/// Create a new homebrew monster.
#[tauri::command]
pub fn create_homebrew_monster(
    state: State<'_, AppState>,
    input: TauriCreateHomebrewMonsterInput,
) -> ApiResponse<CampaignHomebrewMonster> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let svc_input = CreateHomebrewMonsterInput {
        campaign_id: input.campaign_id,
        name: input.name,
        data: input.data,
        cr: input.cr,
        creature_type: input.creature_type,
        size: input.size,
        cloned_from_name: input.cloned_from_name,
        cloned_from_source: input.cloned_from_source,
    };

    to_api_response(HomebrewService::new(&mut db).create_monster(svc_input))
}

/// Update a homebrew monster.
#[tauri::command]
pub fn update_homebrew_monster(
    state: State<'_, AppState>,
    id: String,
    input: TauriUpdateHomebrewMonsterInput,
) -> ApiResponse<CampaignHomebrewMonster> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let svc_input = UpdateHomebrewMonsterInput {
        name: input.name,
        data: input.data,
        cr: input.cr,
        creature_type: input.creature_type,
        size: input.size,
    };

    to_api_response(HomebrewService::new(&mut db).update_monster(&id, svc_input))
}

/// Delete a homebrew monster.
#[tauri::command]
pub fn delete_homebrew_monster(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<bool> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(HomebrewService::new(&mut db).delete_monster(&id).map(|_| true))
}
