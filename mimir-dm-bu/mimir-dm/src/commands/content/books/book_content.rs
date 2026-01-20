//! Book content serving commands.
//!
//! Provides Tauri commands for retrieving and serving book content from the local
//! archive structure. Handles JSON content extraction and base64 image encoding.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use tracing::{error, info};

/// Get book content from the archive structure.
///
/// Retrieves the main book content JSON file from the extracted archive
/// directory structure. Searches for book-*.json or book.json files.
///
/// # Parameters
/// - `book_id` - The unique book identifier (e.g., "PHB", "DMG")
/// - `app_paths` - Application paths configuration for locating the data directory
///
/// # Returns
/// `ApiResponse` containing the parsed JSON content of the book.
///
/// # Errors
/// Returns an error response if:
/// - The book directory does not exist
/// - No book content file is found in the directory
/// - The JSON content cannot be parsed
#[tauri::command]
pub async fn get_book_content(
    book_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<serde_json::Value>, ApiError> {
    info!("Getting book content for: {}", book_id);

    // Get book directory
    let book_dir = state.paths.data_dir.join("books").join(&book_id);

    info!("Looking for book at: {:?}", book_dir);

    if !book_dir.exists() {
        error!("Book directory does not exist: {:?}", book_dir);
        return Ok(ApiResponse::error(format!("Book not found: {}", book_id)));
    }

    // List contents of book directory for debugging
    info!("Book directory contents:");
    if let Ok(entries) = fs::read_dir(&book_dir) {
        for entry in entries.flatten() {
            info!(
                "  - {:?} ({})",
                entry.file_name(),
                if entry.path().is_dir() { "dir" } else { "file" }
            );
        }
    }

    // Find the main book content file
    info!("Searching for book content file...");
    let book_content_path = find_book_content_file(&book_dir)?.ok_or_else(|| {
        error!("No book content file found in {:?}", book_dir);
        format!("No book content found for: {}", book_id)
    })?;

    // Read and parse JSON
    match fs::read_to_string(&book_content_path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(json) => Ok(ApiResponse::success(json)),
            Err(e) => {
                error!("Failed to parse book JSON: {}", e);
                Ok(ApiResponse::error(format!(
                    "Failed to parse book content: {}",
                    e
                )))
            }
        },
        Err(e) => {
            error!("Failed to read book file: {}", e);
            Ok(ApiResponse::error(format!(
                "Failed to read book content: {}",
                e
            )))
        }
    }
}

/// Serve an image from a book's archive structure as base64.
///
/// Reads an image file from the book's extracted archive and returns it
/// as a base64-encoded data URL suitable for embedding in HTML/CSS.
/// Sanitizes paths to prevent directory traversal attacks.
///
/// # Parameters
/// - `book_id` - The unique book identifier (e.g., "PHB", "DMG")
/// - `image_path` - Relative path to the image within the book archive
/// - `app_paths` - Application paths configuration for locating the data directory
///
/// # Returns
/// `ApiResponse` containing a base64 data URL (e.g., "data:image/png;base64,...")
///
/// # Errors
/// Returns an error response if the image file is not found or cannot be read.
#[tauri::command]
pub async fn serve_book_image(
    book_id: String,
    image_path: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<String>, ApiError> {
    info!("Serving image {} from book {}", image_path, book_id);

    let books_dir = state.paths.data_dir.join("books");

    // Sanitize inputs to prevent directory traversal
    let sanitized_book = book_id.replace("..", "").replace("/", "").replace("\\", "");
    let sanitized_image = image_path.replace("..", "");

    // The image path from JSON is like "book/PHB/image.webp" but files are at "img/book/PHB/image.webp"
    // So we need to prepend "img/" if it's not already there
    let image_path_with_img = if sanitized_image.starts_with("img/") {
        sanitized_image.clone()
    } else {
        format!("img/{}", sanitized_image)
    };

    let full_image_path = books_dir.join(&sanitized_book).join(&image_path_with_img);

    if !full_image_path.exists() {
        error!("Image not found: {:?}", full_image_path);
        return Ok(ApiResponse::error("Image not found".to_string()));
    }

    // Read the image file
    match fs::read(&full_image_path) {
        Ok(image_data) => {
            // Determine MIME type based on extension
            let mime_type = match full_image_path.extension().and_then(|ext| ext.to_str()) {
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
                "Successfully served image: {} ({}KB)",
                image_path_with_img,
                image_data.len() / 1024
            );
            Ok(ApiResponse::success(data_url))
        }
        Err(e) => {
            error!("Failed to read image file: {}", e);
            Ok(ApiResponse::error(format!("Failed to read image: {}", e)))
        }
    }
}

// Helper functions

/// Find the main book content file in the archive structure.
///
/// Searches for book-*.json files in the book/ subdirectory, or book.json in the root.
pub fn find_book_content_file(dir: &Path) -> Result<Option<PathBuf>, ApiError> {
    info!("find_book_content_file: searching in {:?}", dir);

    // Check for book directory with book-*.json files
    let book_dir = dir.join("book");
    info!("Checking for book subdirectory at: {:?}", book_dir);

    if book_dir.exists() {
        info!("Book subdirectory exists, listing contents:");
        for entry in
            fs::read_dir(&book_dir).map_err(|e| format!("Failed to read book directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            info!("  - Found: {:?} (is_file: {})", path, path.is_file());

            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    info!("    Checking filename: {}", name);
                    if name.starts_with("book-") && name.ends_with(".json") {
                        info!("    Found book content file: {:?}", path);
                        return Ok(Some(path));
                    }
                }
            }
        }
        info!("No matching book-*.json files found in book subdirectory");
    } else {
        info!("Book subdirectory does not exist");
    }

    // Check for direct book.json in root
    let root_book = dir.join("book.json");
    info!("Checking for book.json in root: {:?}", root_book);
    if root_book.exists() {
        info!("Found book.json in root");
        return Ok(Some(root_book));
    }

    info!("No book content file found");
    Ok(None)
}
