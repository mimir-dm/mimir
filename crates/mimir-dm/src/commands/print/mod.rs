//! Print command handlers for PDF generation.
//!
//! Contains Tauri commands for generating and managing PDF output
//! from Typst templates.

use crate::state::AppState;
use crate::types::ApiResponse;
use mimir_dm_print::PrintService;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info};

/// Information about an available print template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintTemplateInfo {
    /// Template identifier (e.g., "character/sheet")
    pub id: String,
    /// Display name (e.g., "Character Sheet")
    pub name: String,
    /// Category (e.g., "character", "spell", "monster")
    pub category: String,
}

/// Result of PDF generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintResult {
    /// PDF data as base64-encoded string
    pub pdf_base64: String,
    /// Size of the PDF in bytes
    pub size_bytes: usize,
}

/// Get the templates root path.
///
/// In development, this uses the crate's templates directory.
/// In production, templates should be bundled with the app.
fn get_templates_root() -> std::path::PathBuf {
    // Try to use bundled templates first (production)
    // For now, fall back to the development path
    let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("mimir-dm-print")
        .join("templates");

    if dev_path.exists() {
        dev_path
    } else {
        // In production, templates would be bundled differently
        // For now, log a warning and return the dev path
        tracing::warn!(
            "Templates directory not found at {:?}, PDF generation may fail",
            dev_path
        );
        dev_path
    }
}

/// Create a PrintService instance.
fn create_print_service() -> PrintService {
    PrintService::new(get_templates_root())
}

/// List all available print templates.
///
/// Returns a list of templates organized by category.
#[tauri::command]
pub async fn list_print_templates() -> Result<ApiResponse<Vec<PrintTemplateInfo>>, String> {
    debug!("Listing print templates");

    let service = create_print_service();

    match service.list_templates() {
        Ok(templates) => {
            let template_infos: Vec<PrintTemplateInfo> = templates
                .into_iter()
                .map(|t| PrintTemplateInfo {
                    id: t.id,
                    name: t.name,
                    category: t.category,
                })
                .collect();

            info!("Found {} print templates", template_infos.len());
            Ok(ApiResponse::success(template_infos))
        }
        Err(e) => {
            error!("Failed to list templates: {:?}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list templates: {}",
                e
            )))
        }
    }
}

/// Generate a PDF from a template.
///
/// # Parameters
/// - `template_id` - Template identifier (e.g., "character/sheet.typ")
/// - `data` - JSON data to inject into the template
///
/// # Returns
/// Base64-encoded PDF data
#[tauri::command]
pub async fn generate_pdf(
    template_id: String,
    data: serde_json::Value,
) -> Result<ApiResponse<PrintResult>, String> {
    info!("Generating PDF from template: {}", template_id);
    debug!("Template data: {:?}", data);

    let service = create_print_service();

    // Ensure template has .typ extension
    let template_path = if template_id.ends_with(".typ") {
        template_id
    } else {
        format!("{}.typ", template_id)
    };

    match service.render_to_pdf(&template_path, data) {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &pdf_bytes,
            );

            info!("PDF generated successfully ({} bytes)", size_bytes);

            Ok(ApiResponse::success(PrintResult {
                pdf_base64,
                size_bytes,
            }))
        }
        Err(e) => {
            error!("Failed to generate PDF: {:?}", e);
            Ok(ApiResponse::error(format!("Failed to generate PDF: {}", e)))
        }
    }
}

/// Calculate the maximum spell level a character can cast based on class and level.
/// Returns 0 for non-casters or if no spells are available.
fn calculate_max_spell_level(class_name: &str, class_level: i32) -> i32 {
    // Full casters: Bard, Cleric, Druid, Sorcerer, Wizard, Warlock (has Pact Magic but same spell levels)
    let full_casters = ["Bard", "Cleric", "Druid", "Sorcerer", "Wizard", "Warlock"];

    // Half casters: Paladin, Ranger, Artificer
    let half_casters = ["Paladin", "Ranger", "Artificer"];

    // Third casters are subclass-based (Eldritch Knight, Arcane Trickster) - we'll treat them as 1/3
    // For now, if Fighter or Rogue, assume they might be third casters
    let third_casters = ["Fighter", "Rogue"];

    if full_casters.iter().any(|&c| c.eq_ignore_ascii_case(class_name)) {
        // Full caster spell level progression
        match class_level {
            1..=2 => 1,
            3..=4 => 2,
            5..=6 => 3,
            7..=8 => 4,
            9..=10 => 5,
            11..=12 => 6,
            13..=14 => 7,
            15..=16 => 8,
            17..=20 => 9,
            _ => 0,
        }
    } else if half_casters.iter().any(|&c| c.eq_ignore_ascii_case(class_name)) {
        // Half caster spell level progression (starts at level 2)
        match class_level {
            2..=4 => 1,
            5..=8 => 2,
            9..=12 => 3,
            13..=16 => 4,
            17..=20 => 5,
            _ => 0,
        }
    } else if third_casters.iter().any(|&c| c.eq_ignore_ascii_case(class_name)) {
        // Third caster spell level progression (subclass dependent, starts at level 3)
        match class_level {
            3..=6 => 1,
            7..=12 => 2,
            13..=18 => 3,
            19..=20 => 4,
            _ => 0,
        }
    } else {
        // Non-caster or unknown class
        0
    }
}

