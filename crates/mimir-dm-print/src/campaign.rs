//! Campaign document rendering using DocumentBuilder
//!
//! This module provides campaign-specific PDF rendering functionality
//! using the composable DocumentBuilder pattern.

use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::Value;
use tracing::{debug, info, instrument};

use crate::builder::{DocumentBuilder, RenderContext, Renderable};
use crate::error::Result;
use crate::map_renderer::{load_image_from_file, render_map_for_print, MapPrintOptions, RenderMap, RenderToken};
use crate::maps::slice_map_into_tiles;
use crate::sections::{
    MapPreview, MarkdownSection, MonsterAppendix, NpcAppendix, TiledMapSection, TokenCutoutSheet,
};

/// Options for campaign/module PDF export
#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    /// Include table of contents
    pub include_toc: bool,
    /// Include monster appendix
    pub include_monsters: bool,
    /// Include NPC appendix
    pub include_npcs: bool,

    // Map Preview options
    /// Include map previews (fit to single page)
    pub include_map_previews: bool,
    /// Show grid overlay on previews
    pub preview_grid: bool,
    /// Show LOS walls on previews
    pub preview_los_walls: bool,
    /// Show starting positions on previews
    pub preview_positions: bool,

    // Map Play options
    /// Include tiled maps (1"=5ft scale)
    pub include_tiled_maps: bool,
    /// Show grid overlay on tiles
    pub play_grid: bool,
    /// Show LOS walls on tiles
    pub play_los_walls: bool,
    /// Include token cutouts
    pub include_token_cutouts: bool,
}

impl ExportOptions {
    /// Create options for a reference document (docs, monsters, npcs, map previews)
    pub fn reference_doc() -> Self {
        Self {
            include_toc: true,
            include_monsters: true,
            include_npcs: true,
            include_map_previews: true,
            preview_grid: true,
            preview_los_walls: false,
            preview_positions: false,
            include_tiled_maps: false,
            play_grid: true,
            play_los_walls: false,
            include_token_cutouts: false,
        }
    }

    /// Create options for a physical play kit (tiled maps, token cutouts)
    pub fn physical_play_kit() -> Self {
        Self {
            include_toc: false,
            include_monsters: false,
            include_npcs: false,
            include_map_previews: false,
            preview_grid: true,
            preview_los_walls: false,
            preview_positions: false,
            include_tiled_maps: true,
            play_grid: true,
            play_los_walls: false,
            include_token_cutouts: true,
        }
    }

    /// Create options for a complete export
    pub fn complete() -> Self {
        Self {
            include_toc: true,
            include_monsters: true,
            include_npcs: true,
            include_map_previews: true,
            preview_grid: true,
            preview_los_walls: false,
            preview_positions: true,
            include_tiled_maps: true,
            play_grid: true,
            play_los_walls: false,
            include_token_cutouts: true,
        }
    }
}

/// Data for campaign export
pub struct CampaignExportData {
    /// Campaign or module name
    pub name: String,
    /// Document file paths
    pub documents: Vec<PathBuf>,
    /// Monster data (JSON array)
    pub monsters: Option<Value>,
    /// NPC data (JSON array)
    pub npcs: Option<Value>,
    /// Map data with tokens
    pub maps: Vec<(RenderMap, Vec<RenderToken>)>,
    /// Base path for resolving files
    pub base_path: PathBuf,
    /// Path to templates directory
    pub templates_root: PathBuf,
}

