//! Token management command handlers.
//!
//! Commands for managing map tokens - placing, moving, and controlling
//! visibility of monsters, PCs, NPCs, traps, and markers on battle maps.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use tracing::{error, info, debug};
use mimir_dm_core::models::campaign::{NewToken, Token, TokenSummary, TokenSize, TokenType, UpdateToken, VisionType};
use mimir_dm_core::services::TokenService;
use mimir_dm_core::schema::{catalog_monsters, tokens as tokens_schema};
use diesel::prelude::*;
use serde::Deserialize;
use tauri::State;

/// Request to create a new token
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub map_id: i32,
    pub name: String,
    pub token_type: String,
    pub size: String,
    pub x: f32,
    pub y: f32,
    pub visible_to_players: Option<bool>,
    pub color: Option<String>,
    pub image_path: Option<String>,
    pub monster_id: Option<i32>,
    pub character_id: Option<i32>,
    pub notes: Option<String>,
    pub vision_type: Option<String>,
    pub vision_range_ft: Option<f32>,
}

/// Request to update a token
#[derive(Debug, Deserialize)]
pub struct UpdateTokenRequest {
    pub name: Option<String>,
    pub token_type: Option<String>,
    pub size: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub visible_to_players: Option<bool>,
    pub color: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub vision_type: Option<String>,
    pub vision_range_ft: Option<f32>,
}

/// Request to update multiple token positions
#[derive(Debug, Deserialize)]
pub struct BulkPositionUpdate {
    pub id: i32,
    pub x: f32,
    pub y: f32,
}

