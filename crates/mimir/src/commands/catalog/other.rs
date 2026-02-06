//! Other Catalog Commands
//!
//! Commands for optional features, tables, variant rules, vehicles, cults, psionics, rewards, and objects.

use mimir_core::models::catalog::{
    CatalogTable, CatalogTableFilter, Cult, CultFilter, Object, ObjectFilter, OptionalFeature,
    OptionalFeatureFilter, Psionic, PsionicFilter, Reward, RewardFilter, VariantRule,
    VariantRuleFilter, Vehicle, VehicleFilter,
};
use mimir_core::services::{
    CatalogEntityService, CatalogTableService, CultService, ObjectService, OptionalFeatureService,
    PsionicService, RewardService, VariantRuleService, VehicleService, DEFAULT_QUERY_LIMIT,
};
use serde_json::Value;
use tauri::State;

use crate::commands::{entities_to_json, entity_to_json, to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// Optional Features
// =============================================================================

/// Search optional features with optional filters.
#[tauri::command]
pub fn search_optional_features(
    state: State<'_, AppState>,
    filter: Option<OptionalFeatureFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = OptionalFeatureService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get an optional feature by database ID.
#[tauri::command]
pub fn get_optional_feature(state: State<'_, AppState>, id: i32) -> ApiResponse<OptionalFeature> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = OptionalFeatureService::new(&mut db).get(id);
    match result {
        Ok(Some(feature)) => ApiResponse::ok(feature),
        Ok(None) => ApiResponse::err(format!("Optional feature not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get an optional feature by name and source.
#[tauri::command]
pub fn get_optional_feature_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = OptionalFeatureService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(feature)) => ApiResponse::ok(entity_to_json(&feature)),
        Ok(None) => ApiResponse::err(format!("Optional feature not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all optional feature sources.
#[tauri::command]
pub fn list_optional_feature_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = OptionalFeatureService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total optional features.
#[tauri::command]
pub fn count_optional_features(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = OptionalFeatureService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Catalog Tables
// =============================================================================

/// Search catalog tables with optional filters.
#[tauri::command]
pub fn search_tables(
    state: State<'_, AppState>,
    filter: Option<CatalogTableFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = CatalogTableService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a catalog table by database ID.
#[tauri::command]
pub fn get_table(state: State<'_, AppState>, id: i32) -> ApiResponse<CatalogTable> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CatalogTableService::new(&mut db).get(id);
    match result {
        Ok(Some(table)) => ApiResponse::ok(table),
        Ok(None) => ApiResponse::err(format!("Table not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a catalog table by name and source.
#[tauri::command]
pub fn get_table_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CatalogTableService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(table)) => ApiResponse::ok(entity_to_json(&table)),
        Ok(None) => ApiResponse::err(format!("Table not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all table sources.
#[tauri::command]
pub fn list_table_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CatalogTableService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total tables.
#[tauri::command]
pub fn count_tables(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CatalogTableService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Variant Rules
// =============================================================================

/// Search variant rules with optional filters.
#[tauri::command]
pub fn search_variant_rules(
    state: State<'_, AppState>,
    filter: Option<VariantRuleFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = VariantRuleService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a variant rule by database ID.
#[tauri::command]
pub fn get_variant_rule(state: State<'_, AppState>, id: i32) -> ApiResponse<VariantRule> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = VariantRuleService::new(&mut db).get(id);
    match result {
        Ok(Some(rule)) => ApiResponse::ok(rule),
        Ok(None) => ApiResponse::err(format!("Variant rule not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a variant rule by name and source.
#[tauri::command]
pub fn get_variant_rule_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = VariantRuleService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(rule)) => ApiResponse::ok(entity_to_json(&rule)),
        Ok(None) => ApiResponse::err(format!("Variant rule not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all variant rule sources.
#[tauri::command]
pub fn list_variant_rule_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = VariantRuleService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total variant rules.
#[tauri::command]
pub fn count_variant_rules(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = VariantRuleService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Vehicles
// =============================================================================

/// Search vehicles with optional filters.
#[tauri::command]
pub fn search_vehicles(
    state: State<'_, AppState>,
    filter: Option<VehicleFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = VehicleService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a vehicle by database ID.
#[tauri::command]
pub fn get_vehicle(state: State<'_, AppState>, id: i32) -> ApiResponse<Vehicle> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = VehicleService::new(&mut db).get(id);
    match result {
        Ok(Some(vehicle)) => ApiResponse::ok(vehicle),
        Ok(None) => ApiResponse::err(format!("Vehicle not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a vehicle by name and source.
#[tauri::command]
pub fn get_vehicle_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = VehicleService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(vehicle)) => ApiResponse::ok(entity_to_json(&vehicle)),
        Ok(None) => ApiResponse::err(format!("Vehicle not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all vehicle sources.
#[tauri::command]
pub fn list_vehicle_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = VehicleService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total vehicles.
#[tauri::command]
pub fn count_vehicles(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = VehicleService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Cults
// =============================================================================

/// Search cults with optional filters.
#[tauri::command]
pub fn search_cults(
    state: State<'_, AppState>,
    filter: Option<CultFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = CultService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => {
            // Convert entities and add item_type/subtype fields
            let results: Vec<Value> = entities
                .iter()
                .map(|e| {
                    let mut json = entity_to_json(e);
                    if let Value::Object(ref mut map) = json {
                        // Determine item_type from __prop field, or infer from name
                        let item_type = map
                            .get("__prop")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_lowercase())
                            .unwrap_or_else(|| {
                                // Fallback: check if name starts with "Cult"
                                let name = map
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                if name.to_lowercase().starts_with("cult") {
                                    "cult".to_string()
                                } else {
                                    "boon".to_string()
                                }
                            });
                        map.insert("item_type".to_string(), Value::String(item_type));

                        // Extract subtype from "type" field
                        if let Some(type_val) = map.get("type").cloned() {
                            map.insert("subtype".to_string(), type_val);
                        }
                    }
                    json
                })
                .collect();
            ApiResponse::ok(results)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a cult by database ID.
#[tauri::command]
pub fn get_cult(state: State<'_, AppState>, id: i32) -> ApiResponse<Cult> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CultService::new(&mut db).get(id);
    match result {
        Ok(Some(cult)) => ApiResponse::ok(cult),
        Ok(None) => ApiResponse::err(format!("Cult not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a cult by name and source.
#[tauri::command]
pub fn get_cult_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CultService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(cult)) => {
            let mut json = entity_to_json(&cult);
            if let Value::Object(ref mut map) = json {
                // Determine item_type from __prop field, or infer from name
                let item_type = map
                    .get("__prop")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_else(|| {
                        // Fallback: check if name starts with "Cult"
                        let name = map
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if name.to_lowercase().starts_with("cult") {
                            "cult".to_string()
                        } else {
                            "boon".to_string()
                        }
                    });
                map.insert("item_type".to_string(), Value::String(item_type));

                // Extract subtype from "type" field
                if let Some(type_val) = map.get("type").cloned() {
                    map.insert("subtype".to_string(), type_val);
                }
            }
            ApiResponse::ok(json)
        }
        Ok(None) => ApiResponse::err(format!("Cult not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all cult sources.
#[tauri::command]
pub fn list_cult_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CultService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total cults.
#[tauri::command]
pub fn count_cults(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = CultService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Psionics
// =============================================================================

/// Search psionics with optional filters.
#[tauri::command]
pub fn search_psionics(
    state: State<'_, AppState>,
    filter: Option<PsionicFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = PsionicService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a psionic by database ID.
#[tauri::command]
pub fn get_psionic(state: State<'_, AppState>, id: i32) -> ApiResponse<Psionic> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = PsionicService::new(&mut db).get(id);
    match result {
        Ok(Some(psionic)) => ApiResponse::ok(psionic),
        Ok(None) => ApiResponse::err(format!("Psionic not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a psionic by name and source.
#[tauri::command]
pub fn get_psionic_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = PsionicService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(psionic)) => ApiResponse::ok(entity_to_json(&psionic)),
        Ok(None) => ApiResponse::err(format!("Psionic not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all psionic sources.
#[tauri::command]
pub fn list_psionic_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = PsionicService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total psionics.
#[tauri::command]
pub fn count_psionics(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = PsionicService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Rewards
// =============================================================================

/// Search rewards with optional filters.
#[tauri::command]
pub fn search_rewards(
    state: State<'_, AppState>,
    filter: Option<RewardFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = RewardService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a reward by database ID.
#[tauri::command]
pub fn get_reward(state: State<'_, AppState>, id: i32) -> ApiResponse<Reward> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = RewardService::new(&mut db).get(id);
    match result {
        Ok(Some(reward)) => ApiResponse::ok(reward),
        Ok(None) => ApiResponse::err(format!("Reward not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a reward by name and source.
#[tauri::command]
pub fn get_reward_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = RewardService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(reward)) => ApiResponse::ok(entity_to_json(&reward)),
        Ok(None) => ApiResponse::err(format!("Reward not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all reward sources.
#[tauri::command]
pub fn list_reward_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = RewardService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total rewards.
#[tauri::command]
pub fn count_rewards(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = RewardService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Objects
// =============================================================================

/// Search objects with optional filters.
#[tauri::command]
pub fn search_objects(
    state: State<'_, AppState>,
    filter: Option<ObjectFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = ObjectService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get an object by database ID.
#[tauri::command]
pub fn get_object(state: State<'_, AppState>, id: i32) -> ApiResponse<Object> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ObjectService::new(&mut db).get(id);
    match result {
        Ok(Some(object)) => ApiResponse::ok(object),
        Ok(None) => ApiResponse::err(format!("Object not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get an object by name and source.
#[tauri::command]
pub fn get_object_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ObjectService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(object)) => ApiResponse::ok(entity_to_json(&object)),
        Ok(None) => ApiResponse::err(format!("Object not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all object sources.
#[tauri::command]
pub fn list_object_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ObjectService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total objects.
#[tauri::command]
pub fn count_objects(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ObjectService::new(&mut db).count();
    to_api_response(result)
}
