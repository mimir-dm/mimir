//! Book and Source Discovery
//!
//! Discovers available books and sources from a 5etools data directory.
//! Adapted from mimir-5etools-splitter/src/parser.rs.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Metadata for a 5etools book or sourcebook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    /// Display name of the book.
    pub name: String,
    /// Unique identifier for the book.
    pub id: String,
    /// Source code (e.g., "PHB", "DMG", "MM").
    pub source: String,
    /// Book group/category (e.g., "core", "supplement").
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
    /// Type of cover (e.g., "external", "internal").
    #[serde(rename = "type")]
    pub cover_type: String,
    /// Path to the cover image.
    pub path: String,
}

/// Internal struct for parsing books.json.
#[derive(Debug, Deserialize)]
struct BooksJson {
    book: Vec<Book>,
}

/// Load all books from a 5etools repository/directory.
///
/// Reads `data/books.json` and returns metadata for all available books.
///
/// # Arguments
/// * `repo_path` - Path to the 5etools data directory (containing `data/` subdirectory)
///
/// # Example
/// ```ignore
/// let books = load_all_books(Path::new("/path/to/5etools"))?;
/// for book in books {
///     println!("{}: {} ({})", book.source, book.name, book.id);
/// }
/// ```
pub fn load_all_books(repo_path: &Path) -> Result<Vec<Book>> {
    let books_file = repo_path.join("data").join("books.json");
    let content = fs::read_to_string(&books_file)
        .with_context(|| format!("Failed to read books.json at {:?}", books_file))?;

    let books_json: BooksJson =
        serde_json::from_str(&content).context("Failed to parse books.json")?;

    Ok(books_json.book)
}

/// Get all unique source codes from the available books.
///
/// Convenience function that extracts just the source codes.
pub fn get_all_source_codes(repo_path: &Path) -> Result<Vec<String>> {
    let books = load_all_books(repo_path)?;
    Ok(books.into_iter().map(|b| b.source).collect())
}

/// Check if a data file exists for a given source.
///
/// Checks for files matching the pattern `data/{dir}/{prefix}-{source}.json`.
///
/// # Arguments
/// * `repo_path` - Path to the 5etools data directory
/// * `dir` - Subdirectory name (e.g., "bestiary", "spells")
/// * `prefix` - File prefix (e.g., "bestiary", "spells")
/// * `source` - Source code (e.g., "PHB")
pub fn file_exists_for_source(repo_path: &Path, dir: &str, prefix: &str, source: &str) -> bool {
    let filename = format!("{}-{}.json", prefix, source.to_lowercase());
    let file_path = repo_path.join("data").join(dir).join(filename);
    file_path.exists()
}

/// Load and parse a JSON file.
pub fn load_json_file(path: &Path) -> Result<serde_json::Value> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))?;

    serde_json::from_str(&content).with_context(|| format!("Failed to parse JSON from: {:?}", path))
}

/// Get all file paths matching a source pattern.
///
/// Finds files in the following patterns:
/// - Direct: `{prefix}-{source}.json`
/// - Fluff: `fluff-{prefix}-{source}.json`
/// - Sub-files: `{prefix}-{source}-*.json` (for adventures with multiple parts)
///
/// # Arguments
/// * `repo_path` - Path to the 5etools data directory
/// * `dir` - Subdirectory name (e.g., "bestiary", "spells")
/// * `prefix` - File prefix (e.g., "bestiary", "spells")
/// * `source` - Source code (e.g., "PHB")
pub fn get_matching_files(
    repo_path: &Path,
    dir: &str,
    prefix: &str,
    source: &str,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let data_dir = repo_path.join("data").join(dir);

    if !data_dir.exists() {
        return files;
    }

    let source_lower = source.to_lowercase();

    // Direct file (e.g., spells-phb.json)
    let direct_file = data_dir.join(format!("{}-{}.json", prefix, source_lower));
    if direct_file.exists() {
        files.push(direct_file);
    }

    // Fluff file (e.g., fluff-spells-phb.json)
    let fluff_file = data_dir.join(format!("fluff-{}-{}.json", prefix, source_lower));
    if fluff_file.exists() {
        files.push(fluff_file);
    }

    // Files with source as prefix (for adventures with sub-parts)
    if let Ok(entries) = fs::read_dir(&data_dir) {
        let pattern = format!("{}-{}-", prefix, source_lower);
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with(&pattern) && filename.ends_with(".json") {
                    files.push(path);
                }
            }
        }
    }

    files
}