/// Generate a character sheet PDF.
///
/// Convenience command for character sheet generation with proper data structure.
/// When include_spell_cards is true (default), the FULL class spell list is included
/// up to the character's max castable spell level, so players can "cut out" the spells
/// they want to use during play.
/// Feature details are always fetched from the catalog.
#[tauri::command]
pub async fn generate_character_sheet(
    state: State<'_, AppState>,
    character_id: i32,
    template: Option<String>,
    include_spell_cards: Option<bool>,
) -> Result<ApiResponse<PrintResult>, String> {
    use mimir_dm_core::models::catalog::class::{ClassFeature, SubclassFeature};
    use mimir_dm_core::models::catalog::item::Item;
    use mimir_dm_core::models::catalog::Spell;
    use mimir_dm_core::models::catalog::SpellFilters;
    use mimir_dm_core::services::{CharacterService, ClassService, ItemService, SpellService};

    info!("Generating character sheet for character {}", character_id);

    // Get character data from database
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut char_service = CharacterService::new(&mut conn);
    let (_character, character_data) = char_service
        .get_character(character_id)
        .map_err(|e| format!("Failed to get character: {}", e))?;

    // Determine if we should include spell cards (default to true)
    let should_include_spells = include_spell_cards.unwrap_or(true);

    // Collect ALL spells for the character's class(es) up to their max spell level
    let mut spell_details: Vec<Spell> = Vec::new();

    if should_include_spells && !character_data.classes.is_empty() {
        let mut spell_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        // Process each class the character has
        for class_level in &character_data.classes {
            let max_spell_level = calculate_max_spell_level(&class_level.class_name, class_level.level);

            if max_spell_level > 0 {
                // Build filter for this class - include cantrips (0) through max spell level
                let levels: Vec<i32> = (0..=max_spell_level).collect();

                let filters = SpellFilters {
                    query: None,
                    levels,
                    schools: Vec::new(),
                    sources: Vec::new(),
                    tags: Vec::new(),
                    classes: vec![class_level.class_name.clone()],
                    limit: None,
                    offset: None,
                };

                // Search for all spells for this class
                match SpellService::search_spells(&mut spell_conn, filters) {
                    Ok(summaries) => {
                        info!(
                            "Found {} {} spells up to level {}",
                            summaries.len(),
                            class_level.class_name,
                            max_spell_level
                        );

                        // Fetch full spell details for each spell
                        for summary in summaries {
                            match SpellService::get_spell_details(&mut spell_conn, &summary.name, &summary.source) {
                                Ok(Some(spell)) => {
                                    spell_details.push(spell);
                                }
                                Ok(None) => {
                                    debug!("Spell not found in catalog: {} from {}", summary.name, summary.source);
                                }
                                Err(e) => {
                                    error!("Failed to fetch spell {}: {}", summary.name, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to search spells for class {}: {}", class_level.class_name, e);
                    }
                }
            } else {
                debug!("Class {} at level {} has no spellcasting", class_level.class_name, class_level.level);
            }
        }

        // Sort spells by level then name for consistent output
        spell_details.sort_by(|a, b| {
            a.level.cmp(&b.level).then_with(|| a.name.cmp(&b.name))
        });

        // Remove duplicates (in case of multiclass overlap)
        spell_details.dedup_by(|a, b| a.name == b.name && a.source == b.source);

        info!(
            "Fetched {} total spell details for character sheet",
            spell_details.len()
        );
    }

    // Fetch feature details from catalog
    let mut class_feature_details: Vec<ClassFeature> = Vec::new();
    let mut subclass_feature_details: Vec<SubclassFeature> = Vec::new();

    {
        let mut feature_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;
        let mut class_service = ClassService::new(&mut feature_conn);

        for feature_ref in &character_data.class_features {
            if let Some(ref subclass_name) = feature_ref.subclass_name {
                // Try to fetch as subclass feature
                match class_service.get_subclass_feature(
                    &feature_ref.name,
                    &feature_ref.class_name,
                    subclass_name,
                    &feature_ref.source,
                ) {
                    Ok(Some(feature)) => {
                        debug!(
                            "Fetched subclass feature details for: {} ({} {})",
                            feature_ref.name, feature_ref.class_name, subclass_name
                        );
                        subclass_feature_details.push(feature);
                    }
                    Ok(None) => {
                        debug!(
                            "Subclass feature not found in catalog: {} ({} {})",
                            feature_ref.name, feature_ref.class_name, subclass_name
                        );
                    }
                    Err(e) => {
                        error!("Failed to fetch subclass feature {}: {}", feature_ref.name, e);
                    }
                }
            } else {
                // Fetch as class feature
                match class_service.get_class_feature(
                    &feature_ref.name,
                    &feature_ref.class_name,
                    &feature_ref.source,
                ) {
                    Ok(Some(feature)) => {
                        debug!(
                            "Fetched class feature details for: {} ({})",
                            feature_ref.name, feature_ref.class_name
                        );
                        class_feature_details.push(feature);
                    }
                    Ok(None) => {
                        debug!(
                            "Class feature not found in catalog: {} ({})",
                            feature_ref.name, feature_ref.class_name
                        );
                    }
                    Err(e) => {
                        error!("Failed to fetch class feature {}: {}", feature_ref.name, e);
                    }
                }
            }
        }

        info!(
            "Fetched {} class features and {} subclass features for character sheet",
            class_feature_details.len(),
            subclass_feature_details.len()
        );
    }

    // Fetch item details from catalog for inventory items
    let mut item_details: Vec<Item> = Vec::new();
    {
        let mut item_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;
        let mut item_service = ItemService::new(&mut item_conn);

        for inventory_item in &character_data.inventory {
            let source = inventory_item.source.as_deref().unwrap_or("PHB");
            match item_service.get_item_by_name_and_source(&inventory_item.name, source) {
                Ok(Some(item)) => {
                    debug!(
                        "Fetched item details for: {} from {}",
                        inventory_item.name, source
                    );
                    item_details.push(item);
                }
                Ok(None) => {
                    debug!(
                        "Item not found in catalog: {} from {}",
                        inventory_item.name, source
                    );
                }
                Err(e) => {
                    error!("Failed to fetch item {}: {}", inventory_item.name, e);
                }
            }
        }

        info!(
            "Fetched {} item details for character sheet",
            item_details.len()
        );
    }

    // Convert character to JSON
    let character_json = serde_json::to_value(&character_data)
        .map_err(|e| format!("Failed to serialize character: {}", e))?;

    // Convert spells to JSON
    let spells_json = serde_json::to_value(&spell_details)
        .map_err(|e| format!("Failed to serialize spells: {}", e))?;

    // Convert features to JSON
    let class_features_json = serde_json::to_value(&class_feature_details)
        .map_err(|e| format!("Failed to serialize class features: {}", e))?;
    let subclass_features_json = serde_json::to_value(&subclass_feature_details)
        .map_err(|e| format!("Failed to serialize subclass features: {}", e))?;

    // Convert item details to JSON
    let item_details_json = serde_json::to_value(&item_details)
        .map_err(|e| format!("Failed to serialize item details: {}", e))?;

    // Build combined data structure
    let data = serde_json::json!({
        "character": character_json,
        "spells": spells_json,
        "class_features_details": class_features_json,
        "subclass_features_details": subclass_features_json,
        "item_details": item_details_json,
        "include_spell_cards": should_include_spells && !spell_details.is_empty()
    });

    // Always use the combined template which handles spells, equipment, and features
    // The template conditionally shows sections based on what data is available
    let template_id = template.unwrap_or_else(|| "character/sheet-with-spells".to_string());

    generate_pdf(template_id, data).await
}

/// Generate a spell card or list PDF.
///
/// # Parameters
/// - `template` - Template to use (card, list, cards-multiup)
/// - `spells` - Array of spell data
/// - `options` - Additional options (title, show_description, show_cut_lines)
#[tauri::command]
pub async fn generate_spell_pdf(
    template: String,
    spells: Vec<serde_json::Value>,
    options: Option<serde_json::Value>,
) -> Result<ApiResponse<PrintResult>, String> {
    info!("Generating spell PDF with template: {}", template);

    let template_id = format!("spells/{}", template);

    // Build data structure based on template
    let data = match template.as_str() {
        "card" => {
            // Single spell card - use first spell
            spells.into_iter().next().unwrap_or(serde_json::json!({}))
        }
        "list" => {
            let mut data = serde_json::json!({
                "spells": spells
            });
            if let Some(serde_json::Value::Object(opts_map)) = options {
                if let serde_json::Value::Object(ref mut data_map) = data {
                    for (k, v) in opts_map {
                        data_map.insert(k, v);
                    }
                }
            }
            data
        }
        "cards-multiup" => {
            let mut data = serde_json::json!({
                "spells": spells,
                "show_cut_lines": true
            });
            if let Some(serde_json::Value::Object(opts_map)) = options {
                if let serde_json::Value::Object(ref mut data_map) = data {
                    for (k, v) in opts_map {
                        data_map.insert(k, v);
                    }
                }
            }
            data
        }
        _ => serde_json::json!({ "spells": spells }),
    };

    generate_pdf(template_id, data).await
}

/// Generate a spell list PDF for a specific class.
///
/// Fetches all spells available to the specified class and generates
/// a formatted spell list PDF organized by level.
///
/// # Parameters
/// - `class_name` - Name of the class (e.g., "Wizard", "Cleric")
/// - `include_description` - Whether to include spell descriptions (default: false)
/// - `levels` - Optional array of levels to include (e.g., [0, 1, 2] for cantrips through 2nd level)
///
/// # Returns
/// PrintResult with base64-encoded PDF
#[tauri::command]
pub async fn generate_class_spell_list(
    state: State<'_, AppState>,
    class_name: String,
    include_description: Option<bool>,
    levels: Option<Vec<i32>>,
) -> Result<ApiResponse<PrintResult>, String> {
    use mimir_dm_core::models::catalog::SpellFilters;
    use mimir_dm_core::services::SpellService;

    info!("Generating spell list PDF for class: {}", class_name);

    // Fetch spells for the class
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let filters = SpellFilters {
        query: None,
        levels: levels.unwrap_or_default(),
        schools: Vec::new(),
        sources: Vec::new(),
        tags: Vec::new(),
        classes: vec![class_name.clone()],
        limit: None,
        offset: None,
    };

    let spells = SpellService::search_spells(&mut conn, filters)
        .map_err(|e| format!("Failed to search spells: {}", e))?;

    info!("Found {} spells for class {}", spells.len(), class_name);

    if spells.is_empty() {
        return Ok(ApiResponse::error(format!(
            "No spells found for class: {}",
            class_name
        )));
    }

    // Convert SpellSummary to JSON for template
    let spell_data: Vec<serde_json::Value> = spells
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "level": s.level,
                "school": s.school,
                "casting_time": s.casting_time,
                "range": s.range,
                "components": s.components,
                "concentration": s.concentration,
                "ritual": s.ritual,
                "description": s.description,
                "source": s.source
            })
        })
        .collect();

    // Build template data
    let data = serde_json::json!({
        "title": format!("{} Spells", class_name),
        "spells": spell_data,
        "show_description": include_description.unwrap_or(false)
    });

    generate_pdf("spells/list".to_string(), data).await
}

