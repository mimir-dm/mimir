//! Map management commands for Visual Display System.
//!
//! Provides Tauri commands for uploading, managing, and serving battle maps,
//! dungeon maps, and regional maps for visual display during in-person play sessions.
//!
//! Maps are stored in Universal VTT (.dd2vtt) format which contains the image,
//! grid config, LOS walls, portals, and lights.
//!
//! Storage paths:
//! - Campaign maps: `{data_dir}/campaigns/{campaign_id}/maps/{uuid}.dd2vtt`
//! - Module maps: `{data_dir}/modules/{module_id}/maps/{uuid}.dd2vtt`

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::ImageReader;
use mimir_dm_core::{
    models::campaign::{GridType, Map, MapSummary, NewMap},
    services::MapService,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use tauri::State;
use tracing::{error, info, warn};
use uuid::Uuid;

/// UVTT file format version we generate
const UVTT_FORMAT_VERSION: f32 = 0.3;

// ============================================================================
// UVTT Types
// ============================================================================

/// Summary of a UVTT file's contents for preview/display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UvttSummary {
    /// Grid dimensions (columns x rows)
    pub grid_size: (u32, u32),
    /// Pixels per grid cell
    pub pixels_per_grid: u32,
    /// Map dimensions in pixels
    pub dimensions_px: (u32, u32),
    /// Number of LOS wall segments
    pub wall_count: usize,
    /// Number of portals (doors)
    pub portal_count: usize,
    /// Number of light sources
    pub light_count: usize,
}

