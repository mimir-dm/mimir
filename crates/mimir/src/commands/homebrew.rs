//! Homebrew Item Commands
//!
//! Tauri commands for managing campaign homebrew items.

use mimir_core::models::campaign::CampaignHomebrewItem;
use mimir_core::services::{CreateHomebrewItemInput, HomebrewService, UpdateHomebrewItemInput};
use serde::Deserialize;
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

/// Input for creating a homebrew item (Tauri-facing, Deserialize).
#[derive(Debug, Deserialize)]
pub struct TauriCreateHomebrewItemInput {
    pub campaign_id: String,
    pub name: String,
    pub item_type: Option<String>,
    pub rarity: Option<String>,
    pub data: String,
    pub cloned_from_name: Option<String>,
    pub cloned_from_source: Option<String>,
}

/// Input for updating a homebrew item (Tauri-facing, Deserialize).
#[derive(Debug, Deserialize)]
pub struct TauriUpdateHomebrewItemInput {
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(HomebrewService::new(&mut db).list_items(&campaign_id))
}

/// Get a homebrew item by ID.
#[tauri::command]
pub fn get_homebrew_item(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<CampaignHomebrewItem> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(HomebrewService::new(&mut db).get_item(&id))
}

/// Get a homebrew item by campaign_id and name.
#[tauri::command]
pub fn get_homebrew_item_by_name(
    state: State<'_, AppState>,
    campaign_id: String,
    name: String,
) -> ApiResponse<Option<CampaignHomebrewItem>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(HomebrewService::new(&mut db).get_item_by_name(&campaign_id, &name))
}

/// Create a new homebrew item.
#[tauri::command]
pub fn create_homebrew_item(
    state: State<'_, AppState>,
    input: TauriCreateHomebrewItemInput,
) -> ApiResponse<CampaignHomebrewItem> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let svc_input = CreateHomebrewItemInput {
        campaign_id: input.campaign_id,
        name: input.name,
        data: input.data,
        item_type: input.item_type,
        rarity: input.rarity,
        cloned_from_name: input.cloned_from_name,
        cloned_from_source: input.cloned_from_source,
    };

    to_api_response(HomebrewService::new(&mut db).create_item(svc_input))
}

/// Update a homebrew item.
#[tauri::command]
pub fn update_homebrew_item(
    state: State<'_, AppState>,
    id: String,
    input: TauriUpdateHomebrewItemInput,
) -> ApiResponse<CampaignHomebrewItem> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let svc_input = UpdateHomebrewItemInput {
        name: input.name,
        data: input.data,
        item_type: input.item_type,
        rarity: input.rarity,
    };

    to_api_response(HomebrewService::new(&mut db).update_item(&id, svc_input))
}

/// Delete a homebrew item.
#[tauri::command]
pub fn delete_homebrew_item(
    state: State<'_, AppState>,
    id: String,
) -> ApiResponse<bool> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    to_api_response(HomebrewService::new(&mut db).delete_item(&id).map(|_| true))
}
