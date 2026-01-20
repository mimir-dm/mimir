//! Book archive upload functionality.
//!
//! Provides Tauri commands for uploading and importing book archives created
//! by mimir-5esplit. Handles tar.gz extraction, validation, database recording,
//! and automatic catalog import.

use super::book_content::find_book_content_file;
use super::book_library::BookInfo;
use super::catalog_import::import_all_catalogs_from_book;
use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use chrono::Utc;
use diesel::prelude::*;
use flate2::read::GzDecoder;
use mimir_dm_core::models::catalog::{NewUploadedBook, UploadedBook};
use mimir_dm_core::schema::uploaded_books;
use std::fs;
use std::path::Path;
use tar::Archive;
use tauri::State;
use tracing::{error, info, warn};

/// Upload and extract a book archive (tar.gz format from mimir-5esplit).
///
/// Extracts the archive to a temporary directory for validation, creates a
/// database record, then moves files to their final locations. Automatically
/// imports catalog data (spells, items, monsters, etc.) from the book.
///
/// # Parameters
/// - `archive_path` - Path to the tar.gz archive file to upload
/// - `state` - Application state containing database connection and paths
///
/// # Returns
/// `ApiResponse` containing `BookInfo` with the book ID and display name.
///
/// # Errors
/// Returns an error response if:
/// - The archive file does not exist
/// - The archive cannot be opened or extracted
/// - No book directory is found in the archive
/// - A book with the same ID already exists
/// - Database operations fail
/// - File operations fail during import
#[tauri::command]
pub async fn upload_book_archive(
    archive_path: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<BookInfo>, ApiError> {
    info!("Uploading book archive from: {}", archive_path);

    // Verify archive exists
    let archive_file = Path::new(&archive_path);
    if !archive_file.exists() {
        return Ok(ApiResponse::error(format!(
            "Archive not found: {}",
            archive_path
        )));
    }

    // Open the archive
    let tar_gz =
        fs::File::open(archive_file).map_err(|e| format!("Failed to open archive: {}", e))?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    // Create books directory if it doesn't exist
    let books_dir = state.paths.data_dir.join("books");
    if !books_dir.exists() {
        fs::create_dir_all(&books_dir)
            .map_err(|e| format!("Failed to create books directory: {}", e))?;
        info!("Created books directory at: {:?}", books_dir);
    }

    // Extract to a temporary directory first to validate structure
    let temp_dir =
        tempfile::TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;

    archive
        .unpack(temp_dir.path())
        .map_err(|e| format!("Failed to extract archive: {}", e))?;

    // Find the book directory (should be the only top-level directory)
    let mut book_id = None;
    let mut book_metadata = None;
    let mut book_data = None;

    for entry in fs::read_dir(temp_dir.path())
        .map_err(|e| format!("Failed to read temp directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Found the book directory - use its name as the ID
            book_id = Some(entry.file_name().to_string_lossy().to_string());

            // Look for metadata.json from mimir-5esplit
            let metadata_path = path.join("metadata.json");
            if metadata_path.exists() {
                let metadata_content = fs::read_to_string(&metadata_path)
                    .map_err(|e| format!("Failed to read metadata: {}", e))?;
                book_metadata = serde_json::from_str::<serde_json::Value>(&metadata_content).ok();
            }

            // Look for the main book content
            let book_content_path = find_book_content_file(&path)?;
            if let Some(content_path) = book_content_path {
                let content = fs::read_to_string(&content_path)
                    .map_err(|e| format!("Failed to read book content: {}", e))?;
                book_data = serde_json::from_str::<serde_json::Value>(&content).ok();
            }

            break;
        }
    }

    let book_id = book_id.ok_or_else(|| "No book directory found in archive".to_string())?;

    // Check database for collision before doing any work
    match state.db.get_connection() {
        Ok(mut conn) => {
            let existing: Result<UploadedBook, _> = uploaded_books::table
                .filter(uploaded_books::id.eq(&book_id))
                .first(&mut conn);

            if existing.is_ok() {
                return Ok(ApiResponse::error(format!(
                    "Book '{}' is already uploaded. Please remove it first.",
                    book_id
                )));
            }
        }
        Err(e) => {
            error!("Database connection error during collision check: {}", e);
            return Ok(ApiResponse::error(
                "Database error - please try again".to_string(),
            ));
        }
    }

    // Extract book name from metadata or book data
    let book_name = book_metadata
        .as_ref()
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .or_else(|| {
            book_data
                .as_ref()
                .and_then(|d| d.get("data"))
                .and_then(|d| d.get(0))
                .and_then(|d| d.get("name"))
                .and_then(|n| n.as_str())
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| book_id.clone());

    // Check if book already exists on disk - if so, remove it to allow re-import
    let final_book_dir = books_dir.join(&book_id);
    if final_book_dir.exists() {
        info!(
            "Book directory already exists at {:?}, removing for re-import",
            final_book_dir
        );
        fs::remove_dir_all(&final_book_dir)
            .map_err(|e| format!("Failed to remove existing book directory: {}", e))?;
    }

    // Create archives directory if it doesn't exist
    let archives_dir = state.paths.data_dir.join("archives");
    if !archives_dir.exists() {
        fs::create_dir_all(&archives_dir)
            .map_err(|e| format!("Failed to create archives directory: {}", e))?;
        info!("Created archives directory at: {:?}", archives_dir);
    }

    // Strategy: Do all database work first, then move files
    // This way if database fails, we haven't moved anything yet

    // Create database record first
    let archive_destination = archives_dir.join(format!("{}.tar.gz", book_id));

    // Remove existing archive if present
    if archive_destination.exists() {
        info!(
            "Archive already exists at {:?}, removing for re-import",
            archive_destination
        );
        fs::remove_file(&archive_destination)
            .map_err(|e| format!("Failed to remove existing archive: {}", e))?;
    }
    let new_book = NewUploadedBook {
        id: book_id.clone(),
        name: book_name.clone(),
        location: final_book_dir.to_string_lossy().to_string(),
        archive_path: archive_destination.to_string_lossy().to_string(),
        uploaded_at: Utc::now().to_rfc3339(),
        metadata_json: book_metadata.map(|m| m.to_string()),
    };

    match state.db.get_connection() {
        Ok(mut conn) => {
            match diesel::insert_into(uploaded_books::table)
                .values(&new_book)
                .execute(&mut conn)
            {
                Ok(_) => {
                    info!("Successfully recorded book '{}' in database", book_name);
                }
                Err(e) => {
                    error!("Failed to insert book into database: {}", e);
                    return Ok(ApiResponse::error(
                        "Failed to record book in database".to_string(),
                    ));
                }
            }
        }
        Err(e) => {
            error!("Database connection error during insert: {}", e);
            return Ok(ApiResponse::error(
                "Database error - upload failed".to_string(),
            ));
        }
    }

    // Now that database is committed, move the files
    // If this fails, we have the database record so user can retry or we can implement repair

    // Copy archive to archives directory
    if let Err(e) = fs::copy(archive_file, &archive_destination) {
        error!("Failed to copy archive after database insert: {}", e);
        // Clean up database record
        if let Ok(mut conn) = state.db.get_connection() {
            let _ = diesel::delete(uploaded_books::table.filter(uploaded_books::id.eq(&book_id)))
                .execute(&mut conn);
        }
        return Ok(ApiResponse::error(
            "Failed to copy archive file".to_string(),
        ));
    }

    // Move the entire extracted book directory to its final location
    let temp_book_dir = temp_dir.path().join(&book_id);
    if let Err(e) = fs::rename(&temp_book_dir, &final_book_dir)
        .or_else(|_| copy_dir_recursive(&temp_book_dir, &final_book_dir))
    {
        error!("Failed to move book directory after database insert: {}", e);
        // Clean up archive and database record
        let _ = fs::remove_file(&archive_destination);
        if let Ok(mut conn) = state.db.get_connection() {
            let _ = diesel::delete(uploaded_books::table.filter(uploaded_books::id.eq(&book_id)))
                .execute(&mut conn);
        }
        return Ok(ApiResponse::error(
            "Failed to move book directory".to_string(),
        ));
    }

    info!("Successfully imported book '{}'", book_name);

    // Import catalog content automatically
    match state.db.get_connection() {
        Ok(mut catalog_conn) => {
            import_all_catalogs_from_book(&mut catalog_conn, &final_book_dir, &book_id);
        }
        Err(e) => {
            warn!("Book uploaded successfully but couldn't connect to database for catalog import: {}", e);
        }
    }

    // Return simple BookInfo
    Ok(ApiResponse::success(BookInfo {
        id: book_id,
        name: book_name,
    }))
}

/// Copy directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}