/// UVTT file structure for parsing/serializing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UvttFile {
    pub format: f32,
    pub resolution: UvttResolution,
    pub image: String,
    #[serde(default)]
    pub line_of_sight: Vec<Vec<UvttPoint>>,
    #[serde(default)]
    pub portals: Vec<UvttPortal>,
    #[serde(default)]
    pub lights: Vec<UvttLight>,
    #[serde(default)]
    pub environment: Option<UvttEnvironment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UvttResolution {
    pub map_origin: UvttPoint,
    pub map_size: UvttPoint,
    pub pixels_per_grid: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UvttPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UvttPortal {
    pub position: UvttPoint,
    pub bounds: [UvttPoint; 2],
    #[serde(default)]
    pub rotation: f64,
    #[serde(default = "default_true")]
    pub closed: bool,
    #[serde(default)]
    pub freestanding: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UvttLight {
    pub position: UvttPoint,
    pub range: f64,
    #[serde(default = "default_intensity")]
    pub intensity: f64,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_true")]
    pub shadows: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UvttEnvironment {
    #[serde(default)]
    pub baked_lighting: bool,
    /// Ambient light as ARGB hex (e.g., "ffffffff" = bright, "ff000000" = darkness)
    #[serde(default = "default_ambient_light")]
    pub ambient_light: String,
}

fn default_ambient_light() -> String {
    "ffffffff".to_string() // Full bright by default
}

fn default_intensity() -> f64 {
    1.0
}

fn default_color() -> String {
    "#ffffff".to_string()
}

impl UvttFile {
    /// Parse from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| format!("Failed to parse UVTT file: {}", e))
    }

    /// Serialize to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Failed to serialize UVTT file: {}", e))
    }

    /// Get map width in pixels
    pub fn width_px(&self) -> u32 {
        (self.resolution.map_size.x * self.resolution.pixels_per_grid as f64) as u32
    }

    /// Get map height in pixels
    pub fn height_px(&self) -> u32 {
        (self.resolution.map_size.y * self.resolution.pixels_per_grid as f64) as u32
    }

    /// Create summary for frontend display
    pub fn summary(&self) -> UvttSummary {
        UvttSummary {
            grid_size: (
                self.resolution.map_size.x as u32,
                self.resolution.map_size.y as u32,
            ),
            pixels_per_grid: self.resolution.pixels_per_grid,
            dimensions_px: (self.width_px(), self.height_px()),
            wall_count: self.line_of_sight.len(),
            portal_count: self.portals.len(),
            light_count: self.lights.len(),
        }
    }

    /// Create a UVTT wrapper for a plain image (no LOS/portals/lights)
    pub fn from_image(image_base64: String, width_px: u32, height_px: u32, grid_size_px: u32) -> Self {
        let grid_cols = width_px as f64 / grid_size_px as f64;
        let grid_rows = height_px as f64 / grid_size_px as f64;

        Self {
            format: UVTT_FORMAT_VERSION,
            resolution: UvttResolution {
                map_origin: UvttPoint { x: 0.0, y: 0.0 },
                map_size: UvttPoint {
                    x: grid_cols,
                    y: grid_rows,
                },
                pixels_per_grid: grid_size_px,
            },
            image: image_base64,
            line_of_sight: Vec::new(),
            portals: Vec::new(),
            lights: Vec::new(),
            environment: Some(UvttEnvironment {
                baked_lighting: false,
                ambient_light: default_ambient_light(),
            }),
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to upload a new map (either UVTT or image).
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadMapRequest {
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub name: String,
    /// Base64-encoded file data (.dd2vtt or image)
    pub file_data: String,
    /// Original filename for format detection
    pub filename: String,
}

/// Response after uploading a map.
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadMapResponse {
    pub id: i32,
    pub name: String,
    pub file_path: String,
    /// Summary of UVTT contents
    pub summary: UvttSummary,
}

/// Request to list maps.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListMapsRequest {
    pub campaign_id: i32,
    pub module_id: Option<i32>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the storage directory for a map based on campaign/module.
fn get_maps_dir(data_dir: &PathBuf, campaign_id: i32, module_id: Option<i32>) -> PathBuf {
    match module_id {
        Some(mid) => data_dir.join("modules").join(mid.to_string()).join("maps"),
        None => data_dir
            .join("campaigns")
            .join(campaign_id.to_string())
            .join("maps"),
    }
}

/// Check if a filename indicates a UVTT file.
fn is_uvtt_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".dd2vtt") || lower.ends_with(".uvtt")
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Upload a new map (UVTT or image file).
///
/// For UVTT files: stores directly, extracts metadata.
/// For images: wraps in UVTT format with default grid (70px), no LOS.
#[tauri::command]
pub async fn upload_map(
    request: UploadMapRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<UploadMapResponse>, ApiError> {
    info!(
        "Uploading map '{}' for campaign {} (module: {:?})",
        request.name, request.campaign_id, request.module_id
    );

    // Decode base64 file data
    let file_bytes = match STANDARD.decode(&request.file_data) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to decode base64 file data: {}", e);
            return Ok(ApiResponse::error(format!("Invalid file data: {}", e)));
        }
    };

    // Parse or create UVTT structure
    let uvtt = if is_uvtt_file(&request.filename) {
        // Parse existing UVTT file
        match UvttFile::from_bytes(&file_bytes) {
            Ok(uvtt) => uvtt,
            Err(e) => {
                error!("Failed to parse UVTT file: {}", e);
                return Ok(ApiResponse::error(format!("Invalid UVTT file: {}", e)));
            }
        }
    } else {
        // Image file - wrap in UVTT format
        // Detect image dimensions
        let (width, height) = ImageReader::new(Cursor::new(&file_bytes))
            .with_guessed_format()
            .ok()
            .and_then(|reader| reader.into_dimensions().ok())
            .unwrap_or_else(|| {
                warn!("Failed to detect image dimensions, using defaults");
                (1400, 1050)
            });

        let image_base64 = STANDARD.encode(&file_bytes);

        // Default 70px grid - will be configured after upload
        UvttFile::from_image(image_base64, width, height, 70)
    };

    // Create storage directory
    let maps_dir = get_maps_dir(&state.paths.data_dir, request.campaign_id, request.module_id);
    if let Err(e) = fs::create_dir_all(&maps_dir) {
        error!("Failed to create maps directory: {}", e);
        return Ok(ApiResponse::error(format!(
            "Failed to create maps directory: {}",
            e
        )));
    }

    // Generate unique filename
    let unique_id = Uuid::new_v4();
    let stored_filename = format!("{}.dd2vtt", unique_id);
    let file_path = maps_dir.join(&stored_filename);

    // Serialize and save UVTT file
    let uvtt_bytes = match uvtt.to_bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to serialize UVTT: {}", e);
            return Ok(ApiResponse::error(format!("Failed to save map: {}", e)));
        }
    };

    if let Err(e) = fs::write(&file_path, &uvtt_bytes) {
        error!("Failed to write UVTT file: {}", e);
        return Ok(ApiResponse::error(format!("Failed to save map: {}", e)));
    }

    info!(
        "Saved UVTT map to {:?} ({}KB, {}x{} grid)",
        file_path,
        uvtt_bytes.len() / 1024,
        uvtt.resolution.map_size.x as u32,
        uvtt.resolution.map_size.y as u32,
    );

    // Create database record
    let width_px = uvtt.width_px() as i32;
    let height_px = uvtt.height_px() as i32;
    let grid_size_px = uvtt.resolution.pixels_per_grid as i32;

    let new_map = NewMap::new(
        request.campaign_id,
        request.name.clone(),
        stored_filename.clone(),
        width_px,
        height_px,
        width_px, // Original dimensions same as current for UVTT
        height_px,
    )
    .with_grid(GridType::Square, grid_size_px, 0, 0);

    let new_map = if let Some(module_id) = request.module_id {
        new_map.with_module(module_id)
    } else {
        new_map
    };

    let mut conn = state.db.get_connection()?;
    let mut service = MapService::new(&mut conn);

    match service.create_map(new_map) {
        Ok(map) => {
            info!("Map created with ID: {}", map.id);
            let summary = uvtt.summary();
            Ok(ApiResponse::success(UploadMapResponse {
                id: map.id,
                name: map.name,
                file_path: stored_filename,
                summary,
            }))
        }
        Err(e) => {
            // Clean up the saved file on failure
            let _ = fs::remove_file(&file_path);
            error!("Failed to create map record: {}", e);
            Ok(ApiResponse::error(format!("Failed to create map: {}", e)))
        }
    }
}

