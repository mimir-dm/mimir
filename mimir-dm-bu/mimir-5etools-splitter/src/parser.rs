//! Parser for 5etools book metadata and JSON files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Metadata for a 5etools book or sourcebook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    /// Display name of the book.
    pub name: String,
    /// Unique identifier for the book.
    pub id: String,
    /// Source code (e.g., "PHB", "DMG").
    pub source: String,
    /// Book group/category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Publication date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    /// Author(s) of the book.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Cover image information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<Cover>,
    /// Table of contents entries.
    #[serde(default)]
    pub contents: Vec<serde_json::Value>,
}

/// Book cover image information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cover {
    /// Type of cover (e.g., "external").
    #[serde(rename = "type")]
    pub cover_type: String,
    /// Path to the cover image.
    pub path: String,
}

#[derive(Debug, Deserialize)]
struct BooksJson {
    book: Vec<Book>,
}

/// Load all books from the repository
pub fn load_all_books(repo_path: &Path) -> Result<Vec<Book>> {
    let books_file = repo_path.join("data").join("books.json");
    let content = fs::read_to_string(&books_file)
        .context(format!("Failed to read books.json at {:?}", books_file))?;

    let books_json: BooksJson =
        serde_json::from_str(&content).context("Failed to parse books.json")?;

    Ok(books_json.book)
}

/// Load adventures metadata
pub fn load_adventures(repo_path: &Path) -> Result<HashMap<String, serde_json::Value>> {
    let adventures_file = repo_path.join("data").join("adventures.json");
    if !adventures_file.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&adventures_file).context("Failed to read adventures.json")?;

    let data: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse adventures.json")?;

    let mut adventures = HashMap::new();
    if let Some(adventure_array) = data.get("adventure").and_then(|v| v.as_array()) {
        for adventure in adventure_array {
            if let Some(id) = adventure.get("id").and_then(|v| v.as_str()) {
                adventures.insert(id.to_string(), adventure.clone());
            }
        }
    }

    Ok(adventures)
}

/// Check if a file exists for a given book source
pub fn file_exists_for_source(repo_path: &Path, dir: &str, prefix: &str, source: &str) -> bool {
    let filename = format!("{}-{}.json", prefix, source.to_lowercase());
    let file_path = repo_path.join("data").join(dir).join(filename);
    file_path.exists()
}

/// Load JSON data from a file
pub fn load_json_file(path: &Path) -> Result<serde_json::Value> {
    let content = fs::read_to_string(path).context(format!("Failed to read file: {:?}", path))?;

    serde_json::from_str(&content).context(format!("Failed to parse JSON from: {:?}", path))
}

/// Filter entries by source
pub fn filter_by_source(
    data: &serde_json::Value,
    source: &str,
    key: &str,
) -> Vec<serde_json::Value> {
    let mut filtered = Vec::new();

    if let Some(array) = data.get(key).and_then(|v| v.as_array()) {
        for item in array {
            if let Some(item_source) = item.get("source").and_then(|v| v.as_str()) {
                if item_source == source {
                    filtered.push(item.clone());
                }
            }
        }
    }

    filtered
}

/// Get all file paths matching a pattern
pub fn get_matching_files(
    repo_path: &Path,
    dir: &str,
    prefix: &str,
    source: &str,
) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let data_dir = repo_path.join("data").join(dir);

    if !data_dir.exists() {
        return files;
    }

    // Direct file (e.g., spells-phb.json)
    let direct_file = data_dir.join(format!("{}-{}.json", prefix, source.to_lowercase()));
    if direct_file.exists() {
        files.push(direct_file);
    }

    // Fluff file (e.g., fluff-spells-phb.json)
    let fluff_file = data_dir.join(format!("fluff-{}-{}.json", prefix, source.to_lowercase()));
    if fluff_file.exists() {
        files.push(fluff_file);
    }

    // Files with source as prefix (for adventures with sub-parts)
    if let Ok(entries) = fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                let pattern = format!("{}-{}-", prefix, source.to_lowercase());
                if filename.starts_with(&pattern) && filename.ends_with(".json") {
                    files.push(path);
                }
            }
        }
    }

    files
}
