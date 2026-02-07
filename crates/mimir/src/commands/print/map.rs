//! Map Print Commands
//!
//! Tauri commands for printing maps to PDF.

use base64::Engine;
use mimir_core::services::MapService;
use mimir_print::map_renderer::{MapPrintOptions as RenderMapPrintOptions, RenderMap};
use mimir_print::sections::{MapPreview, TiledMapSection};
use mimir_print::{DocumentBuilder, PrintState};
use serde_json::Value;
use std::path::PathBuf;
use tauri::State;
use tracing::{error, info};

use crate::state::AppState;

use super::{ApiResponse, MapPrintOptions, PrintResult};

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
        show_los_walls: opts.preview_los_walls.unwrap_or(false)
            || opts.play_los_walls.unwrap_or(false),
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
        let tiled = TiledMapSection::new(render_map, vec![], base_path).with_options(tiled_options);
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
