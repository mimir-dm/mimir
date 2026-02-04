//! Catalog Source Commands
//!
//! Tauri commands for managing catalog sources (importing 5etools data, listing sources, etc.)

use mimir_core::dal::catalog::{self as catalog_dal};
use mimir_core::import::CatalogImportService;
use mimir_core::models::catalog::{BookContent, CatalogSource};
use mimir_core::utils::now_rfc3339;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tar::Archive;
use tauri::State;
use tracing::{error, info, warn};

use super::ApiResponse;
use crate::state::AppState;

/// Response for source listing with frontend-compatible fields.
#[derive(Debug, Serialize)]
pub struct SourceInfo {
    /// Unique source code (acts as ID)
    pub id: String,
    /// Display name
    pub name: String,
    /// Whether enabled
    pub enabled: bool,
    /// When imported
    pub imported_at: String,
}

impl From<CatalogSource> for SourceInfo {
    fn from(source: CatalogSource) -> Self {
        Self {
            id: source.code,
            name: source.name,
            enabled: source.enabled != 0,
            imported_at: source.imported_at,
        }
    }
}

/// Response for import operation.
#[derive(Debug, Serialize)]
pub struct ImportResponse {
    /// Number of sources imported
    pub sources_imported: usize,
    /// Number of sources that failed
    pub sources_failed: usize,
    /// Total entities imported
    pub total_entities: usize,
    /// Summary message
    pub message: String,
}

/// List all imported catalog sources (books).
///
/// This is the command the frontend calls to populate the book list.
#[tauri::command]
pub fn list_catalog_sources(state: State<'_, AppState>) -> ApiResponse<Vec<SourceInfo>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = catalog_dal::list_sources(&mut db);
    match result {
        Ok(sources) => ApiResponse::ok(sources.into_iter().map(SourceInfo::from).collect()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Import catalog data from a tar.gz archive containing 5etools data.
///
/// Streams directly from the archive without extracting to disk.
#[tauri::command]
pub fn import_catalog_from_zip(
    state: State<'_, AppState>,
    archive_path: String,
) -> ApiResponse<ImportResponse> {
    info!("Starting catalog import from: {}", archive_path);

    let archive_path = Path::new(&archive_path);
    if !archive_path.exists() {
        return ApiResponse::err(format!("File not found: {}", archive_path.display()));
    }

    // Get database connection
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Stream import directly from tarball - no extraction needed
    let mut service = CatalogImportService::new(&mut db);

    match service.import_from_tarball(archive_path) {
        Ok(result) => {
            let message = if result.sources_failed.is_empty() {
                format!(
                    "Successfully imported {} sources with {} entities",
                    result.sources_imported.len(),
                    result.total_entities
                )
            } else {
                format!(
                    "Imported {} sources ({} failed) with {} entities",
                    result.sources_imported.len(),
                    result.sources_failed.len(),
                    result.total_entities
                )
            };

            info!("{}", message);
            info!("Full import result: {:?}", result.summary());
            ApiResponse::ok(ImportResponse {
                sources_imported: result.sources_imported.len(),
                sources_failed: result.sources_failed.len(),
                total_entities: result.total_entities,
                message,
            })
        }
        Err(e) => {
            error!("Import failed: {}", e);
            ApiResponse::err(format!("Import failed: {}", e))
        }
    }
}

/// Set the enabled status for a catalog source.
#[tauri::command]
pub fn set_source_enabled(
    state: State<'_, AppState>,
    source_code: String,
    enabled: bool,
) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match catalog_dal::set_enabled(&mut db, &source_code, enabled) {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::err(format!("Failed to update source: {}", e)),
    }
}

/// Delete a catalog source and all its associated entities.
///
/// This cascade deletes all entities (monsters, spells, items, etc.)
/// from this source before removing the source record.
#[tauri::command]
pub fn delete_catalog_source(
    state: State<'_, AppState>,
    source_code: String,
) -> ApiResponse<()> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    match catalog_dal::delete_source_cascade(&mut db, &source_code) {
        Ok(_) => {
            info!("Deleted catalog source and all entities: {}", source_code);
            ApiResponse::ok(())
        }
        Err(e) => ApiResponse::err(format!("Failed to delete source: {}", e)),
    }
}

// =============================================================================
// Image Import
// =============================================================================

/// Import images from a tar.gz archive.
///
/// The 5etools images are in a separate repository. This command
/// streams directly from the archive to the destination without
/// extracting to a temp directory.
#[tauri::command]
pub fn import_catalog_images(
    state: State<'_, AppState>,
    archive_path: String,
) -> ApiResponse<ImportResponse> {
    info!("Starting image import from: {}", archive_path);

    let archive_path = Path::new(&archive_path);
    if !archive_path.exists() {
        return ApiResponse::err(format!("File not found: {}", archive_path.display()));
    }

    // Destination for images
    let img_dest = state.paths.assets_dir.join("catalog");
    info!("Streaming images to: {}", img_dest.display());

    // Ensure destination exists
    if let Err(e) = std::fs::create_dir_all(&img_dest) {
        return ApiResponse::err(format!("Failed to create destination directory: {}", e));
    }

    // Stream from tar.gz directly to destination
    match stream_images_from_tarball(archive_path, &img_dest) {
        Ok(count) => {
            let message = format!("Successfully imported {} images", count);
            info!("{}", message);
            ApiResponse::ok(ImportResponse {
                sources_imported: 0,
                sources_failed: 0,
                total_entities: count,
                message,
            })
        }
        Err(e) => ApiResponse::err(e),
    }
}

/// Stream images from a tar.gz archive directly to the destination.
///
/// Handles the 5etools-img archive structure where files are nested like:
/// `5etools-img-2.24.0/book/PHB/001-intro.webp`
///
/// Strips the top-level directory prefix and writes directly to dest.
fn stream_images_from_tarball(archive_path: &Path, dest: &Path) -> Result<usize, String> {
    let file = File::open(archive_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let decoder = GzDecoder::new(BufReader::new(file));
    let mut archive = Archive::new(decoder);

    let entries = archive.entries()
        .map_err(|e| format!("Failed to read archive entries: {}", e))?;

    let mut count = 0;
    let mut prefix_to_strip: Option<String> = None;

    for (i, entry_result) in entries.enumerate() {
        let mut entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                warn!("Skipping entry {}: {}", i, e);
                continue;
            }
        };

        let path = match entry.path() {
            Ok(p) => p.to_path_buf(),
            Err(e) => {
                warn!("Skipping entry with invalid path: {}", e);
                continue;
            }
        };

        // Skip directories
        if entry.header().entry_type().is_dir() {
            // Detect the prefix to strip (e.g., "5etools-img-2.24.0/")
            if prefix_to_strip.is_none() {
                let path_str = path.to_string_lossy();
                if path_str.starts_with("5etools-img-") {
                    prefix_to_strip = Some(path_str.trim_end_matches('/').to_string());
                    info!("Detected archive prefix: {}", prefix_to_strip.as_ref().unwrap());
                }
            }
            continue;
        }

        // Only process image files
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let is_image = matches!(ext.as_deref(), Some("webp" | "png" | "jpg" | "jpeg" | "gif" | "svg"));
        if !is_image {
            continue;
        }

        // Strip the prefix directory from the path
        let relative_path = if let Some(ref prefix) = prefix_to_strip {
            let path_str = path.to_string_lossy();
            if let Some(stripped) = path_str.strip_prefix(prefix) {
                Path::new(stripped.trim_start_matches('/')).to_path_buf()
            } else {
                path.clone()
            }
        } else {
            path.clone()
        };

        // Build destination path
        let dest_path = dest.join(&relative_path);

        // Create parent directories
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    warn!("Failed to create directory {:?}: {}", parent, e);
                    continue;
                }
            }
        }

        // Extract the file
        if let Err(e) = entry.unpack(&dest_path) {
            warn!("Failed to extract {:?}: {}", dest_path, e);
            continue;
        }

        count += 1;

        // Log progress every 5000 files
        if count > 0 && count % 5000 == 0 {
            info!("Extracted {} images...", count);
        }
    }

    info!("Image extraction complete: {} files", count);
    Ok(count)
}

