//! World-Related Catalog Commands
//!
//! Commands for conditions, diseases, languages, traps, hazards, actions, and deities.

use mimir_core::models::catalog::{
    Action, ActionFilter, Condition, ConditionFilter, Deity, DeityFilter, Hazard, HazardFilter,
    Language, LanguageFilter, Trap, TrapFilter,
};
use mimir_core::services::{
    ActionService, CatalogEntityService, ConditionService, DeityService, HazardService,
    LanguageService, TrapService, DEFAULT_QUERY_LIMIT,
};
use serde_json::Value;
use tauri::State;

use crate::commands::{entities_to_json, entity_to_json, to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// Conditions
// =============================================================================

/// Search conditions and diseases with optional filters.
#[tauri::command]
pub fn search_conditions(
    state: State<'_, AppState>,
    filter: Option<ConditionFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT);
    let offset = offset.unwrap_or(0);

    // Fetch conditions
    let conditions_result = ConditionService::new(&mut db).search_paginated(&filter, limit, offset);
    let mut results: Vec<Value> = match conditions_result {
        Ok(entities) => entities
            .iter()
            .map(|e| {
                let mut json = entity_to_json(e);
                if let Value::Object(ref mut map) = json {
                    map.insert("item_type".to_string(), Value::String("Condition".to_string()));
                }
                json
            })
            .collect(),
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Fetch diseases and add them
    let diseases = mimir_core::dal::catalog::list_diseases(&mut db);
    if let Ok(disease_list) = diseases {
        for disease in disease_list {
            // Apply name filter if present
            if let Some(ref name_filter) = filter.name_contains {
                if !disease.name.to_lowercase().contains(&name_filter.to_lowercase()) {
                    continue;
                }
            }
            // Apply source filter if present
            if let Some(ref sources) = filter.sources {
                if !sources.is_empty() && !sources.contains(&disease.source) {
                    continue;
                }
            }
            if let Some(ref source) = filter.source {
                if &disease.source != source {
                    continue;
                }
            }

            let mut json = entity_to_json(&disease);
            if let Value::Object(ref mut map) = json {
                map.insert("item_type".to_string(), Value::String("Disease".to_string()));
            }
            results.push(json);
        }
    }

    // Sort by name
    results.sort_by(|a, b| {
        let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        name_a.cmp(name_b)
    });

    ApiResponse::ok(results)
}

/// Get a condition by database ID.
#[tauri::command]
pub fn get_condition(state: State<'_, AppState>, id: i32) -> ApiResponse<Condition> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ConditionService::new(&mut db).get(id);
    match result {
        Ok(Some(condition)) => ApiResponse::ok(condition),
        Ok(None) => ApiResponse::err(format!("Condition not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a condition by name and source.
#[tauri::command]
pub fn get_condition_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ConditionService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(condition)) => ApiResponse::ok(entity_to_json(&condition)),
        Ok(None) => ApiResponse::err(format!("Condition not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all condition sources.
#[tauri::command]
pub fn list_condition_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ConditionService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total conditions.
#[tauri::command]
pub fn count_conditions(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ConditionService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Languages
// =============================================================================

/// Search languages with optional filters.
#[tauri::command]
pub fn search_languages(
    state: State<'_, AppState>,
    filter: Option<LanguageFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = LanguageService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a language by database ID.
#[tauri::command]
pub fn get_language(state: State<'_, AppState>, id: i32) -> ApiResponse<Language> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = LanguageService::new(&mut db).get(id);
    match result {
        Ok(Some(language)) => ApiResponse::ok(language),
        Ok(None) => ApiResponse::err(format!("Language not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a language by name and source.
#[tauri::command]
pub fn get_language_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = LanguageService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(language)) => ApiResponse::ok(entity_to_json(&language)),
        Ok(None) => ApiResponse::err(format!("Language not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all language sources.
#[tauri::command]
pub fn list_language_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = LanguageService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total languages.
#[tauri::command]
pub fn count_languages(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = LanguageService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Traps
// =============================================================================

/// Helper to format trap/hazard type codes.
fn format_trap_type(type_code: Option<&str>) -> String {
    match type_code {
        Some("MAG") => "Magical".to_string(),
        Some("MECH") => "Mechanical".to_string(),
        Some("GEN") => "General".to_string(),
        Some("TRP") => "Trap".to_string(),
        Some("SMPL") => "Simple".to_string(),
        Some("CMPX") => "Complex".to_string(),
        Some("WTH") => "Weather".to_string(),
        Some("ENV") => "Environmental".to_string(),
        Some("WLD") => "Wilderness".to_string(),
        Some("EST") => "Eldritch".to_string(),
        Some(other) => other.to_string(),
        None => "—".to_string(),
    }
}

/// Search traps and hazards with optional filters.
#[tauri::command]
pub fn search_traps(
    state: State<'_, AppState>,
    filter: Option<TrapFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT);
    let offset = offset.unwrap_or(0);

    // Fetch traps
    let traps_result = TrapService::new(&mut db).search_paginated(&filter, limit, offset);
    let mut results: Vec<Value> = match traps_result {
        Ok(entities) => entities
            .iter()
            .map(|e| {
                let mut json = entity_to_json(e);
                if let Value::Object(ref mut map) = json {
                    map.insert("category".to_string(), Value::String("Trap".to_string()));
                    // Extract trapHazType from JSON and format it
                    let trap_type = map
                        .get("trapHazType")
                        .and_then(|v| v.as_str());
                    map.insert("trap_type".to_string(), Value::String(format_trap_type(trap_type)));
                }
                json
            })
            .collect(),
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Fetch hazards and add them
    let hazard_filter = HazardFilter {
        name_contains: filter.name_contains.clone(),
        source: filter.source.clone(),
        sources: filter.sources.clone(),
    };
    let hazards = HazardService::new(&mut db).search_paginated(&hazard_filter, limit, offset);
    if let Ok(hazard_list) = hazards {
        for hazard in hazard_list {
            let mut json = entity_to_json(&hazard);
            if let Value::Object(ref mut map) = json {
                map.insert("category".to_string(), Value::String("Hazard".to_string()));
                // Extract trapHazType from JSON and format it
                let trap_type = map
                    .get("trapHazType")
                    .and_then(|v| v.as_str());
                map.insert("trap_type".to_string(), Value::String(format_trap_type(trap_type)));
            }
            results.push(json);
        }
    }

    // Sort by name
    results.sort_by(|a, b| {
        let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        name_a.cmp(name_b)
    });

    ApiResponse::ok(results)
}

/// Get a trap by database ID.
#[tauri::command]
pub fn get_trap(state: State<'_, AppState>, id: i32) -> ApiResponse<Trap> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = TrapService::new(&mut db).get(id);
    match result {
        Ok(Some(trap)) => ApiResponse::ok(trap),
        Ok(None) => ApiResponse::err(format!("Trap not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a trap by name and source.
#[tauri::command]
pub fn get_trap_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = TrapService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(trap)) => ApiResponse::ok(entity_to_json(&trap)),
        Ok(None) => ApiResponse::err(format!("Trap not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all trap sources.
#[tauri::command]
pub fn list_trap_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = TrapService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total traps.
#[tauri::command]
pub fn count_traps(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = TrapService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Hazards
// =============================================================================

/// Search hazards with optional filters.
#[tauri::command]
pub fn search_hazards(
    state: State<'_, AppState>,
    filter: Option<HazardFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = HazardService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a hazard by database ID.
#[tauri::command]
pub fn get_hazard(state: State<'_, AppState>, id: i32) -> ApiResponse<Hazard> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = HazardService::new(&mut db).get(id);
    match result {
        Ok(Some(hazard)) => ApiResponse::ok(hazard),
        Ok(None) => ApiResponse::err(format!("Hazard not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a hazard by name and source.
#[tauri::command]
pub fn get_hazard_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = HazardService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(hazard)) => ApiResponse::ok(entity_to_json(&hazard)),
        Ok(None) => ApiResponse::err(format!("Hazard not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all hazard sources.
#[tauri::command]
pub fn list_hazard_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = HazardService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total hazards.
#[tauri::command]
pub fn count_hazards(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = HazardService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Actions
// =============================================================================

/// Search actions with optional filters.
#[tauri::command]
pub fn search_actions(
    state: State<'_, AppState>,
    filter: Option<ActionFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = ActionService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get an action by database ID.
#[tauri::command]
pub fn get_action(state: State<'_, AppState>, id: i32) -> ApiResponse<Action> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ActionService::new(&mut db).get(id);
    match result {
        Ok(Some(action)) => ApiResponse::ok(action),
        Ok(None) => ApiResponse::err(format!("Action not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get an action by name and source.
#[tauri::command]
pub fn get_action_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ActionService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(action)) => ApiResponse::ok(entity_to_json(&action)),
        Ok(None) => ApiResponse::err(format!("Action not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all action sources.
#[tauri::command]
pub fn list_action_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ActionService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total actions.
#[tauri::command]
pub fn count_actions(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ActionService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Deities
// =============================================================================

/// Search deities with optional filters.
#[tauri::command]
pub fn search_deities(
    state: State<'_, AppState>,
    filter: Option<DeityFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = DeityService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a deity by database ID.
#[tauri::command]
pub fn get_deity(state: State<'_, AppState>, id: i32) -> ApiResponse<Deity> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = DeityService::new(&mut db).get(id);
    match result {
        Ok(Some(deity)) => ApiResponse::ok(deity),
        Ok(None) => ApiResponse::err(format!("Deity not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a deity by name and source.
#[tauri::command]
pub fn get_deity_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = DeityService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(deity)) => ApiResponse::ok(entity_to_json(&deity)),
        Ok(None) => ApiResponse::err(format!("Deity not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all deity sources.
#[tauri::command]
pub fn list_deity_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = DeityService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total deities.
#[tauri::command]
pub fn count_deities(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = DeityService::new(&mut db).count();
    to_api_response(result)
}