/// Entity type to file location mapping.
///
/// Maps entity types to their directory and file prefix in 5etools data.
#[derive(Debug, Clone, Copy)]
pub struct EntityFileInfo {
    /// Directory under `data/` (e.g., "bestiary", "spells")
    pub dir: &'static str,
    /// File prefix (e.g., "bestiary", "spells")
    pub prefix: &'static str,
    /// JSON key containing the entity array (e.g., "monster", "spell")
    pub json_key: &'static str,
}

/// Get file info for each entity type.
pub const ENTITY_FILE_INFO: &[(&str, EntityFileInfo)] = &[
    (
        "monster",
        EntityFileInfo {
            dir: "bestiary",
            prefix: "bestiary",
            json_key: "monster",
        },
    ),
    (
        "spell",
        EntityFileInfo {
            dir: "spells",
            prefix: "spells",
            json_key: "spell",
        },
    ),
    (
        "item",
        EntityFileInfo {
            dir: "items",
            prefix: "items",
            json_key: "item",
        },
    ),
    (
        "class",
        EntityFileInfo {
            dir: "class",
            prefix: "class",
            json_key: "class",
        },
    ),
    (
        "race",
        EntityFileInfo {
            dir: ".",
            prefix: "races",
            json_key: "race",
        },
    ),
    (
        "background",
        EntityFileInfo {
            dir: ".",
            prefix: "backgrounds",
            json_key: "background",
        },
    ),
    (
        "feat",
        EntityFileInfo {
            dir: ".",
            prefix: "feats",
            json_key: "feat",
        },
    ),
    (
        "action",
        EntityFileInfo {
            dir: ".",
            prefix: "actions",
            json_key: "action",
        },
    ),
    (
        "condition",
        EntityFileInfo {
            dir: ".",
            prefix: "conditionsdiseases",
            json_key: "condition",
        },
    ),
    (
        "disease",
        EntityFileInfo {
            dir: ".",
            prefix: "conditionsdiseases",
            json_key: "disease",
        },
    ),
    (
        "language",
        EntityFileInfo {
            dir: ".",
            prefix: "languages",
            json_key: "language",
        },
    ),
    (
        "optionalfeature",
        EntityFileInfo {
            dir: ".",
            prefix: "optionalfeatures",
            json_key: "optionalfeature",
        },
    ),
    (
        "reward",
        EntityFileInfo {
            dir: ".",
            prefix: "rewards",
            json_key: "reward",
        },
    ),
    (
        "deity",
        EntityFileInfo {
            dir: ".",
            prefix: "deities",
            json_key: "deity",
        },
    ),
    (
        "trap",
        EntityFileInfo {
            dir: ".",
            prefix: "trapshazards",
            json_key: "trap",
        },
    ),
    (
        "hazard",
        EntityFileInfo {
            dir: ".",
            prefix: "trapshazards",
            json_key: "hazard",
        },
    ),
    (
        "object",
        EntityFileInfo {
            dir: ".",
            prefix: "objects",
            json_key: "object",
        },
    ),
    (
        "vehicle",
        EntityFileInfo {
            dir: ".",
            prefix: "vehicles",
            json_key: "vehicle",
        },
    ),
    (
        "cult",
        EntityFileInfo {
            dir: ".",
            prefix: "cultsboons",
            json_key: "cult",
        },
    ),
    (
        "boon",
        EntityFileInfo {
            dir: ".",
            prefix: "cultsboons",
            json_key: "boon",
        },
    ),
    (
        "psionic",
        EntityFileInfo {
            dir: ".",
            prefix: "psionics",
            json_key: "psionic",
        },
    ),
    (
        "table",
        EntityFileInfo {
            dir: ".",
            prefix: "tables",
            json_key: "table",
        },
    ),
    (
        "variantrule",
        EntityFileInfo {
            dir: ".",
            prefix: "variantrules",
            json_key: "variantrule",
        },
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_book_deserialization() {
        let json = json!({
            "name": "Player's Handbook",
            "id": "phb",
            "source": "PHB",
            "group": "core",
            "published": "2014-08-19"
        });
        let book: Book = serde_json::from_value(json).unwrap();
        assert_eq!(book.name, "Player's Handbook");
        assert_eq!(book.source, "PHB");
        assert_eq!(book.group, Some("core".to_string()));
    }

    #[test]
    fn test_entity_file_info() {
        // Find monster info
        let monster_info = ENTITY_FILE_INFO
            .iter()
            .find(|(name, _)| *name == "monster")
            .map(|(_, info)| info);
        assert!(monster_info.is_some());
        let info = monster_info.unwrap();
        assert_eq!(info.dir, "bestiary");
        assert_eq!(info.prefix, "bestiary");
        assert_eq!(info.json_key, "monster");
    }
}