/// Generate a monster stat block or card PDF.
///
/// # Parameters
/// - `template` - Template to use (stat-block, card, encounter, cards-multiup)
/// - `monsters` - Array of monster data
/// - `options` - Additional options (title, notes, show_cut_lines)
#[tauri::command]
pub async fn generate_monster_pdf(
    template: String,
    monsters: Vec<serde_json::Value>,
    options: Option<serde_json::Value>,
) -> Result<ApiResponse<PrintResult>, String> {
    info!("Generating monster PDF with template: {}", template);

    let template_id = format!("monsters/{}", template);

    // Build data structure based on template
    let data = match template.as_str() {
        "stat-block" | "card" => {
            // Single monster - use first
            monsters.into_iter().next().unwrap_or(serde_json::json!({}))
        }
        "encounter" | "cards-multiup" => {
            let mut data = serde_json::json!({
                "monsters": monsters
            });
            if let Some(serde_json::Value::Object(opts_map)) = options {
                if let serde_json::Value::Object(ref mut data_map) = data {
                    for (k, v) in opts_map {
                        data_map.insert(k, v);
                    }
                }
            }
            data
        }
        _ => serde_json::json!({ "monsters": monsters }),
    };

    generate_pdf(template_id, data).await
}

/// Generate a session prep sheet or NPC card PDF.
///
/// # Parameters
/// - `template` - Template to use (prep, npc-card, npc-cards-multiup, handout)
/// - `data` - Session or NPC data
#[tauri::command]
pub async fn generate_session_pdf(
    template: String,
    data: serde_json::Value,
) -> Result<ApiResponse<PrintResult>, String> {
    info!("Generating session PDF with template: {}", template);

    let template_id = format!("session/{}", template);
    generate_pdf(template_id, data).await
}

/// Save a PDF to the file system.
///
/// # Parameters
/// - `pdf_base64` - Base64-encoded PDF data
/// - `path` - File path to save to
#[tauri::command]
pub async fn save_pdf(pdf_base64: String, path: String) -> Result<ApiResponse<()>, String> {
    info!("Saving PDF to: {}", path);

    let pdf_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &pdf_base64)
        .map_err(|e| format!("Failed to decode PDF: {}", e))?;

    std::fs::write(&path, &pdf_bytes).map_err(|e| format!("Failed to write file: {}", e))?;

    info!("PDF saved successfully ({} bytes)", pdf_bytes.len());
    Ok(ApiResponse::success(()))
}