/// Build a campaign/module PDF using DocumentBuilder
///
/// This is the unified entry point for all campaign/module exports.
/// Use `ExportOptions` to control what content is included.
#[instrument(skip(data), fields(name = %data.name, doc_count = data.documents.len()))]
pub fn build_campaign_pdf(data: CampaignExportData, options: ExportOptions) -> Result<Vec<u8>> {
    info!(
        "Building campaign PDF: {} documents, {} maps",
        data.documents.len(),
        data.maps.len()
    );

    let context = RenderContext::new(std::env::temp_dir().join("mimir-export"))
        .with_base_path(data.base_path.clone());

    // Create temp directory if needed
    std::fs::create_dir_all(&context.temp_dir)?;

    let mut builder = DocumentBuilder::new(&data.name)
        .with_templates_root(data.templates_root)
        .with_toc(options.include_toc)
        .with_context(context);

    // Add markdown documents
    for doc_path in &data.documents {
        debug!("Adding document: {:?}", doc_path);
        match MarkdownSection::from_file(doc_path) {
            Ok(section) => {
                builder = builder.append(section);
            }
            Err(e) => {
                tracing::warn!("Failed to read document {:?}: {}", doc_path, e);
            }
        }
    }

    // Add map previews
    if options.include_map_previews {
        let preview_opts = MapPrintOptions {
            show_grid: options.preview_grid,
            show_los_walls: options.preview_los_walls,
            show_positions: false, // Positions shown on separate page
            ..Default::default()
        };

        for (map, tokens) in &data.maps {
            debug!("Adding map preview: {}", map.name);
            let section = MapPreview::new(map.clone(), tokens.clone(), data.base_path.clone())
                .with_options(preview_opts.clone());
            builder = builder.append(section);

            // Add starting positions map if requested (separate page)
            if options.preview_positions && !tokens.is_empty() {
                let positions_opts = MapPrintOptions {
                    show_grid: options.preview_grid,
                    show_los_walls: options.preview_los_walls,
                    show_positions: true,
                    ..Default::default()
                };

                debug!("Adding starting positions map: {}", map.name);
                let mut positions_map = map.clone();
                positions_map.name = format!("{} - Starting Positions", map.name);
                let positions_section = MapPreview::new(positions_map, tokens.clone(), data.base_path.clone())
                    .with_options(positions_opts);
                builder = builder.append(positions_section);
            }
        }
    }

    // Add monster appendix
    if options.include_monsters {
        if let Some(ref monsters) = data.monsters {
            if let Some(arr) = monsters.as_array() {
                if !arr.is_empty() {
                    debug!("Adding monster appendix with {} monsters", arr.len());
                    let section = MonsterAppendix::new(monsters.clone());
                    builder = builder.append(section);
                }
            }
        }
    }

    // Add NPC appendix
    if options.include_npcs {
        if let Some(ref npcs) = data.npcs {
            if let Some(arr) = npcs.as_array() {
                if !arr.is_empty() {
                    debug!("Adding NPC appendix with {} NPCs", arr.len());
                    let section = NpcAppendix::new(npcs.clone());
                    builder = builder.append(section);
                }
            }
        }
    }

    // Add tiled maps for physical play (using pre-sliced tiles like maps.rs)
    if options.include_tiled_maps {
        // Default pixels per grid for letter paper at 1"=1 grid
        const PIXELS_PER_GRID: u32 = 54;

        let play_opts = MapPrintOptions {
            show_grid: options.play_grid,
            show_los_walls: options.play_los_walls,
            show_positions: false, // No position markers on play tiles
            pixels_per_grid: PIXELS_PER_GRID,
            ..Default::default()
        };

        for (map, _tokens) in &data.maps {
            debug!("Adding tiled map: {}", map.name);

            // Load map image from file path
            let file_path = PathBuf::from(&map.image_path);
            let image_bytes = match load_image_from_file(&file_path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    tracing::warn!("Failed to load map image for tiled map: {}", e);
                    continue;
                }
            };
            let image_base64 = STANDARD.encode(&image_bytes);

            // Render without tokens (physical tokens used during play)
            let rendered = match render_map_for_print(
                map,
                &[], // No tokens on play tiles
                &data.base_path,
                &image_base64,
                &play_opts,
            ) {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!("Failed to render map for tiling: {}", e);
                    continue;
                }
            };

            // Slice into tiles
            let (tiles, tiles_x, tiles_y) = match slice_map_into_tiles(&rendered, PIXELS_PER_GRID) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!("Failed to slice map into tiles: {}", e);
                    continue;
                }
            };

            debug!("Sliced map into {}x{} tiles ({} total)", tiles_x, tiles_y, tiles.len());
            builder = builder.append(TiledMapSection::from_tiles(
                map.name.clone(),
                tiles,
                tiles_x,
                tiles_y,
            ));
        }
    }

    // Add token cutouts
    if options.include_token_cutouts {
        let all_tokens: Vec<RenderToken> = data
            .maps
            .iter()
            .flat_map(|(_, tokens)| tokens.clone())
            .collect();

        if !all_tokens.is_empty() {
            debug!("Adding token cutouts for {} tokens", all_tokens.len());
            let section = TokenCutoutSheet::new(all_tokens, data.base_path.clone());
            builder = builder.append(section);
        }
    }

    builder.to_pdf()
}

/// Build a single document PDF
///
/// Convenience function for exporting a single markdown document.
#[instrument(skip_all, fields(path = %file_path.display()))]
pub fn build_single_document_pdf(
    file_path: &PathBuf,
    title: Option<&str>,
    templates_root: PathBuf,
) -> Result<Vec<u8>> {
    info!("Building single document PDF");

    let section = MarkdownSection::from_file(file_path)?;
    let doc_title = title
        .map(|s| s.to_string())
        .or_else(|| section.toc_title())
        .unwrap_or_else(|| "Document".to_string());

    DocumentBuilder::new(doc_title)
        .with_templates_root(templates_root)
        .with_toc(false)
        .append(section)
        .to_pdf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_options_defaults() {
        let opts = ExportOptions::default();
        assert!(!opts.include_toc);
        assert!(!opts.include_monsters);
    }

    #[test]
    fn test_export_options_reference_doc() {
        let opts = ExportOptions::reference_doc();
        assert!(opts.include_toc);
        assert!(opts.include_monsters);
        assert!(opts.include_npcs);
        assert!(opts.include_map_previews);
        assert!(!opts.include_tiled_maps);
    }

    #[test]
    fn test_export_options_physical_play_kit() {
        let opts = ExportOptions::physical_play_kit();
        assert!(!opts.include_toc);
        assert!(opts.include_tiled_maps);
        assert!(opts.include_token_cutouts);
    }
}
