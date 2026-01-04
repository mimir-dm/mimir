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

/// Options for printing a map
///
/// Both preview and play sections can be included in a single PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPrintOptions {
    // Preview section
    /// Include preview page (fit to single page)
    #[serde(default = "default_true")]
    pub include_preview: bool,
    /// Show grid overlay on preview
    #[serde(default)]
    pub preview_grid: bool,
    /// Show LOS walls on preview
    #[serde(default)]
    pub preview_los_walls: bool,
    /// Show starting positions on preview
    #[serde(default)]
    pub preview_positions: bool,

    // Play section
    /// Include play tiles (1"=5ft scale)
    #[serde(default)]
    pub include_play: bool,
    /// Show grid overlay on tiles
    #[serde(default)]
    pub play_grid: bool,
    /// Show LOS walls on tiles
    #[serde(default)]
    pub play_los_walls: bool,
    /// Include token cutout sheets
    #[serde(default)]
    pub play_cutouts: bool,
}

fn default_true() -> bool {
    true
}

impl Default for MapPrintOptions {
    fn default() -> Self {
        Self {
            include_preview: true,
            preview_grid: false,
            preview_los_walls: false,
            preview_positions: false,
            include_play: false,
            play_grid: false,
            play_los_walls: false,
            play_cutouts: false,
        }
    }
}

/// Options for exporting a module to PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleExportOptions {
    // Content section
    /// Include module documents and notes (default: true)
    #[serde(default = "default_true")]
    pub include_documents: bool,
    /// Include monster stat blocks for tagged monsters (default: true)
    #[serde(default = "default_true")]
    pub include_monsters: bool,
    /// Include campaign NPC sheets (default: false)
    #[serde(default)]
    pub include_npcs: bool,

    // Map Preview section
    /// Include map previews (fit to single page) (default: true)
    #[serde(default = "default_true")]
    pub include_preview: bool,
    /// Show grid overlay on preview (default: true)
    #[serde(default = "default_true")]
    pub preview_grid: bool,
    /// Show LOS walls on preview (default: false)
    #[serde(default)]
    pub preview_los_walls: bool,
    /// Show starting positions on preview (default: false)
    #[serde(default)]
    pub preview_positions: bool,

    // Map Play section
    /// Include play tiles (1"=5ft scale) (default: false)
    #[serde(default)]
    pub include_play: bool,
    /// Show grid overlay on tiles (default: true)
    #[serde(default = "default_true")]
    pub play_grid: bool,
    /// Show LOS walls on tiles (default: false)
    #[serde(default)]
    pub play_los_walls: bool,
    /// Include token cutout sheets (default: true when play enabled)
    #[serde(default = "default_true")]
    pub play_cutouts: bool,
}

impl Default for ModuleExportOptions {
    fn default() -> Self {
        Self {
            include_documents: true,
            include_monsters: true,
            include_npcs: false,
            include_preview: true,
            preview_grid: true,
            preview_los_walls: false,
            preview_positions: false,
            include_play: false,
            play_grid: true,
            play_los_walls: false,
            play_cutouts: true,
        }
    }
}

/// Options for exporting a campaign to PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignExportOptions {
    // Reference Document options
    /// Include campaign-level documents (default: true)
    #[serde(default = "default_true")]
    pub include_campaign_docs: bool,
    /// Include module content (documents and monsters) (default: true)
    #[serde(default = "default_true")]
    pub include_module_content: bool,
    /// Include campaign NPC sheets (default: false)
    #[serde(default)]
    pub include_npcs: bool,

    // Module Maps options (sub-options for module content)
    /// Include module map previews scaled to fit one page (default: false)
    #[serde(default)]
    pub include_module_map_previews: bool,
    /// Include module maps at 1"=5ft scale for tabletop play (default: false)
    #[serde(default)]
    pub include_module_tiled_maps: bool,
    /// Include printable paper standees for module tokens (default: false)
    #[serde(default)]
    pub include_token_cutouts: bool,

    // Campaign Maps options
    /// Include campaign map previews scaled to fit one page (default: true)
    #[serde(default = "default_true")]
    pub include_campaign_map_previews: bool,
    /// Include campaign maps at 1"=5ft scale for tabletop play (default: false)
    #[serde(default)]
    pub include_campaign_tiled_maps: bool,
}

