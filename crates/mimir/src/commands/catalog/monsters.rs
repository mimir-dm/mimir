//! Monster Catalog Commands

use mimir_core::models::catalog::{Monster, MonsterFilter};
use mimir_core::services::{CatalogEntityService, MonsterService, DEFAULT_QUERY_LIMIT};
use serde_json::Value;
use tauri::State;

use crate::commands::{entities_to_json, entity_to_json, to_api_response, ApiResponse};
use crate::state::AppState;

/// Search monsters with optional filters.
#[tauri::command]
pub fn search_monsters(
    state: State<'_, AppState>,
    filter: Option<MonsterFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    println!("[search_monsters] filter: {:?}, limit: {:?}, offset: {:?}", filter, limit, offset);

    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    println!("[search_monsters] resolved filter: {:?}", filter);

    let result = MonsterService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => {
            println!("[search_monsters] found {} entities", entities.len());
            ApiResponse::ok(entities_to_json(entities))
        },
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a monster by database ID.
#[tauri::command]
pub fn get_monster(state: State<'_, AppState>, id: i32) -> ApiResponse<Monster> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = MonsterService::new(&mut db).get(id);
    match result {
        Ok(Some(monster)) => ApiResponse::ok(monster),
        Ok(None) => ApiResponse::err(format!("Monster not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a monster by name and source.
#[tauri::command]
pub fn get_monster_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = MonsterService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(monster)) => ApiResponse::ok(entity_to_json(&monster)),
        Ok(None) => ApiResponse::err(format!("Monster not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all monster sources.
#[tauri::command]
pub fn list_monster_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = MonsterService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total monsters.
#[tauri::command]
pub fn count_monsters(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = MonsterService::new(&mut db).count();
    to_api_response(result)
}
