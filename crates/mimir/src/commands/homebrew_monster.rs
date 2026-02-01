//! Homebrew Monster Commands
//!
//! Tauri commands for managing campaign homebrew monsters.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{CampaignHomebrewMonster, NewCampaignHomebrewMonster, UpdateCampaignHomebrewMonster};
use serde::Deserialize;
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Input for creating a homebrew monster.
#[derive(Debug, Deserialize)]
pub struct CreateHomebrewMonsterInput {
    pub campaign_id: String,
    pub name: String,
    pub cr: Option<String>,
    pub creature_type: Option<String>,
    pub size: Option<String>,
    pub data: String,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew monster.
#[derive(Debug, Deserialize)]
pub struct UpdateHomebrewMonsterInput {
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(dal::list_campaign_homebrew_monsters(&mut db, &campaign_id))
}

/// Get a homebrew monster by ID.
#[tauri::command]
pub fn get_homebrew_monster(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<CampaignHomebrewMonster> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(dal::get_campaign_homebrew_monster(&mut db, &id))
}

/// Create a new homebrew monster.
#[tauri::command]
pub fn create_homebrew_monster(
    state: State<'_, AppState>,
    input: CreateHomebrewMonsterInput,
) -> ApiResponse<CampaignHomebrewMonster> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = uuid::Uuid::new_v4().to_string();
    let mut new_monster = NewCampaignHomebrewMonster::new(&id, &input.campaign_id, &input.name, &input.data);
    new_monster.cr = input.cr.as_deref();
    new_monster.creature_type = input.creature_type.as_deref();
    new_monster.size = input.size.as_deref();
    new_monster.cloned_from_name = input.cloned_from_name.as_deref();
    new_monster.cloned_from_source = input.cloned_from_source.as_deref();

    match dal::insert_campaign_homebrew_monster(&mut db, &new_monster) {
        Ok(_) => match dal::get_campaign_homebrew_monster(&mut db, &id) {
            Ok(monster) => ApiResponse::ok(monster),
            Err(e) => ApiResponse::err(e.to_string()),
        },
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Update a homebrew monster.
#[tauri::command]
pub fn update_homebrew_monster(
    state: State<'_, AppState>,
    id: String,
    input: UpdateHomebrewMonsterInput,
) -> ApiResponse<CampaignHomebrewMonster> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let name_ref = input.name.as_deref();
    let data_ref = input.data.as_deref();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let update = UpdateCampaignHomebrewMonster {
        name: name_ref,
        cr: input.cr.as_ref().map(|o| o.as_deref()),
        creature_type: input.creature_type.as_ref().map(|o| o.as_deref()),
        size: input.size.as_ref().map(|o| o.as_deref()),
        data: data_ref,
        updated_at: Some(&now),
    };

    match dal::update_campaign_homebrew_monster(&mut db, &id, &update) {
        Ok(_) => match dal::get_campaign_homebrew_monster(&mut db, &id) {
            Ok(monster) => ApiResponse::ok(monster),
            Err(e) => ApiResponse::err(e.to_string()),
        },
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a homebrew monster.
#[tauri::command]
pub fn delete_homebrew_monster(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<bool> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(dal::delete_campaign_homebrew_monster(&mut db, &id).map(|n| n > 0))
}
