//! Character-Related Catalog Commands
//!
//! Commands for races, backgrounds, classes, subclasses, feats, and features.

use mimir_core::models::catalog::{
    Background, BackgroundFilter, Class, ClassFilter, Feat, FeatFilter, Race,
    RaceFilter, Subclass,
};
use mimir_core::services::{
    BackgroundService, CatalogEntityService, ClassFeatureService, ClassService, FeatService,
    RaceService, SubclassFeatureService, SubclassService, DEFAULT_QUERY_LIMIT,
};
use serde_json::Value;
use tauri::State;

use crate::commands::{entities_to_json, entity_to_json, to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// Races
// =============================================================================

/// Search races with optional filters.
#[tauri::command]
pub fn search_races(
    state: State<'_, AppState>,
    filter: Option<RaceFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = RaceService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a race by database ID.
#[tauri::command]
pub fn get_race(state: State<'_, AppState>, id: i32) -> ApiResponse<Race> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = RaceService::new(&mut db).get(id);
    match result {
        Ok(Some(race)) => ApiResponse::ok(race),
        Ok(None) => ApiResponse::err(format!("Race not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a race by name and source.
#[tauri::command]
pub fn get_race_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = RaceService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(race)) => ApiResponse::ok(entity_to_json(&race)),
        Ok(None) => ApiResponse::err(format!("Race not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all race sources.
#[tauri::command]
pub fn list_race_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = RaceService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total races.
#[tauri::command]
pub fn count_races(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = RaceService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Backgrounds
// =============================================================================

/// Search backgrounds with optional filters.
#[tauri::command]
pub fn search_backgrounds(
    state: State<'_, AppState>,
    filter: Option<BackgroundFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = BackgroundService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a background by database ID.
#[tauri::command]
pub fn get_background(state: State<'_, AppState>, id: i32) -> ApiResponse<Background> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = BackgroundService::new(&mut db).get(id);
    match result {
        Ok(Some(background)) => ApiResponse::ok(background),
        Ok(None) => ApiResponse::err(format!("Background not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a background by name and source.
#[tauri::command]
pub fn get_background_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = BackgroundService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(background)) => ApiResponse::ok(entity_to_json(&background)),
        Ok(None) => ApiResponse::err(format!("Background not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all background sources.
#[tauri::command]
pub fn list_background_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = BackgroundService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total backgrounds.
#[tauri::command]
pub fn count_backgrounds(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = BackgroundService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Classes
// =============================================================================

/// Search classes with optional filters.
#[tauri::command]
pub fn search_classes(
    state: State<'_, AppState>,
    filter: Option<ClassFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = ClassService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a class by database ID.
#[tauri::command]
pub fn get_class(state: State<'_, AppState>, id: i32) -> ApiResponse<Class> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ClassService::new(&mut db).get(id);
    match result {
        Ok(Some(class)) => ApiResponse::ok(class),
        Ok(None) => ApiResponse::err(format!("Class not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a class by name and source.
#[tauri::command]
pub fn get_class_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ClassService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(class)) => ApiResponse::ok(entity_to_json(&class)),
        Ok(None) => ApiResponse::err(format!("Class not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all class sources.
#[tauri::command]
pub fn list_class_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ClassService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total classes.
#[tauri::command]
pub fn count_classes(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = ClassService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Class Features
// =============================================================================

/// Get a class feature by name and class.
#[tauri::command]
pub fn get_class_feature(
    state: State<'_, AppState>,
    name: String,
    class_name: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Value>::err(e),
    };

    let result = ClassFeatureService::new(&mut db).get_by_name_and_class(&name, &class_name);
    match result {
        Ok(Some(feature)) => ApiResponse::ok(entity_to_json(&feature)),
        Ok(None) => ApiResponse::err(format!("Class feature not found: {} ({})", name, class_name)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all class features for a class.
#[tauri::command]
pub fn list_class_features(
    state: State<'_, AppState>,
    class_name: String,
    class_source: String,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Vec<Value>>::err(e),
    };

    let result = ClassFeatureService::new(&mut db).list_by_class(&class_name, &class_source);
    match result {
        Ok(features) => ApiResponse::ok(entities_to_json(features)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Subclasses
// =============================================================================

/// Get a subclass by database ID.
#[tauri::command]
pub fn get_subclass(state: State<'_, AppState>, id: i32) -> ApiResponse<Subclass> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = SubclassService::new(&mut db).get(id);
    match result {
        Ok(subclass) => ApiResponse::ok(subclass),
        Err(e) => ApiResponse::<Subclass>::err(e.to_string()),
    }
}

/// Get a subclass by name, class, and source.
#[tauri::command]
pub fn get_subclass_by_name(
    state: State<'_, AppState>,
    name: String,
    class_name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = SubclassService::new(&mut db).get_by_name_and_class(&name, &class_name, &source);
    match result {
        Ok(Some(subclass)) => ApiResponse::ok(entity_to_json(&subclass)),
        Ok(None) => ApiResponse::<Value>::err(format!(
            "Subclass not found: {} ({}) from {}",
            name, class_name, source
        )),
        Err(e) => ApiResponse::<Value>::err(e.to_string()),
    }
}

/// List all subclasses for a class.
#[tauri::command]
pub fn list_subclasses_by_class(
    state: State<'_, AppState>,
    class_name: String,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Vec<Value>>::err(e),
    };

    let result = SubclassService::new(&mut db).list_by_class(&class_name);
    match result {
        Ok(subclasses) => ApiResponse::ok(entities_to_json(subclasses)),
        Err(e) => ApiResponse::<Vec<Value>>::err(e.to_string()),
    }
}

/// Count total subclasses.
#[tauri::command]
pub fn count_subclasses(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = SubclassService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Subclass Features
// =============================================================================

/// Get a subclass feature by name and subclass.
#[tauri::command]
pub fn get_subclass_feature(
    state: State<'_, AppState>,
    name: String,
    subclass_name: String,
    subclass_source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Value>::err(e),
    };

    let result =
        SubclassFeatureService::new(&mut db).get_by_name_and_subclass(&name, &subclass_name, &subclass_source);
    match result {
        Ok(Some(feature)) => ApiResponse::ok(entity_to_json(&feature)),
        Ok(None) => ApiResponse::err(format!(
            "Subclass feature not found: {} ({})",
            name, subclass_name
        )),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all subclass features for a subclass.
#[tauri::command]
pub fn list_subclass_features(
    state: State<'_, AppState>,
    subclass_name: String,
    subclass_source: String,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Vec<Value>>::err(e),
    };

    let result = SubclassFeatureService::new(&mut db).list_by_subclass(&subclass_name, &subclass_source);
    match result {
        Ok(features) => ApiResponse::ok(entities_to_json(features)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Feats
// =============================================================================

/// Search feats with optional filters.
#[tauri::command]
pub fn search_feats(
    state: State<'_, AppState>,
    filter: Option<FeatFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let filter = filter.unwrap_or_default();
    let result = FeatService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(DEFAULT_QUERY_LIMIT),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a feat by database ID.
#[tauri::command]
pub fn get_feat(state: State<'_, AppState>, id: i32) -> ApiResponse<Feat> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = FeatService::new(&mut db).get(id);
    match result {
        Ok(Some(feat)) => ApiResponse::ok(feat),
        Ok(None) => ApiResponse::err(format!("Feat not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a feat by name and source.
#[tauri::command]
pub fn get_feat_by_name(
    state: State<'_, AppState>,
    name: String,
    source: String,
) -> ApiResponse<Value> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = FeatService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(feat)) => ApiResponse::ok(entity_to_json(&feat)),
        Ok(None) => ApiResponse::err(format!("Feat not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all feat sources.
#[tauri::command]
pub fn list_feat_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = FeatService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total feats.
#[tauri::command]
pub fn count_feats(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = FeatService::new(&mut db).count();
    to_api_response(result)
}