/// Create a new token on a map.
///
/// # Parameters
/// - `request` - Token creation data
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the created `Token`.
#[tauri::command]
pub async fn create_token(
    request: CreateTokenRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Token>, ApiError> {
    info!("Creating token '{}' on map {}", request.name, request.map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    let mut new_token = NewToken::new(request.map_id, request.name, request.x, request.y)
        .with_type(TokenType::from_str(&request.token_type))
        .with_size(TokenSize::from_str(&request.size));

    if let Some(visible) = request.visible_to_players {
        new_token = new_token.with_visibility(visible);
    }
    if let Some(color) = request.color {
        new_token = new_token.with_color(color);
    }
    if let Some(notes) = request.notes {
        new_token = new_token.with_notes(notes);
    }
    // Use image_path directly - monster_id is not stable across reimports
    if let Some(image_path) = request.image_path {
        debug!("Using provided image_path: {}", image_path);
        new_token.image_path = Some(image_path);
    }
    if let Some(character_id) = request.character_id {
        new_token.character_id = Some(character_id);
    }
    if let Some(vision_type_str) = request.vision_type {
        let vision_type = VisionType::from_str(&vision_type_str);
        new_token = new_token.with_vision(vision_type, request.vision_range_ft);
    }

    match service.create_token(new_token) {
        Ok(token) => {
            info!("Token created with ID: {}", token.id);
            Ok(ApiResponse::success(token))
        }
        Err(e) => {
            error!("Failed to create token: {}", e);
            Ok(ApiResponse::error(format!("Failed to create token: {}", e)))
        }
    }
}

/// Get a token by ID.
///
/// # Parameters
/// - `id` - Database ID of the token
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the `Token` if found.
#[tauri::command]
pub async fn get_token(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Option<Token>>, ApiError> {
    info!("Getting token with ID: {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    match service.get_token(id) {
        Ok(token) => Ok(ApiResponse::success(token)),
        Err(e) => {
            error!("Failed to get token: {}", e);
            Ok(ApiResponse::error(format!("Failed to get token: {}", e)))
        }
    }
}

/// List all tokens for a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a vector of `Token` objects.
#[tauri::command]
pub async fn list_tokens(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Token>>, ApiError> {
    info!("Listing tokens for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    match service.list_tokens_for_map(map_id) {
        Ok(tokens) => {
            info!("Found {} tokens", tokens.len());
            Ok(ApiResponse::success(tokens))
        }
        Err(e) => {
            error!("Failed to list tokens: {}", e);
            Ok(ApiResponse::error(format!("Failed to list tokens: {}", e)))
        }
    }
}

/// List visible tokens for a map (for player display).
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a vector of visible `Token` objects.
#[tauri::command]
pub async fn list_visible_tokens(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Token>>, ApiError> {
    info!("Listing visible tokens for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    match service.list_visible_tokens_for_map(map_id) {
        Ok(tokens) => {
            info!("Found {} visible tokens", tokens.len());
            Ok(ApiResponse::success(tokens))
        }
        Err(e) => {
            error!("Failed to list visible tokens: {}", e);
            Ok(ApiResponse::error(format!("Failed to list visible tokens: {}", e)))
        }
    }
}

/// List token summaries with resolved entity names.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a vector of `TokenSummary` objects.
#[tauri::command]
pub async fn list_token_summaries(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<TokenSummary>>, ApiError> {
    info!("Listing token summaries for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    match service.list_token_summaries(map_id) {
        Ok(summaries) => {
            info!("Found {} token summaries", summaries.len());
            Ok(ApiResponse::success(summaries))
        }
        Err(e) => {
            error!("Failed to list token summaries: {}", e);
            Ok(ApiResponse::error(format!("Failed to list token summaries: {}", e)))
        }
    }
}

/// Update a token.
///
/// # Parameters
/// - `id` - Database ID of the token
/// - `request` - Fields to update
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `Token`.
#[tauri::command]
pub async fn update_token(
    id: i32,
    request: UpdateTokenRequest,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Token>, ApiError> {
    info!("Updating token with ID: {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    let update = UpdateToken {
        name: request.name,
        token_type: request.token_type,
        size: request.size,
        x: request.x,
        y: request.y,
        visible_to_players: request.visible_to_players,
        color: request.color,
        image_path: None,
        notes: request.notes,
        updated_at: None, // Service handles this
        vision_type: request.vision_type,
        vision_range_ft: request.vision_range_ft.map(Some),
    };

    match service.update_token(id, update) {
        Ok(token) => {
            info!("Token updated successfully");
            Ok(ApiResponse::success(token))
        }
        Err(e) => {
            error!("Failed to update token: {}", e);
            Ok(ApiResponse::error(format!("Failed to update token: {}", e)))
        }
    }
}

/// Update a token's position (for drag operations).
///
/// # Parameters
/// - `id` - Database ID of the token
/// - `x` - New X position (grid coordinates)
/// - `y` - New Y position (grid coordinates)
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `Token`.
#[tauri::command]
pub async fn update_token_position(
    id: i32,
    x: f32,
    y: f32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Token>, ApiError> {
    info!("Updating token {} position to ({}, {})", id, x, y);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    match service.update_token_position(id, x, y) {
        Ok(token) => {
            info!("Token position updated");
            Ok(ApiResponse::success(token))
        }
        Err(e) => {
            error!("Failed to update token position: {}", e);
            Ok(ApiResponse::error(format!("Failed to update token position: {}", e)))
        }
    }
}

/// Update multiple token positions at once (for group moves).
///
/// # Parameters
/// - `updates` - Vector of position updates
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `Token` objects.
#[tauri::command]
pub async fn bulk_update_token_positions(
    updates: Vec<BulkPositionUpdate>,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Token>>, ApiError> {
    info!("Bulk updating {} token positions", updates.len());

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    let update_tuples: Vec<(i32, f32, f32)> = updates
        .into_iter()
        .map(|u| (u.id, u.x, u.y))
        .collect();

    match service.bulk_update_positions(update_tuples) {
        Ok(tokens) => {
            info!("Updated {} token positions", tokens.len());
            Ok(ApiResponse::success(tokens))
        }
        Err(e) => {
            error!("Failed to bulk update token positions: {}", e);
            Ok(ApiResponse::error(format!("Failed to bulk update token positions: {}", e)))
        }
    }
}

/// Toggle a token's visibility.
///
/// # Parameters
/// - `id` - Database ID of the token
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `Token`.
#[tauri::command]
pub async fn toggle_token_visibility(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Token>, ApiError> {
    info!("Toggling visibility for token {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    match service.toggle_token_visibility(id) {
        Ok(token) => {
            info!("Token visibility toggled to: {}", token.visible_to_players);
            Ok(ApiResponse::success(token))
        }
        Err(e) => {
            error!("Failed to toggle token visibility: {}", e);
            Ok(ApiResponse::error(format!("Failed to toggle token visibility: {}", e)))
        }
    }
}

/// Set a token's visibility explicitly.
///
/// # Parameters
/// - `id` - Database ID of the token
/// - `visible` - Whether token should be visible to players
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the updated `Token`.
#[tauri::command]
pub async fn set_token_visibility(
    id: i32,
    visible: bool,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Token>, ApiError> {
    info!("Setting token {} visibility to {}", id, visible);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    match service.set_token_visibility(id, visible) {
        Ok(token) => {
            info!("Token visibility set");
            Ok(ApiResponse::success(token))
        }
        Err(e) => {
            error!("Failed to set token visibility: {}", e);
            Ok(ApiResponse::error(format!("Failed to set token visibility: {}", e)))
        }
    }
}

/// Delete a token.
///
/// # Parameters
/// - `id` - Database ID of the token
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` indicating success.
#[tauri::command]
pub async fn delete_token(
    id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!("Deleting token with ID: {}", id);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    match service.delete_token(id) {
        Ok(()) => {
            info!("Token deleted successfully");
            Ok(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to delete token: {}", e);
            Ok(ApiResponse::error(format!("Failed to delete token: {}", e)))
        }
    }
}

/// Delete all tokens for a map.
///
/// # Parameters
/// - `map_id` - Database ID of the map
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing the number of tokens deleted.
#[tauri::command]
pub async fn delete_tokens_for_map(
    map_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<usize>, ApiError> {
    info!("Deleting all tokens for map {}", map_id);

    let mut conn = state.db.get_connection()?;
    let mut service = TokenService::new(&mut conn);

    match service.delete_tokens_for_map(map_id) {
        Ok(count) => {
            info!("Deleted {} tokens", count);
            Ok(ApiResponse::success(count))
        }
        Err(e) => {
            error!("Failed to delete tokens for map: {}", e);
            Ok(ApiResponse::error(format!("Failed to delete tokens for map: {}", e)))
        }
    }
}

/// Serve a token's image as base64.
///
/// Looks up the token, finds the associated monster's source,
/// and serves the token image from the book archive.
///
/// # Parameters
/// - `token_id` - Database ID of the token
/// - `state` - Application state
///
/// # Returns
/// `ApiResponse` containing a base64 data URL for the token image.
#[tauri::command]
pub async fn serve_token_image(
    token_id: i32,
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, ApiError> {
    info!("Serving token image for token {}", token_id);

    let mut conn = state.db.get_connection()?;

    // Get the token
    let token: Option<Token> = tokens_schema::table
        .find(token_id)
        .first(&mut conn)
        .optional()
        .map_err(|e| format!("Failed to get token: {}", e))?;

    let token = match token {
        Some(t) => t,
        None => {
            error!("Token not found: {}", token_id);
            return Ok(ApiResponse::error("Token not found".to_string()));
        }
    };

    // Check if token has an image_path
    let image_path = match &token.image_path {
        Some(path) => path,
        None => {
            debug!("Token {} has no image_path", token_id);
            return Ok(ApiResponse::error("Token has no image".to_string()));
        }
    };

    // Get the monster's source to find the right book
    let source: Option<String> = if let Some(monster_id) = token.monster_id {
        catalog_monsters::table
            .filter(catalog_monsters::id.eq(monster_id))
            .select(catalog_monsters::source)
            .first(&mut conn)
            .optional()
            .map_err(|e| format!("Failed to get monster: {}", e))?
    } else {
        None
    };

    let book_id = match source {
        Some(s) => s,
        None => {
            // Try to extract source from path (e.g., "img/bestiary/tokens/MM/Goblin.webp" -> "MM")
            image_path
                .split('/')
                .nth(3)
                .unwrap_or("MM")
                .to_string()
        }
    };

    // Serve the image
    let books_dir = state.paths.data_dir.join("books");
    let full_image_path = books_dir.join(&book_id).join(image_path);

    if !full_image_path.exists() {
        error!("Token image not found: {:?}", full_image_path);
        return Ok(ApiResponse::error("Token image not found".to_string()));
    }

    match std::fs::read(&full_image_path) {
        Ok(image_data) => {
            use base64::{engine::general_purpose::STANDARD, Engine};

            let mime_type = match full_image_path.extension().and_then(|ext| ext.to_str()) {
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("webp") => "image/webp",
                Some("gif") => "image/gif",
                _ => "image/png",
            };

            let base64_data = STANDARD.encode(&image_data);
            let data_url = format!("data:{};base64,{}", mime_type, base64_data);

            info!(
                "Successfully served token image: {} ({}KB)",
                image_path,
                image_data.len() / 1024
            );
            Ok(ApiResponse::success(data_url))
        }
        Err(e) => {
            error!("Failed to read token image: {}", e);
            Ok(ApiResponse::error(format!("Failed to read token image: {}", e)))
        }
    }
}
