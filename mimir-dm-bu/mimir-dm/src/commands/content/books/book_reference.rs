//! Book reference lookup functionality.
//!
//! Provides Tauri commands for looking up cross-references within book data.
//! Supports resolving references to spells, items, creatures, classes, races,
//! feats, and backgrounds with preview generation for tooltips.
//!
//! Primary lookup is from the catalog database via ReferenceService.
//! Falls back to book JSON files for content not yet imported.

use crate::state::AppState;
use crate::types::{ApiError, ApiResponse};
use mimir_dm_core::services::ReferenceService;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use tauri::State;
use tracing::{debug, info, warn};

/// Reference lookup data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceData {
    pub ref_type: String,
    pub name: String,
    pub source: Option<String>,
    pub data: Value,
    pub preview: String,
}

/// Look up a cross-reference in the catalog database or book data.
///
/// First searches the catalog database for the reference. If not found there,
/// falls back to searching book JSON files.
///
/// # Parameters
/// - `ref_type` - Type of reference ("spell", "item", "creature", "class", "race", "feat", "background", "condition", "action")
/// - `ref_name` - Name of the referenced entity
/// - `ref_source` - Optional source book code (defaults to searching common sources)
/// - `state` - Application state containing database pool and paths
///
/// # Returns
/// `ApiResponse` containing `ReferenceData` with the resolved reference, source,
/// full data, and a formatted preview string.
///
/// # Errors
/// Returns an error response if the reference cannot be found in any available source.
#[tauri::command]
pub async fn lookup_reference(
    ref_type: String,
    ref_name: String,
    ref_source: Option<String>,
    state: State<'_, AppState>,
) -> Result<ApiResponse<ReferenceData>, ApiError> {
    info!(
        "Looking up reference: {} '{}' from {:?}",
        ref_type, ref_name, ref_source
    );

    // Try database lookup first
    let db_result = {
        let mut conn = state.db.get_connection().map_err(|e| {
            warn!("Failed to get database connection: {}", e);
            ApiError::Internal(format!("Database connection error: {}", e))
        })?;

        ReferenceService::lookup(
            &mut conn,
            &ref_type,
            &ref_name,
            ref_source.as_deref(),
        )
    };

    match db_result {
        Ok(Some(data)) => {
            debug!("Found reference in database: {} '{}'", ref_type, ref_name);
            return Ok(ApiResponse::success(ReferenceData {
                ref_type: data.ref_type,
                name: data.name,
                source: data.source,
                data: data.data,
                preview: data.preview,
            }));
        }
        Ok(None) => {
            debug!("Reference not found in database, trying book files");
        }
        Err(e) => {
            warn!("Database lookup error: {}", e);
        }
    }

    // Fall back to book JSON file search
    let books_dir = state.paths.data_dir.join("books");

    // Determine which book to search in
    let source_book = ref_source.as_deref().unwrap_or("PHB").to_lowercase();

    // Map source codes to book IDs
    let book_id = match source_book.as_str() {
        "phb" => "PHB",
        "dmg" => "DMG",
        "mm" => "MM",
        "test-book" => "test-book",
        "test-book-two" => "test-book-two",
        "tb2" => "test-book-two",
        "test" => "test-book",
        _ => &source_book,
    };

    // Try to find the reference in the specified book
    let book_dir = books_dir.join(book_id);
    if !book_dir.exists() {
        // If specific book not found, search all books
        return search_all_books_for_reference(&books_dir, &ref_type, &ref_name).await;
    }

    // Search in the specific book
    match search_book_for_reference(&book_dir, &ref_type, &ref_name).await {
        Ok(Some(data)) => Ok(ApiResponse::success(data)),
        Ok(None) => {
            // Not found in specified book, search all
            search_all_books_for_reference(&books_dir, &ref_type, &ref_name).await
        }
        Err(e) => Err(e),
    }
}

