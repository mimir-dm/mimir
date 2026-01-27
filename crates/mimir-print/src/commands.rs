//! Tauri commands for print functionality
//!
//! These commands are only available when the `tauri-commands` feature is enabled.
//!
//! # Status
//!
//! Currently provides stub implementations that return "not yet implemented" errors.
//! Full functionality will be implemented incrementally as we migrate from v1.

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

/// State wrapper for print functionality
pub struct PrintState {
    /// Path to templates directory
    pub templates_dir: PathBuf,
    /// Path to assets directory
    pub assets_dir: PathBuf,
}

impl PrintState {
    /// Create a new PrintState
    pub fn new(templates_dir: PathBuf, assets_dir: PathBuf) -> Self {
        Self {
            templates_dir,
            assets_dir,
        }
    }
}

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

/// Template info for listing
#[derive(Debug, Serialize)]
pub struct PrintTemplateInfo {
    pub id: String,
    pub name: String,
    pub category: String,
}

/// Print result containing PDF data
#[derive(Debug, Serialize)]
pub struct PrintResult {
    pub pdf_base64: String,
    pub size_bytes: usize,
}

// =============================================================================
// Character Export Options
// =============================================================================

#[derive(Debug, Deserialize, Default)]
pub struct CharacterExportOptions {
    pub include_compact_sheet: Option<bool>,
    pub include_long_form: Option<bool>,
    pub include_battle_card: Option<bool>,
    pub include_spell_cards: Option<bool>,
    pub include_equipment_cards: Option<bool>,
    pub include_equipment_detail: Option<bool>,
}

// =============================================================================
// Module Export Options
// =============================================================================

#[derive(Debug, Deserialize, Default)]
pub struct ModuleExportOptions {
    pub include_documents: Option<bool>,
    pub include_monsters: Option<bool>,
    pub include_traps: Option<bool>,
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

// =============================================================================
// Campaign Export Options
// =============================================================================

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

// =============================================================================
// Map Print Options
// =============================================================================

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

// =============================================================================
// Tauri Commands
// =============================================================================

/// List available print templates
#[tauri::command]
pub fn list_print_templates(
    _state: State<'_, PrintState>,
) -> ApiResponse<Vec<PrintTemplateInfo>> {
    // Return empty list for now - templates will be added as we implement features
    ApiResponse::ok(vec![])
}

/// Export a character to PDF
#[tauri::command]
pub fn export_character(
    _state: State<'_, PrintState>,
    character_id: String,
    options: Option<CharacterExportOptions>,
) -> ApiResponse<PrintResult> {
    ApiResponse::err(format!(
        "Character PDF export not yet implemented (character_id: {}, options: {:?})",
        character_id, options
    ))
}

/// Export a campaign document to PDF
#[tauri::command]
pub fn export_campaign_document(
    _state: State<'_, PrintState>,
    document_id: String,
) -> ApiResponse<PrintResult> {
    ApiResponse::err(format!(
        "Campaign document PDF export not yet implemented (document_id: {})",
        document_id
    ))
}

/// Export all campaign documents to PDF
#[tauri::command]
pub fn export_campaign_documents(
    _state: State<'_, PrintState>,
    campaign_id: String,
    options: Option<CampaignExportOptions>,
) -> ApiResponse<PrintResult> {
    ApiResponse::err(format!(
        "Campaign PDF export not yet implemented (campaign_id: {}, options: {:?})",
        campaign_id, options
    ))
}

/// Export module documents to PDF
#[tauri::command]
pub fn export_module_documents(
    _state: State<'_, PrintState>,
    module_id: String,
    options: Option<ModuleExportOptions>,
) -> ApiResponse<PrintResult> {
    ApiResponse::err(format!(
        "Module PDF export not yet implemented (module_id: {}, options: {:?})",
        module_id, options
    ))
}

/// Print a map to PDF
#[tauri::command]
pub fn print_map(
    _state: State<'_, PrintState>,
    map_id: String,
    options: Option<MapPrintOptions>,
) -> ApiResponse<PrintResult> {
    ApiResponse::err(format!(
        "Map PDF export not yet implemented (map_id: {}, options: {:?})",
        map_id, options
    ))
}

/// Generate character sheet (legacy API)
#[tauri::command]
pub fn generate_character_sheet(
    _state: State<'_, PrintState>,
    character_id: String,
    template: Option<String>,
    include_spell_cards: Option<bool>,
) -> ApiResponse<PrintResult> {
    ApiResponse::err(format!(
        "Character sheet generation not yet implemented (character_id: {}, template: {:?})",
        character_id, template
    ))
}

/// Save PDF to file
#[tauri::command]
pub fn save_pdf(
    pdf_base64: String,
    path: String,
) -> ApiResponse<()> {
    let pdf_bytes = match base64::engine::general_purpose::STANDARD.decode(&pdf_base64) {
        Ok(bytes) => bytes,
        Err(e) => return ApiResponse::err(format!("Invalid base64: {}", e)),
    };

    match std::fs::write(&path, &pdf_bytes) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Failed to write file: {}", e)),
    }
}
