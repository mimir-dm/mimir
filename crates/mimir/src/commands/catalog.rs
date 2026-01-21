//! Catalog Commands
//!
//! Tauri commands for searching and retrieving D&D 5e catalog content
//! (monsters, spells, items, races, backgrounds, classes, feats, conditions,
//! languages, traps, hazards, actions).

use mimir_core::models::catalog::{
    Action, ActionFilter, Background, BackgroundFilter, Class, ClassFilter, Condition,
    ConditionFilter, Feat, FeatFilter, Hazard, HazardFilter, Item, ItemFilter, Language,
    LanguageFilter, Monster, MonsterFilter, Race, RaceFilter, Spell, SpellFilter, Trap, TrapFilter,
};
use mimir_core::services::{
    ActionService, BackgroundService, CatalogEntityService, ClassService, ConditionService,
    FeatService, HazardService, ItemService, LanguageService, MonsterService, RaceService,
    SpellService, TrapService,
};
use tauri::State;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// Monsters
// =============================================================================

/// Search monsters with optional filters.
#[tauri::command]
pub fn search_monsters(
    state: State<'_, AppState>,
    filter: Option<MonsterFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Monster>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = MonsterService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a monster by database ID.
#[tauri::command]
pub fn get_monster(state: State<'_, AppState>, id: i32) -> ApiResponse<Monster> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Monster> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = MonsterService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(monster)) => ApiResponse::ok(monster),
        Ok(None) => ApiResponse::err(format!("Monster not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all monster sources.
#[tauri::command]
pub fn list_monster_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = MonsterService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total monsters.
#[tauri::command]
pub fn count_monsters(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = MonsterService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Spells
// =============================================================================

/// Search spells with optional filters.
#[tauri::command]
pub fn search_spells(
    state: State<'_, AppState>,
    filter: Option<SpellFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Spell>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = SpellService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a spell by database ID.
#[tauri::command]
pub fn get_spell(state: State<'_, AppState>, id: i32) -> ApiResponse<Spell> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Spell> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = SpellService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(spell)) => ApiResponse::ok(spell),
        Ok(None) => ApiResponse::err(format!("Spell not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all spell sources.
#[tauri::command]
pub fn list_spell_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = SpellService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total spells.
#[tauri::command]
pub fn count_spells(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = SpellService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Items
// =============================================================================

/// Search items with optional filters.
#[tauri::command]
pub fn search_items(
    state: State<'_, AppState>,
    filter: Option<ItemFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Item>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = ItemService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get an item by database ID.
#[tauri::command]
pub fn get_item(state: State<'_, AppState>, id: i32) -> ApiResponse<Item> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Item> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ItemService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(item)) => ApiResponse::ok(item),
        Ok(None) => ApiResponse::err(format!("Item not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all item sources.
#[tauri::command]
pub fn list_item_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ItemService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total items.
#[tauri::command]
pub fn count_items(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ItemService::new(&mut db).count();
    to_api_response(result)
}

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
) -> ApiResponse<Vec<Race>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = RaceService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a race by database ID.
#[tauri::command]
pub fn get_race(state: State<'_, AppState>, id: i32) -> ApiResponse<Race> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Race> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = RaceService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(race)) => ApiResponse::ok(race),
        Ok(None) => ApiResponse::err(format!("Race not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all race sources.
#[tauri::command]
pub fn list_race_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = RaceService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total races.
#[tauri::command]
pub fn count_races(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Background>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = BackgroundService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a background by database ID.
#[tauri::command]
pub fn get_background(state: State<'_, AppState>, id: i32) -> ApiResponse<Background> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Background> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = BackgroundService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(background)) => ApiResponse::ok(background),
        Ok(None) => ApiResponse::err(format!("Background not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all background sources.
#[tauri::command]
pub fn list_background_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = BackgroundService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total backgrounds.
#[tauri::command]
pub fn count_backgrounds(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Class>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = ClassService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a class by database ID.
#[tauri::command]
pub fn get_class(state: State<'_, AppState>, id: i32) -> ApiResponse<Class> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Class> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ClassService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(class)) => ApiResponse::ok(class),
        Ok(None) => ApiResponse::err(format!("Class not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all class sources.
#[tauri::command]
pub fn list_class_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ClassService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total classes.
#[tauri::command]
pub fn count_classes(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ClassService::new(&mut db).count();
    to_api_response(result)
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
) -> ApiResponse<Vec<Feat>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = FeatService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a feat by database ID.
#[tauri::command]
pub fn get_feat(state: State<'_, AppState>, id: i32) -> ApiResponse<Feat> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Feat> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = FeatService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(feat)) => ApiResponse::ok(feat),
        Ok(None) => ApiResponse::err(format!("Feat not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all feat sources.
#[tauri::command]
pub fn list_feat_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = FeatService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total feats.
#[tauri::command]
pub fn count_feats(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = FeatService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Conditions
// =============================================================================

/// Search conditions with optional filters.
#[tauri::command]
pub fn search_conditions(
    state: State<'_, AppState>,
    filter: Option<ConditionFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Condition>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = ConditionService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a condition by database ID.
#[tauri::command]
pub fn get_condition(state: State<'_, AppState>, id: i32) -> ApiResponse<Condition> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Condition> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ConditionService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(condition)) => ApiResponse::ok(condition),
        Ok(None) => ApiResponse::err(format!("Condition not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all condition sources.
#[tauri::command]
pub fn list_condition_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ConditionService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total conditions.
#[tauri::command]
pub fn count_conditions(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Language>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = LanguageService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a language by database ID.
#[tauri::command]
pub fn get_language(state: State<'_, AppState>, id: i32) -> ApiResponse<Language> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Language> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = LanguageService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(language)) => ApiResponse::ok(language),
        Ok(None) => ApiResponse::err(format!("Language not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all language sources.
#[tauri::command]
pub fn list_language_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = LanguageService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total languages.
#[tauri::command]
pub fn count_languages(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = LanguageService::new(&mut db).count();
    to_api_response(result)
}

// =============================================================================
// Traps
// =============================================================================

/// Search traps with optional filters.
#[tauri::command]
pub fn search_traps(
    state: State<'_, AppState>,
    filter: Option<TrapFilter>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<Trap>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = TrapService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a trap by database ID.
#[tauri::command]
pub fn get_trap(state: State<'_, AppState>, id: i32) -> ApiResponse<Trap> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Trap> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = TrapService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(trap)) => ApiResponse::ok(trap),
        Ok(None) => ApiResponse::err(format!("Trap not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all trap sources.
#[tauri::command]
pub fn list_trap_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = TrapService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total traps.
#[tauri::command]
pub fn count_traps(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Hazard>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = HazardService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get a hazard by database ID.
#[tauri::command]
pub fn get_hazard(state: State<'_, AppState>, id: i32) -> ApiResponse<Hazard> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Hazard> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = HazardService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(hazard)) => ApiResponse::ok(hazard),
        Ok(None) => ApiResponse::err(format!("Hazard not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all hazard sources.
#[tauri::command]
pub fn list_hazard_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = HazardService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total hazards.
#[tauri::command]
pub fn count_hazards(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Action>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = ActionService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    to_api_response(result)
}

/// Get an action by database ID.
#[tauri::command]
pub fn get_action(state: State<'_, AppState>, id: i32) -> ApiResponse<Action> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Action> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ActionService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(action)) => ApiResponse::ok(action),
        Ok(None) => ApiResponse::err(format!("Action not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all action sources.
#[tauri::command]
pub fn list_action_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ActionService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total actions.
#[tauri::command]
pub fn count_actions(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ActionService::new(&mut db).count();
    to_api_response(result)
}