/// Search a specific book for a reference
async fn search_book_for_reference(
    book_dir: &Path,
    ref_type: &str,
    ref_name: &str,
) -> Result<Option<ReferenceData>, ApiError> {
    debug!(
        "Searching book {:?} for {} '{}'",
        book_dir, ref_type, ref_name
    );

    // Map reference types to data file patterns
    let file_patterns = match ref_type {
        "spell" => vec!["spells-*.json", "*.json", "book-*.json"],
        "item" => vec!["items-*.json", "*.json", "book-*.json"],
        "creature" | "monster" => vec!["bestiary-*.json", "*.json", "book-*.json"],
        "class" => vec!["class-*.json", "*.json", "book-*.json"],
        "race" => vec!["race-*.json", "*.json", "book-*.json"],
        "feat" => vec!["feats-*.json", "*.json", "book-*.json"],
        "background" => vec!["backgrounds-*.json", "*.json", "book-*.json"],
        _ => vec!["book-*.json"],
    };

    // Check data subdirectory first, then type-specific directories
    let data_dir = book_dir.join("data");
    let mut search_dirs = if data_dir.exists() {
        vec![data_dir, book_dir.join("book")]
    } else {
        vec![book_dir.join("book")]
    };

    // Add type-specific directories
    match ref_type {
        "spell" => search_dirs.push(book_dir.join("spells")),
        "item" => search_dirs.push(book_dir.join("items")),
        "creature" | "monster" => search_dirs.push(book_dir.join("bestiary")),
        "class" => search_dirs.push(book_dir.join("class")),
        "race" => search_dirs.push(book_dir.join("races")),
        "feat" => search_dirs.push(book_dir.join("feats")),
        "background" => search_dirs.push(book_dir.join("backgrounds")),
        _ => {}
    }

    for dir in search_dirs {
        if !dir.exists() {
            continue;
        }

        // Search through relevant JSON files
        for pattern in &file_patterns {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Check if filename matches pattern
                    if matches_pattern(file_name, pattern) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                                if let Some(data) =
                                    find_reference_in_json(&json, ref_type, ref_name)
                                {
                                    return Ok(Some(data));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Search all books for a reference
async fn search_all_books_for_reference(
    books_dir: &Path,
    ref_type: &str,
    ref_name: &str,
) -> Result<ApiResponse<ReferenceData>, ApiError> {
    debug!("Searching all books for {} '{}'", ref_type, ref_name);

    if let Ok(entries) = fs::read_dir(books_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(Some(data)) = search_book_for_reference(&path, ref_type, ref_name).await {
                    return Ok(ApiResponse::success(data));
                }
            }
        }
    }

    Ok(ApiResponse::error(format!(
        "Reference not found: {} '{}'",
        ref_type, ref_name
    )))
}

/// Check if a filename matches a pattern (simple glob)
fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return filename.starts_with(parts[0]) && filename.ends_with(parts[1]);
        }
    }
    filename == pattern
}

/// Find a reference in a JSON data structure
fn find_reference_in_json(json: &Value, ref_type: &str, ref_name: &str) -> Option<ReferenceData> {
    let ref_name_lower = ref_name.to_lowercase();

    // Check for typed arrays (spell, item, creature, etc.)
    let type_key = match ref_type {
        "creature" | "monster" => "monster",
        other => other,
    };

    if let Some(array) = json.get(type_key).and_then(|v| v.as_array()) {
        for item in array {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                if name.to_lowercase() == ref_name_lower {
                    return Some(create_reference_data(ref_type, name, item));
                }
            }
        }
    }

    // Check in book data structure
    if let Some(data_array) = json.get("data").and_then(|v| v.as_array()) {
        for section in data_array {
            if let Some(found) = search_entries_for_reference(section, ref_type, &ref_name_lower) {
                return Some(found);
            }
        }
    }

    None
}

/// Search through entries recursively for references
fn search_entries_for_reference(
    entry: &Value,
    ref_type: &str,
    ref_name_lower: &str,
) -> Option<ReferenceData> {
    // Check if this entry is the reference we're looking for
    if let Some(entry_type) = entry.get("type").and_then(|v| v.as_str()) {
        if entry_type == ref_type || (ref_type == "spell" && entry_type == "spellList") {
            if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
                if name.to_lowercase() == *ref_name_lower {
                    return Some(create_reference_data(ref_type, name, entry));
                }
            }
        }
    }

    // Search in entries array
    if let Some(entries) = entry.get("entries").and_then(|v| v.as_array()) {
        for sub_entry in entries {
            if let Some(found) = search_entries_for_reference(sub_entry, ref_type, ref_name_lower) {
                return Some(found);
            }
        }
    }

    None
}

