//! Map Commands
//!
//! Tauri commands for map management (UVTT files).
//! Split into sub-modules by functionality.

pub mod crud;
pub mod uvtt;
pub mod light;
pub mod fog;
pub mod traps;
pub mod pois;

// Re-export all public items for backwards compatibility
pub use crud::*;
pub use uvtt::*;
pub use light::*;
pub use fog::*;
pub use traps::*;
pub use pois::*;

use mimir_core::models::campaign::{LightSource, Map};
use mimir_core::services::MapService;
use serde::Serialize;
use std::path::Path;

// =============================================================================
// Shared Types and Functions
// =============================================================================

/// Map response with UVTT-derived fields for the frontend.
#[derive(Debug, Serialize)]
pub struct MapResponse {
    // Base map fields
    pub id: String,
    pub campaign_id: String,
    pub module_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub uvtt_asset_id: String,
    pub lighting_mode: String,
    pub fog_enabled: i32,
    pub created_at: String,
    pub updated_at: String,
    // UVTT-derived fields
    pub width_px: i32,
    pub height_px: i32,
    pub grid_type: String,
    pub grid_size_px: Option<i32>,
    pub grid_offset_x: i32,
    pub grid_offset_y: i32,
    pub original_width_px: Option<i32>,
    pub original_height_px: Option<i32>,
    pub image_path: String,
}

/// Enrich a Map with UVTT data to create a MapResponse.
pub(crate) fn enrich_map_with_uvtt(map: &Map, service: &mut MapService, _app_dir: &Path) -> MapResponse {
    // Default values if UVTT parsing fails
    let mut width_px = 0;
    let mut height_px = 0;
    let mut grid_size_px = None;

    // Try to read and parse UVTT data
    if let Ok(uvtt_bytes) = service.read_uvtt_file(map) {
        if let Ok(uvtt_json) = serde_json::from_slice::<serde_json::Value>(&uvtt_bytes) {
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

            width_px = (pixels_per_grid as f64 * map_size_x) as i32;
            height_px = (pixels_per_grid as f64 * map_size_y) as i32;
            grid_size_px = Some(pixels_per_grid);
        }
    }

    MapResponse {
        id: map.id.clone(),
        campaign_id: map.campaign_id.clone(),
        module_id: map.module_id.clone(),
        name: map.name.clone(),
        description: map.description.clone(),
        sort_order: map.sort_order,
        uvtt_asset_id: map.uvtt_asset_id.clone(),
        lighting_mode: map.lighting_mode.clone(),
        fog_enabled: map.fog_enabled,
        created_at: map.created_at.clone(),
        updated_at: map.updated_at.clone(),
        width_px,
        height_px,
        grid_type: "square".to_string(), // UVTT files use square grids
        grid_size_px,
        grid_offset_x: 0, // UVTT maps start at origin
        grid_offset_y: 0,
        original_width_px: Some(width_px),
        original_height_px: Some(height_px),
        image_path: map.uvtt_asset_id.clone(), // Asset ID is used to serve the image
    }
}

/// Enrich multiple maps with UVTT data.
pub(crate) fn enrich_maps_with_uvtt(maps: Vec<Map>, service: &mut MapService, app_dir: &Path) -> Vec<MapResponse> {
    maps.iter().map(|map| enrich_map_with_uvtt(map, service, app_dir)).collect()
}

/// Light source response format for frontend.
#[derive(Debug, Serialize)]
pub struct LightSourceResponse {
    pub id: String,
    pub map_id: String,
    pub token_id: Option<String>,
    pub token_name: Option<String>,
    pub name: String,
    pub light_type: String,
    pub x: f64,
    pub y: f64,
    pub bright_radius_ft: i32,
    pub dim_radius_ft: i32,
    pub color: Option<String>,
    pub is_active: bool,
}

/// Transform a LightSource to LightSourceResponse with pixel coordinates.
pub(crate) fn transform_light_source(ls: LightSource, grid_size_px: i32) -> LightSourceResponse {
    // Convert grid coordinates to pixel coordinates (center of grid cell)
    let x = (ls.grid_x as f64 + 0.5) * grid_size_px as f64;
    let y = (ls.grid_y as f64 + 0.5) * grid_size_px as f64;
    let is_active = ls.is_active();

    LightSourceResponse {
        id: ls.id,
        map_id: ls.map_id,
        token_id: None,
        token_name: None,
        name: ls.name.unwrap_or_else(|| "Light".to_string()),
        light_type: "custom".to_string(),
        x,
        y,
        bright_radius_ft: ls.bright_radius,
        dim_radius_ft: ls.dim_radius,
        color: ls.color,
        is_active,
    }
}

/// Get the grid size (pixels per grid) from a map's UVTT file.
pub(crate) fn get_map_grid_size_for_lights(service: &mut MapService, map_id: &str) -> i32 {
    if let Ok(Some(map)) = service.get(map_id) {
        if let Ok(uvtt_bytes) = service.read_uvtt_file(&map) {
            if let Ok(uvtt_json) = serde_json::from_slice::<serde_json::Value>(&uvtt_bytes) {
                return uvtt_json
                    .get("resolution")
                    .and_then(|r| r.get("pixels_per_grid"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(70) as i32;
            }
        }
    }
    70 // Default grid size
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Simple base64 encoding using the standard library.
pub(crate) fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Simple base64 decoding.
pub(crate) fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s)
}
