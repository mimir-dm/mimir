//! Catalog Commands
//!
//! Tauri commands for searching and retrieving D&D 5e catalog content
//! (monsters, spells, items, races, backgrounds, classes, feats, conditions,
//! languages, traps, hazards, actions).

use mimir_core::models::catalog::{
    Action, ActionFilter, Background, BackgroundFilter, CatalogTable, CatalogTableFilter, Class,
    ClassFeature, ClassFilter, Condition, ConditionFilter, Cult, CultFilter, Deity, DeityFilter,
    Disease, Feat, FeatFilter, Hazard, HazardFilter, Item, ItemFilter, Language, LanguageFilter,
    Monster, MonsterFilter, Object, ObjectFilter, OptionalFeature, OptionalFeatureFilter, Psionic,
    PsionicFilter, Race, RaceFilter, Reward, RewardFilter, Spell, SpellFilter, Subclass,
    SubclassFeature, Trap, TrapFilter, VariantRule, VariantRuleFilter, Vehicle, VehicleFilter,
};
use mimir_core::services::{
    ActionService, BackgroundService, CatalogEntityService, CatalogTableService,
    ClassFeatureService, ClassService, ConditionService, CultService, DeityService, FeatService,
    HazardService, ItemService, LanguageService, MonsterService, ObjectService,
    OptionalFeatureService, PsionicService, RaceService, RewardService, SpellService,
    SubclassFeatureService, SubclassService, TrapService, VariantRuleService, VehicleService,
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

impl CatalogEntity for ClassFeature {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { None }
}

impl CatalogEntity for Subclass {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { self.fluff.as_deref() }
}

impl CatalogEntity for SubclassFeature {
    fn id(&self) -> Option<i32> { self.id }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }
    fn data(&self) -> &str { &self.data }
    fn fluff(&self) -> Option<&str> { None }
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

impl CatalogEntity for Disease {
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
    println!("[search_monsters] filter: {:?}, limit: {:?}, offset: {:?}", filter, limit, offset);

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let filter = filter.unwrap_or_default();
    println!("[search_monsters] resolved filter: {:?}", filter);

    let result = MonsterService::new(&mut db).search_paginated(
        &filter,
        limit.unwrap_or(100),
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

/// Get spells available to a specific class.
///
/// Returns all spells on the class's spell list, optionally filtered by level.
#[tauri::command]
pub fn get_spells_by_class(
    state: State<'_, AppState>,
    class_name: String,
    level: Option<i32>,
) -> ApiResponse<Vec<Value>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
// Class Features
// =============================================================================

/// Get a class feature by name and class.
#[tauri::command]
pub fn get_class_feature(
    state: State<'_, AppState>,
    name: String,
    class_name: String,
) -> ApiResponse<Value> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Value>::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Vec<Value>>::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Vec<Value>>::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Value>::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::<Vec<Value>>::err(format!("Database lock error: {}", e)),
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

/// Search conditions and diseases with optional filters.
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
    let limit = limit.unwrap_or(100);
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

/// Search traps and hazards with optional filters.
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
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    // Helper to format trap/hazard type codes
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

// =============================================================================
// Level-Up Catalog Commands
// =============================================================================

/// Get class information needed for level-up decisions.
///
/// Returns structured data including hit die, subclass level, ASI levels,
/// multiclass prerequisites, and spellcasting type.
#[tauri::command]
pub fn get_class_info(
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
        Ok(Some(class)) => {
            // Parse the class data JSON
            let data: Value = match serde_json::from_str(&class.data) {
                Ok(d) => d,
                Err(e) => return ApiResponse::err(format!("Failed to parse class data: {}", e)),
            };

            // Extract hit die
            let hit_die = data.get("hd")
                .and_then(|hd| hd.get("faces"))
                .and_then(|f| f.as_i64())
                .unwrap_or(8) as i32;

            // Determine subclass level by looking for gainSubclassFeature in classFeatures
            let subclass_level = find_subclass_level(&data);

            // Extract spellcasting type from casterProgression
            let spellcasting_type = data.get("casterProgression")
                .and_then(|p| p.as_str())
                .map(|s| match s {
                    "full" => "Full",
                    "1/2" | "half" => "Half",
                    "1/3" | "third" => "Third",
                    "pact" => "PactMagic",
                    _ => "None",
                });

            // Extract spellcasting ability
            let spellcasting_ability = data.get("spellcastingAbility")
                .and_then(|a| a.as_str())
                .map(|s| s.to_string());

            // Extract multiclass prerequisites
            let multiclass_prereqs = extract_multiclass_prereqs(&data);

            // Standard ASI levels (most classes)
            // Some classes like Fighter/Rogue have more
            let asi_levels = determine_asi_levels(&name, &data);

            // Build the response
            let mut response = serde_json::json!({
                "name": class.name,
                "source": class.source,
                "hit_die": hit_die,
                "subclass_level": subclass_level,
                "asi_levels": asi_levels,
                "multiclass_prereqs": multiclass_prereqs,
                "spellcasting_type": spellcasting_type,
                "spellcasting_ability": spellcasting_ability,
            });

            // Add optional feature progression if present (invocations, metamagic, etc.)
            if let Some(opt_prog) = data.get("optionalfeatureProgression") {
                response.as_object_mut().unwrap()
                    .insert("optional_feature_progression".to_string(), opt_prog.clone());
            }

            // Add cantrip/spells known progression if present
            if let Some(cantrips) = data.get("cantripProgression") {
                response.as_object_mut().unwrap()
                    .insert("cantrip_progression".to_string(), cantrips.clone());
            }
            if let Some(spells_known) = data.get("spellsKnownProgression") {
                response.as_object_mut().unwrap()
                    .insert("spells_known_progression".to_string(), spells_known.clone());
            }

            ApiResponse::ok(response)
        }
        Ok(None) => ApiResponse::err(format!("Class not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get spellcasting progression for a class.
///
/// Returns spell slots per level and spells known (if applicable).
#[tauri::command]
pub fn get_class_spellcasting(
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
        Ok(Some(class)) => {
            let data: Value = match serde_json::from_str(&class.data) {
                Ok(d) => d,
                Err(e) => return ApiResponse::err(format!("Failed to parse class data: {}", e)),
            };

            let caster_type = data.get("casterProgression")
                .and_then(|p| p.as_str());

            if caster_type.is_none() {
                return ApiResponse::ok(serde_json::json!({
                    "name": class.name,
                    "source": class.source,
                    "is_spellcaster": false,
                }));
            }

            let mut response = serde_json::json!({
                "name": class.name,
                "source": class.source,
                "is_spellcaster": true,
                "caster_type": caster_type,
                "spellcasting_ability": data.get("spellcastingAbility"),
            });

            // Add cantrip progression
            if let Some(cantrips) = data.get("cantripProgression") {
                response.as_object_mut().unwrap()
                    .insert("cantrip_progression".to_string(), cantrips.clone());
            }

            // Add spells known progression (for known casters like Bard, Sorcerer)
            if let Some(spells_known) = data.get("spellsKnownProgression") {
                response.as_object_mut().unwrap()
                    .insert("spells_known_progression".to_string(), spells_known.clone());
            }

            // Add prepared spell formula (for prepared casters like Cleric, Druid)
            if let Some(prepared) = data.get("preparedSpells") {
                response.as_object_mut().unwrap()
                    .insert("prepared_spells".to_string(), prepared.clone());
            }

            // Extract spell slots from classTableGroups
            if let Some(table_groups) = data.get("classTableGroups").and_then(|g| g.as_array()) {
                let spell_slots = extract_spell_slots_from_table(table_groups);
                if !spell_slots.is_empty() {
                    response.as_object_mut().unwrap()
                        .insert("spell_slots_by_level".to_string(), spell_slots.into());
                }
            }

            // Generate standard spell slot progression based on caster type if not in tables
            if response.get("spell_slots_by_level").is_none() {
                if let Some(ct) = caster_type {
                    let slots = generate_spell_slot_progression(ct);
                    response.as_object_mut().unwrap()
                        .insert("spell_slots_by_level".to_string(), slots.into());
                }
            }

            ApiResponse::ok(response)
        }
        Ok(None) => ApiResponse::err(format!("Class not found: {} ({})", name, source)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all fighting styles.
#[tauri::command]
pub fn list_fighting_styles(state: State<'_, AppState>) -> ApiResponse<Vec<Value>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get all optional features and filter for fighting styles
    let result = OptionalFeatureService::new(&mut db).list_all();
    match result {
        Ok(features) => {
            let fighting_styles: Vec<Value> = features
                .into_iter()
                .filter(|f| {
                    f.feature_type.as_ref()
                        .map(|t| t.starts_with("FS"))
                        .unwrap_or(false)
                })
                .map(|f| {
                    let mut json = entity_to_json(&f);
                    // Add which classes can use this style
                    if let Value::Object(ref mut map) = json {
                        let classes = extract_fighting_style_classes(f.feature_type.as_deref());
                        map.insert("available_to_classes".to_string(), classes.into());
                    }
                    json
                })
                .collect();
            ApiResponse::ok(fighting_styles)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all metamagic options.
#[tauri::command]
pub fn list_metamagic(state: State<'_, AppState>) -> ApiResponse<Vec<Value>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = OptionalFeatureService::new(&mut db).list_by_type("MM");
    match result {
        Ok(features) => ApiResponse::ok(entities_to_json(features)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all Battle Master maneuvers.
#[tauri::command]
pub fn list_maneuvers(state: State<'_, AppState>) -> ApiResponse<Vec<Value>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Maneuvers can be MV or MV:B
    let result = OptionalFeatureService::new(&mut db).list_all();
    match result {
        Ok(features) => {
            let maneuvers: Vec<Value> = features
                .into_iter()
                .filter(|f| {
                    f.feature_type.as_ref()
                        .map(|t| t.starts_with("MV"))
                        .unwrap_or(false)
                })
                .map(|f| entity_to_json(&f))
                .collect();
            ApiResponse::ok(maneuvers)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all Eldritch Invocations with their prerequisites.
#[tauri::command]
pub fn list_invocations(state: State<'_, AppState>) -> ApiResponse<Vec<Value>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = OptionalFeatureService::new(&mut db).list_by_type("EI");
    match result {
        Ok(features) => {
            let invocations: Vec<Value> = features
                .into_iter()
                .map(|f| {
                    let mut json = entity_to_json(&f);
                    // Parse prerequisites to extract level and pact requirements
                    if let Value::Object(ref mut map) = json {
                        let (level_prereq, pact_prereq, spell_prereq) =
                            extract_invocation_prereqs(map.get("prerequisite"));
                        if let Some(level) = level_prereq {
                            map.insert("level_prereq".to_string(), Value::Number(level.into()));
                        }
                        if let Some(pact) = pact_prereq {
                            map.insert("pact_prereq".to_string(), Value::String(pact));
                        }
                        if let Some(spell) = spell_prereq {
                            map.insert("spell_prereq".to_string(), Value::String(spell));
                        }
                    }
                    json
                })
                .collect();
            ApiResponse::ok(invocations)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all feats with their prerequisites parsed.
#[tauri::command]
pub fn list_feats_with_prereqs(
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
        limit.unwrap_or(200),
        offset.unwrap_or(0),
    );
    match result {
        Ok(feats) => {
            let feats_with_prereqs: Vec<Value> = feats
                .into_iter()
                .map(|f| {
                    let mut json = entity_to_json(&f);
                    // Parse prerequisites from the data
                    if let Value::Object(ref mut map) = json {
                        let prereqs = extract_feat_prereqs(map.get("prerequisite"));
                        if !prereqs.is_empty() {
                            map.insert("parsed_prereqs".to_string(), prereqs.into());
                        }
                    }
                    json
                })
                .collect();
            ApiResponse::ok(feats_with_prereqs)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Find the level at which a class gains its subclass.
fn find_subclass_level(data: &Value) -> i32 {
    if let Some(features) = data.get("classFeatures").and_then(|f| f.as_array()) {
        for feature in features {
            // Check if this feature grants subclass
            let grants_subclass = match feature {
                Value::Object(obj) => obj.get("gainSubclassFeature")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                Value::String(_) => false, // Simple string refs don't grant subclass
                _ => false,
            };

            if grants_subclass {
                // Parse the level from the classFeature string or object
                if let Value::Object(obj) = feature {
                    if let Some(cf) = obj.get("classFeature").and_then(|v| v.as_str()) {
                        // Format: "FeatureName|ClassName|ClassSource|Level"
                        let parts: Vec<&str> = cf.split('|').collect();
                        if parts.len() >= 4 {
                            if let Ok(level) = parts[3].parse::<i32>() {
                                return level;
                            }
                        }
                    }
                }
            }
        }
    }

    // Default subclass levels by class name patterns
    3 // Most classes get subclass at 3
}

/// Extract multiclass prerequisites from class data.
fn extract_multiclass_prereqs(data: &Value) -> Value {
    if let Some(mc) = data.get("multiclassing") {
        if let Some(reqs) = mc.get("requirements") {
            return reqs.clone();
        }
    }
    Value::Null
}

/// Determine ASI levels for a class.
fn determine_asi_levels(class_name: &str, data: &Value) -> Vec<i32> {
    // Standard ASI levels
    let standard = vec![4, 8, 12, 16, 19];

    // Fighter and Rogue get extra ASIs
    let fighter_levels = vec![4, 6, 8, 12, 14, 16, 19];
    let rogue_levels = vec![4, 8, 10, 12, 16, 19];

    match class_name.to_lowercase().as_str() {
        "fighter" => fighter_levels,
        "rogue" => rogue_levels,
        _ => {
            // Try to find ASI levels from class features
            if let Some(features) = data.get("classFeatures").and_then(|f| f.as_array()) {
                let mut asi_levels = Vec::new();
                for feature in features {
                    let feature_str = match feature {
                        Value::String(s) => s.clone(),
                        Value::Object(obj) => obj.get("classFeature")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        _ => continue,
                    };

                    if feature_str.to_lowercase().contains("ability score improvement") {
                        // Parse level from format: "Feature|Class|Source|Level"
                        let parts: Vec<&str> = feature_str.split('|').collect();
                        if parts.len() >= 4 {
                            if let Ok(level) = parts[3].parse::<i32>() {
                                asi_levels.push(level);
                            }
                        }
                    }
                }
                if !asi_levels.is_empty() {
                    return asi_levels;
                }
            }
            standard
        }
    }
}

/// Extract spell slot progression from class table groups.
fn extract_spell_slots_from_table(table_groups: &[Value]) -> Vec<Value> {
    for group in table_groups {
        if let Some(labels) = group.get("colLabels").and_then(|l| l.as_array()) {
            // Look for spell slot columns (1st, 2nd, 3rd, etc.)
            let has_spell_slots = labels.iter().any(|l| {
                l.as_str().map(|s| s.contains("st") || s.contains("nd") || s.contains("rd") || s.contains("th")).unwrap_or(false)
            });

            if has_spell_slots {
                if let Some(rows) = group.get("rows").and_then(|r| r.as_array()) {
                    // Return the rows which contain spell slot data per level
                    return rows.clone();
                }
            }
        }
    }
    Vec::new()
}

/// Generate standard spell slot progression based on caster type.
fn generate_spell_slot_progression(caster_type: &str) -> Vec<Vec<i32>> {
    match caster_type {
        "full" => vec![
            // Level 1-20 spell slots [1st, 2nd, 3rd, 4th, 5th, 6th, 7th, 8th, 9th]
            vec![2, 0, 0, 0, 0, 0, 0, 0, 0], // 1
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0], // 2
            vec![4, 2, 0, 0, 0, 0, 0, 0, 0], // 3
            vec![4, 3, 0, 0, 0, 0, 0, 0, 0], // 4
            vec![4, 3, 2, 0, 0, 0, 0, 0, 0], // 5
            vec![4, 3, 3, 0, 0, 0, 0, 0, 0], // 6
            vec![4, 3, 3, 1, 0, 0, 0, 0, 0], // 7
            vec![4, 3, 3, 2, 0, 0, 0, 0, 0], // 8
            vec![4, 3, 3, 3, 1, 0, 0, 0, 0], // 9
            vec![4, 3, 3, 3, 2, 0, 0, 0, 0], // 10
            vec![4, 3, 3, 3, 2, 1, 0, 0, 0], // 11
            vec![4, 3, 3, 3, 2, 1, 0, 0, 0], // 12
            vec![4, 3, 3, 3, 2, 1, 1, 0, 0], // 13
            vec![4, 3, 3, 3, 2, 1, 1, 0, 0], // 14
            vec![4, 3, 3, 3, 2, 1, 1, 1, 0], // 15
            vec![4, 3, 3, 3, 2, 1, 1, 1, 0], // 16
            vec![4, 3, 3, 3, 2, 1, 1, 1, 1], // 17
            vec![4, 3, 3, 3, 3, 1, 1, 1, 1], // 18
            vec![4, 3, 3, 3, 3, 2, 1, 1, 1], // 19
            vec![4, 3, 3, 3, 3, 2, 2, 1, 1], // 20
        ],
        "1/2" | "half" => vec![
            // Half casters start at level 2
            vec![0, 0, 0, 0, 0], // 1
            vec![2, 0, 0, 0, 0], // 2
            vec![3, 0, 0, 0, 0], // 3
            vec![3, 0, 0, 0, 0], // 4
            vec![4, 2, 0, 0, 0], // 5
            vec![4, 2, 0, 0, 0], // 6
            vec![4, 3, 0, 0, 0], // 7
            vec![4, 3, 0, 0, 0], // 8
            vec![4, 3, 2, 0, 0], // 9
            vec![4, 3, 2, 0, 0], // 10
            vec![4, 3, 3, 0, 0], // 11
            vec![4, 3, 3, 0, 0], // 12
            vec![4, 3, 3, 1, 0], // 13
            vec![4, 3, 3, 1, 0], // 14
            vec![4, 3, 3, 2, 0], // 15
            vec![4, 3, 3, 2, 0], // 16
            vec![4, 3, 3, 3, 1], // 17
            vec![4, 3, 3, 3, 1], // 18
            vec![4, 3, 3, 3, 2], // 19
            vec![4, 3, 3, 3, 2], // 20
        ],
        "1/3" | "third" => vec![
            // Third casters (Eldritch Knight, Arcane Trickster) start at level 3
            vec![0, 0, 0, 0], // 1
            vec![0, 0, 0, 0], // 2
            vec![2, 0, 0, 0], // 3
            vec![3, 0, 0, 0], // 4
            vec![3, 0, 0, 0], // 5
            vec![3, 0, 0, 0], // 6
            vec![4, 2, 0, 0], // 7
            vec![4, 2, 0, 0], // 8
            vec![4, 2, 0, 0], // 9
            vec![4, 3, 0, 0], // 10
            vec![4, 3, 0, 0], // 11
            vec![4, 3, 0, 0], // 12
            vec![4, 3, 2, 0], // 13
            vec![4, 3, 2, 0], // 14
            vec![4, 3, 2, 0], // 15
            vec![4, 3, 3, 0], // 16
            vec![4, 3, 3, 0], // 17
            vec![4, 3, 3, 0], // 18
            vec![4, 3, 3, 1], // 19
            vec![4, 3, 3, 1], // 20
        ],
        "pact" => vec![
            // Warlock pact magic - slots per short rest at highest available level
            // Format: [slots, slot_level]
            vec![1, 1], // 1
            vec![2, 1], // 2
            vec![2, 2], // 3
            vec![2, 2], // 4
            vec![2, 3], // 5
            vec![2, 3], // 6
            vec![2, 4], // 7
            vec![2, 4], // 8
            vec![2, 5], // 9
            vec![2, 5], // 10
            vec![3, 5], // 11
            vec![3, 5], // 12
            vec![3, 5], // 13
            vec![3, 5], // 14
            vec![3, 5], // 15
            vec![3, 5], // 16
            vec![4, 5], // 17
            vec![4, 5], // 18
            vec![4, 5], // 19
            vec![4, 5], // 20
        ],
        _ => Vec::new(),
    }
}

/// Extract which classes can use a fighting style based on its type code.
fn extract_fighting_style_classes(feature_type: Option<&str>) -> Vec<String> {
    match feature_type {
        Some(t) => {
            let mut classes = Vec::new();
            if t.contains("FS:F") || t == "FS" {
                classes.push("Fighter".to_string());
            }
            if t.contains("FS:P") || t == "FS" {
                classes.push("Paladin".to_string());
            }
            if t.contains("FS:R") || t == "FS" {
                classes.push("Ranger".to_string());
            }
            if t.contains("FS:B") {
                classes.push("Bard".to_string());
            }
            classes
        }
        None => Vec::new(),
    }
}

/// Extract invocation prerequisites (level, pact boon, spell requirements).
fn extract_invocation_prereqs(prereq: Option<&Value>) -> (Option<i32>, Option<String>, Option<String>) {
    let mut level_prereq = None;
    let mut pact_prereq = None;
    let mut spell_prereq = None;

    if let Some(Value::Array(prereqs)) = prereq {
        for p in prereqs {
            if let Value::Object(obj) = p {
                // Level requirement
                if let Some(lvl) = obj.get("level") {
                    if let Some(warlock_level) = lvl.get("warlock") {
                        level_prereq = warlock_level.as_i64().map(|l| l as i32);
                    } else if let Some(l) = lvl.as_i64() {
                        level_prereq = Some(l as i32);
                    }
                }

                // Pact boon requirement
                if let Some(pact) = obj.get("pact") {
                    pact_prereq = pact.as_str().map(|s| s.to_string());
                }

                // Spell requirement
                if let Some(spell) = obj.get("spell") {
                    if let Some(spells) = spell.as_array() {
                        spell_prereq = spells.first()
                            .and_then(|s| s.as_str())
                            .map(|s| s.replace("#c", "").to_string());
                    }
                }
            }
        }
    }

    (level_prereq, pact_prereq, spell_prereq)
}

/// Extract feat prerequisites into a simple list.
fn extract_feat_prereqs(prereq: Option<&Value>) -> Vec<String> {
    let mut prereqs = Vec::new();

    if let Some(Value::Array(arr)) = prereq {
        for p in arr {
            if let Value::Object(obj) = p {
                // Ability score requirements
                if let Some(ability) = obj.get("ability") {
                    if let Value::Array(abilities) = ability {
                        for a in abilities {
                            if let Value::Object(ab) = a {
                                for (stat, val) in ab {
                                    if let Some(v) = val.as_i64() {
                                        prereqs.push(format!("{} {}", stat.to_uppercase(), v));
                                    }
                                }
                            }
                        }
                    }
                }

                // Race requirements
                if let Some(race) = obj.get("race") {
                    if let Value::Array(races) = race {
                        for r in races {
                            if let Some(name) = r.get("name").and_then(|n| n.as_str()) {
                                prereqs.push(format!("Race: {}", name));
                            }
                        }
                    }
                }

                // Spellcasting requirement
                if obj.get("spellcasting").is_some() || obj.get("spellcastingFeature").is_some() {
                    prereqs.push("Spellcasting".to_string());
                }

                // Proficiency requirements
                if let Some(prof) = obj.get("proficiency") {
                    if let Value::Array(profs) = prof {
                        for pr in profs {
                            if let Value::Object(po) = pr {
                                for (key, _) in po {
                                    prereqs.push(format!("Proficiency: {}", key));
                                }
                            }
                        }
                    }
                }

                // Level requirement
                if let Some(level) = obj.get("level") {
                    if let Some(l) = level.as_i64() {
                        prereqs.push(format!("Level {}", l));
                    }
                }
            }
        }
    }

    prereqs
}