/// Get a map by ID.
///
/// # Parameters
/// - `id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the `Map` if found.
#[tauri::command]
pub async fn get_map(id: i32, state: State<'_, AppState>) -> Result<ApiResponse<Map>, ApiError> {
    info!("Getting map with ID: {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = MapService::new(&mut conn);

    match service.get_map(id) {
        Ok(Some(map)) => {
            info!("Map found: {}", map.name);
            Ok(ApiResponse::success(map))
        }
        Ok(None) => {
            info!("Map not found with ID: {}", id);
            Ok(ApiResponse::error(format!(
                "Map not found with ID: {}",
                id
            )))
        }
        Err(e) => {
            error!("Failed to get map: {}", e);
            Ok(ApiResponse::error(format!("Failed to get map: {}", e)))
        }
    }
}

/// List maps for a campaign or module.
///
/// If module_id is provided, returns only maps for that module.
/// Otherwise, returns all campaign-level maps (not tied to a module).
///
/// # Parameters
/// - `request` - Request with campaign_id and optional module_id
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a vector of `Map` objects.
#[tauri::command]
pub async fn list_maps(
    request: ListMapsRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Map>>, ApiError> {
    info!(
        "Listing maps for campaign {} (module: {:?})",
        request.campaign_id, request.module_id
    );

    let mut conn = state.db.get_connection()?;
    let mut service = MapService::new(&mut conn);

    let maps = if let Some(module_id) = request.module_id {
        service.list_module_maps(module_id)
    } else {
        service.list_campaign_maps(request.campaign_id)
    };

    match maps {
        Ok(maps) => {
            info!("Found {} maps", maps.len());
            Ok(ApiResponse::success(maps))
        }
        Err(e) => {
            error!("Failed to list maps: {}", e);
            Ok(ApiResponse::error(format!("Failed to list maps: {}", e)))
        }
    }
}

/// List all maps for a campaign with module names.
///
/// Returns map summaries including which module each map belongs to.
///
/// # Parameters
/// - `campaign_id` - Database ID of the campaign
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a vector of `MapSummary` objects.
#[tauri::command]
pub async fn list_map_summaries(
    campaign_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<MapSummary>>, ApiError> {
    info!("Listing map summaries for campaign {}", campaign_id);

    let mut conn = state.db.get_connection()?;
    let mut service = MapService::new(&mut conn);

    match service.list_map_summaries(campaign_id) {
        Ok(summaries) => {
            info!("Found {} map summaries", summaries.len());
            Ok(ApiResponse::success(summaries))
        }
        Err(e) => {
            error!("Failed to list map summaries: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to list map summaries: {}",
                e
            )))
        }
    }
}

/// Update a map's grid configuration.
///
/// Convenience command for updating just the grid settings.
///
/// # Parameters
/// - `id` - Database ID of the map
/// - `grid_type` - Grid type ("square", "hex", or "none")
/// - `grid_size_px` - Pixels per grid cell (None to remove grid)
/// - `offset_x` - Grid X offset for alignment
/// - `offset_y` - Grid Y offset for alignment
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `Map`.
#[tauri::command]
pub async fn update_map_grid(
    id: i32,
    grid_type: String,
    grid_size_px: Option<i32>,
    offset_x: i32,
    offset_y: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Map>, ApiError> {
    info!(
        "Updating grid for map {}: type={}, size={:?}, offset=({}, {})",
        id, grid_type, grid_size_px, offset_x, offset_y
    );

    let mut conn = state.db.get_connection()?;
    let mut service = MapService::new(&mut conn);

    match service.update_grid_config(id, &grid_type, grid_size_px, offset_x, offset_y) {
        Ok(map) => {
            info!("Map grid updated successfully");
            Ok(ApiResponse::success(map))
        }
        Err(e) => {
            error!("Failed to update map grid: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to update map grid: {}",
                e
            )))
        }
    }
}