/// Export a single campaign document to PDF.
///
/// Reads the markdown document from disk, converts to Typst, and generates PDF.
///
/// # Parameters
/// - `document_id` - Database ID of the document
///
/// # Returns
/// PrintResult with base64-encoded PDF
#[tauri::command]
pub async fn export_campaign_document(
    state: State<'_, AppState>,
    document_id: i32,
) -> Result<ApiResponse<PrintResult>, String> {
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::dal::campaign::documents::DocumentRepository;

    info!("Exporting campaign document {} to PDF", document_id);

    // Get the document from the database
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let document = DocumentRepository::find_by_id(&mut conn, document_id)
        .map_err(|e| format!("Failed to get document: {}", e))?;

    // Get the campaign name
    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .find_by_id(document.campaign_id)
        .map_err(|e| format!("Failed to get campaign: {}", e))?
        .ok_or_else(|| format!("Campaign {} not found", document.campaign_id))?;

    // Create the print service and render
    let service = create_print_service();
    let file_path = std::path::PathBuf::from(&document.file_path);

    match service.render_campaign_document(&file_path, Some(&campaign.name)) {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &pdf_bytes,
            );

            info!(
                "Campaign document PDF generated successfully ({} bytes)",
                size_bytes
            );

            Ok(ApiResponse::success(PrintResult {
                pdf_base64,
                size_bytes,
            }))
        }
        Err(e) => {
            error!("Failed to generate campaign document PDF: {:?}", e);
            Ok(ApiResponse::error(format!(
                "Failed to generate PDF: {}",
                e
            )))
        }
    }
}

/// Document data for PDF export (parsed from markdown)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExportData {
    /// Document title
    pub title: String,
    /// Document type (e.g., "session_outline")
    pub document_type: String,
    /// Typst content (converted from markdown)
    pub content: String,
}

/// Module data for PDF export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleExportData {
    /// Module name
    pub name: String,
    /// Module number
    pub module_number: i32,
    /// Documents belonging to this module
    pub documents: Vec<DocumentExportData>,
    /// Monsters in this module (full JSON data)
    pub monsters: Vec<serde_json::Value>,
}

