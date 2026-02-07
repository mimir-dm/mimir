//! Print Commands
//!
//! Tauri commands for PDF export using mimir-print infrastructure.
//! Split into sub-modules by print domain.

mod character;
mod document;
mod helpers;
mod map;
mod monster;
mod trap;

// Re-export all commands for use in main.rs invoke_handler
pub use character::*;
pub use document::*;
pub use map::*;
pub use monster::*;
pub use trap::*;

use serde::{Deserialize, Serialize};

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
