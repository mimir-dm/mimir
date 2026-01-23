//! Catalog Commands
//!
//! Tauri commands for searching and retrieving D&D 5e catalog content
//! (monsters, spells, items, races, backgrounds, classes, feats, conditions,
//! languages, traps, hazards, actions).

use mimir_core::models::catalog::{
    Action, ActionFilter, Background, BackgroundFilter, CatalogTable, CatalogTableFilter, Class,
    ClassFilter, Condition, ConditionFilter, Cult, CultFilter, Deity, DeityFilter, Feat, FeatFilter,
    Hazard, HazardFilter, Item, ItemFilter, Language, LanguageFilter, Monster, MonsterFilter,
    Object, ObjectFilter, OptionalFeature, OptionalFeatureFilter, Psionic, PsionicFilter, Race,
    RaceFilter, Reward, RewardFilter, Spell, SpellFilter, Trap, TrapFilter, VariantRule,
    VariantRuleFilter, Vehicle, VehicleFilter,
};
use mimir_core::services::{
    ActionService, BackgroundService, CatalogEntityService, CatalogTableService, ClassService,
    ConditionService, CultService, DeityService, FeatService, HazardService, ItemService,
    LanguageService, MonsterService, ObjectService, OptionalFeatureService, PsionicService,
    RaceService, RewardService, SpellService, TrapService, VariantRuleService, VehicleService,
};
use serde_json::Value;
use tauri::State;

use super::{entity_to_json, entities_to_json, to_api_response, ApiResponse, CatalogEntity};
use crate::state::AppState;

// =============================================================================
// CatalogEntity trait implementations
// =============================================================================

impl CatalogEntity for Monster {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for Spell {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for Item {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for Race {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for Background {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for Class {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for Feat {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for Condition {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for Language {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for Trap {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for Hazard {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for Action {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for Deity {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for OptionalFeature {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for CatalogTable {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for VariantRule {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for Vehicle {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for Cult {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for Psionic {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for Reward {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
}

impl CatalogEntity for Object {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
) -> ApiResponse<Vec<Value>> {
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
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
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
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = DeityService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = DeityService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total deities.
#[tauri::command]
pub fn count_deities(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = DeityService::new(&mut db).count();
    to_api_response(result)
}

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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = OptionalFeatureService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = OptionalFeatureService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total optional features.
#[tauri::command]
pub fn count_optional_features(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = CatalogTableService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CatalogTableService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total tables.
#[tauri::command]
pub fn count_tables(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = VariantRuleService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = VariantRuleService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total variant rules.
#[tauri::command]
pub fn count_variant_rules(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = VehicleService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = VehicleService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total vehicles.
#[tauri::command]
pub fn count_vehicles(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = CultService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    );
    match result {
        Ok(entities) => ApiResponse::ok(entities_to_json(entities)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a cult by database ID.
#[tauri::command]
pub fn get_cult(state: State<'_, AppState>, id: i32) -> ApiResponse<Cult> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CultService::new(&mut db).get_by_name_and_source(&name, &source);
    match result {
        Ok(Some(cult)) => ApiResponse::ok(entity_to_json(&cult)),
        Ok(None) => ApiResponse::err(format!("Cult not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all cult sources.
#[tauri::command]
pub fn list_cult_sources(state: State<'_, AppState>) -> ApiResponse<Vec<String>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = CultService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total cults.
#[tauri::command]
pub fn count_cults(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = PsionicService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = PsionicService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total psionics.
#[tauri::command]
pub fn count_psionics(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = RewardService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = RewardService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total rewards.
#[tauri::command]
pub fn count_rewards(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    let result = ObjectService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ObjectService::new(&mut db).list_sources();
    to_api_response(result)
}

/// Count total objects.
#[tauri::command]
pub fn count_objects(state: State<'_, AppState>) -> ApiResponse<i64> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = ObjectService::new(&mut db).count();
    to_api_response(result)
}