/// Strip 5etools tags from text content.
///
/// Converts tags like `{@item leather armor|phb}` to "leather armor",
/// `{@atk mw}` to "Melee Weapon Attack", etc.
fn strip_5etools_tags(text: &str) -> String {
    use regex::Regex;

    // Lazy static would be better, but for simplicity we'll create regex on each call
    // This is called infrequently (only during PDF export)

    let mut result = text.to_string();

    // Pattern: {@tag content|source} or {@tag content}
    // We need to handle nested tags and various formats

    // Attack type tags
    let atk_re = Regex::new(r"\{@atk\s+([^}]+)\}").unwrap();
    result = atk_re.replace_all(&result, |caps: &regex::Captures| {
        match caps[1].trim() {
            "mw" => "Melee Weapon Attack:".to_string(),
            "rw" => "Ranged Weapon Attack:".to_string(),
            "ms" => "Melee Spell Attack:".to_string(),
            "rs" => "Ranged Spell Attack:".to_string(),
            "mw,rw" | "rw,mw" => "Melee or Ranged Weapon Attack:".to_string(),
            other => other.to_string(),
        }
    }).to_string();

    // Hit bonus: {@hit 4} -> "+4"
    let hit_re = Regex::new(r"\{@hit\s+(\d+)\}").unwrap();
    result = hit_re.replace_all(&result, "+$1").to_string();

    // Damage: {@damage 1d6+2} -> "1d6+2"
    let damage_re = Regex::new(r"\{@damage\s+([^}]+)\}").unwrap();
    result = damage_re.replace_all(&result, "$1").to_string();

    // DC: {@dc 13} -> "DC 13"
    let dc_re = Regex::new(r"\{@dc\s+(\d+)\}").unwrap();
    result = dc_re.replace_all(&result, "DC $1").to_string();

    // Dice: {@dice 1d6} -> "1d6"
    let dice_re = Regex::new(r"\{@dice\s+([^}]+)\}").unwrap();
    result = dice_re.replace_all(&result, "$1").to_string();

    // Condition: {@condition poisoned} -> "poisoned"
    let condition_re = Regex::new(r"\{@condition\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = condition_re.replace_all(&result, "$1").to_string();

    // Skill: {@skill Perception} -> "Perception"
    let skill_re = Regex::new(r"\{@skill\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = skill_re.replace_all(&result, "$1").to_string();

    // Item: {@item leather armor|phb} -> "leather armor"
    let item_re = Regex::new(r"\{@item\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = item_re.replace_all(&result, "$1").to_string();

    // Creature: {@creature goblin|mm} -> "goblin"
    let creature_re = Regex::new(r"\{@creature\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = creature_re.replace_all(&result, "$1").to_string();

    // Spell: {@spell fireball|phb} -> "fireball"
    let spell_re = Regex::new(r"\{@spell\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = spell_re.replace_all(&result, "$1").to_string();

    // Sense: {@sense darkvision} -> "darkvision"
    let sense_re = Regex::new(r"\{@sense\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = sense_re.replace_all(&result, "$1").to_string();

    // Status: {@status unconscious} -> "unconscious"
    let status_re = Regex::new(r"\{@status\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = status_re.replace_all(&result, "$1").to_string();

    // Recharge: {@recharge 5} -> "(Recharge 5-6)"
    let recharge_re = Regex::new(r"\{@recharge\s*(\d*)\}").unwrap();
    result = recharge_re.replace_all(&result, |caps: &regex::Captures| {
        if caps.get(1).is_none_or(|m| m.as_str().is_empty()) {
            "(Recharge)".to_string()
        } else {
            format!("(Recharge {}-6)", &caps[1])
        }
    }).to_string();

    // h (hit points): {@h} -> "Hit: "
    let h_re = Regex::new(r"\{@h\}").unwrap();
    result = h_re.replace_all(&result, "Hit: ").to_string();

    // Filter: {@filter ...} -> just the text before the pipe
    let filter_re = Regex::new(r"\{@filter\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = filter_re.replace_all(&result, "$1").to_string();

    // Quickref: {@quickref ...} -> just the display text
    let quickref_re = Regex::new(r"\{@quickref\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = quickref_re.replace_all(&result, "$1").to_string();

    // Action: {@action Dodge} -> "Dodge"
    let action_re = Regex::new(r"\{@action\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = action_re.replace_all(&result, "$1").to_string();

    // Chance: {@chance 50} -> "50%"
    let chance_re = Regex::new(r"\{@chance\s+(\d+)(?:\|[^}]*)?\}").unwrap();
    result = chance_re.replace_all(&result, "$1%").to_string();

    // Scaledamage/scaledice: {@scaledamage 1d6|...} -> "1d6"
    let scaledamage_re = Regex::new(r"\{@scaledamage\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = scaledamage_re.replace_all(&result, "$1").to_string();
    let scaledice_re = Regex::new(r"\{@scaledice\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = scaledice_re.replace_all(&result, "$1").to_string();

    // Generic fallback for any remaining {@tag content|source} patterns
    // Extract just the content before the pipe
    let generic_re = Regex::new(r"\{@\w+\s+([^|}]+)(?:\|[^}]*)?\}").unwrap();
    result = generic_re.replace_all(&result, "$1").to_string();

    result
}

/// Recursively process a JSON value and strip 5etools tags from all string values.
fn strip_5etools_tags_from_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            *s = strip_5etools_tags(s);
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                strip_5etools_tags_from_json(item);
            }
        }
        serde_json::Value::Object(obj) => {
            for (_, v) in obj {
                strip_5etools_tags_from_json(v);
            }
        }
        _ => {}
    }
}

/// Export all campaign documents as a combined PDF.
///
/// Reads all markdown documents for the campaign, converts to Typst,
/// and generates a single PDF with cover page and table of contents.
/// Campaign-level documents appear first, then each module with its
/// documents and monsters together.
///
/// # Parameters
/// - `campaign_id` - Database ID of the campaign
///
/// # Returns
/// PrintResult with base64-encoded PDF
#[tauri::command]
pub async fn export_campaign_documents(
    state: State<'_, AppState>,
    campaign_id: i32,
) -> Result<ApiResponse<PrintResult>, String> {
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::dal::campaign::documents::DocumentRepository;
    use mimir_dm_core::models::catalog::class::{ClassFeature, SubclassFeature};
    use mimir_dm_core::services::{CharacterService, ClassService, DocumentService, ModuleMonsterService, ModuleService};
    use mimir_dm_print::markdown::parse_campaign_document;

    info!("Exporting all campaign {} documents to PDF", campaign_id);

    // Get the campaign
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut campaign_repo = CampaignRepository::new(&mut conn);
    let campaign = campaign_repo
        .find_by_id(campaign_id)
        .map_err(|e| format!("Failed to get campaign: {}", e))?
        .ok_or_else(|| format!("Campaign {} not found", campaign_id))?;

    // Get all documents for the campaign
    let mut doc_conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut doc_service = DocumentService::new(&mut doc_conn);
    let all_documents = doc_service
        .get_campaign_documents(campaign_id)
        .map_err(|e| format!("Failed to get documents: {}", e))?;

    // Separate campaign-level documents (no module_id) from module documents
    let campaign_documents: Vec<_> = all_documents
        .iter()
        .filter(|d| d.module_id.is_none())
        .collect();

    // Sort campaign documents according to campaign construction flow
    // Includes both required and optional documents from all stages
    let document_order: Vec<&str> = vec![
        // Concept stage
        "campaign_pitch",
        // Session Zero stage (required)
        "starting_scenario",
        "world_primer",
        "character_guidelines",
        "table_expectations",
        "character_integration",
        // Session Zero stage (optional)
        "safety_tools",
        "house_rules",
        // Integration stage (required)
        "campaign_bible",
        // Integration stage (optional)
        "player_secrets",
        "faction_overview",
        // Legacy document types
        "major_npc_tracker",
    ];

    let get_order = |template_id: &str| -> usize {
        document_order
            .iter()
            .position(|&t| t == template_id)
            .unwrap_or(usize::MAX)
    };

    let mut sorted_campaign_docs = campaign_documents.clone();
    sorted_campaign_docs.sort_by(|a, b| {
        let a_order = get_order(&a.template_id);
        let b_order = get_order(&b.template_id);
        match a_order.cmp(&b_order) {
            std::cmp::Ordering::Equal => a.created_at.cmp(&b.created_at),
            other => other,
        }
    });

    // Collect file paths for campaign-level documents
    let campaign_file_paths: Vec<std::path::PathBuf> = sorted_campaign_docs
        .iter()
        .filter_map(|doc| {
            let path = std::path::PathBuf::from(&doc.file_path);
            if path.exists() {
                Some(path)
            } else {
                debug!("Skipping non-existent document file: {:?}", path);
                None
            }
        })
        .collect();

    // Get modules with their documents and monsters
    let mut module_conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut module_service = ModuleService::new(&mut module_conn);
    let modules = module_service
        .list_campaign_modules(campaign_id)
        .map_err(|e| format!("Failed to get modules: {}", e))?;

    let mut module_export_data: Vec<ModuleExportData> = Vec::new();

    for module in modules {
        // Get module documents
        let mut module_doc_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let module_docs = DocumentRepository::find_by_module(&mut module_doc_conn, module.id)
            .map_err(|e| format!("Failed to get documents for module {}: {}", module.id, e))?;

        // Parse module documents
        let mut parsed_module_docs: Vec<DocumentExportData> = Vec::new();
        for doc in module_docs {
            let path = std::path::PathBuf::from(&doc.file_path);
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(markdown) => {
                        match parse_campaign_document(&markdown) {
                            Ok(parsed) => {
                                let title = parsed
                                    .frontmatter
                                    .get("title")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&doc.title)
                                    .to_string();

                                let document_type = parsed
                                    .frontmatter
                                    .get("type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&doc.document_type)
                                    .to_string();

                                parsed_module_docs.push(DocumentExportData {
                                    title,
                                    document_type,
                                    content: parsed.typst_content,
                                });
                            }
                            Err(e) => {
                                debug!("Failed to parse document {}: {}", doc.title, e);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Failed to read document file {:?}: {}", path, e);
                    }
                }
            }
        }

        // Check for play-notes.md file (created during play mode)
        let play_notes_path = std::path::PathBuf::from(&campaign.directory_path)
            .join("modules")
            .join(format!("module_{:02}", module.module_number))
            .join("play-notes.md");

        if play_notes_path.exists() {
            match std::fs::read_to_string(&play_notes_path) {
                Ok(markdown) => {
                    // Try to parse as a document with frontmatter
                    match parse_campaign_document(&markdown) {
                        Ok(parsed) => {
                            let title = parsed
                                .frontmatter
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Play Notes")
                                .to_string();

                            parsed_module_docs.push(DocumentExportData {
                                title,
                                document_type: "session_notes".to_string(),
                                content: parsed.typst_content,
                            });
                            debug!("Added play-notes.md for module {}", module.module_number);
                        }
                        Err(_) => {
                            // If no frontmatter, treat as raw markdown
                            let typst_content = mimir_dm_print::markdown::markdown_to_typst(&markdown);
                            parsed_module_docs.push(DocumentExportData {
                                title: "Play Notes".to_string(),
                                document_type: "session_notes".to_string(),
                                content: typst_content,
                            });
                            debug!("Added raw play-notes.md for module {}", module.module_number);
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to read play-notes.md: {}", e);
                }
            }
        }

        // Get module monsters
        let mut monster_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut monster_service = ModuleMonsterService::new(&mut monster_conn);
        let monsters_with_data = monster_service
            .get_monsters_with_data(module.id)
            .map_err(|e| format!("Failed to get monsters for module {}: {}", module.id, e))?;

        // Convert to JSON values
        let monster_json: Vec<serde_json::Value> = monsters_with_data
            .into_iter()
            .map(|m| {
                if let Some(data) = m.monster_data {
                    let mut monster = data;
                    if let serde_json::Value::Object(ref mut obj) = monster {
                        obj.insert(
                            "quantity".to_string(),
                            serde_json::Value::Number(m.quantity.into()),
                        );
                        if let Some(tag) = &m.encounter_tag {
                            obj.insert(
                                "encounter_tag".to_string(),
                                serde_json::Value::String(tag.clone()),
                            );
                        }
                    }
                    strip_5etools_tags_from_json(&mut monster);
                    monster
                } else {
                    serde_json::json!({
                        "name": m.monster_name,
                        "source": m.monster_source,
                        "quantity": m.quantity,
                        "encounter_tag": m.encounter_tag
                    })
                }
            })
            .collect();

        // Only include modules that have documents or monsters
        if !parsed_module_docs.is_empty() || !monster_json.is_empty() {
            module_export_data.push(ModuleExportData {
                name: module.name.clone(),
                module_number: module.module_number,
                documents: parsed_module_docs,
                monsters: monster_json,
            });
        }
    }

    // Sort modules by module_number
    module_export_data.sort_by_key(|m| m.module_number);

    // Fetch NPCs for the campaign
    let mut npc_conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut char_service = CharacterService::new(&mut npc_conn);
    let campaign_characters = char_service
        .list_characters_for_campaign(campaign_id)
        .map_err(|e| format!("Failed to get characters: {}", e))?;

    // Filter to only NPCs and fetch their data
    let npc_characters: Vec<_> = campaign_characters
        .into_iter()
        .filter(|c| c.is_npc())
        .collect();

    // Fetch character data for each NPC and convert to JSON
    let mut npcs_json: Vec<serde_json::Value> = Vec::new();
    for npc in npc_characters {
        let mut npc_data_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut npc_service = CharacterService::new(&mut npc_data_conn);
        match npc_service.get_character(npc.id) {
            Ok((_character, character_data)) => {
                // Fetch feature details from catalog
                let mut class_feature_details: Vec<ClassFeature> = Vec::new();
                let mut subclass_feature_details: Vec<SubclassFeature> = Vec::new();

                {
                    let mut feature_conn = state
                        .db
                        .get_connection()
                        .map_err(|e| format!("Database error: {}", e))?;
                    let mut class_service = ClassService::new(&mut feature_conn);

                    for feature_ref in &character_data.class_features {
                        if let Some(ref subclass_name) = feature_ref.subclass_name {
                            // Try to fetch as subclass feature
                            if let Ok(Some(feature)) = class_service.get_subclass_feature(
                                &feature_ref.name,
                                &feature_ref.class_name,
                                subclass_name,
                                &feature_ref.source,
                            ) {
                                subclass_feature_details.push(feature);
                            }
                        } else {
                            // Fetch as class feature
                            if let Ok(Some(feature)) = class_service.get_class_feature(
                                &feature_ref.name,
                                &feature_ref.class_name,
                                &feature_ref.source,
                            ) {
                                class_feature_details.push(feature);
                            }
                        }
                    }
                }

                // Convert to JSON
                let npc_json = serde_json::to_value(&character_data)
                    .map_err(|e| format!("Failed to serialize NPC {}: {}", npc.character_name, e))?;

                let class_features_json = serde_json::to_value(&class_feature_details)
                    .map_err(|e| format!("Failed to serialize class features: {}", e))?;
                let subclass_features_json = serde_json::to_value(&subclass_feature_details)
                    .map_err(|e| format!("Failed to serialize subclass features: {}", e))?;

                // Merge feature details into NPC data
                let mut npc_data = npc_json;
                if let serde_json::Value::Object(ref mut map) = npc_data {
                    map.insert("class_features_details".to_string(), class_features_json);
                    map.insert("subclass_features_details".to_string(), subclass_features_json);
                }

                npcs_json.push(npc_data);
            }
            Err(e) => {
                debug!("Failed to get NPC data for {}: {}", npc.character_name, e);
            }
        }
    }

    // Fetch maps and tokens for the campaign
    let mut maps_data: Vec<(mimir_dm_print::RenderMap, Vec<mimir_dm_print::RenderToken>)> = Vec::new();
    {
        use mimir_dm_core::services::{MapService, TokenService};

        let mut map_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut map_service = MapService::new(&mut map_conn);
        let maps = map_service
            .list_campaign_maps(campaign_id)
            .map_err(|e| format!("Failed to get maps: {}", e))?;

        for map_summary in maps {
            // Get full map details
            let mut detail_conn = state
                .db
                .get_connection()
                .map_err(|e| format!("Database error: {}", e))?;

            let mut detail_service = MapService::new(&mut detail_conn);
            if let Ok(Some(map)) = detail_service.get_map(map_summary.id) {
                // Get tokens for this map
                let mut token_conn = state
                    .db
                    .get_connection()
                    .map_err(|e| format!("Database error: {}", e))?;

                let mut token_service = TokenService::new(&mut token_conn);
                let tokens = token_service
                    .list_tokens_for_map(map.id)
                    .unwrap_or_default();

                // Convert to RenderMap
                let render_map = mimir_dm_print::RenderMap {
                    name: map.name.clone(),
                    image_path: map.image_path.clone(),
                    width_px: map.width_px,
                    height_px: map.height_px,
                    grid_type: map.grid_type.clone(),
                    grid_size_px: map.grid_size_px,
                    grid_offset_x: map.grid_offset_x,
                    grid_offset_y: map.grid_offset_y,
                };

                // Convert tokens to RenderToken
                let render_tokens: Vec<mimir_dm_print::RenderToken> = tokens
                    .into_iter()
                    .map(|t| mimir_dm_print::RenderToken {
                        name: t.name,
                        x: t.x,
                        y: t.y,
                        size: t.size,
                        color: t.color,
                        token_type: t.token_type,
                    })
                    .collect();

                maps_data.push((render_map, render_tokens));
            }
        }
    }

    info!(
        "Rendering {} campaign documents, {} modules, {} NPCs, and {} maps for campaign '{}'",
        campaign_file_paths.len(),
        module_export_data.len(),
        npcs_json.len(),
        maps_data.len(),
        campaign.name
    );

    // Create the print service and render combined PDF
    let service = create_print_service();

    // Convert module data to JSON
    let modules_json = serde_json::to_value(&module_export_data)
        .map_err(|e| format!("Failed to serialize module data: {}", e))?;

    // Convert NPCs to JSON value
    let npcs_value = serde_json::Value::Array(npcs_json);

    // Get the campaign base path for resolving map images
    let campaign_base_path = std::path::PathBuf::from(&campaign.directory_path);

    match service.render_campaign_combined_with_all(&campaign_file_paths, &campaign.name, modules_json, npcs_value, maps_data, &campaign_base_path) {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &pdf_bytes,
            );

            info!(
                "Combined campaign PDF generated successfully ({} bytes, {} campaign docs, {} modules)",
                size_bytes,
                campaign_file_paths.len(),
                module_export_data.len()
            );

            Ok(ApiResponse::success(PrintResult {
                pdf_base64,
                size_bytes,
            }))
        }
        Err(e) => {
            error!("Failed to generate combined campaign PDF: {:?}", e);
            Ok(ApiResponse::error(format!(
                "Failed to generate PDF: {}",
                e
            )))
        }
    }
}

/// Export a single module's documents and monsters as PDF.
///
/// # Parameters
/// - `module_id` - Database ID of the module
///
/// # Returns
/// PrintResult with base64-encoded PDF
#[tauri::command]
pub async fn export_module_documents(
    state: State<'_, AppState>,
    module_id: i32,
) -> Result<ApiResponse<PrintResult>, String> {
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::dal::campaign::documents::DocumentRepository;
    use mimir_dm_core::services::{ModuleMonsterService, ModuleService};
    use mimir_dm_print::markdown::parse_campaign_document;

    info!("Exporting module {} documents to PDF", module_id);

    // Get the module
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut module_service = ModuleService::new(&mut conn);
    let module = module_service
        .get_module(module_id)
        .map_err(|e| format!("Failed to get module: {}", e))?
        .ok_or_else(|| format!("Module {} not found", module_id))?;

    // Get the campaign for directory path
    let mut campaign_conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut campaign_repo = CampaignRepository::new(&mut campaign_conn);
    let campaign = campaign_repo
        .find_by_id(module.campaign_id)
        .map_err(|e| format!("Failed to get campaign: {}", e))?
        .ok_or_else(|| format!("Campaign {} not found", module.campaign_id))?;

    // Get module documents
    let mut doc_conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let module_docs = DocumentRepository::find_by_module(&mut doc_conn, module_id)
        .map_err(|e| format!("Failed to get documents for module {}: {}", module_id, e))?;

    // Parse module documents
    let mut parsed_module_docs: Vec<DocumentExportData> = Vec::new();
    for doc in module_docs {
        let path = std::path::PathBuf::from(&doc.file_path);
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(markdown) => {
                    match parse_campaign_document(&markdown) {
                        Ok(parsed) => {
                            let title = parsed
                                .frontmatter
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&doc.title)
                                .to_string();

                            let document_type = parsed
                                .frontmatter
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&doc.document_type)
                                .to_string();

                            parsed_module_docs.push(DocumentExportData {
                                title,
                                document_type,
                                content: parsed.typst_content,
                            });
                        }
                        Err(e) => {
                            debug!("Failed to parse document {}: {}", doc.title, e);
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to read document file {:?}: {}", path, e);
                }
            }
        }
    }

    // Check for play-notes.md file (created during play mode)
    let play_notes_path = std::path::PathBuf::from(&campaign.directory_path)
        .join("modules")
        .join(format!("module_{:02}", module.module_number))
        .join("play-notes.md");

    if play_notes_path.exists() {
        match std::fs::read_to_string(&play_notes_path) {
            Ok(markdown) => {
                // Try to parse as a document with frontmatter
                match parse_campaign_document(&markdown) {
                    Ok(parsed) => {
                        let title = parsed
                            .frontmatter
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Play Notes")
                            .to_string();

                        parsed_module_docs.push(DocumentExportData {
                            title,
                            document_type: "session_notes".to_string(),
                            content: parsed.typst_content,
                        });
                        debug!("Added play-notes.md for module {}", module.module_number);
                    }
                    Err(_) => {
                        // If no frontmatter, treat as raw markdown
                        let typst_content = mimir_dm_print::markdown::markdown_to_typst(&markdown);
                        parsed_module_docs.push(DocumentExportData {
                            title: "Play Notes".to_string(),
                            document_type: "session_notes".to_string(),
                            content: typst_content,
                        });
                        debug!("Added raw play-notes.md for module {}", module.module_number);
                    }
                }
            }
            Err(e) => {
                debug!("Failed to read play-notes.md: {}", e);
            }
        }
    }

    // Get module monsters
    let mut monster_conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut monster_service = ModuleMonsterService::new(&mut monster_conn);
    let monsters_with_data = monster_service
        .get_monsters_with_data(module_id)
        .map_err(|e| format!("Failed to get monsters for module {}: {}", module_id, e))?;

    // Convert to JSON values
    let monster_json: Vec<serde_json::Value> = monsters_with_data
        .into_iter()
        .map(|m| {
            if let Some(data) = m.monster_data {
                let mut monster = data;
                if let serde_json::Value::Object(ref mut obj) = monster {
                    obj.insert(
                        "quantity".to_string(),
                        serde_json::Value::Number(m.quantity.into()),
                    );
                    if let Some(tag) = &m.encounter_tag {
                        obj.insert(
                            "encounter_tag".to_string(),
                            serde_json::Value::String(tag.clone()),
                        );
                    }
                }
                strip_5etools_tags_from_json(&mut monster);
                monster
            } else {
                serde_json::json!({
                    "name": m.monster_name,
                    "source": m.monster_source,
                    "quantity": m.quantity,
                    "encounter_tag": m.encounter_tag
                })
            }
        })
        .collect();

    // Build module export data
    let module_export = ModuleExportData {
        name: module.name.clone(),
        module_number: module.module_number,
        documents: parsed_module_docs,
        monsters: monster_json,
    };

    info!(
        "Rendering module '{}' with {} documents and {} monsters",
        module.name,
        module_export.documents.len(),
        module_export.monsters.len()
    );

    // Create the print service and render PDF
    let service = create_print_service();

    // Convert module data to JSON - pass as single-element array so template can reuse same logic
    let modules_json = serde_json::to_value(vec![module_export])
        .map_err(|e| format!("Failed to serialize module data: {}", e))?;

    // Use the combined template with no campaign docs, just the module
    match service.render_campaign_combined_with_monsters(&[], &module.name, modules_json) {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &pdf_bytes,
            );

            info!(
                "Module PDF generated successfully ({} bytes)",
                size_bytes
            );

            Ok(ApiResponse::success(PrintResult {
                pdf_base64,
                size_bytes,
            }))
        }
        Err(e) => {
            error!("Failed to generate module PDF: {:?}", e);
            Ok(ApiResponse::error(format!(
                "Failed to generate PDF: {}",
                e
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_5etools_attack_tags() {
        assert_eq!(strip_5etools_tags("{@atk mw}"), "Melee Weapon Attack:");
        assert_eq!(strip_5etools_tags("{@atk rw}"), "Ranged Weapon Attack:");
        assert_eq!(strip_5etools_tags("{@atk ms}"), "Melee Spell Attack:");
        assert_eq!(strip_5etools_tags("{@atk rs}"), "Ranged Spell Attack:");
        assert_eq!(strip_5etools_tags("{@atk mw,rw}"), "Melee or Ranged Weapon Attack:");
    }

    #[test]
    fn test_strip_5etools_hit_and_damage() {
        assert_eq!(strip_5etools_tags("{@hit 4}"), "+4");
        assert_eq!(strip_5etools_tags("{@hit 12}"), "+12");
        assert_eq!(strip_5etools_tags("{@damage 1d6+2}"), "1d6+2");
        assert_eq!(strip_5etools_tags("{@damage 2d8}"), "2d8");
    }

    #[test]
    fn test_strip_5etools_dc_and_dice() {
        assert_eq!(strip_5etools_tags("{@dc 13}"), "DC 13");
        assert_eq!(strip_5etools_tags("{@dc 15}"), "DC 15");
        assert_eq!(strip_5etools_tags("{@dice 1d6}"), "1d6");
        assert_eq!(strip_5etools_tags("{@dice 2d10+5}"), "2d10+5");
    }

    #[test]
    fn test_strip_5etools_condition_and_item() {
        assert_eq!(strip_5etools_tags("{@condition poisoned}"), "poisoned");
        assert_eq!(strip_5etools_tags("{@condition frightened|PHB}"), "frightened");
        assert_eq!(strip_5etools_tags("{@item leather armor|phb}"), "leather armor");
        assert_eq!(strip_5etools_tags("{@item longsword}"), "longsword");
    }

    #[test]
    fn test_strip_5etools_creature_and_spell() {
        assert_eq!(strip_5etools_tags("{@creature goblin|mm}"), "goblin");
        assert_eq!(strip_5etools_tags("{@spell fireball|phb}"), "fireball");
        assert_eq!(strip_5etools_tags("{@spell magic missile}"), "magic missile");
    }

    #[test]
    fn test_strip_5etools_recharge() {
        assert_eq!(strip_5etools_tags("{@recharge 5}"), "(Recharge 5-6)");
        assert_eq!(strip_5etools_tags("{@recharge 6}"), "(Recharge 6-6)");
        assert_eq!(strip_5etools_tags("{@recharge}"), "(Recharge)");
    }

    #[test]
    fn test_strip_5etools_h_tag() {
        assert_eq!(strip_5etools_tags("{@h}"), "Hit: ");
    }

    #[test]
    fn test_strip_5etools_complex_text() {
        let input = "{@atk mw} {@hit 4} to hit, reach 5 ft., one target. {@h}{@damage 1d6+2} slashing damage.";
        let expected = "Melee Weapon Attack: +4 to hit, reach 5 ft., one target. Hit: 1d6+2 slashing damage.";
        assert_eq!(strip_5etools_tags(input), expected);
    }

    #[test]
    fn test_strip_5etools_ac_with_item() {
        let input = "15 ({@item leather armor|phb}, {@item shield|phb})";
        let expected = "15 (leather armor, shield)";
        assert_eq!(strip_5etools_tags(input), expected);
    }

    #[test]
    fn test_strip_5etools_skill_and_action() {
        assert_eq!(strip_5etools_tags("{@skill Perception}"), "Perception");
        assert_eq!(strip_5etools_tags("{@skill Stealth|PHB}"), "Stealth");
        assert_eq!(strip_5etools_tags("{@action Dodge}"), "Dodge");
    }

    #[test]
    fn test_strip_5etools_preserves_plain_text() {
        let input = "The goblin can take the Disengage or Hide action as a bonus action.";
        assert_eq!(strip_5etools_tags(input), input);
    }
}