// =============================================================================
// Books (Readable Content)
// =============================================================================

/// Book info for the library listing (Reading mode).
/// Matches the frontend BookInfo interface.
#[derive(Debug, Serialize)]
pub struct LibraryBookInfo {
    /// Source code (acts as unique ID)
    pub id: String,
    /// Display name
    pub name: String,
    /// Whether enabled (always true for books with content)
    pub enabled: bool,
    /// When imported (from the source record)
    pub imported_at: String,
    /// Cover image path (relative to assets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_path: Option<String>,
}

/// List all books with readable content (for Reading mode).
///
/// Returns only sources that have associated book content (PHB, DMG, etc.)
/// This is used by the "Reading" mode in the Library.
#[tauri::command]
pub fn list_library_books(state: State<'_, AppState>) -> ApiResponse<Vec<LibraryBookInfo>> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    // Get all books
    let books_result = catalog_dal::list_books(&mut db);
    let books = match books_result {
        Ok(b) => b,
        Err(e) => return ApiResponse::err(e.to_string()),
    };

    // For each book, get the source info to get imported_at timestamp
    let mut book_infos = Vec::new();
    for book in books {
        // Get the source info for the imported_at timestamp
        let imported_at = match catalog_dal::get_source(&mut db, &book.source) {
            Ok(source) => source.imported_at,
            Err(_) => now_rfc3339(),
        };

        book_infos.push(LibraryBookInfo {
            id: book.source,
            name: book.name,
            enabled: true,
            imported_at,
            cover_path: book.cover_path,
        });
    }

    ApiResponse::ok(book_infos)
}

/// Get book content for reading.
///
/// Returns the full book content (chapters, sections, entries) for rendering
/// in the book reader view.
#[tauri::command]
pub fn get_book_content(
    state: State<'_, AppState>,
    book_id: String,
) -> ApiResponse<BookContent> {
    let mut db = match state.connect() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(e),
    };

    let result = catalog_dal::get_book_by_source(&mut db, &book_id);
    match result {
        Ok(Some(book)) => ApiResponse::ok(BookContent::from(book)),
        Ok(None) => ApiResponse::err(format!("Book not found: {}", book_id)),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Serve a book image as a base64 data URL.
///
/// This reads an image from the catalog assets directory and returns it
/// as a data URL that can be used directly in an img src attribute.
#[tauri::command]
pub fn serve_book_image(
    state: State<'_, AppState>,
    _book_id: String,
    image_path: String,
) -> ApiResponse<String> {
    // Build the full path to the image
    // Images are stored at: assets/catalog/{image_path}
    // image_path is like "book/PHB/001-intro.webp"
    let image_file = state.paths.assets_dir.join("catalog").join(&image_path);

    // Check if file exists
    if !image_file.exists() {
        return ApiResponse::err(format!(
            "Image not found: {} (looked at {})",
            image_path,
            image_file.display()
        ));
    }

    // Read the file
    let image_data = match std::fs::read(&image_file) {
        Ok(data) => data,
        Err(e) => return ApiResponse::err(format!("Failed to read image: {}", e)),
    };

    // Determine MIME type from extension
    let mime_type = match image_file
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("webp") => "image/webp",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };

    // Encode as base64 data URL
    let base64_data = BASE64.encode(&image_data);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    ApiResponse::ok(data_url)
}