impl Default for CampaignExportOptions {
    fn default() -> Self {
        Self {
            include_campaign_docs: true,
            include_module_content: true,
            include_npcs: false,
            include_module_map_previews: false,
            include_module_tiled_maps: false,
            include_token_cutouts: false,
            include_campaign_map_previews: true,
            include_campaign_tiled_maps: false,
        }
    }
}

/// Convert a database token to a RenderToken, handling WebP to PNG conversion.
///
/// Token images stored as WebP need to be converted to PNG for Typst compatibility.
/// Converted images are cached in a temp directory.
fn convert_token_to_render_token(
    token: mimir_dm_core::models::campaign::tokens::Token,
    books_dir: &std::path::Path,
    temp_dir: &std::path::Path,
) -> mimir_dm_print::RenderToken {
    let resolved_image_path = token.image_path.and_then(|p| {
        // Extract source from path (e.g., "img/bestiary/tokens/MM/Goblin.webp" -> "MM")
        let source = p.split('/').nth(3).unwrap_or("MM");
        let full_path = books_dir.join(source).join(&p);
        if full_path.exists() {
            // Check if it's a webp - Typst doesn't support webp, convert to PNG
            if p.ends_with(".webp") {
                // Convert webp to png in temp directory
                let png_name = format!("{}_{}.png", source, p.replace('/', "_").replace(".webp", ""));
                let png_path = temp_dir.join(&png_name);

                // Only convert if not already cached
                if !png_path.exists() {
                    match image::open(&full_path) {
                        Ok(img) => {
                            if let Err(e) = img.save_with_format(&png_path, image::ImageFormat::Png) {
                                debug!("Failed to convert webp to png: {}", e);
                                return None;
                            }
                        }
                        Err(e) => {
                            debug!("Failed to open webp image: {}", e);
                            return None;
                        }
                    }
                }
                Some(png_path.to_string_lossy().to_string())
            } else {
                Some(full_path.to_string_lossy().to_string())
            }
        } else {
            debug!("Token image not found: {:?}", full_path);
            None
        }
    });

    mimir_dm_print::RenderToken {
        name: token.name,
        x: token.x,
        y: token.y,
        size: token.size,
        color: token.color,
        token_type: token.token_type,
        image_path: resolved_image_path,
    }
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
    _template: Option<String>,
    include_spell_cards: Option<bool>,
) -> Result<ApiResponse<PrintResult>, String> {
    use mimir_dm_core::models::catalog::Spell;
    use mimir_dm_core::models::catalog::SpellFilters;
    use mimir_dm_core::services::{CharacterService, SpellService};

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

    // Build PDF directly using the core function
    use mimir_dm_print::generate_character_sheet_pdf;

    let templates_root = get_templates_root();

    match generate_character_sheet_pdf(character_data, spell_details, templates_root) {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &pdf_bytes,
            );
            info!("Character sheet generated ({} bytes)", size_bytes);
            Ok(ApiResponse::success(PrintResult { pdf_base64, size_bytes }))
        }
        Err(e) => {
            error!("Failed to generate character sheet: {:?}", e);
            Ok(ApiResponse::error(format!("Failed to generate PDF: {}", e)))
        }
    }
}

/// Options for exporting a character to PDF with composable sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterExportOptions {
    /// Include compact 2-page character sheet
    #[serde(default = "default_true")]
    pub include_compact_sheet: bool,
    /// Include long form character details (personality, background, RP notes)
    #[serde(default)]
    pub include_long_form: bool,
    /// Include spell cards (silently no-op if no spells)
    #[serde(default = "default_true")]
    pub include_spell_cards: bool,
    /// Include equipment cards (weapons, magic items, special ammo)
    #[serde(default)]
    pub include_equipment_cards: bool,
    /// Include detailed equipment list with descriptions
    #[serde(default)]
    pub include_equipment_detail: bool,
}

impl Default for CharacterExportOptions {
    fn default() -> Self {
        Self {
            include_compact_sheet: true,
            include_long_form: false,
            include_spell_cards: true,
            include_equipment_cards: false,
            include_equipment_detail: false,
        }
    }
}

