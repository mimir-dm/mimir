//! Map management commands for Visual Display System.
//!
//! Provides Tauri commands for uploading, managing, and serving battle maps,
//! dungeon maps, and regional maps for visual display during in-person play sessions.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::ImageReader;
use mimir_dm_core::{
    models::campaign::{Map, MapSummary, NewMap, UpdateMap},
    services::MapService,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use tauri::State;
use tracing::{error, info};
use uuid::Uuid;

/// Maximum dimension (width or height) for uploaded map images.
/// Images larger than this will be scaled down proportionally.
const MAX_MAP_DIMENSION: u32 = 4096;

/// JPEG quality for processed map images (0-100).
const JPEG_QUALITY: u8 = 85;

/// Request to upload a new map image.
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadMapRequest {
    pub campaign_id: i32,
    pub module_id: Option<i32>,
    pub name: String,
    /// Base64-encoded image data
    pub image_data: String,
    /// Original filename for extension detection
    pub filename: String,
    pub width_px: i32,
    pub height_px: i32,
}

/// Request to update map properties.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMapRequest {
    pub name: Option<String>,
    pub grid_type: Option<String>,
    pub grid_size_px: Option<Option<i32>>,
    pub grid_offset_x: Option<i32>,
    pub grid_offset_y: Option<i32>,
}

/// Request to list maps.
#[derive(Debug, Serialize, Deserialize)]
pub struct ListMapsRequest {
    pub campaign_id: i32,
    pub module_id: Option<i32>,
}

