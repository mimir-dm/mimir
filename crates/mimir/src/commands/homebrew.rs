//! Homebrew Item Commands
//!
//! Tauri commands for managing campaign homebrew items.

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{CampaignHomebrewItem, NewCampaignHomebrewItem, UpdateCampaignHomebrewItem};
use serde::Deserialize;
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Input for creating a homebrew item.
#[derive(Debug, Deserialize)]
pub struct CreateHomebrewItemInput {
    pub campaign_id: String,
    pub name: String,
    pub item_type: Option<String>,
    pub rarity: Option<String>,
    pub data: String,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew item.
#[derive(Debug, Deserialize)]
pub struct UpdateHomebrewItemInput {
    pub name: Option<String>,
    pub item_type: Option<Option<String>>,
    pub rarity: Option<Option<String>>,
    pub data: Option<String>,
}

/// List all homebrew items for a campaign.
#[tauri::command]
pub fn list_homebrew_items(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<CampaignHomebrewItem>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(dal::list_campaign_homebrew_items(&mut db, &campaign_id))
}

/// Get a homebrew item by ID.
#[tauri::command]
pub fn get_homebrew_item(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<CampaignHomebrewItem> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(dal::get_campaign_homebrew_item(&mut db, &id))
}

/// Get a homebrew item by campaign_id and name.
#[tauri::command]
pub fn get_homebrew_item_by_name(
    state: State<'_, AppState>,
    campaign_id: String,
    name: String,
) -> ApiResponse<Option<CampaignHomebrewItem>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(dal::get_campaign_homebrew_item_by_name(&mut db, &campaign_id, &name))
}

/// Create a new homebrew item.
#[tauri::command]
pub fn create_homebrew_item(
    state: State<'_, AppState>,
    input: CreateHomebrewItemInput,
) -> ApiResponse<CampaignHomebrewItem> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = NewCampaignHomebrewItem::new(&id, &input.campaign_id, &input.name, &input.data);
    new_item.item_type = input.item_type.as_deref();
    new_item.rarity = input.rarity.as_deref();
    new_item.cloned_from_name = input.cloned_from_name.as_deref();
    new_item.cloned_from_source = input.cloned_from_source.as_deref();

    match dal::insert_campaign_homebrew_item(&mut db, &new_item) {
        Ok(_) => match dal::get_campaign_homebrew_item(&mut db, &id) {
            Ok(item) => ApiResponse::ok(item),
            Err(e) => ApiResponse::err(e.to_string()),
        },
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Update a homebrew item.
#[tauri::command]
pub fn update_homebrew_item(
    state: State<'_, AppState>,
    id: String,
    input: UpdateHomebrewItemInput,
) -> ApiResponse<CampaignHomebrewItem> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let name_ref = input.name.as_deref();
    let data_ref = input.data.as_deref();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let update = UpdateCampaignHomebrewItem {
        name: name_ref,
        item_type: input.item_type.as_ref().map(|o| o.as_deref()),
        rarity: input.rarity.as_ref().map(|o| o.as_deref()),
        data: data_ref,
        updated_at: Some(&now),
    };

    match dal::update_campaign_homebrew_item(&mut db, &id, &update) {
        Ok(_) => match dal::get_campaign_homebrew_item(&mut db, &id) {
            Ok(item) => ApiResponse::ok(item),
            Err(e) => ApiResponse::err(e.to_string()),
        },
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a homebrew item.
#[tauri::command]
pub fn delete_homebrew_item(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<bool> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    to_api_response(dal::delete_campaign_homebrew_item(&mut db, &id).map(|n| n > 0))
}
