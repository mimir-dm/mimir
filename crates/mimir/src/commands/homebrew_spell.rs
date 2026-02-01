//! Homebrew Spell Commands
//!
//! Tauri commands for managing campaign homebrew spells.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{CampaignHomebrewSpell, NewCampaignHomebrewSpell, UpdateCampaignHomebrewSpell};
use serde::Deserialize;
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Input for creating a homebrew spell.
#[derive(Debug, Deserialize)]
pub struct CreateHomebrewSpellInput {
    pub campaign_id: String,
    pub name: String,
    pub level: Option<i32>,
    pub school: Option<String>,
    pub data: String,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew spell.
#[derive(Debug, Deserialize)]
pub struct UpdateHomebrewSpellInput {
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

    to_api_response(dal::list_campaign_homebrew_spells(&mut db, &campaign_id))
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

    to_api_response(dal::get_campaign_homebrew_spell(&mut db, &id))
}

/// Create a new homebrew spell.
#[tauri::command]
pub fn create_homebrew_spell(
    state: State<'_, AppState>,
    input: CreateHomebrewSpellInput,
) -> ApiResponse<CampaignHomebrewSpell> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = uuid::Uuid::new_v4().to_string();
    let mut new_spell = NewCampaignHomebrewSpell::new(&id, &input.campaign_id, &input.name, &input.data);
    if let Some(level) = input.level {
        new_spell = new_spell.with_level(level);
    }
    new_spell.school = input.school.as_deref();
    new_spell.cloned_from_name = input.cloned_from_name.as_deref();
    new_spell.cloned_from_source = input.cloned_from_source.as_deref();

    match dal::insert_campaign_homebrew_spell(&mut db, &new_spell) {
        Ok(_) => match dal::get_campaign_homebrew_spell(&mut db, &id) {
            Ok(spell) => ApiResponse::ok(spell),
            Err(e) => ApiResponse::err(e.to_string()),
        },
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Update a homebrew spell.
#[tauri::command]
pub fn update_homebrew_spell(
    state: State<'_, AppState>,
    id: String,
    input: UpdateHomebrewSpellInput,
) -> ApiResponse<CampaignHomebrewSpell> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let name_ref = input.name.as_deref();
    let data_ref = input.data.as_deref();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let update = UpdateCampaignHomebrewSpell {
        name: name_ref,
        level: input.level,
        school: input.school.as_ref().map(|o| o.as_deref()),
        data: data_ref,
        updated_at: Some(&now),
    };

    match dal::update_campaign_homebrew_spell(&mut db, &id, &update) {
        Ok(_) => match dal::get_campaign_homebrew_spell(&mut db, &id) {
            Ok(spell) => ApiResponse::ok(spell),
            Err(e) => ApiResponse::err(e.to_string()),
        },
        Err(e) => ApiResponse::err(e.to_string()),
    }
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

    to_api_response(dal::delete_campaign_homebrew_spell(&mut db, &id).map(|n| n > 0))
}
