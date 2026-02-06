//! Item Catalog Commands

use mimir_core::models::catalog::{Item, ItemFilter};
use mimir_core::services::{CatalogEntityService, ItemService, DEFAULT_QUERY_LIMIT};
use serde_json::Value;
use tauri::State;

use crate::commands::{entities_to_json, entity_to_json, to_api_response, ApiResponse};
use crate::state::AppState;

/// Search items with optional filters.
#[tauri::command]
pub fn search_items(
    state: State<'_, AppState>,
    filter: Option<ItemFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = ItemService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get an item by database ID.
#[tauri::command]
pub fn get_item(state: State<'_, AppState>, id: i32) -> ApiResponse<Item> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ItemService::new(&mut db).get(id);
    match result {
        Ok(Some(item)) => ApiResponse::ok(item),
        Ok(None) => ApiResponse::err(format!("Item not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get an item by name and source.
#[tauri::command]
pub fn get_item_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
    campaign_id: Option<String>,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Check homebrew items first when source is "HB" and campaign_id is provided
    if source == "HB" {
        if let Some(ref cid) = campaign_id {
            match mimir_core::dal::campaign::get_campaign_homebrew_item_by_name(&mut db, cid, &name) {
                Ok(Some(item)) => {
                    // Return homebrew item data as parsed JSON, similar to catalog items
                    let mut json: Value = serde_json::from_str(&item.data)
                        .unwrap_or(Value::Object(Default::default()));
                    if let Value::Object(ref mut map) = json {
                        map.insert("name".to_string(), Value::String(item.name));
                        map.insert("source".to_string(), Value::String("HB".to_string()));
                        map.insert("homebrew".to_string(), Value::Bool(true));
                        map.insert("homebrew_id".to_string(), Value::String(item.id));
                        if let Some(ref it) = item.item_type {
                            map.insert("type".to_string(), Value::String(it.clone()));
                        }
                        if let Some(ref r) = item.rarity {
                            map.insert("rarity".to_string(), Value::String(r.clone()));
                        }
                    }
                    return ApiResponse::ok(json);
                }
                Ok(None) => {}
                Err(e) => return ApiResponse::err(e.to_string()),
            }
        }
    }

    // Fall back to catalog lookup
    let result = ItemService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(item)) => ApiResponse::ok(entity_to_json(&item)),
        Ok(None) => ApiResponse::err(format!("Item not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all item sources.
#[tauri::command]
pub fn list_item_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ItemService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total items.
#[tauri::command]
pub fn count_items(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ItemService::new(&mut db).count();
    to_api_response(result)
}