/// Create a ReferenceData object from JSON data
fn create_reference_data(ref_type: &str, name: &str, data: &Value) -> ReferenceData {
    let preview = generate_preview(ref_type, data);

    ReferenceData {
        ref_type: ref_type.to_string(),
        name: name.to_string(),
        source: data
            .get("source")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        data: data.clone(),
        preview,
    }
}

/// Generate a preview string for tooltips
fn generate_preview(ref_type: &str, data: &Value) -> String {
    match ref_type {
        "spell" => {
            let level = data.get("level").and_then(|v| v.as_u64()).unwrap_or(0);
            let school = data
                .get("school")
                .and_then(|v| v.as_str())
                .map(|s| get_spell_school_name(s))
                .unwrap_or("Unknown");
            let range = format_spell_range(data.get("range"));

            if level == 0 {
                format!("Cantrip • {}<br/>{}", school, range)
            } else {
                format!("Level {} • {}<br/>{}", level, school, range)
            }
        }
        "item" => {
            let item_type = data.get("type").and_then(|v| v.as_str()).unwrap_or("Item");
            let rarity = data.get("rarity").and_then(|v| v.as_str()).unwrap_or("");
            let value = data
                .get("value")
                .and_then(|v| v.as_u64())
                .map(|v| format!("{} gp", v))
                .unwrap_or_default();

            format!(
                "{}{}<br/>{}",
                item_type,
                if !rarity.is_empty() {
                    format!(" • {}", rarity)
                } else {
                    String::new()
                },
                value
            )
        }
        "creature" | "monster" => {
            let cr = data
                .get("cr")
                .and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(obj) = v.as_object() {
                        obj.get("cr")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "?".to_string());

            let type_str = data
                .get("type")
                .and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(obj) = v.as_object() {
                        obj.get("type")
                            .and_then(|t| t.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "creature".to_string());

            let ac = data
                .get("ac")
                .and_then(|v| {
                    if let Some(n) = v.as_u64() {
                        Some(n.to_string())
                    } else if let Some(arr) = v.as_array() {
                        arr.first().and_then(|a| {
                            if let Some(n) = a.as_u64() {
                                Some(n.to_string())
                            } else if let Some(obj) = a.as_object() {
                                obj.get("ac")
                                    .and_then(|ac| ac.as_u64())
                                    .map(|n| n.to_string())
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "?".to_string());

            let hp = data
                .get("hp")
                .and_then(|v| {
                    if let Some(obj) = v.as_object() {
                        obj.get("average")
                            .and_then(|a| a.as_u64())
                            .map(|n| n.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "?".to_string());

            format!("{} • CR {}<br/>AC {}, HP {}", type_str, cr, ac, hp)
        }
        "class" => {
            let hd = data
                .get("hd")
                .and_then(|v| v.as_object())
                .and_then(|o| o.get("faces").and_then(|f| f.as_u64()))
                .map(|d| format!("d{}", d))
                .unwrap_or_else(|| "d?".to_string());

            format!("Class • {} Hit Die", hd)
        }
        _ => format!(
            "{}: {}",
            ref_type,
            data.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
        ),
    }
}

/// Get spell school full name from abbreviation
fn get_spell_school_name(abbr: &str) -> &'static str {
    match abbr {
        "A" => "Abjuration",
        "C" => "Conjuration",
        "D" => "Divination",
        "E" => "Enchantment",
        "V" => "Evocation",
        "I" => "Illusion",
        "N" => "Necromancy",
        "T" => "Transmutation",
        _ => "Unknown",
    }
}

/// Format spell range for display
fn format_spell_range(range: Option<&Value>) -> String {
    if let Some(range_val) = range {
        if let Some(range_type) = range_val.get("type").and_then(|v| v.as_str()) {
            match range_type {
                "point" => {
                    if let Some(distance) = range_val.get("distance") {
                        if let Some(dist_type) = distance.get("type").and_then(|v| v.as_str()) {
                            if let Some(amount) = distance.get("amount").and_then(|v| v.as_u64()) {
                                return format!("Range: {} {}", amount, dist_type);
                            }
                        }
                    }
                }
                "self" => return "Range: Self".to_string(),
                "touch" => return "Range: Touch".to_string(),
                "sight" => return "Range: Sight".to_string(),
                "unlimited" => return "Range: Unlimited".to_string(),
                _ => {}
            }
        }
    }
    "Range: Varies".to_string()
}
