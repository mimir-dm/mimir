//! Spell Catalog Commands

use mimir_core::models::catalog::{Spell, SpellFilter};
use mimir_core::services::{CatalogEntityService, SpellService, DEFAULT_QUERY_LIMIT};
use serde_json::Value;
use tauri::State;

use crate::commands::{entities_to_json, entity_to_json, to_api_response, ApiResponse};
use crate::state::AppState;

/// Search spells with optional filters.
#[tauri::command]
pub fn search_spells(
    state: State<'_, AppState>,
    filter: Option<SpellFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = SpellService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a spell by database ID.
#[tauri::command]
pub fn get_spell(state: State<'_, AppState>, id: i32) -> ApiResponse<Spell> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = SpellService::new(&mut db).get(id);
    match result {
        Ok(Some(spell)) => ApiResponse::ok(spell),
        Ok(None) => ApiResponse::err(format!("Spell not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a spell by name and source.
#[tauri::command]
pub fn get_spell_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = SpellService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(spell)) => ApiResponse::ok(entity_to_json(&spell)),
        Ok(None) => ApiResponse::err(format!("Spell not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all spell sources.
#[tauri::command]
pub fn list_spell_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = SpellService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total spells.
#[tauri::command]
pub fn count_spells(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = SpellService::new(&mut db).count();
    to_api_response(result)
}

/// Get spells available to a specific class.
///
/// Returns all spells on the class's spell list, optionally filtered by level.
#[tauri::command]
pub fn get_spells_by_class(
    state: State<'_, AppState>,
    class_name: String,
    level: Option<i32>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = if let Some(lvl) = level {
        SpellService::new(&mut db).list_by_class_and_level(&class_name, lvl)
    } else {
        SpellService::new(&mut db).list_by_class(&class_name)
    };

    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}