/// Delete a map.
///
/// Removes both the database record and the stored image file.
///
/// # Parameters
/// - `id` - Database ID of the map to delete
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` with success or error status.
#[tauri::command]
pub async fn delete_map(id: i32, state: State<'_, AppState>) -> Result<ApiResponse<()>, ApiError> {
    info!("Deleting map with ID: {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = MapService::new(&mut conn);

    // Get the map first to know the image path
    let map = match service.get_map(id) {
        Ok(Some(map)) => map,
        Ok(None) => {
            return Ok(ApiResponse::error(format!(
                "Map not found with ID: {}",
                id
            )));
        }
        Err(e) => {
            error!("Failed to get map for deletion: {}", e);
            return Ok(ApiResponse::error(format!("Failed to get map: {}", e)));
        }
    };

    // Delete the database record
    if let Err(e) = service.delete_map(id) {
        error!("Failed to delete map record: {}", e);
        return Ok(ApiResponse::error(format!("Failed to delete map: {}", e)));
    }

    // Delete the image file - handle both UVTT and legacy paths
    let is_uvtt = map.image_path.ends_with(".dd2vtt");
    let image_path = if is_uvtt {
        get_maps_dir(&state.paths.data_dir, map.campaign_id, map.module_id).join(&map.image_path)
    } else {
        state.paths.data_dir.join("maps").join(&map.image_path)
    };

    if image_path.exists() {
        if let Err(e) = fs::remove_file(&image_path) {
            // Log but don't fail - the DB record is already deleted
            error!(
                "Warning: Failed to delete image file {:?}: {}",
                image_path, e
            );
        } else {
            info!("Deleted image file: {:?}", image_path);
        }
    }

    info!("Map deleted successfully");
    Ok(ApiResponse::success(()))
}

/// Serve a map image as base64 data URL.
///
/// Reads the map image from storage and returns it as a base64-encoded
/// data URL suitable for display in the frontend.
///
/// # Parameters
/// - `id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a base64 data URL (e.g., "data:image/png;base64,...")
#[tauri::command]
pub async fn serve_map_image(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, ApiError> {
    info!("Serving image for map {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = MapService::new(&mut conn);

    // Get the map to find the image path
    let map = match service.get_map(id) {
        Ok(Some(map)) => map,
        Ok(None) => {
            return Ok(ApiResponse::error(format!(
                "Map not found with ID: {}",
                id
            )));
        }
        Err(e) => {
            error!("Failed to get map: {}", e);
            return Ok(ApiResponse::error(format!("Failed to get map: {}", e)));
        }
    };

    // Check if this is a UVTT file
    let is_uvtt = map.image_path.ends_with(".dd2vtt");

    // Build the correct path based on storage location
    let image_path = if is_uvtt {
        // UVTT files are stored in campaign/module maps directories
        get_maps_dir(&state.paths.data_dir, map.campaign_id, map.module_id).join(&map.image_path)
    } else {
        // Legacy path for old-style maps
        state.paths.data_dir.join("maps").join(&map.image_path)
    };

    if !image_path.exists() {
        error!("Map image not found: {:?}", image_path);
        return Ok(ApiResponse::error(format!(
            "Map image not found: {:?}",
            image_path
        )));
    }

    // Read the file
    let file_data = match fs::read(&image_path) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to read map file: {}", e);
            return Ok(ApiResponse::error(format!("Failed to read map: {}", e)));
        }
    };

    // For UVTT files, extract the image from the JSON
    if is_uvtt {
        // Parse UVTT JSON and extract image
        let uvtt: serde_json::Value = match serde_json::from_slice(&file_data) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse UVTT file: {}", e);
                return Ok(ApiResponse::error(format!("Invalid UVTT file: {}", e)));
            }
        };

        // Get the image field (base64 encoded)
        let image_base64 = match uvtt.get("image").and_then(|v| v.as_str()) {
            Some(img) => img,
            None => {
                error!("UVTT file missing image field");
                return Ok(ApiResponse::error("UVTT file missing image".to_string()));
            }
        };

        // The UVTT image is already base64, just need to add the data URL prefix
        // UVTT images are typically PNG
        let data_url = format!("data:image/png;base64,{}", image_base64);

        info!(
            "Successfully served UVTT map image: {} ({}KB base64)",
            map.image_path,
            image_base64.len() / 1024
        );
        return Ok(ApiResponse::success(data_url));
    }

    // For regular image files
    match fs::read(&image_path) {
        Ok(image_data) => {
            // Determine MIME type based on extension
            let mime_type = match image_path.extension().and_then(|ext| ext.to_str()) {
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("webp") => "image/webp",
                Some("gif") => "image/gif",
                _ => "image/png", // Default to PNG
            };

            // Encode as base64 data URL
            let base64_data = STANDARD.encode(&image_data);
            let data_url = format!("data:{};base64,{}", mime_type, base64_data);

            info!(
                "Successfully served map image: {} ({}KB)",
                map.image_path,
                image_data.len() / 1024
            );
            Ok(ApiResponse::success(data_url))
        }
        Err(e) => {
            error!("Failed to read map image: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to read map image: {}",
                e
            )))
        }
    }
}

