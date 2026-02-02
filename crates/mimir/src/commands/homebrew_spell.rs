//! Homebrew Spell Commands
//!
//! Tauri commands for managing campaign homebrew spells.

use mimir_core::models::campaign::CampaignHomebrewSpell;
use mimir_core::services::{CreateHomebrewSpellInput, HomebrewService, UpdateHomebrewSpellInput};
use serde::Deserialize;
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Input for creating a homebrew spell (Tauri-facing, Deserialize).
#[derive(Debug, Deserialize)]
pub struct TauriCreateHomebrewSpellInput {
    pub campaign_id: String,
    pub name: String,
    pub level: Option<i32>,
    pub school: Option<String>,
    pub data: String,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew spell (Tauri-facing, Deserialize).
#[derive(Debug, Deserialize)]
pub struct TauriUpdateHomebrewSpellInput {
    pub name: Option<String>,
    pub level: Option<Option<i32>>,
    pub school: Option<Option<String>>,
    pub data: Option<String>,
}

/// List all homebrew spells for a campaign.
#[tauri::command]
pub fn list_homebrew_spells(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<CampaignHomebrewSpell>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(HomebrewService::new(&mut db).list_spells(&campaign_id))
}

/// Get a homebrew spell by ID.
#[tauri::command]
pub fn get_homebrew_spell(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<CampaignHomebrewSpell> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(HomebrewService::new(&mut db).get_spell(&id))
}

/// Create a new homebrew spell.
#[tauri::command]
pub fn create_homebrew_spell(
    state: State<'_, AppState>,
    input: TauriCreateHomebrewSpellInput,
) -> ApiResponse<CampaignHomebrewSpell> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let svc_input = CreateHomebrewSpellInput {
        campaign_id: input.campaign_id,
        name: input.name,
        data: input.data,
        level: input.level,
        school: input.school,
        cloned_from_name: input.cloned_from_name,
        cloned_from_source: input.cloned_from_source,
    };

    to_api_response(HomebrewService::new(&mut db).create_spell(svc_input))
}

/// Update a homebrew spell.
#[tauri::command]
pub fn update_homebrew_spell(
    state: State<'_, AppState>,
    id: String,
    input: TauriUpdateHomebrewSpellInput,
) -> ApiResponse<CampaignHomebrewSpell> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let svc_input = UpdateHomebrewSpellInput {
        name: input.name,
        data: input.data,
        level: input.level,
        school: input.school,
    };

    to_api_response(HomebrewService::new(&mut db).update_spell(&id, svc_input))
}

/// Delete a homebrew spell.
#[tauri::command]
pub fn delete_homebrew_spell(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<bool> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(HomebrewService::new(&mut db).delete_spell(&id).map(|_| true))
}
