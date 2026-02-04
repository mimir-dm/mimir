//! Print Commands
//!
//! Tauri commands for PDF export using mimir-print infrastructure.

use base64::Engine;
use mimir_core::dal::campaign as dal;
use mimir_core::dal::catalog as catalog_dal;
use mimir_core::services::{CampaignService, CharacterService, DocumentService, MapService};
use mimir_print::map_renderer::{MapPrintOptions as RenderMapPrintOptions, RenderMap};
use mimir_print::sections::{
    CharacterBattleCardSection, CharacterData, CharacterSection,
    ClassInfo, CutoutToken, EquipmentCardsSection, InventoryItem,
    MapPreview, MonsterCardSection, Proficiencies, ProficiencyEntry, SpellCardsSection,
    TiledMapSection, TokenCutoutSection, is_card_worthy,
};
use mimir_print::{DocumentBuilder, MarkdownSection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tauri::State;
use tracing::{error, info};

use crate::state::AppState;
use mimir_print::PrintState;

// =============================================================================
// Response Types
// =============================================================================

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }
}

/// Print result containing PDF data
#[derive(Debug, Serialize)]
pub struct PrintResult {
    pub pdf_base64: String,
    pub size_bytes: usize,
}

// =============================================================================
// Export Options
// =============================================================================

