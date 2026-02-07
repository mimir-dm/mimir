//! Character Export Commands
//!
//! Tauri commands for exporting character sheets to PDF.

use base64::Engine;
use mimir_core::dal::campaign as dal;
use mimir_core::dal::catalog as catalog_dal;
use mimir_core::services::CharacterService;
use mimir_print::sections::{
    CharacterBattleCardSection, CharacterData, CharacterSection, ClassInfo, EquipmentCardsSection,
    Proficiencies, ProficiencyEntry, SpellCardsSection, is_card_worthy,
};
use mimir_print::{DocumentBuilder, PrintState};
use serde_json::Value;
use tauri::State;
use tracing::{error, info};

use crate::state::AppState;

use super::helpers::{
    caster_level_multiplier, compute_ac, compute_hit_die_string, compute_hp_max,
    enrich_inventory_item, max_spell_level_for_class, spell_slots_for_caster_level,
    spellcasting_ability_for_class,
};
use super::{ApiResponse, CharacterExportOptions, PrintResult};

/// Export a character to PDF
#[tauri::command]
pub fn export_character(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    character_id: String,
    options: Option<CharacterExportOptions>,
) -> ApiResponse<PrintResult> {
    info!("Exporting character {} to PDF", character_id);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the character
    let character = match CharacterService::new(&mut db).get(&character_id) {
        Ok(Some(c)) => c,
        Ok(None) => return ApiResponse::err(format!("Character not found: {}", character_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get character: {}", e)),
    };

    // Get character classes
    let classes = match dal::list_character_classes(&mut db, &character_id) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get character classes: {}", e);
            vec![] // Continue without classes
        }
    };

    // Get character inventory
    let inventory = match CharacterService::new(&mut db).get_inventory(&character_id) {
        Ok(inv) => inv,
        Err(e) => {
            error!("Failed to get character inventory: {}", e);
            vec![] // Continue without inventory
        }
    };

    // Get character proficiencies
    let proficiencies_raw = match dal::list_character_proficiencies(&mut db, &character_id) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to get character proficiencies: {}", e);
            vec![]
        }
    };

    // Build proficiencies struct grouped by type
    let proficiencies = {
        let mut profs = Proficiencies::default();
        for p in proficiencies_raw {
            let expertise = p.has_expertise();
            match p.proficiency_type.as_str() {
                "skill" => profs.skills.push(ProficiencyEntry {
                    name: p.name,
                    expertise,
                }),
                "save" => profs.saves.push(p.name),
                "language" => profs.languages.push(p.name),
                "armor" => profs.armor.push(p.name),
                "weapon" => profs.weapons.push(p.name),
                "tool" => profs.tools.push(p.name),
                _ => {}
            }
        }
        profs
    };

    // Build CharacterData for the section
    let mut char_data = CharacterData {
        name: character.name.clone(),
        player_name: character.player_name.clone(),
        is_npc: character.is_npc(),

        race_name: character.race_name.clone(),
        background_name: character.background_name.clone(),

        strength: character.strength,
        dexterity: character.dexterity,
        constitution: character.constitution,
        intelligence: character.intelligence,
        wisdom: character.wisdom,
        charisma: character.charisma,

        cp: character.cp,
        sp: character.sp,
        ep: character.ep,
        gp: character.gp,
        pp: character.pp,

        traits: character.traits.clone(),
        ideals: character.ideals.clone(),
        bonds: character.bonds.clone(),
        flaws: character.flaws.clone(),

        role: character.role.clone(),
        location: character.location.clone(),
        faction: character.faction.clone(),

        classes: classes
            .into_iter()
            .map(|c| {
                let is_starting = c.is_starting_class();
                ClassInfo {
                    class_name: c.class_name,
                    level: c.level,
                    subclass_name: c.subclass_name,
                    is_starting,
                }
            })
            .collect(),

        inventory: inventory
            .iter()
            .map(|i| enrich_inventory_item(&mut db, i))
            .collect(),

        proficiencies,
        speed: 30, // Default speed - could be looked up from race catalog
        ac: 10,    // computed below

        hit_points_max: 0,   // computed below
        hit_die: String::new(), // computed below
        spellcasting_ability: None,
        spell_save_dc: None,
        spell_attack_bonus: None,
        spell_slots: vec![0; 9],
    };

    // Compute AC from equipped armor
    let dex_mod = (character.dexterity - 10).div_euclid(2);
    char_data.ac = compute_ac(&char_data.inventory, dex_mod);

    // Compute derived fields
    let con_mod = (character.constitution - 10).div_euclid(2);
    char_data.hit_points_max = compute_hp_max(&char_data.classes, con_mod);
    char_data.hit_die = compute_hit_die_string(&char_data.classes);

    // Compute spellcasting: find primary caster class
    let primary_caster = char_data
        .classes
        .iter()
        .find_map(|c| spellcasting_ability_for_class(&c.class_name).map(|a| (a, &c.class_name)));

    if let Some((ability_abbrev, _class_name)) = primary_caster {
        let ability_mod = match ability_abbrev {
            "INT" => (character.intelligence - 10).div_euclid(2),
            "WIS" => (character.wisdom - 10).div_euclid(2),
            "CHA" => (character.charisma - 10).div_euclid(2),
            _ => 0,
        };
        let prof_bonus = {
            let total_level: i32 = char_data.classes.iter().map(|c| c.level).sum();
            if total_level <= 4 {
                2
            } else if total_level <= 8 {
                3
            } else if total_level <= 12 {
                4
            } else if total_level <= 16 {
                5
            } else {
                6
            }
        };

        char_data.spellcasting_ability = Some(ability_abbrev.to_string());
        char_data.spell_save_dc = Some(8 + prof_bonus + ability_mod);
        char_data.spell_attack_bonus = Some(prof_bonus + ability_mod);

        // Compute combined caster level for multiclass spell slots
        let caster_level: f64 = char_data
            .classes
            .iter()
            .map(|c| c.level as f64 * caster_level_multiplier(&c.class_name))
            .sum();
        let caster_level = caster_level.floor() as i32;

        if caster_level > 0 {
            char_data.spell_slots = spell_slots_for_caster_level(caster_level);
        }
    }

    let char_data = char_data;

    // Get export options with defaults
    let opts = options.unwrap_or_default();
    let include_compact = opts.include_compact_sheet.unwrap_or(true);
    let include_battle_card = opts.include_battle_card.unwrap_or(false);
    let include_spell_cards = opts.include_spell_cards.unwrap_or(false);
    let include_equipment_cards = opts.include_equipment_cards.unwrap_or(false);

    // Log received options
    info!("=== Character Export Options ===");
    info!("  include_compact_sheet: {}", include_compact);
    info!("  include_battle_card: {}", include_battle_card);
    info!("  include_spell_cards: {}", include_spell_cards);
    info!("  include_equipment_cards: {}", include_equipment_cards);
    info!("================================");

    // Build PDF with selected sections
    let mut builder = DocumentBuilder::new(&character.name)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false);

    let mut has_content = false;

    // Add compact sheet (default)
    if include_compact {
        info!("[SECTION] Adding CharacterSection (compact sheet)");
        builder = builder.append(CharacterSection::new(char_data.clone()));
        has_content = true;
    }

    // Add battle card
    if include_battle_card {
        info!("[SECTION] Adding CharacterBattleCardSection");
        builder = builder.append(CharacterBattleCardSection::from_single(char_data.clone()));
        has_content = true;
    }

    // Add spell cards - get spells available to character's classes (filtered by level and sources)
    if include_spell_cards {
        info!("[SECTION] Spell cards requested - looking up class spell lists...");

        // Get character's allowed sources for filtering
        let allowed_sources: Option<std::collections::HashSet<String>> =
            match dal::list_character_source_codes(&mut db, &character_id) {
                Ok(sources) if !sources.is_empty() => {
                    info!("  Character has {} allowed sources configured", sources.len());
                    Some(sources.into_iter().collect())
                }
                Ok(_) => {
                    info!("  No source restrictions - showing all spells");
                    None
                }
                Err(e) => {
                    info!(
                        "  Could not load character sources ({}), showing all spells",
                        e
                    );
                    None
                }
            };

        let mut spell_data: Vec<Value> = Vec::new();
        let mut seen_spells: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Get spells for each of the character's classes
        for class_info in &char_data.classes {
            // Calculate max spell level this class can cast at their level
            let max_spell_level =
                max_spell_level_for_class(&class_info.class_name, class_info.level);
            info!(
                "  Looking up spells for {} {} (max spell level: {})",
                class_info.class_name, class_info.level, max_spell_level
            );

            if max_spell_level == 0 {
                info!("    Class has no spellcasting at this level");
                continue;
            }

            match catalog_dal::list_spells_by_class(&mut db, &class_info.class_name) {
                Ok(class_spells) => {
                    let filtered_count = class_spells
                        .iter()
                        .filter(|s| s.level <= max_spell_level)
                        .filter(|s| {
                            allowed_sources
                                .as_ref()
                                .is_none_or(|sources| sources.contains(&s.source))
                        })
                        .count();
                    info!(
                        "    Found {} spells total, {} after level/source filtering",
                        class_spells.len(),
                        filtered_count
                    );

                    for spell in class_spells {
                        // Filter by max spell level (cantrips are level 0)
                        if spell.level > max_spell_level {
                            continue;
                        }

                        // Filter by character's allowed sources (if configured)
                        if let Some(ref sources) = allowed_sources {
                            if !sources.contains(&spell.source) {
                                continue;
                            }
                        }

                        // Create unique key to avoid duplicates (same spell on multiple class lists)
                        let spell_key = format!("{}|{}", spell.name, spell.source);
                        if seen_spells.contains(&spell_key) {
                            continue;
                        }
                        seen_spells.insert(spell_key);

                        match spell.parse_data() {
                            Ok(mut data) => {
                                // Add source class to the data
                                if let Some(obj) = data.as_object_mut() {
                                    obj.insert(
                                        "source_class".to_string(),
                                        Value::String(class_info.class_name.clone()),
                                    );
                                }
                                spell_data.push(data);
                            }
                            Err(e) => {
                                error!("    Failed to parse spell '{}': {}", spell.name, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "    Failed to get spells for class '{}': {}",
                        class_info.class_name, e
                    );
                }
            }
        }

        if spell_data.is_empty() {
            info!("  No spells found for any class - skipping SpellCardsSection");
        } else {
            info!(
                "[SECTION] Adding SpellCardsSection with {} unique spells",
                spell_data.len()
            );
            builder = builder.append(SpellCardsSection::new(spell_data));
            has_content = true;
        }
    } else {
        info!("[SECTION] Spell cards NOT requested");
    }

    // Add equipment cards - look up items from catalog for full data
    if include_equipment_cards {
        info!("[SECTION] Equipment cards requested - looking up inventory...");
        // Get full inventory with source info
        let inventory = match CharacterService::new(&mut db).get_inventory(&character_id) {
            Ok(inv) => {
                info!("  Found {} inventory items", inv.len());
                inv
            }
            Err(e) => {
                error!("  Failed to get inventory for equipment cards: {}", e);
                vec![]
            }
        };

        if inventory.is_empty() {
            info!("  No inventory items - skipping equipment cards section");
        } else {
            let mut item_data: Vec<Value> = Vec::new();
            for inv_item in &inventory {
                info!(
                    "  Looking up item '{}' from source '{}'",
                    inv_item.item_name, inv_item.item_source
                );
                match catalog_dal::get_item_by_name(
                    &mut db,
                    &inv_item.item_name,
                    &inv_item.item_source,
                ) {
                    Ok(Some(catalog_item)) => match catalog_item.parse_data() {
                        Ok(mut data) => {
                            // Add inventory-specific fields
                            if let Some(obj) = data.as_object_mut() {
                                obj.insert(
                                    "quantity".to_string(),
                                    Value::Number(inv_item.quantity.into()),
                                );
                                obj.insert(
                                    "equipped".to_string(),
                                    Value::Bool(inv_item.is_equipped()),
                                );
                                obj.insert(
                                    "attuned".to_string(),
                                    Value::Bool(inv_item.is_attuned()),
                                );
                                if let Some(ref notes) = inv_item.notes {
                                    obj.insert("notes".to_string(), Value::String(notes.clone()));
                                }
                            }
                            // Only include card-worthy items
                            if is_card_worthy(&data) {
                                info!("    -> Card-worthy item added");
                                item_data.push(data);
                            } else {
                                info!("    -> Item not card-worthy, skipping");
                            }
                        }
                        Err(e) => {
                            error!("    -> Failed to parse item data: {}", e);
                        }
                    },
                    Ok(None) => {
                        info!("    -> Item not found in catalog");
                    }
                    Err(e) => {
                        error!("    -> Error looking up item: {}", e);
                    }
                }
            }

            if item_data.is_empty() {
                info!("  No card-worthy items - skipping EquipmentCardsSection");
            } else {
                info!(
                    "[SECTION] Adding EquipmentCardsSection with {} items",
                    item_data.len()
                );
                builder = builder.append(EquipmentCardsSection::new(item_data));
                has_content = true;
            }
        }
    } else {
        info!("[SECTION] Equipment cards NOT requested");
    }

    if !has_content {
        error!("No content selected for export - has_content is false");
        return ApiResponse::err("No content selected for export");
    }

    // Build PDF
    info!("=== Building PDF ===");
    info!("  has_content: {}", has_content);
    let pdf_result = builder.to_pdf();

    match pdf_result {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Character PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate character PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

/// Generate character sheet (legacy API)
#[tauri::command]
pub fn generate_character_sheet(
    _app_state: State<'_, AppState>,
    _print_state: State<'_, PrintState>,
    character_id: String,
    template: Option<String>,
    _include_spell_cards: Option<bool>,
) -> ApiResponse<PrintResult> {
    ApiResponse::err(format!(
        "Character sheet generation not yet implemented (character_id: {}, template: {:?})",
        character_id, template
    ))
}