/// Process an uploaded map image: resize if needed and convert to JPEG.
///
/// # Arguments
/// * `image_bytes` - Raw image bytes (PNG, JPEG, WebP, etc.)
/// * `max_dimension` - Maximum width or height in pixels
///
/// # Returns
/// Tuple of (processed_jpeg_bytes, width, height) or an error message.
fn process_map_image(
    image_bytes: &[u8],
    max_dimension: u32,
) -> Result<(Vec<u8>, u32, u32), String> {
    // Decode the image from bytes
    let img = ImageReader::new(Cursor::new(image_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to detect image format: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let (orig_width, orig_height) = (img.width(), img.height());
    info!(
        "Processing map image: {}x{} (max: {})",
        orig_width, orig_height, max_dimension
    );

    // Resize if either dimension exceeds the max
    let img = if orig_width > max_dimension || orig_height > max_dimension {
        let scale = max_dimension as f32 / orig_width.max(orig_height) as f32;
        let new_width = (orig_width as f32 * scale) as u32;
        let new_height = (orig_height as f32 * scale) as u32;

        info!(
            "Resizing map from {}x{} to {}x{}",
            orig_width, orig_height, new_width, new_height
        );

        img.resize(new_width, new_height, FilterType::Lanczos3)
    } else {
        img
    };

    let (final_width, final_height) = (img.width(), img.height());

    // Convert to RGB (JPEG doesn't support alpha)
    let rgb_img = img.to_rgb8();

    // Encode as JPEG
    let mut jpeg_bytes = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, JPEG_QUALITY);
    encoder
        .encode_image(&rgb_img)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

    info!(
        "Processed map: {}x{} -> {} KB JPEG",
        final_width,
        final_height,
        jpeg_bytes.len() / 1024
    );

    Ok((jpeg_bytes, final_width, final_height))
}

/// Upload a new map image.
///
/// Accepts base64-encoded image data, stores it in the app data directory,
/// and creates a database record for the map. Images are automatically
/// resized to max 4096px and converted to JPEG for optimal performance.
///
/// # Parameters
/// - `request` - Upload request with image data and metadata
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the created `Map` record.
#[tauri::command]
pub async fn upload_map(
    request: UploadMapRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Map>, ApiError> {
    info!(
        "Uploading map '{}' for campaign {} (module: {:?})",
        request.name, request.campaign_id, request.module_id
    );

    // Create maps directory if it doesn't exist
    let maps_dir = state.paths.data_dir.join("maps");
    if let Err(e) = fs::create_dir_all(&maps_dir) {
        error!("Failed to create maps directory: {}", e);
        return Ok(ApiResponse::error(format!(
            "Failed to create maps directory: {}",
            e
        )));
    }

    // Decode base64 image data
    let raw_bytes = match STANDARD.decode(&request.image_data) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to decode base64 image: {}", e);
            return Ok(ApiResponse::error(format!("Invalid image data: {}", e)));
        }
    };

    // Process image: resize if needed and convert to JPEG
    let (processed_bytes, width, height) = match process_map_image(&raw_bytes, MAX_MAP_DIMENSION) {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to process image: {}", e);
            return Ok(ApiResponse::error(format!("Failed to process image: {}", e)));
        }
    };

    // Generate unique filename (always .jpg now)
    let unique_id = Uuid::new_v4();
    let stored_filename = format!("{}.jpg", unique_id);
    let image_path = maps_dir.join(&stored_filename);

    // Save the processed image
    if let Err(e) = fs::write(&image_path, &processed_bytes) {
        error!("Failed to write image file: {}", e);
        return Ok(ApiResponse::error(format!("Failed to save image: {}", e)));
    }

    info!(
        "Saved processed map to {:?} ({}KB, {}x{})",
        image_path,
        processed_bytes.len() / 1024,
        width,
        height
    );

    // Create database record with processed dimensions and original dimensions
    let new_map = NewMap::new(
        request.campaign_id,
        request.name,
        stored_filename,
        width as i32,
        height as i32,
        request.width_px,  // Original dimensions from frontend
        request.height_px,
    );

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
            Ok(ApiResponse::success(map))
        }
        Err(e) => {
            // Clean up the saved image on failure
            let _ = fs::remove_file(&image_path);
            error!("Failed to create map record: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to create map: {}",
                e
            )))
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
pub async fn get_map(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Map>, ApiError> {
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

/// Update a map's properties.
///
/// # Parameters
/// - `id` - Database ID of the map
/// - `request` - Fields to update
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `Map`.
#[tauri::command]
pub async fn update_map(
    id: i32,
    request: UpdateMapRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Map>, ApiError> {
    info!("Updating map with ID: {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = MapService::new(&mut conn);

    let update = UpdateMap {
        name: request.name,
        grid_type: request.grid_type,
        grid_size_px: request.grid_size_px,
        grid_offset_x: request.grid_offset_x,
        grid_offset_y: request.grid_offset_y,
        updated_at: None, // Service handles this
        fog_enabled: None,
        ambient_light: None,
    };

    match service.update_map(id, update) {
        Ok(map) => {
            info!("Map updated successfully");
            Ok(ApiResponse::success(map))
        }
        Err(e) => {
            error!("Failed to update map: {}", e);
            Ok(ApiResponse::error(format!("Failed to update map: {}", e)))
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
pub async fn delete_map(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
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
            return Ok(ApiResponse::error(format!(
                "Failed to get map: {}",
                e
            )));
        }
    };

    // Delete the database record
    if let Err(e) = service.delete_map(id) {
        error!("Failed to delete map record: {}", e);
        return Ok(ApiResponse::error(format!(
            "Failed to delete map: {}",
            e
        )));
    }

    // Delete the image file
    let image_path = state.paths.data_dir.join("maps").join(&map.image_path);
    if image_path.exists() {
        if let Err(e) = fs::remove_file(&image_path) {
            // Log but don't fail - the DB record is already deleted
            error!("Warning: Failed to delete image file {:?}: {}", image_path, e);
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
            return Ok(ApiResponse::error(format!(
                "Failed to get map: {}",
                e
            )));
        }
    };

    // Check if this is a UVTT file
    let is_uvtt = map.image_path.ends_with(".dd2vtt");

    // Build the correct path based on storage location
    let image_path = if is_uvtt {
        // UVTT files are stored in campaign/module maps directories
        if let Some(module_id) = map.module_id {
            state
                .paths
                .data_dir
                .join("modules")
                .join(module_id.to_string())
                .join("maps")
                .join(&map.image_path)
        } else {
            state
                .paths
                .data_dir
                .join("campaigns")
                .join(map.campaign_id.to_string())
                .join("maps")
                .join(&map.image_path)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    /// Create a test PNG image of given dimensions
    fn create_test_png(width: u32, height: u32) -> Vec<u8> {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_fn(width, height, |x, y| {
                // Simple gradient pattern
                Rgb([
                    (x % 256) as u8,
                    (y % 256) as u8,
                    ((x + y) % 256) as u8,
                ])
            });

        let mut bytes = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .expect("Failed to encode test PNG");
        bytes
    }

    #[test]
    fn test_process_small_image_no_resize() {
        // Image smaller than max - should not be resized
        let png_bytes = create_test_png(1024, 768);
        let (jpeg_bytes, width, height) = process_map_image(&png_bytes, 4096).unwrap();

        assert_eq!(width, 1024);
        assert_eq!(height, 768);
        assert!(!jpeg_bytes.is_empty());
        // JPEG magic bytes
        assert_eq!(&jpeg_bytes[0..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn test_process_wide_image_resized() {
        // Wide image exceeding max width - should be scaled down
        let png_bytes = create_test_png(8000, 4000);
        let (jpeg_bytes, width, height) = process_map_image(&png_bytes, 4096).unwrap();

        assert_eq!(width, 4096);
        assert_eq!(height, 2048); // Proportional scaling
        assert!(!jpeg_bytes.is_empty());
    }

    #[test]
    fn test_process_tall_image_resized() {
        // Tall image exceeding max height - should be scaled down
        let png_bytes = create_test_png(3000, 6000);
        let (jpeg_bytes, width, height) = process_map_image(&png_bytes, 4096).unwrap();

        assert_eq!(width, 2048); // Proportional scaling
        assert_eq!(height, 4096);
        assert!(!jpeg_bytes.is_empty());
    }

    #[test]
    fn test_process_exact_max_size_no_resize() {
        // Image exactly at max size - should not be resized
        let png_bytes = create_test_png(4096, 2048);
        let (_, width, height) = process_map_image(&png_bytes, 4096).unwrap();

        assert_eq!(width, 4096);
        assert_eq!(height, 2048);
    }

    #[test]
    fn test_process_invalid_image_data() {
        // Random bytes - should fail gracefully
        let invalid_bytes = vec![0u8, 1, 2, 3, 4, 5];
        let result = process_map_image(&invalid_bytes, 4096);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to"));
    }

    #[test]
    fn test_output_is_valid_jpeg() {
        let png_bytes = create_test_png(800, 600);
        let (jpeg_bytes, _, _) = process_map_image(&png_bytes, 4096).unwrap();

        // Verify we can decode the output as a valid image
        let decoded = ImageReader::new(Cursor::new(&jpeg_bytes))
            .with_guessed_format()
            .unwrap()
            .decode();

        assert!(decoded.is_ok());
        let img = decoded.unwrap();
        assert_eq!(img.width(), 800);
        assert_eq!(img.height(), 600);
    }
}
