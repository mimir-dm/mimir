//! Map Commands
//!
//! Tauri commands for map management (UVTT files).

use mimir_core::dal::campaign as dal;
use mimir_core::models::campaign::{LightingMode, LightSource, Map, NewLightSource, UpdateLightSource};
use mimir_core::services::{CreateMapInput, MapService, UpdateMapInput};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;
use uuid::Uuid;

use super::{to_api_response, ApiResponse};
use crate::state::AppState;

// =============================================================================
// Map Response (enriched with UVTT data)
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
fn enrich_map_with_uvtt(map: &Map, service: &mut MapService, _app_dir: &Path) -> MapResponse {
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
fn enrich_maps_with_uvtt(maps: Vec<Map>, service: &mut MapService, app_dir: &Path) -> Vec<MapResponse> {
    maps.iter().map(|map| enrich_map_with_uvtt(map, service, app_dir)).collect()
}

// =============================================================================
// List Commands
// =============================================================================

/// List all maps for a campaign (including module maps).
#[tauri::command]
pub fn list_campaign_maps(state: State<'_, AppState>, campaign_id: String) -> ApiResponse<Vec<MapResponse>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    match service.list_for_campaign(&campaign_id) {
        Ok(maps) => ApiResponse::ok(enrich_maps_with_uvtt(maps, &mut service, &state.paths.app_dir)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List only campaign-level maps (not in any module).
#[tauri::command]
pub fn list_campaign_level_maps(
    state: State<'_, AppState>,
    campaign_id: String,
) -> ApiResponse<Vec<MapResponse>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    match service.list_campaign_level(&campaign_id) {
        Ok(maps) => ApiResponse::ok(enrich_maps_with_uvtt(maps, &mut service, &state.paths.app_dir)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// List all maps for a module.
#[tauri::command]
pub fn list_module_maps(state: State<'_, AppState>, module_id: String) -> ApiResponse<Vec<MapResponse>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    match service.list_for_module(&module_id) {
        Ok(maps) => ApiResponse::ok(enrich_maps_with_uvtt(maps, &mut service, &state.paths.app_dir)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// CRUD Commands
// =============================================================================

/// Get a map by ID.
#[tauri::command]
pub fn get_map(state: State<'_, AppState>, id: String) -> ApiResponse<MapResponse> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    match service.get(&id) {
        Ok(Some(map)) => ApiResponse::ok(enrich_map_with_uvtt(&map, &mut service, &state.paths.app_dir)),
        Ok(None) => ApiResponse::err(format!("Map not found: {}", id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Parse lighting mode from string.
fn parse_lighting_mode(s: Option<&str>) -> LightingMode {
    match s {
        Some("dim") => LightingMode::Dim,
        Some("dark") => LightingMode::Dark,
        _ => LightingMode::Bright,
    }
}

/// Request for creating a map.
#[derive(Debug, serde::Deserialize)]
pub struct CreateMapRequest {
    pub campaign_id: String,
    pub module_id: Option<String>,
    pub name: String,
    pub filename: String,
    pub description: Option<String>,
    pub lighting_mode: Option<String>,
    /// Base64-encoded UVTT file data
    pub uvtt_data_base64: String,
}

/// Create a new map from a UVTT file.
#[tauri::command]
pub fn create_map(state: State<'_, AppState>, request: CreateMapRequest) -> ApiResponse<Map> {
    // Decode base64 data
    let uvtt_data = match base64_decode(&request.uvtt_data_base64) {
        Ok(data) => data,
        Err(e) => return ApiResponse::err(format!("Invalid base64 data: {}", e)),
    };

    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut input = if let Some(module_id) = request.module_id {
        CreateMapInput::for_module(
            &request.campaign_id,
            module_id,
            &request.name,
            &request.filename,
            uvtt_data,
        )
    } else {
        CreateMapInput::for_campaign(&request.campaign_id, &request.name, &request.filename, uvtt_data)
    };

    if let Some(desc) = request.description {
        input = input.with_description(desc);
    }

    let lighting_mode = parse_lighting_mode(request.lighting_mode.as_deref());
    input = input.with_lighting_mode(lighting_mode);

    let result = MapService::new(&mut db, &state.paths.app_dir).create(input);
    to_api_response(result)
}

/// Request for updating a map.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateMapRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub lighting_mode: Option<String>,
}

/// Update a map.
#[tauri::command]
pub fn update_map(
    state: State<'_, AppState>,
    id: String,
    request: UpdateMapRequest,
) -> ApiResponse<Map> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let input = UpdateMapInput {
        name: request.name,
        description: request.description,
        sort_order: request.sort_order,
        lighting_mode: request.lighting_mode.as_deref().map(|s| parse_lighting_mode(Some(s))),
        module_id: None, // Don't allow moving maps via update
    };

    let result = MapService::new(&mut db, &state.paths.app_dir).update(&id, input);
    to_api_response(result)
}

/// Delete a map and its associated UVTT asset.
#[tauri::command]
pub fn delete_map(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = MapService::new(&mut db, &state.paths.app_dir).delete(&id);
    to_api_response(result)
}

// =============================================================================
// UVTT Data Commands
// =============================================================================

/// Read the UVTT file data for a map (returned as base64).
#[tauri::command]
pub fn read_map_uvtt(state: State<'_, AppState>, map_id: String) -> ApiResponse<String> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);

    // Get the map
    let map = match service.get(&map_id) {
        Ok(Some(map)) => map,
        Ok(None) => return ApiResponse::err(format!("Map not found: {}", map_id)),
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Read the UVTT file
    match service.read_uvtt_file(&map) {
        Ok(data) => ApiResponse::ok(base64_encode(&data)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// A 2D point in grid units.
#[derive(Debug, Clone, Serialize)]
pub struct UvttPoint {
    pub x: f64,
    pub y: f64,
}

/// UVTT portal (door) structure.
#[derive(Debug, Clone, Serialize)]
pub struct UvttPortal {
    pub position: UvttPoint,
    pub bounds: [UvttPoint; 2],
    pub rotation: f64,
    pub closed: bool,
    pub freestanding: bool,
}

/// UVTT light source structure.
#[derive(Debug, Clone, Serialize)]
pub struct UvttLight {
    pub position: UvttPoint,
    pub range: f64,
    pub intensity: f64,
    pub color: String,
    pub shadows: bool,
}

/// UVTT environment settings.
#[derive(Debug, Clone, Serialize)]
pub struct UvttEnvironment {
    pub baked_lighting: bool,
    pub ambient_light: String,
}

/// UVTT map size.
#[derive(Debug, Clone, Serialize)]
pub struct UvttMapSize {
    pub x: f64,
    pub y: f64,
}

/// UVTT resolution data.
#[derive(Debug, Clone, Serialize)]
pub struct UvttResolution {
    pub pixels_per_grid: i32,
    pub map_size: UvttMapSize,
}

/// Parsed UVTT data with resolution, walls, portals, lights, and environment.
#[derive(Debug, Serialize)]
pub struct UvttData {
    pub resolution: UvttResolution,
    /// Wall segments for line of sight blocking. Each inner Vec is a polyline.
    pub line_of_sight: Option<Vec<Vec<UvttPoint>>>,
    /// Portals (doors) that can be opened/closed.
    pub portals: Option<Vec<UvttPortal>>,
    /// Light sources embedded in the map.
    pub lights: Option<Vec<UvttLight>>,
    /// Environment settings including ambient light.
    pub environment: Option<UvttEnvironment>,
}

/// Get parsed UVTT data for a map (resolution, grid size, etc).
#[tauri::command]
pub fn get_uvtt_map(state: State<'_, AppState>, id: String) -> ApiResponse<UvttData> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);

    // Get the map
    let map = match service.get(&id) {
        Ok(Some(map)) => map,
        Ok(None) => return ApiResponse::err(format!("Map not found: {}", id)),
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Read the UVTT file
    let uvtt_bytes = match service.read_uvtt_file(&map) {
        Ok(data) => data,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Parse as JSON
    let uvtt_json: serde_json::Value = match serde_json::from_slice(&uvtt_bytes) {
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

    // Extract line_of_sight (wall segments)
    let line_of_sight: Option<Vec<Vec<UvttPoint>>> = uvtt_json
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
                                Some(UvttPoint { x, y })
                            })
                            .collect()
                    })
                })
                .collect()
        });

    // Extract portals (doors)
    let portals: Option<Vec<UvttPortal>> = uvtt_json
        .get("portals")
        .and_then(|p| p.as_array())
        .map(|portals| {
            portals
                .iter()
                .filter_map(|portal| {
                    let position = portal.get("position")?;
                    let pos_x = position.get("x").and_then(|v| v.as_f64())?;
                    let pos_y = position.get("y").and_then(|v| v.as_f64())?;

                    let bounds = portal.get("bounds").and_then(|b| b.as_array())?;
                    if bounds.len() < 2 {
                        return None;
                    }
                    let b0_x = bounds[0].get("x").and_then(|v| v.as_f64())?;
                    let b0_y = bounds[0].get("y").and_then(|v| v.as_f64())?;
                    let b1_x = bounds[1].get("x").and_then(|v| v.as_f64())?;
                    let b1_y = bounds[1].get("y").and_then(|v| v.as_f64())?;

                    Some(UvttPortal {
                        position: UvttPoint { x: pos_x, y: pos_y },
                        bounds: [
                            UvttPoint { x: b0_x, y: b0_y },
                            UvttPoint { x: b1_x, y: b1_y },
                        ],
                        rotation: portal.get("rotation").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        closed: portal.get("closed").and_then(|v| v.as_bool()).unwrap_or(true),
                        freestanding: portal.get("freestanding").and_then(|v| v.as_bool()).unwrap_or(false),
                    })
                })
                .collect()
        });

    // Extract lights
    let lights: Option<Vec<UvttLight>> = uvtt_json
        .get("lights")
        .and_then(|l| l.as_array())
        .map(|lights| {
            lights
                .iter()
                .filter_map(|light| {
                    let position = light.get("position")?;
                    let pos_x = position.get("x").and_then(|v| v.as_f64())?;
                    let pos_y = position.get("y").and_then(|v| v.as_f64())?;

                    Some(UvttLight {
                        position: UvttPoint { x: pos_x, y: pos_y },
                        range: light.get("range").and_then(|v| v.as_f64()).unwrap_or(5.0),
                        intensity: light.get("intensity").and_then(|v| v.as_f64()).unwrap_or(1.0),
                        color: light
                            .get("color")
                            .and_then(|v| v.as_str())
                            .unwrap_or("ffffffff")
                            .to_string(),
                        shadows: light.get("shadows").and_then(|v| v.as_bool()).unwrap_or(true),
                    })
                })
                .collect()
        });

    // Extract environment
    let environment: Option<UvttEnvironment> = uvtt_json.get("environment").map(|env| {
        UvttEnvironment {
            baked_lighting: env.get("baked_lighting").and_then(|v| v.as_bool()).unwrap_or(false),
            ambient_light: env
                .get("ambient_light")
                .and_then(|v| v.as_str())
                .unwrap_or("ffffffff")
                .to_string(),
        }
    });

    ApiResponse::ok(UvttData {
        resolution: UvttResolution {
            pixels_per_grid,
            map_size: UvttMapSize {
                x: map_size_x,
                y: map_size_y,
            },
        },
        line_of_sight,
        portals,
        lights,
        environment,
    })
}

/// Serve the map image as a data URL.
#[tauri::command]
pub fn serve_map_image(state: State<'_, AppState>, id: String) -> ApiResponse<String> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let mut service = MapService::new(&mut db, &state.paths.app_dir);

    // Get the map
    let map = match service.get(&id) {
        Ok(Some(map)) => map,
        Ok(None) => return ApiResponse::err(format!("Map not found: {}", id)),
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Read the UVTT file
    let uvtt_bytes = match service.read_uvtt_file(&map) {
        Ok(data) => data,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // Parse as JSON
    let uvtt_json: serde_json::Value = match serde_json::from_slice(&uvtt_bytes) {
        Ok(v) => v,
        Err(e) => return ApiResponse::err(format!("Failed to parse UVTT JSON: {}", e)),
    };

    // Extract the base64 image
    let image_b64 = match uvtt_json.get("image").and_then(|v| v.as_str()) {
        Some(img) => img,
        None => return ApiResponse::err("No image found in UVTT file".to_string()),
    };

    // Return as data URL (UVTT images are typically PNG)
    let data_url = format!("data:image/png;base64,{}", image_b64);
    ApiResponse::ok(data_url)
}

// =============================================================================
// Light Source Commands
// =============================================================================

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
fn transform_light_source(ls: LightSource, grid_size_px: i32) -> LightSourceResponse {
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
fn get_map_grid_size_for_lights(service: &mut MapService, map_id: &str) -> i32 {
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

/// List all light sources for a map.
#[tauri::command]
pub fn list_light_sources(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<LightSourceResponse>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get grid size for coordinate conversion
    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    let grid_size_px = get_map_grid_size_for_lights(&mut service, &map_id);

    match dal::list_light_sources(&mut db, &map_id) {
        Ok(lights) => {
            let responses: Vec<LightSourceResponse> = lights
                .into_iter()
                .map(|l| transform_light_source(l, grid_size_px))
                .collect();
            ApiResponse::ok(responses)
        }
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a light source.
#[derive(Debug, Deserialize)]
pub struct CreateLightSourceRequest {
    pub map_id: String,
    pub name: String,
    pub light_type: String,
    pub x: f64,
    pub y: f64,
    pub bright_radius_ft: i32,
    pub dim_radius_ft: i32,
    pub color: Option<String>,
    pub is_active: bool,
}

/// Helper to get a light source and return it as a response with proper coordinates.
fn get_light_response(
    db: &mut diesel::SqliteConnection,
    app_dir: &std::path::Path,
    light_id: &str,
) -> Result<LightSourceResponse, String> {
    let ls = dal::get_light_source(db, light_id).map_err(|e| e.to_string())?;
    let mut service = MapService::new(db, app_dir);
    let grid_size_px = get_map_grid_size_for_lights(&mut service, &ls.map_id);
    Ok(transform_light_source(ls, grid_size_px))
}

/// Create a new light source.
#[tauri::command]
pub fn create_light_source(
    state: State<'_, AppState>,
    request: CreateLightSourceRequest,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get grid size to convert pixel coordinates to grid coordinates
    let mut service = MapService::new(&mut db, &state.paths.app_dir);
    let grid_size_px = get_map_grid_size_for_lights(&mut service, &request.map_id);

    // Convert pixel coordinates to grid coordinates
    let grid_x = (request.x / grid_size_px as f64) as i32;
    let grid_y = (request.y / grid_size_px as f64) as i32;

    let id = Uuid::new_v4().to_string();
    let name_owned = request.name.clone();
    let color_owned = request.color.clone();

    let mut light = NewLightSource::new(
        &id,
        &request.map_id,
        grid_x,
        grid_y,
        request.bright_radius_ft,
        request.dim_radius_ft,
    )
    .with_name(&name_owned);

    if let Some(ref color) = color_owned {
        light = light.with_color(color);
    }

    if !request.is_active {
        light = light.inactive();
    }

    if let Err(e) = dal::insert_light_source(&mut db, &light) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
    }
}

/// Toggle a light source on/off.
#[tauri::command]
pub fn toggle_light_source(state: State<'_, AppState>, id: String) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get current state
    let light = match dal::get_light_source(&mut db, &id) {
        Ok(l) => l,
        Err(e) => return ApiResponse::err(format!("Light source not found: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = if light.is_active() {
        UpdateLightSource::turn_off(&now)
    } else {
        UpdateLightSource::turn_on(&now)
    };

    if let Err(e) = dal::update_light_source(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
    }
}

/// Delete a light source.
#[tauri::command]
pub fn delete_light_source(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::delete_light_source(&mut db, &id) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete all light sources on a map.
#[tauri::command]
pub fn delete_all_light_sources(state: State<'_, AppState>, map_id: String) -> ApiResponse<i32> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::delete_all_light_sources(&mut db, &map_id) {
        Ok(count) => ApiResponse::ok(count as i32),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Create a torch light source (20ft bright, 40ft dim).
#[tauri::command]
pub fn create_torch(
    state: State<'_, AppState>,
    map_id: String,
    x: i32,
    y: i32,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = Uuid::new_v4().to_string();
    let light = NewLightSource::torch(&id, &map_id, x, y);

    if let Err(e) = dal::insert_light_source(&mut db, &light) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
    }
}

/// Create a lantern light source (30ft bright, 60ft dim).
#[tauri::command]
pub fn create_lantern(
    state: State<'_, AppState>,
    map_id: String,
    x: i32,
    y: i32,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = Uuid::new_v4().to_string();
    let light = NewLightSource::lantern(&id, &map_id, x, y);

    if let Err(e) = dal::insert_light_source(&mut db, &light) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
    }
}

/// Request for updating a light source.
#[derive(Debug, Deserialize)]
pub struct UpdateLightSourceRequest {
    pub name: Option<Option<String>>,
    pub bright_radius_ft: Option<i32>,
    pub dim_radius_ft: Option<i32>,
    pub color: Option<Option<String>>,
    pub is_active: Option<bool>,
}

/// Update a light source.
#[tauri::command]
pub fn update_light_source(
    state: State<'_, AppState>,
    id: String,
    request: UpdateLightSourceRequest,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();

    // Build update struct based on what fields are provided
    let name: Option<Option<&str>> = match &request.name {
        Some(inner) => Some(inner.as_deref()),
        None => None,
    };
    let color: Option<Option<&str>> = match &request.color {
        Some(inner) => Some(inner.as_deref()),
        None => None,
    };

    let update = UpdateLightSource {
        grid_x: None,
        grid_y: None,
        name,
        bright_radius: request.bright_radius_ft,
        dim_radius: request.dim_radius_ft,
        color,
        active: request.is_active.map(|a| if a { 1 } else { 0 }),
        updated_at: Some(&now),
    };

    if let Err(e) = dal::update_light_source(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
    }
}

/// Move a light source to a new position.
#[tauri::command]
pub fn move_light_source(
    state: State<'_, AppState>,
    id: String,
    x: i32,
    y: i32,
) -> ApiResponse<LightSourceResponse> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateLightSource::set_position(x, y, &now);

    if let Err(e) = dal::update_light_source(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match get_light_response(&mut db, &state.paths.app_dir, &id) {
        Ok(response) => ApiResponse::ok(response),
        Err(e) => ApiResponse::err(e),
    }
}

// =============================================================================
// Fog of War Commands
// =============================================================================

use mimir_core::models::campaign::{FogRevealedArea, FogState, NewFogRevealedArea, UpdateMap};

/// Get the fog state for a map (enabled + revealed areas).
#[tauri::command]
pub fn get_fog_state(state: State<'_, AppState>, map_id: String) -> ApiResponse<FogState> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get map to check fog_enabled
    let map = match dal::get_map(&mut db, &map_id) {
        Ok(m) => m,
        Err(e) => return ApiResponse::err(format!("Map not found: {}", e)),
    };

    // Get revealed areas
    let revealed_areas = match dal::list_fog_revealed_areas(&mut db, &map_id) {
        Ok(areas) => areas,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    ApiResponse::ok(FogState::new(
        map_id,
        map.is_fog_enabled(),
        revealed_areas,
    ))
}

/// Toggle fog of war on/off for a map.
#[tauri::command]
pub fn toggle_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<bool> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get current fog state
    let map = match dal::get_map(&mut db, &map_id) {
        Ok(m) => m,
        Err(e) => return ApiResponse::err(format!("Map not found: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let new_enabled = !map.is_fog_enabled();
    let update = UpdateMap::set_fog_enabled(new_enabled, &now);

    if let Err(e) = dal::update_map(&mut db, &map_id, &update) {
        return ApiResponse::err(e.to_string());
    }

    ApiResponse::ok(new_enabled)
}

/// Enable fog of war for a map.
#[tauri::command]
pub fn enable_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateMap::enable_fog(&now);

    match dal::update_map(&mut db, &map_id, &update) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Disable fog of war for a map.
#[tauri::command]
pub fn disable_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateMap::disable_fog(&now);

    match dal::update_map(&mut db, &map_id, &update) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for revealing a rectangular area.
#[derive(Debug, Deserialize)]
pub struct RevealRectRequest {
    pub map_id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Reveal a rectangular area on the map.
#[tauri::command]
pub fn reveal_rect(
    state: State<'_, AppState>,
    request: RevealRectRequest,
) -> ApiResponse<FogRevealedArea> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = Uuid::new_v4().to_string();
    let area = NewFogRevealedArea::rect(
        &id,
        &request.map_id,
        request.x,
        request.y,
        request.width,
        request.height,
    );

    if let Err(e) = dal::insert_fog_revealed_area(&mut db, &area) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_fog_revealed_area(&mut db, &id) {
        Ok(area) => ApiResponse::ok(area),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for revealing a circular area.
#[derive(Debug, Deserialize)]
pub struct RevealCircleRequest {
    pub map_id: String,
    pub center_x: f64,
    pub center_y: f64,
    pub radius: f64,
}

/// Reveal a circular area on the map (stored as bounding box).
#[tauri::command]
pub fn reveal_circle(
    state: State<'_, AppState>,
    request: RevealCircleRequest,
) -> ApiResponse<FogRevealedArea> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = Uuid::new_v4().to_string();
    let area = NewFogRevealedArea::circle(
        &id,
        &request.map_id,
        request.center_x,
        request.center_y,
        request.radius,
    );

    if let Err(e) = dal::insert_fog_revealed_area(&mut db, &area) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_fog_revealed_area(&mut db, &id) {
        Ok(area) => ApiResponse::ok(area),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for revealing the entire map.
#[derive(Debug, Deserialize)]
pub struct RevealAllRequest {
    pub map_id: String,
    pub width: f64,
    pub height: f64,
}

/// Reveal the entire map.
#[tauri::command]
pub fn reveal_all(
    state: State<'_, AppState>,
    request: RevealAllRequest,
) -> ApiResponse<FogRevealedArea> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = Uuid::new_v4().to_string();
    let area = NewFogRevealedArea::rect(
        &id,
        &request.map_id,
        0.0,
        0.0,
        request.width,
        request.height,
    );

    if let Err(e) = dal::insert_fog_revealed_area(&mut db, &area) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_fog_revealed_area(&mut db, &id) {
        Ok(area) => ApiResponse::ok(area),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a revealed area.
#[tauri::command]
pub fn delete_revealed_area(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::delete_fog_revealed_area(&mut db, &id) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Reset fog by clearing all revealed areas for a map.
#[tauri::command]
pub fn reset_fog(state: State<'_, AppState>, map_id: String) -> ApiResponse<i32> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::delete_all_fog_revealed_areas(&mut db, &map_id) {
        Ok(count) => ApiResponse::ok(count as i32),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Map Trap Commands
// =============================================================================

use mimir_core::models::campaign::{MapTrap, NewMapTrap, UpdateMapTrap};

/// List all traps for a map.
#[tauri::command]
pub fn list_map_traps(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<MapTrap>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::list_map_traps(&mut db, &map_id) {
        Ok(traps) => ApiResponse::ok(traps),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a map trap by ID.
#[tauri::command]
pub fn get_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a new map trap.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMapTrapRequest {
    pub map_id: String,
    pub name: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub description: Option<String>,
    pub trigger_description: Option<String>,
    pub effect_description: Option<String>,
    pub dc: Option<i32>,
    pub visible: Option<bool>,
}

/// Create a new map trap.
#[tauri::command]
pub fn create_map_trap(
    state: State<'_, AppState>,
    request: CreateMapTrapRequest,
) -> ApiResponse<MapTrap> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = Uuid::new_v4().to_string();
    let mut trap = NewMapTrap::new(&id, &request.map_id, &request.name, request.grid_x, request.grid_y);

    if let Some(desc) = &request.description {
        trap = trap.with_description(desc);
    }
    if let Some(trigger) = &request.trigger_description {
        trap = trap.with_trigger(trigger);
    }
    if let Some(effect) = &request.effect_description {
        trap = trap.with_effect(effect);
    }
    if let Some(dc) = request.dc {
        trap = trap.with_dc(dc);
    }
    if request.visible == Some(true) {
        trap = trap.visible();
    }

    if let Err(e) = dal::insert_map_trap(&mut db, &trap) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for updating a map trap.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMapTrapRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_description: Option<String>,
    pub effect_description: Option<String>,
    pub dc: Option<i32>,
}

/// Update a map trap.
#[tauri::command]
pub fn update_map_trap(
    state: State<'_, AppState>,
    id: String,
    request: UpdateMapTrapRequest,
) -> ApiResponse<MapTrap> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let name_str = request.name;
    let desc_str = request.description;
    let trigger_str = request.trigger_description;
    let effect_str = request.effect_description;

    let update = UpdateMapTrap {
        name: name_str.as_deref(),
        description: desc_str.as_ref().map(|s| Some(s.as_str())),
        trigger_description: trigger_str.as_ref().map(|s| Some(s.as_str())),
        effect_description: effect_str.as_ref().map(|s| Some(s.as_str())),
        dc: request.dc.map(Some),
        updated_at: Some(&now),
        ..Default::default()
    };

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Move a map trap to a new position.
#[tauri::command]
pub fn move_map_trap(
    state: State<'_, AppState>,
    id: String,
    grid_x: i32,
    grid_y: i32,
) -> ApiResponse<MapTrap> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateMapTrap::set_position(grid_x, grid_y, &now);

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Toggle trap visibility for players.
#[tauri::command]
pub fn toggle_map_trap_visibility(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get current state
    let trap = match dal::get_map_trap(&mut db, &id) {
        Ok(t) => t,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateMapTrap::set_visible(!trap.is_visible(), &now);

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Trigger a trap.
#[tauri::command]
pub fn trigger_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateMapTrap::trigger(&now);

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Reset (re-arm) a triggered trap.
#[tauri::command]
pub fn reset_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<MapTrap> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateMapTrap::reset(&now);

    if let Err(e) = dal::update_map_trap(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_trap(&mut db, &id) {
        Ok(trap) => ApiResponse::ok(trap),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a map trap.
#[tauri::command]
pub fn delete_map_trap(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::delete_map_trap(&mut db, &id) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Map POI (Point of Interest) Commands
// =============================================================================

use mimir_core::models::campaign::{MapPoi, NewMapPoi, UpdateMapPoi};

/// List all POIs for a map.
#[tauri::command]
pub fn list_map_pois(state: State<'_, AppState>, map_id: String) -> ApiResponse<Vec<MapPoi>> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::list_map_pois(&mut db, &map_id) {
        Ok(pois) => ApiResponse::ok(pois),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Get a map POI by ID.
#[tauri::command]
pub fn get_map_poi(state: State<'_, AppState>, id: String) -> ApiResponse<MapPoi> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for creating a new map POI.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMapPoiRequest {
    pub map_id: String,
    pub name: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub visible: Option<bool>,
}

/// Create a new map POI.
#[tauri::command]
pub fn create_map_poi(
    state: State<'_, AppState>,
    request: CreateMapPoiRequest,
) -> ApiResponse<MapPoi> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let id = Uuid::new_v4().to_string();
    let mut poi = NewMapPoi::new(&id, &request.map_id, &request.name, request.grid_x, request.grid_y);

    if let Some(desc) = &request.description {
        poi = poi.with_description(desc);
    }
    if let Some(icon) = &request.icon {
        poi = poi.with_icon(icon);
    }
    if let Some(color) = &request.color {
        poi = poi.with_color(color);
    }
    if request.visible == Some(true) {
        poi = poi.visible();
    }

    if let Err(e) = dal::insert_map_poi(&mut db, &poi) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Request for updating a map POI.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMapPoiRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

/// Update a map POI.
#[tauri::command]
pub fn update_map_poi(
    state: State<'_, AppState>,
    id: String,
    request: UpdateMapPoiRequest,
) -> ApiResponse<MapPoi> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let name_str = request.name;
    let desc_str = request.description;
    let icon_str = request.icon;
    let color_str = request.color;

    let update = UpdateMapPoi {
        name: name_str.as_deref(),
        description: desc_str.as_ref().map(|s| Some(s.as_str())),
        icon: icon_str.as_deref(),
        color: color_str.as_ref().map(|s| Some(s.as_str())),
        updated_at: Some(&now),
        ..Default::default()
    };

    if let Err(e) = dal::update_map_poi(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Move a map POI to a new position.
#[tauri::command]
pub fn move_map_poi(
    state: State<'_, AppState>,
    id: String,
    grid_x: i32,
    grid_y: i32,
) -> ApiResponse<MapPoi> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateMapPoi::set_position(grid_x, grid_y, &now);

    if let Err(e) = dal::update_map_poi(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Toggle POI visibility for players.
#[tauri::command]
pub fn toggle_map_poi_visibility(state: State<'_, AppState>, id: String) -> ApiResponse<MapPoi> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Get current state
    let poi = match dal::get_map_poi(&mut db, &id) {
        Ok(p) => p,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let update = UpdateMapPoi::set_visible(!poi.is_visible(), &now);

    if let Err(e) = dal::update_map_poi(&mut db, &id, &update) {
        return ApiResponse::err(e.to_string());
    }

    match dal::get_map_poi(&mut db, &id) {
        Ok(poi) => ApiResponse::ok(poi),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Delete a map POI.
#[tauri::command]
pub fn delete_map_poi(state: State<'_, AppState>, id: String) -> ApiResponse<()> {
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match dal::delete_map_poi(&mut db, &id) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Simple base64 encoding using the standard library.
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Simple base64 decoding.
fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s)
}