/// Export a character to PDF with composable sections.
///
/// Allows users to select which sections to include:
/// - Compact Sheet (2-page WotC-style)
/// - Long Form (personality, background, RP notes)
/// - Spell Cards (silently skipped if no spells)
/// - Equipment Detail (full inventory with descriptions)
///
/// Sections appear in a fixed order regardless of selection order.
#[tauri::command]
pub async fn export_character(
    state: State<'_, AppState>,
    character_id: i32,
    options: Option<CharacterExportOptions>,
) -> Result<ApiResponse<PrintResult>, String> {
    use mimir_dm_core::models::catalog::Spell;
    use mimir_dm_core::models::catalog::SpellFilters;
    use mimir_dm_core::services::{CharacterService, SpellService};

    let opts = options.unwrap_or_default();
    info!(
        "Exporting character {} with options: compact={}, long_form={}, spells={}, equipment_cards={}, equipment_detail={}",
        character_id,
        opts.include_compact_sheet,
        opts.include_long_form,
        opts.include_spell_cards,
        opts.include_equipment_cards,
        opts.include_equipment_detail
    );

    // Get character data from database
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut char_service = CharacterService::new(&mut conn);
    let (_character, character_data) = char_service
        .get_character(character_id)
        .map_err(|e| format!("Failed to get character: {}", e))?;

    // Collect spells if spell cards are requested
    let mut spell_details: Vec<Spell> = Vec::new();

    if opts.include_spell_cards && !character_data.classes.is_empty() {
        let mut spell_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        // Process each class the character has
        for class_level in &character_data.classes {
            let max_spell_level = calculate_max_spell_level(&class_level.class_name, class_level.level);

            if max_spell_level > 0 {
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

                match SpellService::search_spells(&mut spell_conn, filters) {
                    Ok(summaries) => {
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
            }
        }

        // Sort and dedupe spells
        spell_details.sort_by(|a, b| {
            a.level.cmp(&b.level).then_with(|| a.name.cmp(&b.name))
        });
        spell_details.dedup_by(|a, b| a.name == b.name && a.source == b.source);

        info!(
            "Fetched {} total spell details for character export",
            spell_details.len()
        );
    }

    // Fetch equipment catalog data if equipment cards are requested
    let equipment_data: Vec<serde_json::Value> = if opts.include_equipment_cards && !character_data.inventory.is_empty() {
        use mimir_dm_core::services::ItemService;

        let mut item_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut item_service = ItemService::new(&mut item_conn);

        // Look up each inventory item in the catalog
        character_data
            .inventory
            .iter()
            .filter_map(|inv_item| {
                // Try to get full item details from catalog
                let source = inv_item.source.as_deref().unwrap_or("PHB");
                match item_service.get_item_by_name_and_source(&inv_item.name, source) {
                    Ok(Some(catalog_item)) => {
                        // Serialize the Item struct to JSON
                        if let Ok(mut item_json) = serde_json::to_value(&catalog_item) {
                            // Add user notes if present
                            if let (Some(notes), serde_json::Value::Object(ref mut obj)) = (&inv_item.notes, &mut item_json) {
                                obj.insert("notes".to_string(), serde_json::Value::String(notes.clone()));
                            }
                            Some(item_json)
                        } else {
                            // Create minimal JSON from inventory data
                            Some(serde_json::json!({
                                "name": inv_item.name,
                                "source": source,
                                "notes": inv_item.notes
                            }))
                        }
                    }
                    Ok(None) => {
                        // Item not in catalog - create minimal entry with notes
                        if inv_item.notes.is_some() {
                            Some(serde_json::json!({
                                "name": inv_item.name,
                                "notes": inv_item.notes
                            }))
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        debug!("Failed to fetch item {}: {}", inv_item.name, e);
                        None
                    }
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    info!("Found {} equipment items for cards", equipment_data.len());

    // Build PDF using export_character_pdf_with_equipment
    use mimir_dm_print::{export_character_pdf_with_equipment, CharacterExportOptions as PrintExportOptions};

    let templates_root = get_templates_root();
    let print_options = PrintExportOptions {
        include_compact_sheet: opts.include_compact_sheet,
        include_long_form: opts.include_long_form,
        include_spell_cards: opts.include_spell_cards,
        include_equipment_cards: opts.include_equipment_cards,
        include_equipment_detail: opts.include_equipment_detail,
    };

    match export_character_pdf_with_equipment(character_data, spell_details, equipment_data, templates_root, print_options) {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &pdf_bytes,
            );
            info!("Character export generated ({} bytes)", size_bytes);
            Ok(ApiResponse::success(PrintResult { pdf_base64, size_bytes }))
        }
        Err(e) => {
            error!("Failed to export character: {:?}", e);
            Ok(ApiResponse::error(format!("Failed to generate PDF: {}", e)))
        }
    }
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

/// Print a map to PDF with configurable options.
///
/// Generates a PDF from a map with optional grid overlay, LOS walls,
/// position markers, and token cutouts.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `options` - Print options (mode, show_grid, show_los_walls, etc.)
///
/// # Returns
/// PrintResult with base64-encoded PDF
#[tauri::command]
pub async fn print_map(
    state: State<'_, AppState>,
    map_id: i32,
    options: Option<MapPrintOptions>,
) -> Result<ApiResponse<PrintResult>, String> {
    use crate::commands::campaign::maps::UvttFile;
    use mimir_dm_core::services::{MapService, TokenService};
    use mimir_dm_print::{generate_map_pdf, MapPdfOptions, RenderMap, RenderToken};

    let options = options.unwrap_or_default();
    info!(
        "Printing map {} with options: preview={}, play={}",
        map_id, options.include_preview, options.include_play
    );

    // Get the map from database
    let mut conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut map_service = MapService::new(&mut conn);
    let map = map_service
        .get_map(map_id)
        .map_err(|e| format!("Failed to get map: {}", e))?
        .ok_or_else(|| format!("Map not found with ID: {}", map_id))?;

    // Load the UVTT file to get LOS walls and grid config
    let maps_dir = if let Some(module_id) = map.module_id {
        state.paths.data_dir.join("modules").join(module_id.to_string()).join("maps")
    } else {
        state.paths.data_dir.join("campaigns").join(map.campaign_id.to_string()).join("maps")
    };

    let uvtt_path = maps_dir.join(&map.image_path);
    let uvtt_bytes = std::fs::read(&uvtt_path)
        .map_err(|e| format!("Failed to read UVTT file: {}", e))?;
    let uvtt = UvttFile::from_bytes(&uvtt_bytes)
        .map_err(|e| format!("Failed to parse UVTT file: {}", e))?;

    // Get tokens for this map
    let mut token_conn = state
        .db
        .get_connection()
        .map_err(|e| format!("Database error: {}", e))?;

    let mut token_service = TokenService::new(&mut token_conn);
    let tokens = token_service
        .list_tokens_for_map(map_id)
        .unwrap_or_default();

    // Convert to RenderToken format
    let books_dir = state.paths.data_dir.join("books");
    let temp_dir = std::env::temp_dir().join("mimir-token-images");
    std::fs::create_dir_all(&temp_dir).ok();

    let render_tokens: Vec<RenderToken> = tokens
        .into_iter()
        .map(|t| convert_token_to_render_token(t, &books_dir, &temp_dir))
        .collect();

    // Build RenderMap from database map
    let render_map = RenderMap {
        name: map.name.clone(),
        image_path: map.image_path.clone(),
        width_px: map.width_px,
        height_px: map.height_px,
        grid_type: map.grid_type.clone(),
        grid_size_px: map.grid_size_px,
        grid_offset_x: map.grid_offset_x,
        grid_offset_y: map.grid_offset_y,
    };

    // Build options for PDF generation
    let needs_los_walls = options.preview_los_walls || options.play_los_walls;
    let pdf_options = MapPdfOptions {
        include_preview: options.include_preview,
        preview_grid: options.preview_grid,
        preview_los_walls: options.preview_los_walls,
        preview_positions: options.preview_positions,
        include_play: options.include_play,
        play_grid: options.play_grid,
        play_los_walls: options.play_los_walls,
        play_cutouts: options.play_cutouts,
        los_walls: if needs_los_walls {
            uvtt.line_of_sight
                .iter()
                .map(|wall| wall.iter().map(|p| (p.x, p.y)).collect())
                .collect()
        } else {
            Vec::new()
        },
        pixels_per_grid: uvtt.resolution.pixels_per_grid,
    };

    // Generate PDF using core function
    let templates_root = get_templates_root();

    match generate_map_pdf(
        &render_map,
        &render_tokens,
        &uvtt.image,
        &pdf_options,
        temp_dir,
        templates_root,
    ) {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &pdf_bytes,
            );

            info!("Map PDF generated successfully ({} bytes)", size_bytes);

            Ok(ApiResponse::success(PrintResult {
                pdf_base64,
                size_bytes,
            }))
        }
        Err(e) => {
            error!("Failed to generate map PDF: {:?}", e);
            Ok(ApiResponse::error(format!("Failed to generate PDF: {}", e)))
        }
    }
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

    // Render single document using new campaign API
    use mimir_dm_print::build_single_document_pdf;

    let file_path = std::path::PathBuf::from(&document.file_path);
    let templates_root = get_templates_root();

    match build_single_document_pdf(&file_path, Some(&campaign.name), templates_root) {
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
/// - `options` - Export options (optional, uses defaults if not provided)
///
/// # Returns
/// PrintResult with base64-encoded PDF
#[tauri::command]
pub async fn export_campaign_documents(
    state: State<'_, AppState>,
    campaign_id: i32,
    options: Option<CampaignExportOptions>,
) -> Result<ApiResponse<PrintResult>, String> {
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::dal::campaign::documents::DocumentRepository;
    use mimir_dm_core::models::catalog::class::{ClassFeature, SubclassFeature};
    use mimir_dm_core::services::{CharacterService, ClassService, DocumentService, ModuleMonsterService, ModuleService};

    let opts = options.unwrap_or_default();
    info!("Exporting all campaign {} documents to PDF with options: {:?}", campaign_id, opts);

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

    // Get campaign-level documents (if requested)
    let campaign_file_paths: Vec<std::path::PathBuf> = if opts.include_campaign_docs {
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
        sorted_campaign_docs
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
            .collect()
    } else {
        Vec::new()
    };

    // Get module document file paths and monsters (if requested)
    let mut module_file_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut all_monsters: Vec<serde_json::Value> = Vec::new();

    if opts.include_module_content {
        let mut module_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut module_service = ModuleService::new(&mut module_conn);
        let mut modules = module_service
            .list_campaign_modules(campaign_id)
            .map_err(|e| format!("Failed to get modules: {}", e))?;

        // Sort modules by module_number
        modules.sort_by_key(|m| m.module_number);

        for module in modules {
            // Get module documents
            let mut module_doc_conn = state
                .db
                .get_connection()
                .map_err(|e| format!("Database error: {}", e))?;

            let module_docs = DocumentRepository::find_by_module(&mut module_doc_conn, module.id)
                .map_err(|e| format!("Failed to get documents for module {}: {}", module.id, e))?;

            // Collect document file paths
            for doc in module_docs {
                let path = std::path::PathBuf::from(&doc.file_path);
                if path.exists() {
                    module_file_paths.push(path);
                } else {
                    debug!("Skipping non-existent document file: {:?}", path);
                }
            }

            // Check for play-notes.md file (created during play mode)
            let play_notes_path = std::path::PathBuf::from(&campaign.directory_path)
                .join("modules")
                .join(format!("module_{:02}", module.module_number))
                .join("play-notes.md");

            if play_notes_path.exists() {
                module_file_paths.push(play_notes_path);
                debug!("Added play-notes.md for module {}", module.module_number);
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

            // Convert to JSON values and add to combined list
            for m in monsters_with_data {
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
                    all_monsters.push(monster);
                } else {
                    all_monsters.push(serde_json::json!({
                        "name": m.monster_name,
                        "source": m.monster_source,
                        "quantity": m.quantity,
                        "encounter_tag": m.encounter_tag
                    }));
                }
            }
        }
    }

    // Fetch NPCs for the campaign (if requested)
    let mut npcs_json: Vec<serde_json::Value> = Vec::new();

    if opts.include_npcs {
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
            .filter(|c| c.is_npc)
            .collect();

        // Fetch character data for each NPC and convert to JSON
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
    }

    // ==========================================================================
    // CAMPAIGN MAPS (regional/world maps - no tokens)
    // ==========================================================================
    let mut campaign_maps_data: Vec<(mimir_dm_print::RenderMap, Vec<mimir_dm_print::RenderToken>)> = Vec::new();

    if opts.include_campaign_map_previews || opts.include_campaign_tiled_maps {
        use mimir_dm_core::services::MapService;

        let mut map_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut map_service = MapService::new(&mut map_conn);
        let maps = map_service
            .list_campaign_maps(campaign_id)
            .map_err(|e| format!("Failed to get campaign maps: {}", e))?;

        for map_summary in maps {
            let mut detail_conn = state
                .db
                .get_connection()
                .map_err(|e| format!("Database error: {}", e))?;

            let mut detail_service = MapService::new(&mut detail_conn);
            if let Ok(Some(map)) = detail_service.get_map(map_summary.id) {
                // Resolve full path to map image
                let map_image_full_path = if map.image_path.ends_with(".dd2vtt") || map.image_path.ends_with(".uvtt") {
                    state.paths.data_dir
                        .join("campaigns")
                        .join(campaign_id.to_string())
                        .join("maps")
                        .join(&map.image_path)
                } else {
                    state.paths.data_dir.join("maps").join(&map.image_path)
                };

                let render_map = mimir_dm_print::RenderMap {
                    name: map.name.clone(),
                    image_path: map_image_full_path.to_string_lossy().to_string(),
                    width_px: map.width_px,
                    height_px: map.height_px,
                    grid_type: map.grid_type.clone(),
                    grid_size_px: map.grid_size_px,
                    grid_offset_x: map.grid_offset_x,
                    grid_offset_y: map.grid_offset_y,
                };

                // Campaign maps don't have tokens - pass empty vec
                campaign_maps_data.push((render_map, Vec::new()));
            }
        }
    }

    // ==========================================================================
    // MODULE MAPS (dungeon/encounter maps - with tokens)
    // ==========================================================================
    let mut module_maps_data: Vec<(mimir_dm_print::RenderMap, Vec<mimir_dm_print::RenderToken>)> = Vec::new();

    if opts.include_module_map_previews || opts.include_module_tiled_maps {
        use mimir_dm_core::services::{MapService, TokenService};

        // Get all modules for this campaign
        let mut mod_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut mod_service = mimir_dm_core::services::ModuleService::new(&mut mod_conn);
        let modules = mod_service
            .list_campaign_modules(campaign_id)
            .map_err(|e| format!("Failed to get modules: {}", e))?;

        for module in modules {
            let mut map_conn = state
                .db
                .get_connection()
                .map_err(|e| format!("Database error: {}", e))?;

            let mut map_service = MapService::new(&mut map_conn);
            let maps = map_service
                .list_module_maps(module.id)
                .map_err(|e| format!("Failed to get module maps: {}", e))?;

            for map_summary in maps {
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

                    // Resolve full path to map image
                    let map_image_full_path = if map.image_path.ends_with(".dd2vtt") || map.image_path.ends_with(".uvtt") {
                        state.paths.data_dir
                            .join("modules")
                            .join(module.id.to_string())
                            .join("maps")
                            .join(&map.image_path)
                    } else {
                        state.paths.data_dir.join("maps").join(&map.image_path)
                    };

                    let render_map = mimir_dm_print::RenderMap {
                        name: map.name.clone(),
                        image_path: map_image_full_path.to_string_lossy().to_string(),
                        width_px: map.width_px,
                        height_px: map.height_px,
                        grid_type: map.grid_type.clone(),
                        grid_size_px: map.grid_size_px,
                        grid_offset_x: map.grid_offset_x,
                        grid_offset_y: map.grid_offset_y,
                    };

                    // Convert tokens to RenderToken using shared helper
                    let books_dir = state.paths.data_dir.join("books");
                    let temp_dir = std::env::temp_dir().join("mimir-token-images");
                    std::fs::create_dir_all(&temp_dir).ok();

                    let render_tokens: Vec<mimir_dm_print::RenderToken> = tokens
                        .into_iter()
                        .map(|t| convert_token_to_render_token(t, &books_dir, &temp_dir))
                        .collect();

                    module_maps_data.push((render_map, render_tokens));
                }
            }
        }
    }

    // Combine all documents in order: campaign docs first, then module docs
    let mut all_documents: Vec<std::path::PathBuf> = campaign_file_paths;
    all_documents.extend(module_file_paths);

    info!(
        "Rendering {} documents, {} monsters, {} NPCs, {} campaign maps, {} module maps for campaign '{}'",
        all_documents.len(),
        all_monsters.len(),
        npcs_json.len(),
        campaign_maps_data.len(),
        module_maps_data.len(),
        campaign.name
    );

    // Build PDF using new DocumentBuilder API
    use mimir_dm_print::{build_campaign_pdf, CampaignExportData, ExportOptions};

    let campaign_base_path = std::path::PathBuf::from(&campaign.directory_path);
    let templates_root = get_templates_root();

    let export_data = CampaignExportData {
        name: campaign.name.clone(),
        documents: all_documents.clone(),
        monsters: if all_monsters.is_empty() {
            None
        } else {
            Some(serde_json::Value::Array(all_monsters))
        },
        npcs: if npcs_json.is_empty() {
            None
        } else {
            Some(serde_json::Value::Array(npcs_json))
        },
        campaign_maps: campaign_maps_data,
        module_maps: module_maps_data,
        base_path: campaign_base_path,
        templates_root,
    };

    let export_options = ExportOptions {
        include_toc: true,
        include_monsters: opts.include_module_content, // Monsters come from modules
        include_npcs: opts.include_npcs,
        // Campaign map options
        include_campaign_map_previews: opts.include_campaign_map_previews,
        include_campaign_tiled_maps: opts.include_campaign_tiled_maps,
        // Module map options
        include_module_map_previews: opts.include_module_map_previews,
        include_module_tiled_maps: opts.include_module_tiled_maps,
        include_token_cutouts: opts.include_token_cutouts,
        // Shared rendering options
        preview_grid: true,
        preview_los_walls: false,
        preview_positions: false,
        play_grid: true,
        play_los_walls: false,
    };

    match build_campaign_pdf(export_data, export_options) {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &pdf_bytes,
            );

            info!(
                "Combined campaign PDF generated successfully ({} bytes, {} docs)",
                size_bytes,
                all_documents.len()
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
/// - `options` - Export options (optional, uses defaults if not provided)
///
/// # Returns
/// PrintResult with base64-encoded PDF
#[tauri::command]
pub async fn export_module_documents(
    state: State<'_, AppState>,
    module_id: i32,
    options: Option<ModuleExportOptions>,
) -> Result<ApiResponse<PrintResult>, String> {
    use mimir_dm_core::dal::campaign::campaigns::CampaignRepository;
    use mimir_dm_core::dal::campaign::documents::DocumentRepository;
    use mimir_dm_core::services::{ModuleMonsterService, ModuleService};

    let opts = options.unwrap_or_default();
    info!("Exporting module {} documents to PDF with options: {:?}", module_id, opts);

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

    // Get module document file paths (if requested)
    let mut document_paths: Vec<std::path::PathBuf> = Vec::new();

    if opts.include_documents {
        let mut doc_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let module_docs = DocumentRepository::find_by_module(&mut doc_conn, module_id)
            .map_err(|e| format!("Failed to get documents for module {}: {}", module_id, e))?;

        // Collect document file paths
        for doc in module_docs {
            let path = std::path::PathBuf::from(&doc.file_path);
            if path.exists() {
                document_paths.push(path);
            } else {
                debug!("Skipping non-existent document file: {:?}", path);
            }
        }

        // Check for play-notes.md file (created during play mode)
        let play_notes_path = std::path::PathBuf::from(&campaign.directory_path)
            .join("modules")
            .join(format!("module_{:02}", module.module_number))
            .join("play-notes.md");

        if play_notes_path.exists() {
            document_paths.push(play_notes_path);
            debug!("Added play-notes.md for module {}", module.module_number);
        }
    }

    // Get module monsters (if requested)
    let monster_json: Vec<serde_json::Value> = if opts.include_monsters {
        let mut monster_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut monster_service = ModuleMonsterService::new(&mut monster_conn);
        let monsters_with_data = monster_service
            .get_monsters_with_data(module_id)
            .map_err(|e| format!("Failed to get monsters for module {}: {}", module_id, e))?;

        // Convert to JSON values
        monsters_with_data
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
            .collect()
    } else {
        Vec::new()
    };

    // Fetch maps for this module (if map previews or play tiles are requested)
    let mut maps_data: Vec<(mimir_dm_print::RenderMap, Vec<mimir_dm_print::RenderToken>)> = Vec::new();

    if opts.include_preview || opts.include_play {
        use mimir_dm_core::services::{MapService, TokenService};

        let mut map_conn = state
            .db
            .get_connection()
            .map_err(|e| format!("Database error: {}", e))?;

        let mut map_service = MapService::new(&mut map_conn);
        let maps = map_service
            .list_module_maps(module_id)
            .map_err(|e| format!("Failed to get maps for module {}: {}", module_id, e))?;

        for map in maps {
            // Get tokens for this map
            let mut token_conn = state
                .db
                .get_connection()
                .map_err(|e| format!("Database error: {}", e))?;

            let mut token_service = TokenService::new(&mut token_conn);
            let tokens = token_service
                .list_tokens_for_map(map.id)
                .unwrap_or_default();

            // Resolve full path to map image
            let map_image_full_path = if map.image_path.ends_with(".dd2vtt") || map.image_path.ends_with(".uvtt") {
                state.paths.data_dir
                    .join("modules")
                    .join(module_id.to_string())
                    .join("maps")
                    .join(&map.image_path)
            } else {
                state.paths.data_dir.join("maps").join(&map.image_path)
            };

            let render_map = mimir_dm_print::RenderMap {
                name: map.name.clone(),
                image_path: map_image_full_path.to_string_lossy().to_string(),
                width_px: map.width_px,
                height_px: map.height_px,
                grid_type: map.grid_type.clone(),
                grid_size_px: map.grid_size_px,
                grid_offset_x: map.grid_offset_x,
                grid_offset_y: map.grid_offset_y,
            };

            // Convert tokens to RenderToken using shared helper
            let books_dir = state.paths.data_dir.join("books");
            let temp_dir = std::env::temp_dir().join("mimir-token-images");
            std::fs::create_dir_all(&temp_dir).ok();

            let render_tokens: Vec<mimir_dm_print::RenderToken> = tokens
                .into_iter()
                .map(|t| convert_token_to_render_token(t, &books_dir, &temp_dir))
                .collect();

            maps_data.push((render_map, render_tokens));
        }
    }

    info!(
        "Rendering module '{}' with {} documents, {} monsters, {} maps",
        module.name,
        document_paths.len(),
        monster_json.len(),
        maps_data.len()
    );

    // Build PDF using new DocumentBuilder API
    use mimir_dm_print::{build_campaign_pdf, CampaignExportData, ExportOptions};

    let campaign_base_path = std::path::PathBuf::from(&campaign.directory_path);
    let templates_root = get_templates_root();

    let export_data = CampaignExportData {
        name: module.name.clone(),
        documents: document_paths.clone(),
        monsters: if monster_json.is_empty() {
            None
        } else {
            Some(serde_json::Value::Array(monster_json))
        },
        npcs: None, // NPCs not yet implemented for module export
        campaign_maps: Vec::new(), // Module export has no campaign-level maps
        module_maps: maps_data,    // All maps are module maps
        base_path: campaign_base_path,
        templates_root,
    };

    if opts.include_npcs {
        tracing::warn!(
            "include_npcs was requested but is not yet implemented for module export. \
            NPCs are campaign-scoped and will be added in a future update."
        );
    }

    let export_options = ExportOptions {
        include_toc: true,
        include_monsters: opts.include_monsters,
        include_npcs: false, // NPCs not yet implemented for module export
        // Campaign map options (none for module export)
        include_campaign_map_previews: false,
        include_campaign_tiled_maps: false,
        // Module map options
        include_module_map_previews: opts.include_preview,
        include_module_tiled_maps: opts.include_play,
        include_token_cutouts: opts.include_play && opts.play_cutouts,
        // Shared rendering options
        preview_grid: opts.preview_grid,
        preview_los_walls: opts.preview_los_walls,
        preview_positions: opts.preview_positions,
        play_grid: opts.play_grid,
        play_los_walls: opts.play_los_walls,
    };

    match build_campaign_pdf(export_data, export_options) {
        Ok(pdf_bytes) => {
            let size_bytes = pdf_bytes.len();
            let pdf_base64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &pdf_bytes,
            );

            info!(
                "Module PDF generated successfully ({} bytes, {} docs)",
                size_bytes,
                document_paths.len()
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
