//! Catalog Source Commands
//!
//! Tauri commands for managing catalog sources (importing 5etools data, listing sources, etc.)

use mimir_core::dal::catalog::{self as catalog_dal};
use mimir_core::import::CatalogImportService;
use mimir_core::models::catalog::CatalogSource;
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tauri::State;
use tempfile::TempDir;
use tracing::{error, info, warn};
use zip::ZipArchive;

use super::{to_api_response, ApiResponse};
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    let result = catalog_dal::list_sources(&mut db);
    match result {
        Ok(sources) => ApiResponse::ok(sources.into_iter().map(SourceInfo::from).collect()),
        Err(e) => ApiResponse::err(e.to_string()),
    }
}

/// Import catalog data from a zip archive containing 5etools data.
///
/// The zip file should contain the 5etools data directory structure:
/// - data/ folder with JSON files
/// - img/ folder with images (optional)
#[tauri::command]
pub fn import_catalog_from_zip(
    state: State<'_, AppState>,
    archive_path: String,
) -> ApiResponse<ImportResponse> {
    info!("Starting catalog import from: {}", archive_path);

    // Validate the file exists
    let archive_path = Path::new(&archive_path);
    if !archive_path.exists() {
        return ApiResponse::err(format!("File not found: {}", archive_path.display()));
    }

    // Create a temporary directory for extraction
    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(e) => return ApiResponse::err(format!("Failed to create temp directory: {}", e)),
    };

    // Extract the zip file
    info!("Extracting archive to: {}", temp_dir.path().display());
    if let Err(e) = extract_zip(archive_path, temp_dir.path()) {
        return ApiResponse::err(format!("Failed to extract archive: {}", e));
    }

    // Find the data directory (might be nested in a folder)
    let data_dir = find_data_directory(temp_dir.path());
    let repo_path = match data_dir {
        Some(path) => path,
        None => {
            return ApiResponse::err(
                "Invalid archive: Could not find 'data' directory. \
                 The archive should contain a 5etools data structure with a 'data' folder."
                    .to_string(),
            )
        }
    };

    info!("Found data directory at: {}", repo_path.display());

    // Get database connection and run import
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    // Configure image directory if present
    let img_source = repo_path.join("img");
    let img_dest = state.paths.assets_dir.join("catalog");

    let mut service = CatalogImportService::new(&mut db);
    if img_source.exists() {
        info!("Found img directory, will copy images to: {}", img_dest.display());
        service = service.with_image_copy(img_source, img_dest);
    }

    // Run the import
    match service.import_from_directory(&repo_path) {
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
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
    let mut db = match state.db.lock() {
        Ok(db) => db,
        Err(e) => return ApiResponse::err(format!("Database lock error: {}", e)),
    };

    match catalog_dal::delete_source_cascade(&mut db, &source_code) {
        Ok(_) => {
            info!("Deleted catalog source and all entities: {}", source_code);
            ApiResponse::ok(())
        }
        Err(e) => ApiResponse::err(format!("Failed to delete source: {}", e)),
    }
}

/// Extract a zip archive to the specified directory.
fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

    let total_files = archive.len();
    info!("Extracting {} files...", total_files);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read file {} in archive: {}", i, e))?;

        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => {
                warn!("Skipping file with invalid name at index {}", i);
                continue;
            }
        };

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory {:?}: {}", outpath, e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
                }
            }
            let mut outfile = File::create(&outpath)
                .map_err(|e| format!("Failed to create file {:?}: {}", outpath, e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file {:?}: {}", outpath, e))?;
        }

        // Log progress every 1000 files
        if i > 0 && i % 1000 == 0 {
            info!("Extracted {}/{} files...", i, total_files);
        }
    }

    info!("Extraction complete: {} files", total_files);
    Ok(())
}

/// Find the data directory within the extracted archive.
///
/// The 5etools data might be:
/// - Directly in the root (data/ folder)
/// - Nested in a single folder (5etools-master/data/)
fn find_data_directory(root: &Path) -> Option<std::path::PathBuf> {
    // Check if data directory exists directly in root
    let direct_data = root.join("data");
    if direct_data.exists() && direct_data.is_dir() {
        return Some(root.to_path_buf());
    }

    // Check if there's a single subdirectory containing data
    if let Ok(entries) = std::fs::read_dir(root) {
        let subdirs: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();

        // If there's exactly one subdirectory, check for data inside it
        if subdirs.len() == 1 {
            let nested = subdirs[0].path();
            let nested_data = nested.join("data");
            if nested_data.exists() && nested_data.is_dir() {
                return Some(nested);
            }
        }
    }

    None
}
