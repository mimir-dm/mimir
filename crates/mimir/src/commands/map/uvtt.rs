//! UVTT Data Commands
//!
//! Commands for reading and parsing UVTT (Universal VTT) file data.

use mimir_core::services::MapService;
use serde::Serialize;
use tauri::State;

use super::base64_encode;
use crate::commands::ApiResponse;
use crate::state::AppState;

// =============================================================================
// UVTT Data Types
// =============================================================================

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

// =============================================================================
// UVTT Commands
// =============================================================================

/// Read the UVTT file data for a map (returned as base64).
#[tauri::command]
pub fn read_map_uvtt(state: State<'_, AppState>, map_id: String) -> ApiResponse<String> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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

/// Get parsed UVTT data for a map (resolution, grid size, etc).
#[tauri::command]
pub fn get_uvtt_map(state: State<'_, AppState>, id: String) -> ApiResponse<UvttData> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
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