/// Get UVTT file contents for a map.
#[tauri::command]
pub async fn get_uvtt_map(
    campaign_id: i32,
    module_id: Option<i32>,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<UvttFile>, ApiError> {
    let maps_dir = get_maps_dir(&state.paths.data_dir, campaign_id, module_id);
    let full_path = maps_dir.join(&file_path);

    if !full_path.exists() {
        return Ok(ApiResponse::error(format!(
            "Map file not found: {}",
            file_path
        )));
    }

    let bytes = match fs::read(&full_path) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to read UVTT file: {}", e);
            return Ok(ApiResponse::error(format!("Failed to read map: {}", e)));
        }
    };

    match UvttFile::from_bytes(&bytes) {
        Ok(uvtt) => Ok(ApiResponse::success(uvtt)),
        Err(e) => Ok(ApiResponse::error(format!("Failed to parse map: {}", e))),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uvtt_from_image() {
        let uvtt = UvttFile::from_image(
            "SGVsbG8=".to_string(), // "Hello" in base64
            1400,
            1050,
            70,
        );

        assert_eq!(uvtt.format, 0.3);
        assert_eq!(uvtt.resolution.pixels_per_grid, 70);
        assert_eq!(uvtt.resolution.map_size.x, 20.0);
        assert_eq!(uvtt.resolution.map_size.y, 15.0);
        assert!(uvtt.line_of_sight.is_empty());
        assert!(uvtt.portals.is_empty());
        assert!(uvtt.lights.is_empty());
    }

    #[test]
    fn test_uvtt_summary() {
        let uvtt = UvttFile::from_image("SGVsbG8=".to_string(), 1400, 1050, 70);
        let summary = uvtt.summary();

        assert_eq!(summary.grid_size, (20, 15));
        assert_eq!(summary.pixels_per_grid, 70);
        assert_eq!(summary.dimensions_px, (1400, 1050));
        assert_eq!(summary.wall_count, 0);
        assert_eq!(summary.portal_count, 0);
        assert_eq!(summary.light_count, 0);
    }

    #[test]
    fn test_is_uvtt_file() {
        assert!(is_uvtt_file("map.dd2vtt"));
        assert!(is_uvtt_file("Map.DD2VTT"));
        assert!(is_uvtt_file("test.uvtt"));
        assert!(!is_uvtt_file("map.png"));
        assert!(!is_uvtt_file("map.jpg"));
    }

    #[test]
    fn test_get_maps_dir() {
        let data_dir = PathBuf::from("/data");

        // Campaign-level map
        let dir = get_maps_dir(&data_dir, 1, None);
        assert_eq!(dir, PathBuf::from("/data/campaigns/1/maps"));

        // Module-level map
        let dir = get_maps_dir(&data_dir, 1, Some(5));
        assert_eq!(dir, PathBuf::from("/data/modules/5/maps"));
    }

    #[test]
    fn test_uvtt_roundtrip() {
        let original = UvttFile {
            format: 0.3,
            resolution: UvttResolution {
                map_origin: UvttPoint { x: 0.0, y: 0.0 },
                map_size: UvttPoint { x: 20.0, y: 15.0 },
                pixels_per_grid: 70,
            },
            image: "SGVsbG8=".to_string(),
            environment: None,
            line_of_sight: vec![vec![
                UvttPoint { x: 0.0, y: 0.0 },
                UvttPoint { x: 10.0, y: 0.0 },
            ]],
            portals: vec![UvttPortal {
                position: UvttPoint { x: 5.0, y: 0.0 },
                bounds: [
                    UvttPoint { x: 4.5, y: 0.0 },
                    UvttPoint { x: 5.5, y: 0.0 },
                ],
                rotation: 0.0,
                closed: true,
                freestanding: false,
            }],
            lights: vec![UvttLight {
                position: UvttPoint { x: 5.0, y: 5.0 },
                range: 6.0,
                intensity: 0.8,
                color: "#ffaa00".to_string(),
                shadows: true,
            }],
        };

        let bytes = original.to_bytes().unwrap();
        let parsed = UvttFile::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.format, original.format);
        assert_eq!(
            parsed.resolution.pixels_per_grid,
            original.resolution.pixels_per_grid
        );
        assert_eq!(parsed.line_of_sight.len(), 1);
        assert_eq!(parsed.portals.len(), 1);
        assert_eq!(parsed.lights.len(), 1);
        assert!(parsed.portals[0].closed);
        assert_eq!(parsed.lights[0].color, "#ffaa00");
    }
}
