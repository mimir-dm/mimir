//! Book library listing and removal commands.
//!
//! Provides Tauri commands for managing the uploaded book library. Supports
//! listing all available books and removing books with full cleanup of
//! associated catalog data and files.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use diesel::prelude::*;
use mimir_dm_core::models::catalog::UploadedBook;
use mimir_dm_core::schema::uploaded_books;
use mimir_dm_core::services::{
    ActionService, BackgroundService, ClassService, ConditionService, DeityService, FeatService,
    ItemService, LanguageService, MonsterService, ObjectService, RaceService, RewardService,
    SpellService, TrapService, VehicleService,
};
use std::fs;
use std::path::Path;
use tauri::State;
use tracing::{error, info, warn};

/// Book information for library listing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BookInfo {
    pub id: String,   // Book ID (e.g., "phb", "dmg")
    pub name: String, // Display name (e.g., "Player's Handbook")
}

/// List all books in the library.
///
/// Retrieves all uploaded books from the database and returns their
/// IDs and display names for the library view.
///
/// # Parameters
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` containing a vector of `BookInfo` objects.
///
/// # Errors
/// Returns an error response if the database connection or query fails.
#[tauri::command]
pub async fn list_library_books(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<BookInfo>>, ApiError> {
    info!("Listing library books from database");

    match state.db.get_connection() {
        Ok(mut conn) => match uploaded_books::table.load::<UploadedBook>(&mut conn) {
            Ok(books) => {
                let book_list: Vec<BookInfo> = books
                    .into_iter()
                    .map(|book| BookInfo {
                        id: book.id,
                        name: book.name,
                    })
                    .collect();

                info!("Found {} books in library", book_list.len());
                Ok(ApiResponse::success(book_list))
            }
            Err(e) => {
                error!("Failed to query books from database: {}", e);
                Ok(ApiResponse::error(
                    "Failed to load books from database".to_string(),
                ))
            }
        },
        Err(e) => {
            error!("Database connection error when listing books: {}", e);
            Ok(ApiResponse::error("Database connection error".to_string()))
        }
    }
}

/// Remove a book from the library.
///
/// Performs a complete cleanup: removes the database record, deletes all
/// associated catalog data (spells, items, monsters, etc.), removes the
/// extracted book directory, and deletes the original archive file.
///
/// # Parameters
/// - `book_id` - The unique book identifier to remove
/// - `state` - Application state containing the database connection
///
/// # Returns
/// `ApiResponse` with success or error status.
///
/// # Errors
/// Returns an error response if:
/// - The book is not found in the database
/// - Database operations fail (file cleanup may still partially succeed)
#[tauri::command]
pub async fn remove_book_from_library(
    book_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ApiError> {
    info!("Removing book from library: {}", book_id);

    // First, get book info from database to know what to clean up
    let book_record = match state.db.get_connection() {
        Ok(mut conn) => {
            match uploaded_books::table
                .filter(uploaded_books::id.eq(&book_id))
                .first::<UploadedBook>(&mut conn)
            {
                Ok(book) => Some(book),
                Err(diesel::NotFound) => {
                    return Ok(ApiResponse::error(format!(
                        "Book '{}' not found in database",
                        book_id
                    )));
                }
                Err(e) => {
                    error!("Database error when looking up book: {}", e);
                    return Ok(ApiResponse::error(
                        "Database error during lookup".to_string(),
                    ));
                }
            }
        }
        Err(e) => {
            error!("Database connection error during lookup: {}", e);
            return Ok(ApiResponse::error("Database connection error".to_string()));
        }
    };

    if let Some(book) = book_record {
        // Use database transaction for atomic cleanup
        match state.db.get_connection() {
            Ok(mut conn) => {
                let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
                    // Delete from database first
                    diesel::delete(uploaded_books::table.filter(uploaded_books::id.eq(&book_id)))
                        .execute(conn)?;

                    // Delete related catalog data
                    let _ = SpellService::remove_spells_by_source(conn, &book_id);
                    let _ = ActionService::remove_actions_by_source(conn, &book_id);
                    let _ = ConditionService::remove_conditions_by_source(conn, &book_id);
                    let _ = LanguageService::remove_languages_by_source(conn, &book_id);
                    let _ = RewardService::remove_rewards_by_source(conn, &book_id);
                    let _ = BackgroundService::remove_backgrounds_by_source(conn, &book_id);
                    let _ = FeatService::remove_feats_by_source(conn, &book_id);
                    let _ = RaceService::remove_races_by_source(conn, &book_id);
                    let _ = ObjectService::remove_objects_by_source(conn, &book_id);
                    let _ = TrapService::remove_traps_from_source(conn, &book_id);
                    let _ = ItemService::remove_items_from_source(conn, &book_id);
                    let _ = MonsterService::remove_monsters_from_source(conn, &book_id);
                    let _ = DeityService::remove_deities_from_source(conn, &book_id);
                    let _ = VehicleService::remove_vehicles_from_source(conn, &book_id);
                    let _ = ClassService::remove_classes_from_source(conn, &book_id);
                    // We don't want catalog cleanup errors to fail the book removal

                    Ok(())
                });

                match transaction_result {
                    Ok(_) => {
                        info!("Successfully removed book '{}' from database", book_id);

                        // Now clean up files
                        let mut cleanup_errors = Vec::new();

                        // Remove extracted directory
                        let book_dir = Path::new(&book.location);
                        if book_dir.exists() {
                            if let Err(e) = fs::remove_dir_all(book_dir) {
                                error!("Failed to remove book directory: {}", e);
                                cleanup_errors.push(format!("directory: {}", e));
                            }
                        }

                        // Remove archive file
                        let archive_path = Path::new(&book.archive_path);
                        if archive_path.exists() {
                            if let Err(e) = fs::remove_file(archive_path) {
                                error!("Failed to remove archive file: {}", e);
                                cleanup_errors.push(format!("archive: {}", e));
                            }
                        }

                        if cleanup_errors.is_empty() {
                            info!("Successfully removed all files for book '{}'", book_id);
                            Ok(ApiResponse::success(()))
                        } else {
                            warn!("Book '{}' removed from database but some files couldn't be deleted: {}",
                                  book_id, cleanup_errors.join(", "));
                            Ok(ApiResponse::success(())) // Still success since database is clean
                        }
                    }
                    Err(e) => {
                        error!("Failed to remove book from database: {}", e);
                        Ok(ApiResponse::error(
                            "Failed to remove book from database".to_string(),
                        ))
                    }
                }
            }
            Err(e) => {
                error!("Database connection error during removal: {}", e);
                Ok(ApiResponse::error("Database connection error".to_string()))
            }
        }
    } else {
        Ok(ApiResponse::error(format!("Book '{}' not found", book_id)))
    }
}
