//! Level-Up Catalog Commands
//!
//! Commands for class info, spellcasting, and character option helpers.

use mimir_core::models::catalog::FeatFilter;
use mimir_core::services::{CatalogEntityService, ClassService, FeatService, OptionalFeatureService};
use serde_json::Value;
use tauri::State;

use super::helpers::{
    determine_asi_levels, extract_feat_prereqs, extract_fighting_style_classes,
    extract_invocation_prereqs, extract_multiclass_prereqs, extract_spell_slots_from_table,
    find_subclass_level, generate_spell_slot_progression,
};
use crate::commands::{entities_to_json, entity_to_json, ApiResponse};
use crate::state::AppState;

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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