#[derive(Debug, Deserialize, Default)]
pub struct CharacterExportOptions {
    pub include_compact_sheet: Option<bool>,
    pub include_battle_card: Option<bool>,
    pub include_spell_cards: Option<bool>,
    pub include_equipment_cards: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ModuleExportOptions {
    pub include_documents: Option<bool>,
    pub include_monsters: Option<bool>,
    pub include_traps: Option<bool>,
    pub include_pois: Option<bool>,
    pub include_npcs: Option<bool>,
    pub include_play_notes: Option<bool>,
    pub include_preview: Option<bool>,
    pub preview_grid: Option<bool>,
    pub preview_los_walls: Option<bool>,
    pub preview_positions: Option<bool>,
    pub include_play: Option<bool>,
    pub play_grid: Option<bool>,
    pub play_los_walls: Option<bool>,
    pub play_cutouts: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CampaignExportOptions {
    pub include_campaign_docs: Option<bool>,
    pub include_module_content: Option<bool>,
    pub include_npcs: Option<bool>,
    pub include_module_map_previews: Option<bool>,
    pub include_module_tiled_maps: Option<bool>,
    pub include_token_cutouts: Option<bool>,
    pub include_campaign_map_previews: Option<bool>,
    pub include_campaign_tiled_maps: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct MapPrintOptions {
    pub include_preview: Option<bool>,
    pub preview_grid: Option<bool>,
    pub preview_los_walls: Option<bool>,
    pub preview_positions: Option<bool>,
    pub include_play: Option<bool>,
    pub play_grid: Option<bool>,
    pub play_los_walls: Option<bool>,
    pub play_cutouts: Option<bool>,
}

/// Template info for listing
#[derive(Debug, Serialize)]
pub struct PrintTemplateInfo {
    pub id: String,
    pub name: String,
    pub category: String,
}

// =============================================================================
// Character Sheet Computation Helpers
// =============================================================================

/// Get the hit die size for a class (e.g. "Fighter" -> 10)
fn hit_die_for_class(class_name: &str) -> i32 {
    match class_name {
        "Barbarian" => 12,
        "Fighter" | "Paladin" | "Ranger" => 10,
        "Artificer" | "Bard" | "Cleric" | "Druid" | "Monk" | "Rogue" | "Warlock" => 8,
        "Sorcerer" | "Wizard" => 6,
        _ => 8, // default to d8
    }
}

/// Get the spellcasting ability abbreviation for a class, if it's a caster
fn spellcasting_ability_for_class(class_name: &str) -> Option<&'static str> {
    match class_name {
        "Artificer" | "Wizard" => Some("INT"),
        "Cleric" | "Druid" | "Ranger" => Some("WIS"),
        "Bard" | "Paladin" | "Sorcerer" | "Warlock" => Some("CHA"),
        _ => None,
    }
}

/// Get the caster level multiplier for multiclass spell slot calculation
fn caster_level_multiplier(class_name: &str) -> f64 {
    match class_name {
        "Bard" | "Cleric" | "Druid" | "Sorcerer" | "Wizard" => 1.0,
        "Artificer" | "Paladin" | "Ranger" => 0.5,
        // Third casters (subclass-based, but handle if they appear)
        "Eldritch Knight" | "Arcane Trickster" => 1.0 / 3.0,
        _ => 0.0,
    }
}

/// Standard 5e spell slot table indexed by caster level (1-20), returning slots for levels 1-9
fn spell_slots_for_caster_level(caster_level: i32) -> Vec<i32> {
    match caster_level {
        1  => vec![2, 0, 0, 0, 0, 0, 0, 0, 0],
        2  => vec![3, 0, 0, 0, 0, 0, 0, 0, 0],
        3  => vec![4, 2, 0, 0, 0, 0, 0, 0, 0],
        4  => vec![4, 3, 0, 0, 0, 0, 0, 0, 0],
        5  => vec![4, 3, 2, 0, 0, 0, 0, 0, 0],
        6  => vec![4, 3, 3, 0, 0, 0, 0, 0, 0],
        7  => vec![4, 3, 3, 1, 0, 0, 0, 0, 0],
        8  => vec![4, 3, 3, 2, 0, 0, 0, 0, 0],
        9  => vec![4, 3, 3, 3, 1, 0, 0, 0, 0],
        10 => vec![4, 3, 3, 3, 2, 0, 0, 0, 0],
        11 => vec![4, 3, 3, 3, 2, 1, 0, 0, 0],
        12 => vec![4, 3, 3, 3, 2, 1, 0, 0, 0],
        13 => vec![4, 3, 3, 3, 2, 1, 1, 0, 0],
        14 => vec![4, 3, 3, 3, 2, 1, 1, 0, 0],
        15 => vec![4, 3, 3, 3, 2, 1, 1, 1, 0],
        16 => vec![4, 3, 3, 3, 2, 1, 1, 1, 0],
        17 => vec![4, 3, 3, 3, 2, 1, 1, 1, 1],
        18 => vec![4, 3, 3, 3, 3, 1, 1, 1, 1],
        19 => vec![4, 3, 3, 3, 3, 2, 1, 1, 1],
        20 => vec![4, 3, 3, 3, 3, 2, 2, 1, 1],
        _ if caster_level < 1 => vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
        _ => vec![4, 3, 3, 3, 3, 2, 2, 1, 1], // cap at 20
    }
}

/// Compute hit points max: level 1 = max die + CON mod, subsequent = avg die + CON mod
fn compute_hp_max(classes: &[mimir_print::sections::ClassInfo], con_mod: i32) -> i32 {
    if classes.is_empty() {
        return 0;
    }

    let mut hp = 0;
    let mut is_first_level = true;

    // Find starting class for first-level HP
    let starting = classes.iter().find(|c| c.is_starting).unwrap_or(&classes[0]);
    let starting_die = hit_die_for_class(&starting.class_name);
    // Level 1: max die + CON mod
    hp += starting_die + con_mod;
    let starting_remaining = starting.level - 1;
    // Remaining levels of starting class: avg + CON mod
    hp += starting_remaining * (starting_die / 2 + 1 + con_mod);

    for class in classes {
        if class.is_starting || (!is_first_level && std::ptr::eq(class, &classes[0])) {
            is_first_level = false;
            continue;
        }
        is_first_level = false;
        let die = hit_die_for_class(&class.class_name);
        hp += class.level * (die / 2 + 1 + con_mod);
    }

    hp.max(1)
}

/// Build hit die string like "5d10 + 3d8"
fn compute_hit_die_string(classes: &[mimir_print::sections::ClassInfo]) -> String {
    classes
        .iter()
        .map(|c| format!("{}d{}", c.level, hit_die_for_class(&c.class_name)))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Enrich an inventory item with catalog data (weapon stats, armor AC, etc.)
fn enrich_inventory_item(
    db: &mut diesel::SqliteConnection,
    inv_item: &mimir_core::models::CharacterInventory,
) -> InventoryItem {
    let equipped = inv_item.is_equipped();
    let attuned = inv_item.is_attuned();

    // Try to look up catalog item for weapon/armor stats
    let (item_type, damage, damage_type, armor_ac, finesse) =
        match catalog_dal::get_item_by_name(db, &inv_item.item_name, &inv_item.item_source) {
            Ok(Some(catalog_item)) => {
                if let Ok(data) = catalog_item.parse_data() {
                    let it = data.get("type")
                        .or_else(|| data.get("item_type"))
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let dmg = data.get("dmg1").and_then(|v| v.as_str()).map(String::from);
                    let dt = data.get("dmg_type")
                        .or_else(|| data.get("dmgType"))
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let ac = data.get("ac").and_then(|v| v.as_i64()).map(|v| v as i32);
                    let fin = data.get("property")
                        .and_then(|v| v.as_array())
                        .map_or(false, |arr| arr.iter().any(|p| p.as_str() == Some("F")));
                    (it, dmg, dt, ac, fin)
                } else {
                    (None, None, None, None, false)
                }
            }
            _ => (None, None, None, None, false),
        };

    InventoryItem {
        name: inv_item.item_name.clone(),
        quantity: inv_item.quantity,
        equipped,
        attuned,
        item_type,
        damage,
        damage_type,
        armor_ac,
        finesse,
    }
}

/// Compute AC from equipped armor and DEX modifier
fn compute_ac(inventory: &[InventoryItem], dex_mod: i32) -> i32 {
    let mut base_ac = 10 + dex_mod; // Default: no armor
    let mut shield_bonus = 0;

    for item in inventory {
        if !item.equipped {
            continue;
        }
        match item.item_type.as_deref() {
            Some("LA") => {
                // Light armor: base AC + DEX mod
                if let Some(ac) = item.armor_ac {
                    base_ac = ac + dex_mod;
                }
            }
            Some("MA") => {
                // Medium armor: base AC + DEX mod (max 2)
                if let Some(ac) = item.armor_ac {
                    base_ac = ac + dex_mod.min(2);
                }
            }
            Some("HA") => {
                // Heavy armor: flat AC, no DEX
                if let Some(ac) = item.armor_ac {
                    base_ac = ac;
                }
            }
            Some("S") => {
                // Shield: +2 (or whatever the AC value is)
                shield_bonus = item.armor_ac.unwrap_or(2);
            }
            _ => {}
        }
    }

    base_ac + shield_bonus
}

/// Calculate the maximum spell level a class can cast at a given class level.
/// Returns 0 if the class has no spellcasting at that level.
fn max_spell_level_for_class(class_name: &str, class_level: i32) -> i32 {
    match class_name {
        // Full casters: spell level = (class_level + 1) / 2, max 9
        "Bard" | "Cleric" | "Druid" | "Sorcerer" | "Wizard" => {
            ((class_level + 1) / 2).min(9)
        }
        // Warlock: uses pact magic, different progression but similar max
        "Warlock" => {
            match class_level {
                1 => 1,
                2 => 1,
                3..=4 => 2,
                5..=6 => 3,
                7..=8 => 4,
                9..=10 => 5,
                11..=16 => 5, // Mystic Arcanum gives 6-9 but as 1/day, slots stay at 5
                17..=20 => 5,
                _ => 0,
            }
        }
        // Half casters: start at 2, spell level = (class_level + 1) / 4 + some offset
        "Paladin" | "Ranger" => {
            if class_level < 2 {
                0
            } else {
                match class_level {
                    2..=4 => 1,
                    5..=8 => 2,
                    9..=12 => 3,
                    13..=16 => 4,
                    17..=20 => 5,
                    _ => 0,
                }
            }
        }
        // Artificer: half caster but starts at 1
        "Artificer" => {
            match class_level {
                1..=4 => 1,
                5..=8 => 2,
                9..=12 => 3,
                13..=16 => 4,
                17..=20 => 5,
                _ => 0,
            }
        }
        // Third casters (subclasses, but if they appear as class names)
        "Eldritch Knight" | "Arcane Trickster" => {
            if class_level < 3 {
                0
            } else {
                match class_level {
                    3..=6 => 1,
                    7..=12 => 2,
                    13..=18 => 3,
                    19..=20 => 4,
                    _ => 0,
                }
            }
        }
        // Non-casters or unknown classes
        _ => 0,
    }
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// List available print templates
#[tauri::command]
pub fn list_print_templates(
    print_state: State<'_, PrintState>,
) -> ApiResponse<Vec<PrintTemplateInfo>> {
    use mimir_print::PrintService;

    let service = PrintService::new(print_state.templates_dir.clone());
    match service.list_templates() {
        Ok(templates) => {
            let infos: Vec<PrintTemplateInfo> = templates
                .into_iter()
                .map(|t| PrintTemplateInfo {
                    id: t.id,
                    name: t.name,
                    category: t.category,
                })
                .collect();
            ApiResponse::ok(infos)
        }
        Err(e) => ApiResponse::err(format!("Failed to list templates: {}", e)),
    }
}

/// Export a single campaign document to PDF
#[tauri::command]
pub fn export_campaign_document(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    document_id: String,
) -> ApiResponse<PrintResult> {
    info!("Exporting campaign document {} to PDF", document_id);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the document
    let document = match DocumentService::new(&mut db).get(&document_id) {
        Ok(Some(doc)) => doc,
        Ok(None) => return ApiResponse::err(format!("Document not found: {}", document_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get document: {}", e)),
    };

    // Get the campaign for the title
    let campaign = match CampaignService::new(&mut db).get(&document.campaign_id) {
        Ok(Some(c)) => c,
        Ok(None) => {
            return ApiResponse::err(format!("Campaign not found: {}", document.campaign_id))
        }
        Err(e) => return ApiResponse::err(format!("Failed to get campaign: {}", e)),
    };

    // Build the PDF using markdown section
    let section = match MarkdownSection::from_markdown(&document.content) {
        Ok(s) => s.with_title(&document.title),
        Err(e) => return ApiResponse::err(format!("Failed to parse markdown: {}", e)),
    };

    let pdf_result = DocumentBuilder::new(&campaign.name)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false)
        .append(section)
        .to_pdf();

    match pdf_result {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Campaign document PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate campaign document PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

/// Export all campaign documents to PDF
#[tauri::command]
pub fn export_campaign_documents(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    campaign_id: String,
    options: Option<CampaignExportOptions>,
) -> ApiResponse<PrintResult> {
    use mimir_core::services::ModuleService;

    info!("Exporting campaign {} to PDF", campaign_id);

    let opts = options.unwrap_or_default();

    // Log received options
    info!("=== Campaign Export Options ===");
    info!("  include_campaign_docs: {:?}", opts.include_campaign_docs);
    info!("  include_module_content: {:?}", opts.include_module_content);
    info!("  include_npcs: {:?}", opts.include_npcs);
    info!("  include_module_map_previews: {:?}", opts.include_module_map_previews);
    info!("  include_module_tiled_maps: {:?}", opts.include_module_tiled_maps);
    info!("  include_token_cutouts: {:?}", opts.include_token_cutouts);
    info!("  include_campaign_map_previews: {:?}", opts.include_campaign_map_previews);
    info!("  include_campaign_tiled_maps: {:?}", opts.include_campaign_tiled_maps);
    info!("================================");

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the campaign
    let campaign = match CampaignService::new(&mut db).get(&campaign_id) {
        Ok(Some(c)) => c,
        Ok(None) => return ApiResponse::err(format!("Campaign not found: {}", campaign_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get campaign: {}", e)),
    };

    // Build combined PDF
    let mut builder = DocumentBuilder::new(&campaign.name)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(true)
        .with_toc(true);

    let mut has_content = false;

    // 1. Campaign-level documents
    if opts.include_campaign_docs.unwrap_or(true) {
        info!("[SECTION] Campaign documents requested");
        let documents = match DocumentService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(docs) => {
                info!("  Found {} campaign documents", docs.len());
                docs
            },
            Err(e) => {
                error!("  Failed to list campaign documents: {}", e);
                vec![]
            }
        };

        for doc in documents {
            match MarkdownSection::from_markdown(&doc.content) {
                Ok(section) => {
                    info!("  Adding document: {}", doc.title);
                    builder = builder.append(section.with_title(&doc.title));
                    has_content = true;
                }
                Err(e) => {
                    error!("  Failed to parse document {}: {}", doc.id, e);
                }
            }
        }
    } else {
        info!("[SECTION] Campaign documents NOT requested");
    }

    // 2. Module content (documents + monsters)
    if opts.include_module_content.unwrap_or(false) {
        info!("[SECTION] Module content requested");
        let modules = match ModuleService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(m) => {
                info!("  Found {} modules", m.len());
                m
            },
            Err(e) => {
                error!("  Failed to list modules: {}", e);
                vec![]
            }
        };

        for module in modules {
            info!("  Processing module: {}", module.name);

            // Module documents
            let module_docs = match DocumentService::new(&mut db).list_for_module(&module.id) {
                Ok(docs) => {
                    info!("    Found {} documents", docs.len());
                    docs
                },
                Err(e) => {
                    error!("    Failed to list module {} documents: {}", module.id, e);
                    vec![]
                }
            };

            for doc in module_docs {
                match MarkdownSection::from_markdown(&doc.content) {
                    Ok(section) => {
                        let title = format!("{}: {}", module.name, doc.title);
                        info!("    Adding document: {}", title);
                        builder = builder.append(section.with_title(&title));
                        has_content = true;
                    }
                    Err(e) => {
                        error!("    Failed to parse document {}: {}", doc.id, e);
                    }
                }
            }

            // Module monsters as cards
            let module_monsters = match dal::list_module_monsters(&mut db, &module.id) {
                Ok(monsters) => {
                    info!("    Found {} monsters", monsters.len());
                    monsters
                },
                Err(e) => {
                    error!("    Failed to list module {} monsters: {}", module.id, e);
                    vec![]
                }
            };

            if !module_monsters.is_empty() {
                let mut monster_data: Vec<Value> = Vec::new();
                for mm in &module_monsters {
                    if let Ok(Some(catalog_monster)) =
                        catalog_dal::get_monster_by_name(&mut db, &mm.monster_name, &mm.monster_source)
                    {
                        if let Ok(mut data) = catalog_monster.parse_data() {
                            if let Some(ref display_name) = mm.display_name {
                                if let Some(obj) = data.as_object_mut() {
                                    obj.insert("name".to_string(), Value::String(display_name.clone()));
                                }
                            }
                            for _ in 0..mm.quantity {
                                monster_data.push(data.clone());
                            }
                        }
                    }
                }

                if !monster_data.is_empty() {
                    info!("    Adding MonsterCardSection with {} monsters", monster_data.len());
                    let section = MonsterCardSection::new(monster_data);
                    builder = builder.append(section);
                    has_content = true;
                }
            }

            // Module traps as cards (from map_traps table)
            {
                use mimir_print::sections::TrapCardSection;
                use std::collections::HashSet;

                let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
                let maps = match map_service.list_for_module(&module.id) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("    Failed to list module maps for traps: {}", e);
                        vec![]
                    }
                };

                let mut seen_traps: HashSet<String> = HashSet::new();
                let mut trap_data: Vec<Value> = Vec::new();

                for map in &maps {
                    if let Ok(map_traps) = dal::list_map_traps(&mut db, &map.id) {
                        for trap in map_traps {
                            if seen_traps.contains(&trap.name) {
                                continue;
                            }
                            seen_traps.insert(trap.name.clone());

                            // Try catalog lookup
                            let sources_to_try = ["DMG", "XGE", "TCE", "PHB"];
                            let mut found_catalog = false;

                            for source in sources_to_try {
                                if let Ok(Some(catalog_trap)) =
                                    catalog_dal::get_trap_by_name(&mut db, &trap.name, source)
                                {
                                    if let Ok(data) = catalog_trap.parse_data() {
                                        trap_data.push(data);
                                        found_catalog = true;
                                        break;
                                    }
                                }
                            }

                            // Create custom trap card if not in catalog
                            if !found_catalog {
                                let custom_trap = serde_json::json!({
                                    "name": trap.name,
                                    "trapHazType": "TRAP",
                                    "effect": [
                                        {
                                            "type": "entries",
                                            "name": "Trigger",
                                            "entries": [trap.trigger_description.as_deref().unwrap_or("Unknown trigger")]
                                        },
                                        {
                                            "type": "entries",
                                            "name": "Effect",
                                            "entries": [trap.effect_description.as_deref().unwrap_or("Unknown effect")]
                                        }
                                    ],
                                    "countermeasures": [trap.description.as_deref().unwrap_or("No countermeasures listed")],
                                    "dc": trap.dc
                                });
                                trap_data.push(custom_trap);
                            }
                        }
                    }
                }

                if !trap_data.is_empty() {
                    info!("    Adding TrapCardSection with {} traps", trap_data.len());
                    let section = TrapCardSection::new(trap_data);
                    builder = builder.append(section);
                    has_content = true;
                }
            }

            // Module POIs (from map_pois table)
            {
                let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
                let maps = match map_service.list_for_module(&module.id) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("    Failed to list module maps for POIs: {}", e);
                        vec![]
                    }
                };

                let mut poi_content = String::new();
                let mut total_pois = 0;

                for map in &maps {
                    if let Ok(map_pois) = dal::list_map_pois(&mut db, &map.id) {
                        if !map_pois.is_empty() {
                            poi_content.push_str(&format!("### {}\n\n", map.name));
                            for poi in &map_pois {
                                total_pois += 1;
                                poi_content.push_str(&format!(
                                    "**{}** ({})\n",
                                    poi.name,
                                    format!("{},{}", poi.grid_x, poi.grid_y)
                                ));
                                if let Some(ref desc) = poi.description {
                                    poi_content.push_str(&format!("{}\n", desc));
                                }
                                poi_content.push('\n');
                            }
                        }
                    }
                }

                if !poi_content.is_empty() {
                    info!("    Adding POI section with {} points of interest", total_pois);
                    let section_title = format!("{}: Points of Interest", module.name);
                    let section = MarkdownSection::from_content(&poi_content, Some(&section_title));
                    builder = builder.append(section);
                    has_content = true;
                }
            }
        }
    } else {
        info!("[SECTION] Module content NOT requested");
    }

    // 3. NPCs (characters marked as NPCs)
    if opts.include_npcs.unwrap_or(false) {
        info!("[SECTION] NPCs requested");
        let npcs = match CharacterService::new(&mut db).list_npcs(&campaign_id) {
            Ok(n) => {
                info!("  Found {} NPCs", n.len());
                n
            },
            Err(e) => {
                error!("  Failed to list NPCs: {}", e);
                vec![]
            }
        };

        for npc in npcs {
            info!("  Processing NPC: {}", npc.name);
            let classes = dal::list_character_classes(&mut db, &npc.id).unwrap_or_default();
            let inventory = CharacterService::new(&mut db)
                .get_inventory(&npc.id)
                .unwrap_or_default();
            let proficiencies_raw = dal::list_character_proficiencies(&mut db, &npc.id).unwrap_or_default();

            // Build proficiencies struct
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

            let mut char_data = CharacterData {
                name: npc.name.clone(),
                player_name: npc.player_name.clone(),
                is_npc: npc.is_npc(),
                race_name: npc.race_name.clone(),
                background_name: npc.background_name.clone(),
                strength: npc.strength,
                dexterity: npc.dexterity,
                constitution: npc.constitution,
                intelligence: npc.intelligence,
                wisdom: npc.wisdom,
                charisma: npc.charisma,
                cp: npc.cp,
                sp: npc.sp,
                ep: npc.ep,
                gp: npc.gp,
                pp: npc.pp,
                traits: npc.traits.clone(),
                ideals: npc.ideals.clone(),
                bonds: npc.bonds.clone(),
                flaws: npc.flaws.clone(),
                role: npc.role.clone(),
                location: npc.location.clone(),
                faction: npc.faction.clone(),
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
                speed: 30,
                ac: 10, // computed below
                hit_points_max: 0,
                hit_die: String::new(),
                spellcasting_ability: None,
                spell_save_dc: None,
                spell_attack_bonus: None,
                spell_slots: vec![0; 9],
            };

            // Compute AC from equipped armor
            let npc_dex_mod = (npc.dexterity - 10).div_euclid(2);
            char_data.ac = compute_ac(&char_data.inventory, npc_dex_mod);

            // Compute HP and hit dice for NPC
            let con_mod = (npc.constitution - 10).div_euclid(2);
            let mut char_data = char_data;
            char_data.hit_points_max = compute_hp_max(&char_data.classes, con_mod);
            char_data.hit_die = compute_hit_die_string(&char_data.classes);

            info!("    Adding CharacterSection for NPC: {}", char_data.name);
            builder = builder.append(CharacterSection::new(char_data));
            has_content = true;
        }
    } else {
        info!("[SECTION] NPCs NOT requested");
    }

    // 4. Campaign-level map previews
    if opts.include_campaign_map_previews.unwrap_or(false) {
        info!("[SECTION] Campaign map previews requested");
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_campaign_level(&campaign_id) {
            Ok(m) => {
                info!("  Found {} campaign-level maps", m.len());
                m
            },
            Err(e) => {
                error!("  Failed to list campaign maps: {}", e);
                vec![]
            }
        };

        for map in maps {
            if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                    // Extract base64 image from UVTT and decode it
                    if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                        match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                            Ok(image_bytes) => {
                                info!("  Adding map preview: {}", map.name);
                                let preview = MapPreview::from_rendered(map.name.clone(), image_bytes);
                                builder = builder.append(preview);
                                has_content = true;
                            }
                            Err(e) => {
                                error!("  Failed to decode map image for {}: {}", map.name, e);
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("[SECTION] Campaign map previews NOT requested");
    }

    // 5. Module map previews
    if opts.include_module_map_previews.unwrap_or(false) {
        info!("[SECTION] Module map previews requested");
        let modules = match ModuleService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(m) => {
                info!("  Found {} modules to check for maps", m.len());
                m
            },
            Err(e) => {
                error!("  Failed to list modules: {}", e);
                vec![]
            }
        };

        for module in modules {
            let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
            let maps = match map_service.list_for_module(&module.id) {
                Ok(m) => {
                    info!("  Module '{}' has {} maps", module.name, m.len());
                    m
                },
                Err(e) => {
                    error!("  Failed to list module {} maps: {}", module.id, e);
                    vec![]
                }
            };

            for map in maps {
                if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                    if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                        // Extract base64 image from UVTT and decode it
                        if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                            match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                                Ok(image_bytes) => {
                                    let name = format!("{}: {}", module.name, map.name);
                                    info!("    Adding map preview: {}", name);
                                    let preview = MapPreview::from_rendered(name, image_bytes);
                                    builder = builder.append(preview);
                                    has_content = true;
                                }
                                Err(e) => {
                                    error!("    Failed to decode map image for {}: {}", map.name, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("[SECTION] Module map previews NOT requested");
    }

    // 6. Campaign tiled maps (for tabletop play at 1"=5ft scale)
    if opts.include_campaign_tiled_maps.unwrap_or(false) {
        info!("[SECTION] Campaign tiled maps requested");
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_campaign_level(&campaign_id) {
            Ok(m) => {
                info!("  Found {} campaign-level maps for tiled export", m.len());
                m
            },
            Err(e) => {
                error!("  Failed to list campaign maps for tiled export: {}", e);
                vec![]
            }
        };

        for map in maps {
            if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                    let pixels_per_grid = uvtt_json
                        .get("resolution")
                        .and_then(|r| r.get("pixels_per_grid"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(70) as i32;

                    // Extract base64 image from UVTT and decode it
                    if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                        match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                            Ok(image_bytes) => {
                                let tiled = TiledMapSection::from_rendered(
                                    map.name.clone(),
                                    image_bytes,
                                    pixels_per_grid,
                                );
                                builder = builder.append(tiled);
                                has_content = true;
                            }
                            Err(e) => {
                                error!("Failed to decode map image for tiled {}: {}", map.name, e);
                            }
                        }
                    }
                }
            }
        }
    }

    // 7. Module tiled maps (for tabletop play at 1"=5ft scale)
    if opts.include_module_tiled_maps.unwrap_or(false) {
        info!("[SECTION] Module tiled maps requested");
        let modules = match ModuleService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(m) => {
                info!("  Found {} modules for tiled maps", m.len());
                m
            },
            Err(e) => {
                error!("  Failed to list modules for tiled maps: {}", e);
                vec![]
            }
        };

        for module in modules {
            let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
            let maps = match map_service.list_for_module(&module.id) {
                Ok(m) => {
                    info!("  Module '{}' has {} maps for tiled export", module.name, m.len());
                    m
                },
                Err(e) => {
                    error!("  Failed to list module {} maps for tiled export: {}", module.id, e);
                    vec![]
                }
            };

            for map in maps {
                if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                    if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                        let resolution = uvtt_json.get("resolution");
                        let pixels_per_grid = resolution
                            .and_then(|r| r.get("pixels_per_grid"))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(70) as i32;

                        // Extract base64 image from UVTT and decode it
                        if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                            match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                                Ok(image_bytes) => {
                                    let name = format!("{}: {}", module.name, map.name);
                                    let tiled = TiledMapSection::from_rendered(
                                        name,
                                        image_bytes,
                                        pixels_per_grid,
                                    );
                                    builder = builder.append(tiled);
                                    has_content = true;
                                }
                                Err(e) => {
                                    error!("Failed to decode map image for tiled {}: {}", map.name, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 8. Token cutouts (paper standees for tabletop play)
    if opts.include_token_cutouts.unwrap_or(false) {
        info!("[SECTION] Token cutouts requested");
        use mimir_core::services::ModuleService;

        let modules = match ModuleService::new(&mut db).list_for_campaign(&campaign_id) {
            Ok(m) => {
                info!("  Found {} modules for token cutouts", m.len());
                m
            },
            Err(e) => {
                error!("  Failed to list modules for token cutouts: {}", e);
                vec![]
            }
        };

        let mut cutout_tokens: Vec<CutoutToken> = Vec::new();

        for module in modules {
            // Get module monsters
            let module_monsters = match dal::list_module_monsters(&mut db, &module.id) {
                Ok(monsters) => {
                    info!("  Module '{}' has {} monsters for cutouts", module.name, monsters.len());
                    monsters
                },
                Err(e) => {
                    error!("  Failed to list module {} monsters for cutouts: {}", module.id, e);
                    continue;
                }
            };

            for mm in module_monsters {
                // Look up catalog monster to get size
                let size = match catalog_dal::get_monster_by_name(
                    &mut db,
                    &mm.monster_name,
                    &mm.monster_source,
                ) {
                    Ok(Some(catalog_monster)) => {
                        catalog_monster.size.unwrap_or_else(|| "Medium".to_string())
                    }
                    _ => "Medium".to_string(),
                };

                // Try to load token image from assets
                let img_base = app_state
                    .paths
                    .assets_dir
                    .join("catalog")
                    .join("bestiary")
                    .join("tokens")
                    .join(&mm.monster_source);

                let extensions = ["webp", "png", "jpg", "jpeg"];
                let mut image_bytes: Option<Vec<u8>> = None;

                for ext in &extensions {
                    let path = img_base.join(format!("{}.{}", &mm.monster_name, ext));
                    if path.exists() {
                        match std::fs::read(&path) {
                            Ok(bytes) => {
                                image_bytes = Some(bytes);
                                break;
                            }
                            Err(e) => {
                                error!("Failed to read token image {:?}: {}", path, e);
                            }
                        }
                    }
                }

                // Use display name if set, otherwise monster name
                let display_name = mm.display_name.unwrap_or(mm.monster_name.clone());

                let mut token = CutoutToken::new(display_name, size, "monster".to_string())
                    .with_quantity(mm.quantity as u32);

                if let Some(bytes) = image_bytes {
                    token = token.with_image(bytes);
                }

                cutout_tokens.push(token);
            }
        }

        if !cutout_tokens.is_empty() {
            info!("  Adding TokenCutoutSection with {} tokens", cutout_tokens.len());
            let section = TokenCutoutSection::new(cutout_tokens);
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No tokens with images found");
        }
    } else {
        info!("[SECTION] Token cutouts NOT requested");
    }

    if !has_content {
        error!("No content found to export - has_content is false");
        return ApiResponse::err("No content found to export");
    }

    info!("=== Building Campaign PDF ===");
    match builder.to_pdf() {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Campaign PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate campaign PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

/// Export module documents to PDF
#[tauri::command]
pub fn export_module_documents(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    module_id: String,
    options: Option<ModuleExportOptions>,
) -> ApiResponse<PrintResult> {
    info!("Exporting module {} to PDF", module_id);

    let opts = options.unwrap_or_default();

    // Log received options
    info!("=== Module Export Options ===");
    info!("  include_documents: {:?}", opts.include_documents);
    info!("  include_monsters: {:?}", opts.include_monsters);
    info!("  include_traps: {:?}", opts.include_traps);
    info!("  include_pois: {:?}", opts.include_pois);
    info!("  include_npcs: {:?}", opts.include_npcs);
    info!("  include_play_notes: {:?}", opts.include_play_notes);
    info!("  include_preview: {:?}", opts.include_preview);
    info!("  preview_grid: {:?}", opts.preview_grid);
    info!("  preview_los_walls: {:?}", opts.preview_los_walls);
    info!("  preview_positions: {:?}", opts.preview_positions);
    info!("  include_play: {:?}", opts.include_play);
    info!("  play_grid: {:?}", opts.play_grid);
    info!("  play_los_walls: {:?}", opts.play_los_walls);
    info!("  play_cutouts: {:?}", opts.play_cutouts);
    info!("==============================");

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the module
    use mimir_core::services::ModuleService;
    let module = match ModuleService::new(&mut db).get(&module_id) {
        Ok(Some(m)) => m,
        Ok(None) => return ApiResponse::err(format!("Module not found: {}", module_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get module: {}", e)),
    };

    // Build combined PDF
    let mut builder = DocumentBuilder::new(&module.name)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(true)
        .with_toc(true);

    let mut has_content = false;

    // 1. Module documents
    if opts.include_documents.unwrap_or(true) {
        info!("[SECTION] Module documents requested");
        let documents = match DocumentService::new(&mut db).list_for_module(&module_id) {
            Ok(docs) => {
                info!("  Found {} documents", docs.len());
                docs
            },
            Err(e) => {
                error!("  Failed to list documents: {}", e);
                vec![]
            }
        };

        for doc in documents {
            match MarkdownSection::from_markdown(&doc.content) {
                Ok(section) => {
                    info!("  Adding document: {}", doc.title);
                    builder = builder.append(section.with_title(&doc.title));
                    has_content = true;
                }
                Err(e) => {
                    error!("  Failed to parse document {}: {}", doc.id, e);
                }
            }
        }
    } else {
        info!("[SECTION] Module documents NOT requested");
    }

    // 2. Monster stat blocks
    if opts.include_monsters.unwrap_or(true) {
        info!("[SECTION] Monster cards requested");
        let module_monsters = match dal::list_module_monsters(&mut db, &module_id) {
            Ok(monsters) => {
                info!("  Found {} module monsters", monsters.len());
                monsters
            },
            Err(e) => {
                error!("  Failed to list module monsters: {}", e);
                vec![]
            }
        };

        let mut monster_data: Vec<Value> = Vec::new();
        for mm in &module_monsters {
            if let Ok(Some(catalog_monster)) = catalog_dal::get_monster_by_name(
                &mut db,
                &mm.monster_name,
                &mm.monster_source,
            ) {
                if let Ok(mut data) = catalog_monster.parse_data() {
                    // Apply display name override if set
                    if let Some(ref display_name) = mm.display_name {
                        if let Some(obj) = data.as_object_mut() {
                            obj.insert("name".to_string(), Value::String(display_name.clone()));
                        }
                    }
                    // Add quantity copies if > 1
                    for _ in 0..mm.quantity {
                        monster_data.push(data.clone());
                    }
                }
            }
        }

        if !monster_data.is_empty() {
            info!("  Adding MonsterCardSection with {} monsters", monster_data.len());
            let section = MonsterCardSection::new(monster_data);
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No monster data found");
        }
    } else {
        info!("[SECTION] Monster cards NOT requested");
    }

    // 2b. Trap cards (from map_traps table)
    if opts.include_traps.unwrap_or(false) {
        use mimir_print::sections::TrapCardSection;
        use std::collections::HashSet;

        info!("[SECTION] Trap cards requested");

        // Get all maps for this module
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_for_module(&module_id) {
            Ok(m) => m,
            Err(e) => {
                error!("  Failed to list module maps for traps: {}", e);
                vec![]
            }
        };

        // Collect unique trap names from all maps
        let mut seen_traps: HashSet<String> = HashSet::new();
        let mut trap_data: Vec<Value> = Vec::new();

        for map in &maps {
            match dal::list_map_traps(&mut db, &map.id) {
                Ok(map_traps) => {
                    for trap in map_traps {
                        // Skip if we've already added this trap type
                        if seen_traps.contains(&trap.name) {
                            continue;
                        }
                        seen_traps.insert(trap.name.clone());

                        // Try to look up trap in catalog (default to DMG source)
                        let sources_to_try = ["DMG", "XGE", "TCE", "PHB"];
                        let mut found_catalog = false;

                        for source in sources_to_try {
                            if let Ok(Some(catalog_trap)) =
                                catalog_dal::get_trap_by_name(&mut db, &trap.name, source)
                            {
                                if let Ok(data) = catalog_trap.parse_data() {
                                    info!("  Found trap in catalog: {} ({})", trap.name, source);
                                    trap_data.push(data);
                                    found_catalog = true;
                                    break;
                                }
                            }
                        }

                        // If not in catalog, create a basic trap card from map trap data
                        if !found_catalog {
                            info!("  Creating custom trap card: {}", trap.name);
                            let custom_trap = serde_json::json!({
                                "name": trap.name,
                                "trapHazType": "TRAP",
                                "effect": [
                                    {
                                        "type": "entries",
                                        "name": "Trigger",
                                        "entries": [trap.trigger_description.as_deref().unwrap_or("Unknown trigger")]
                                    },
                                    {
                                        "type": "entries",
                                        "name": "Effect",
                                        "entries": [trap.effect_description.as_deref().unwrap_or("Unknown effect")]
                                    }
                                ],
                                "countermeasures": [trap.description.as_deref().unwrap_or("No countermeasures listed")],
                                "dc": trap.dc
                            });
                            trap_data.push(custom_trap);
                        }
                    }
                }
                Err(e) => {
                    error!("  Failed to list traps for map {}: {}", map.id, e);
                }
            }
        }

        if !trap_data.is_empty() {
            info!("  Adding TrapCardSection with {} traps", trap_data.len());
            let section = TrapCardSection::new(trap_data);
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No traps found in module maps");
        }
    } else {
        info!("[SECTION] Trap cards NOT requested");
    }

    // 2c. Points of Interest (from map_pois table)
    if opts.include_pois.unwrap_or(false) {
        info!("[SECTION] POIs requested");

        // Get all maps for this module
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_for_module(&module_id) {
            Ok(m) => m,
            Err(e) => {
                error!("  Failed to list module maps for POIs: {}", e);
                vec![]
            }
        };

        // Collect POIs from all maps, grouped by map
        let mut poi_content = String::new();
        let mut total_pois = 0;

        for map in &maps {
            match dal::list_map_pois(&mut db, &map.id) {
                Ok(map_pois) => {
                    if !map_pois.is_empty() {
                        poi_content.push_str(&format!("## {}\n\n", map.name));
                        for poi in &map_pois {
                            total_pois += 1;
                            poi_content.push_str(&format!(
                                "### {} ({})\n",
                                poi.name,
                                format!("{},{}", poi.grid_x, poi.grid_y)
                            ));
                            if let Some(ref desc) = poi.description {
                                poi_content.push_str(&format!("{}\n", desc));
                            }
                            poi_content.push('\n');
                        }
                    }
                }
                Err(e) => {
                    error!("  Failed to list POIs for map {}: {}", map.id, e);
                }
            }
        }

        if !poi_content.is_empty() {
            info!("  Adding POI section with {} points of interest", total_pois);
            let section = MarkdownSection::from_content(&poi_content, Some("Points of Interest"));
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No POIs found in module maps");
        }
    } else {
        info!("[SECTION] POIs NOT requested");
    }

    // 3. Map previews
    if opts.include_preview.unwrap_or(true) {
        info!("[SECTION] Map previews requested");
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_for_module(&module_id) {
            Ok(m) => {
                info!("  Found {} maps", m.len());
                m
            },
            Err(e) => {
                error!("  Failed to list module maps: {}", e);
                vec![]
            }
        };

        for map in maps {
            if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                    // Extract base64 image from UVTT and decode it
                    if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                        match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                            Ok(image_bytes) => {
                                info!("  Adding map preview: {}", map.name);
                                let preview = MapPreview::from_rendered(map.name.clone(), image_bytes);
                                builder = builder.append(preview);
                                has_content = true;
                            }
                            Err(e) => {
                                error!("  Failed to decode map image for {}: {}", map.name, e);
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("[SECTION] Map previews NOT requested");
    }

    // 4. Tiled maps for play
    if opts.include_play.unwrap_or(false) {
        info!("[SECTION] Tiled maps requested");
        let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
        let maps = match map_service.list_for_module(&module_id) {
            Ok(m) => {
                info!("  Found {} maps for tiled export", m.len());
                m
            },
            Err(e) => {
                error!("  Failed to list module maps: {}", e);
                vec![]
            }
        };

        for map in maps {
            if let Ok(uvtt_bytes) = map_service.read_uvtt_file(&map) {
                if let Ok(uvtt_json) = serde_json::from_slice::<Value>(&uvtt_bytes) {
                    let pixels_per_grid = uvtt_json
                        .get("resolution")
                        .and_then(|r| r.get("pixels_per_grid"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(70) as i32;

                    // Extract base64 image from UVTT and decode it
                    if let Some(image_base64) = uvtt_json.get("image").and_then(|v| v.as_str()) {
                        match base64::engine::general_purpose::STANDARD.decode(image_base64) {
                            Ok(image_bytes) => {
                                let tiled = TiledMapSection::from_rendered(
                                    map.name.clone(),
                                    image_bytes,
                                    pixels_per_grid,
                                );
                                info!("  Adding tiled map: {}", map.name);
                                builder = builder.append(tiled);
                                has_content = true;
                            }
                            Err(e) => {
                                error!("  Failed to decode map image for tiled {}: {}", map.name, e);
                            }
                        }
                    }
                }
            }
        }
    } else {
        info!("[SECTION] Tiled maps NOT requested");
    }

    // 5. Token cutouts
    if opts.play_cutouts.unwrap_or(false) {
        info!("[SECTION] Token cutouts requested");
        let module_monsters = match dal::list_module_monsters(&mut db, &module_id) {
            Ok(monsters) => {
                info!("  Found {} monsters for cutouts", monsters.len());
                monsters
            },
            Err(e) => {
                error!("  Failed to list module monsters for cutouts: {}", e);
                vec![]
            }
        };

        let mut cutout_tokens: Vec<CutoutToken> = Vec::new();

        for mm in module_monsters {
            let size = match catalog_dal::get_monster_by_name(
                &mut db,
                &mm.monster_name,
                &mm.monster_source,
            ) {
                Ok(Some(catalog_monster)) => {
                    catalog_monster.size.unwrap_or_else(|| "Medium".to_string())
                }
                _ => "Medium".to_string(),
            };

            // Try to load token image
            let img_base = app_state
                .paths
                .assets_dir
                .join("catalog")
                .join("bestiary")
                .join("tokens")
                .join(&mm.monster_source);

            let extensions = ["webp", "png", "jpg", "jpeg"];
            let mut image_bytes: Option<Vec<u8>> = None;

            for ext in &extensions {
                let path = img_base.join(format!("{}.{}", &mm.monster_name, ext));
                if path.exists() {
                    if let Ok(bytes) = std::fs::read(&path) {
                        image_bytes = Some(bytes);
                        break;
                    }
                }
            }

            let display_name = mm.display_name.unwrap_or(mm.monster_name.clone());
            let mut token = CutoutToken::new(display_name, size, "monster".to_string())
                .with_quantity(mm.quantity as u32);

            if let Some(bytes) = image_bytes {
                token = token.with_image(bytes);
            }

            cutout_tokens.push(token);
        }

        if !cutout_tokens.is_empty() {
            info!("  Adding TokenCutoutSection with {} tokens", cutout_tokens.len());
            let section = TokenCutoutSection::new(cutout_tokens);
            builder = builder.append(section);
            has_content = true;
        } else {
            info!("  No tokens with images found");
        }
    } else {
        info!("[SECTION] Token cutouts NOT requested");
    }

    if !has_content {
        error!("No content selected or found to export - has_content is false");
        return ApiResponse::err("No content selected or found to export");
    }

    info!("=== Building Module PDF ===");
    match builder.to_pdf() {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!("Module PDF generated successfully ({} bytes)", size_bytes);

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate module PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

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
        ac: 10, // computed below

        hit_points_max: 0,  // computed below
        hit_die: String::new(),  // computed below
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
    let primary_caster = char_data.classes.iter()
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
            if total_level <= 4 { 2 } else if total_level <= 8 { 3 }
            else if total_level <= 12 { 4 } else if total_level <= 16 { 5 } else { 6 }
        };

        char_data.spellcasting_ability = Some(ability_abbrev.to_string());
        char_data.spell_save_dc = Some(8 + prof_bonus + ability_mod);
        char_data.spell_attack_bonus = Some(prof_bonus + ability_mod);

        // Compute combined caster level for multiclass spell slots
        let caster_level: f64 = char_data.classes.iter()
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
                    info!("  Could not load character sources ({}), showing all spells", e);
                    None
                }
            };

        let mut spell_data: Vec<Value> = Vec::new();
        let mut seen_spells: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Get spells for each of the character's classes
        for class_info in &char_data.classes {
            // Calculate max spell level this class can cast at their level
            let max_spell_level = max_spell_level_for_class(&class_info.class_name, class_info.level);
            info!("  Looking up spells for {} {} (max spell level: {})",
                class_info.class_name, class_info.level, max_spell_level);

            if max_spell_level == 0 {
                info!("    Class has no spellcasting at this level");
                continue;
            }

            match catalog_dal::list_spells_by_class(&mut db, &class_info.class_name) {
                Ok(class_spells) => {
                    let filtered_count = class_spells.iter()
                        .filter(|s| s.level <= max_spell_level)
                        .filter(|s| allowed_sources.as_ref().is_none_or(|sources| sources.contains(&s.source)))
                        .count();
                    info!("    Found {} spells total, {} after level/source filtering",
                        class_spells.len(), filtered_count);

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
                                    obj.insert("source_class".to_string(), Value::String(class_info.class_name.clone()));
                                }
                                spell_data.push(data);
                            },
                            Err(e) => {
                                error!("    Failed to parse spell '{}': {}", spell.name, e);
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("    Failed to get spells for class '{}': {}", class_info.class_name, e);
                }
            }
        }

        if spell_data.is_empty() {
            info!("  No spells found for any class - skipping SpellCardsSection");
        } else {
            info!("[SECTION] Adding SpellCardsSection with {} unique spells", spell_data.len());
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
            },
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
                info!("  Looking up item '{}' from source '{}'", inv_item.item_name, inv_item.item_source);
                match catalog_dal::get_item_by_name(&mut db, &inv_item.item_name, &inv_item.item_source) {
                    Ok(Some(catalog_item)) => {
                        match catalog_item.parse_data() {
                            Ok(mut data) => {
                                // Add inventory-specific fields
                                if let Some(obj) = data.as_object_mut() {
                                    obj.insert("quantity".to_string(), Value::Number(inv_item.quantity.into()));
                                    obj.insert("equipped".to_string(), Value::Bool(inv_item.is_equipped()));
                                    obj.insert("attuned".to_string(), Value::Bool(inv_item.is_attuned()));
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
                            },
                            Err(e) => {
                                error!("    -> Failed to parse item data: {}", e);
                            }
                        }
                    },
                    Ok(None) => {
                        info!("    -> Item not found in catalog");
                    },
                    Err(e) => {
                        error!("    -> Error looking up item: {}", e);
                    }
                }
            }

            if item_data.is_empty() {
                info!("  No card-worthy items - skipping EquipmentCardsSection");
            } else {
                info!("[SECTION] Adding EquipmentCardsSection with {} items", item_data.len());
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

/// Print a map to PDF
#[tauri::command]
pub fn print_map(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    map_id: String,
    options: Option<MapPrintOptions>,
) -> ApiResponse<PrintResult> {
    info!("=== print_map called ===");
    info!("  map_id: {}", map_id);

    let opts = options.unwrap_or_default();

    // Log options
    info!("=== Options ===");
    info!("  include_preview: {:?}", opts.include_preview);
    info!("  include_play: {:?}", opts.include_play);
    info!("  preview_grid: {:?}", opts.preview_grid);
    info!("  preview_los_walls: {:?}", opts.preview_los_walls);
    info!("  preview_positions: {:?}", opts.preview_positions);
    info!("  play_grid: {:?}", opts.play_grid);
    info!("  play_los_walls: {:?}", opts.play_los_walls);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the map
    let mut map_service = MapService::new(&mut db, &app_state.paths.app_dir);
    let map = match map_service.get(&map_id) {
        Ok(Some(m)) => m,
        Ok(None) => return ApiResponse::err(format!("Map not found: {}", map_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get map: {}", e)),
    };

    // Read UVTT file to get map dimensions and grid info
    let uvtt_bytes = match map_service.read_uvtt_file(&map) {
        Ok(data) => data,
        Err(e) => return ApiResponse::err(format!("Failed to read UVTT file: {}", e)),
    };

    // Parse UVTT JSON
    let uvtt_json: Value = match serde_json::from_slice(&uvtt_bytes) {
        Ok(v) => v,
        Err(e) => return ApiResponse::err(format!("Failed to parse UVTT JSON: {}", e)),
    };

    // Extract resolution data
    let resolution = uvtt_json.get("resolution");
    let pixels_per_grid = resolution
        .and_then(|r| r.get("pixels_per_grid"))
        .and_then(|v| v.as_i64())
        .unwrap_or(70) as i32;

    let map_size_x = resolution
        .and_then(|r| r.get("map_size"))
        .and_then(|ms| ms.get("x"))
        .and_then(|v| v.as_f64())
        .unwrap_or(25.0);

    let map_size_y = resolution
        .and_then(|r| r.get("map_size"))
        .and_then(|ms| ms.get("y"))
        .and_then(|v| v.as_f64())
        .unwrap_or(25.0);

    let width_px = (pixels_per_grid as f64 * map_size_x) as i32;
    let height_px = (pixels_per_grid as f64 * map_size_y) as i32;

    // Extract LOS walls from UVTT
    let los_walls: Vec<Vec<(f64, f64)>> = uvtt_json
        .get("line_of_sight")
        .and_then(|los| los.as_array())
        .map(|segments| {
            segments
                .iter()
                .filter_map(|segment| {
                    segment.as_array().map(|points| {
                        points
                            .iter()
                            .filter_map(|p| {
                                let x = p.get("x").and_then(|v| v.as_f64())?;
                                let y = p.get("y").and_then(|v| v.as_f64())?;
                                Some((x, y))
                            })
                            .collect()
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Create RenderMap from the UVTT data
    let render_map = RenderMap {
        name: map.name.clone(),
        image_path: map.uvtt_asset_id.clone(), // Will be resolved via UVTT extraction
        width_px,
        height_px,
        grid_type: "square".to_string(),
        grid_size_px: Some(pixels_per_grid),
        grid_offset_x: 0,
        grid_offset_y: 0,
    };

    // Build the base path for assets
    let base_path = PathBuf::from(&app_state.paths.app_dir);

    // Create render options
    let render_options = RenderMapPrintOptions {
        show_grid: opts.preview_grid.unwrap_or(true) || opts.play_grid.unwrap_or(true),
        show_los_walls: opts.preview_los_walls.unwrap_or(false) || opts.play_los_walls.unwrap_or(false),
        show_positions: opts.preview_positions.unwrap_or(false),
        los_walls: los_walls.clone(),
        pixels_per_grid: pixels_per_grid as u32,
    };

    // Build the PDF based on options
    let include_preview = opts.include_preview.unwrap_or(true);
    let include_play = opts.include_play.unwrap_or(false);

    info!("=== Map Data ===");
    info!("  map_name: {}", map.name);
    info!("  width_px: {}, height_px: {}", width_px, height_px);
    info!("  pixels_per_grid: {}", pixels_per_grid);
    info!("  los_walls_count: {}", los_walls.len());

    if !include_preview && !include_play {
        error!("No content selected - neither preview nor play mode enabled");
        return ApiResponse::err("Must include either preview or play mode");
    }

    let mut builder = DocumentBuilder::new(&map.name)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false);

    info!("=== Sections ===");

    // Add map preview section if requested
    if include_preview {
        info!("[SECTION] Adding MapPreview section");
        info!("  show_grid: {}", render_options.show_grid);
        info!("  show_los_walls: {}", render_options.show_los_walls);
        info!("  show_positions: {}", render_options.show_positions);
        let preview = MapPreview::new(render_map.clone(), vec![], base_path.clone())
            .with_options(render_options.clone());
        builder = builder.append(preview);
    } else {
        info!("[SECTION] MapPreview NOT requested");
    }

    // Add tiled map section if requested
    if include_play {
        info!("[SECTION] Adding TiledMapSection for play");
        info!("  play_grid: {}", opts.play_grid.unwrap_or(true));
        info!("  play_los_walls: {}", opts.play_los_walls.unwrap_or(false));
        let tiled_options = RenderMapPrintOptions {
            show_grid: opts.play_grid.unwrap_or(true),
            show_los_walls: opts.play_los_walls.unwrap_or(false),
            show_positions: false, // Play tiles don't show positions
            los_walls: los_walls.clone(),
            pixels_per_grid: pixels_per_grid as u32,
        };
        let tiled = TiledMapSection::new(render_map, vec![], base_path)
            .with_options(tiled_options);
        builder = builder.append(tiled);
    } else {
        info!("[SECTION] TiledMapSection NOT requested");
    }

    info!("=== Building PDF ===");

    match builder.to_pdf() {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!("Map PDF generated successfully ({} bytes)", size_bytes);

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate map PDF: {:?}", e);
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

/// Save PDF to file
#[tauri::command]
pub fn save_pdf(pdf_base64: String, path: String) -> ApiResponse<()> {
    info!("=== save_pdf called ===");
    info!("  path: {}", path);
    info!("  base64_length: {} bytes", pdf_base64.len());

    let pdf_bytes = match base64::engine::general_purpose::STANDARD.decode(&pdf_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid base64 data: {}", e);
            return ApiResponse::err(format!("Invalid base64: {}", e));
        }
    };

    info!("  decoded_size: {} bytes", pdf_bytes.len());

    match std::fs::write(&path, &pdf_bytes) {
        Ok(_) => {
            info!("PDF saved successfully to: {}", path);
            ApiResponse::ok(())
        }
        Err(e) => {
            error!("Failed to write PDF file: {}", e);
            ApiResponse::err(format!("Failed to write file: {}", e))
        }
    }
}

// =============================================================================
// Monster Card Export
// =============================================================================

/// Export options for monster cards
#[derive(Debug, Deserialize, Default)]
pub struct MonsterExportOptions {
    pub show_cut_lines: Option<bool>,
}

/// Export module monsters as cards to PDF
#[tauri::command]
pub fn export_module_monsters(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    module_id: String,
    options: Option<MonsterExportOptions>,
) -> ApiResponse<PrintResult> {
    info!("=== export_module_monsters called ===");
    info!("  module_id: {}", module_id);

    let opts = options.unwrap_or_default();

    // Log options
    info!("=== Options ===");
    info!("  show_cut_lines: {:?}", opts.show_cut_lines);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get the module
    use mimir_core::services::ModuleService;
    let module = match ModuleService::new(&mut db).get(&module_id) {
        Ok(Some(m)) => m,
        Ok(None) => return ApiResponse::err(format!("Module not found: {}", module_id)),
        Err(e) => return ApiResponse::err(format!("Failed to get module: {}", e)),
    };

    // Get module monsters
    let module_monsters = match dal::list_module_monsters(&mut db, &module_id) {
        Ok(monsters) => monsters,
        Err(e) => return ApiResponse::err(format!("Failed to list module monsters: {}", e)),
    };

    info!("=== Module Data ===");
    info!("  module_name: {}", module.name);
    info!("  module_monsters_count: {}", module_monsters.len());

    if module_monsters.is_empty() {
        error!("No monsters found for this module");
        return ApiResponse::err("No monsters found for this module");
    }

    for mm in &module_monsters {
        info!("  - {} ({}) x{}", mm.monster_name, mm.monster_source, mm.quantity);
    }

    // Look up each monster's full data from the catalog
    let mut monster_data: Vec<Value> = Vec::new();
    for mm in &module_monsters {
        match catalog_dal::get_monster_by_name(&mut db, &mm.monster_name, &mm.monster_source) {
            Ok(Some(catalog_monster)) => {
                // Parse the monster's data JSON
                match catalog_monster.parse_data() {
                    Ok(mut data) => {
                        // Apply display name override if set
                        if let Some(ref display_name) = mm.display_name {
                            if let Some(obj) = data.as_object_mut() {
                                obj.insert("name".to_string(), Value::String(display_name.clone()));
                            }
                        }
                        // Add quantity copies if > 1
                        for _ in 0..mm.quantity {
                            monster_data.push(data.clone());
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to parse monster data for {}: {}",
                            mm.monster_name, e
                        );
                    }
                }
            }
            Ok(None) => {
                error!(
                    "Catalog monster not found: {} ({})",
                    mm.monster_name, mm.monster_source
                );
            }
            Err(e) => {
                error!("Failed to look up monster {}: {}", mm.monster_name, e);
            }
        }
    }

    if monster_data.is_empty() {
        error!("No valid monster data found for this module");
        return ApiResponse::err("No valid monster data found for this module");
    }

    info!("=== Section ===");
    info!("[SECTION] Adding MonsterCardSection with {} monsters", monster_data.len());

    // Create monster cards section
    let mut section = MonsterCardSection::new(monster_data);
    if let Some(show_cut) = opts.show_cut_lines {
        section = section.with_cut_lines(show_cut);
    }

    // Build PDF
    info!("=== Building PDF ===");
    let title = format!("{} - Monster Cards", module.name);
    let pdf_result = DocumentBuilder::new(&title)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false)
        .append(section)
        .to_pdf();

    match pdf_result {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Monster cards PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate monster cards PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

/// Export a single monster as a card to PDF
#[tauri::command]
pub fn export_monster_card(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    monster_name: String,
    monster_source: String,
    options: Option<MonsterExportOptions>,
) -> ApiResponse<PrintResult> {
    info!("=== export_monster_card called ===");
    info!("  monster_name: {}", monster_name);
    info!("  monster_source: {}", monster_source);

    let opts = options.unwrap_or_default();

    // Log options
    info!("=== Options ===");
    info!("  show_cut_lines: {:?}", opts.show_cut_lines);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Look up the monster from catalog
    let catalog_monster =
        match catalog_dal::get_monster_by_name(&mut db, &monster_name, &monster_source) {
            Ok(Some(m)) => m,
            Ok(None) => {
                return ApiResponse::err(format!(
                    "Monster not found: {} ({})",
                    monster_name, monster_source
                ))
            }
            Err(e) => return ApiResponse::err(format!("Failed to look up monster: {}", e)),
        };

    // Parse monster data
    let monster_data = match catalog_monster.parse_data() {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse monster data: {}", e);
            return ApiResponse::err(format!("Failed to parse monster data: {}", e));
        }
    };

    info!("=== Section ===");
    info!("[SECTION] Adding MonsterCardSection for single monster");

    // Create monster cards section with single monster
    let mut section = MonsterCardSection::new(vec![monster_data]);
    if let Some(show_cut) = opts.show_cut_lines {
        section = section.with_cut_lines(show_cut);
    }

    // Build PDF
    info!("=== Building PDF ===");
    let title = format!("{} - Monster Card", monster_name);
    let pdf_result = DocumentBuilder::new(&title)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false)
        .append(section)
        .to_pdf();

    match pdf_result {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Monster card PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate monster card PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

// =============================================================================
// Trap Card Export
// =============================================================================

/// Export options for trap cards
#[derive(Debug, Deserialize, Default)]
pub struct TrapExportOptions {
    pub show_cut_lines: Option<bool>,
}

/// Export a single trap as a card to PDF
#[tauri::command]
pub fn export_trap_card(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    trap_name: String,
    trap_source: String,
    options: Option<TrapExportOptions>,
) -> ApiResponse<PrintResult> {
    use mimir_print::sections::TrapCardSection;

    info!("=== export_trap_card called ===");
    info!("  trap_name: {}", trap_name);
    info!("  trap_source: {}", trap_source);

    let opts = options.unwrap_or_default();

    // Log options
    info!("=== Options ===");
    info!("  show_cut_lines: {:?}", opts.show_cut_lines);

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Look up the trap from catalog
    let catalog_trap = match catalog_dal::get_trap_by_name(&mut db, &trap_name, &trap_source) {
        Ok(Some(t)) => t,
        Ok(None) => {
            return ApiResponse::err(format!(
                "Trap not found: {} ({})",
                trap_name, trap_source
            ))
        }
        Err(e) => return ApiResponse::err(format!("Failed to look up trap: {}", e)),
    };

    // Parse trap data
    let trap_data = match catalog_trap.parse_data() {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse trap data: {}", e);
            return ApiResponse::err(format!("Failed to parse trap data: {}", e));
        }
    };

    info!("=== Section ===");
    info!("[SECTION] Adding TrapCardSection for single trap");

    // Create trap cards section with single trap
    let mut section = TrapCardSection::new(vec![trap_data]);
    if let Some(show_cut) = opts.show_cut_lines {
        section = section.with_cut_lines(show_cut);
    }

    // Build PDF
    info!("=== Building PDF ===");
    let title = format!("{} - Trap Card", trap_name);
    let pdf_result = DocumentBuilder::new(&title)
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false)
        .append(section)
        .to_pdf();

    match pdf_result {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Trap card PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate trap card PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

/// Export multiple traps as cards to PDF
#[tauri::command]
pub fn export_trap_cards(
    app_state: State<'_, AppState>,
    print_state: State<'_, PrintState>,
    traps: Vec<(String, String)>, // Vec of (name, source) tuples
    options: Option<TrapExportOptions>,
) -> ApiResponse<PrintResult> {
    use mimir_print::sections::TrapCardSection;

    info!("=== export_trap_cards called ===");
    info!("  traps_count: {}", traps.len());

    let opts = options.unwrap_or_default();

    // Log options
    info!("=== Options ===");
    info!("  show_cut_lines: {:?}", opts.show_cut_lines);

    if traps.is_empty() {
        error!("No traps specified");
        return ApiResponse::err("No traps specified");
    }

    // Log requested traps
    info!("=== Requested Traps ===");
    for (name, source) in &traps {
        info!("  - {} ({})", name, source);
    }

    // Get database connection
    let mut db = match app_state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Look up each trap from catalog
    let mut trap_data: Vec<Value> = Vec::new();
    for (name, source) in &traps {
        match catalog_dal::get_trap_by_name(&mut db, name, source) {
            Ok(Some(catalog_trap)) => match catalog_trap.parse_data() {
                Ok(data) => {
                    trap_data.push(data);
                }
                Err(e) => {
                    error!("Failed to parse trap data for {}: {}", name, e);
                }
            },
            Ok(None) => {
                error!("Catalog trap not found: {} ({})", name, source);
            }
            Err(e) => {
                error!("Failed to look up trap {}: {}", name, e);
            }
        }
    }

    if trap_data.is_empty() {
        error!("No valid trap data found");
        return ApiResponse::err("No valid trap data found");
    }

    info!("=== Section ===");
    info!("[SECTION] Adding TrapCardSection with {} traps", trap_data.len());

    // Create trap cards section
    let mut section = TrapCardSection::new(trap_data);
    if let Some(show_cut) = opts.show_cut_lines {
        section = section.with_cut_lines(show_cut);
    }

    // Build PDF
    info!("=== Building PDF ===");
    let pdf_result = DocumentBuilder::new("Trap Cards")
        .with_templates_root(print_state.templates_dir.clone())
        .with_title_page(false)
        .with_toc(false)
        .append(section)
        .to_pdf();

    match pdf_result {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

            info!(
                "Trap cards PDF generated successfully ({} bytes)",
                size_bytes
            );

            ApiResponse::ok(PrintResult {
                pdf_base64,
                size_bytes,
            })
        }
        Err(e) => {
            error!("Failed to generate trap cards PDF: {:?}", e);
            ApiResponse::err(format!("Failed to generate PDF: {}", e))
        }
    }
}

