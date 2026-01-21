//! Token Image Utilities
//!
//! Handles token image paths, slugification, and file copying for monster tokens.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Slugify a monster name for use in filesystem paths.
///
/// Converts "Adult Red Dragon" to "adult-red-dragon".
pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                // Skip special characters
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        // Collapse multiple hyphens
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Generate the relative token path for storage in the database.
///
/// Returns a path like "tokens/MM/adult-red-dragon.png".
pub fn token_relative_path(source: &str, monster_name: &str, extension: &str) -> String {
    let slug = slugify(monster_name);
    format!("tokens/{}/{}.{}", source, slug, extension)
}

/// Resolve a relative token path to an absolute filesystem path.
pub fn resolve_token_path(app_data_dir: &Path, token_image_path: &str) -> PathBuf {
    app_data_dir.join(token_image_path)
}

/// Find a token image in the 5etools image directory.
///
/// Searches for PNG and WEBP formats.
/// Returns the source path and extension if found.
pub fn find_source_token(img_dir: &Path, source: &str, monster_name: &str) -> Option<(PathBuf, String)> {
    let base_path = img_dir.join("bestiary").join("tokens").join(source);

    // Try exact name match first
    for ext in &["png", "webp"] {
        let path = base_path.join(format!("{}.{}", monster_name, ext));
        if path.exists() {
            return Some((path, ext.to_string()));
        }
    }

    // Try with common name transformations
    let slug = slugify(monster_name);
    for ext in &["png", "webp"] {
        let path = base_path.join(format!("{}.{}", slug, ext));
        if path.exists() {
            return Some((path, ext.to_string()));
        }
    }

    None
}

/// Copy a token image from the source directory to the app data directory.
///
/// Returns the relative path stored in the database, or None if no token was found.
pub fn copy_token(
    img_dir: &Path,
    app_data_dir: &Path,
    source: &str,
    monster_name: &str,
) -> Result<Option<String>> {
    // Find the source token
    let Some((source_path, extension)) = find_source_token(img_dir, source, monster_name) else {
        return Ok(None);
    };

    // Generate destination path
    let rel_path = token_relative_path(source, monster_name, &extension);
    let dest_path = resolve_token_path(app_data_dir, &rel_path);

    // Create parent directory
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create token directory: {:?}", parent))?;
    }

    // Copy the file
    fs::copy(&source_path, &dest_path)
        .with_context(|| format!("Failed to copy token from {:?} to {:?}", source_path, dest_path))?;

    Ok(Some(rel_path))
}

/// Batch copy tokens for multiple monsters.
///
/// Returns the count of tokens successfully copied.
pub fn copy_tokens_batch(
    img_dir: &Path,
    app_data_dir: &Path,
    monsters: &[(String, String)], // (source, name) pairs
) -> (usize, Vec<String>) {
    let mut copied = 0;
    let mut errors = Vec::new();

    for (source, name) in monsters {
        match copy_token(img_dir, app_data_dir, source, name) {
            Ok(Some(_)) => copied += 1,
            Ok(None) => {
                // No token found - not an error, just skip
            }
            Err(e) => {
                errors.push(format!("{}/{}: {}", source, name, e));
            }
        }
    }

    (copied, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_slugify_simple() {
        assert_eq!(slugify("Goblin"), "goblin");
    }

    #[test]
    fn test_slugify_spaces() {
        assert_eq!(slugify("Adult Red Dragon"), "adult-red-dragon");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Lich (Acererak)"), "lich-acererak");
    }

    #[test]
    fn test_slugify_apostrophe() {
        // Apostrophes are removed (not replaced with hyphen)
        assert_eq!(slugify("Death's Head"), "deaths-head");
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(slugify("Very   Spaced   Name"), "very-spaced-name");
    }

    #[test]
    fn test_slugify_hyphens() {
        assert_eq!(slugify("Half-Dragon"), "half-dragon");
    }

    #[test]
    fn test_token_relative_path() {
        let path = token_relative_path("MM", "Adult Red Dragon", "png");
        assert_eq!(path, "tokens/MM/adult-red-dragon.png");
    }

    #[test]
    fn test_resolve_token_path() {
        let app_data = Path::new("/home/user/.mimir");
        let resolved = resolve_token_path(app_data, "tokens/MM/goblin.png");
        assert_eq!(resolved, PathBuf::from("/home/user/.mimir/tokens/MM/goblin.png"));
    }

    #[test]
    fn test_find_source_token_png() {
        let temp = TempDir::new().unwrap();
        let img_dir = temp.path();

        // Create token directory structure
        let token_dir = img_dir.join("bestiary/tokens/MM");
        fs::create_dir_all(&token_dir).unwrap();
        fs::write(token_dir.join("Goblin.png"), "fake png").unwrap();

        let result = find_source_token(img_dir, "MM", "Goblin");
        assert!(result.is_some());
        let (path, ext) = result.unwrap();
        assert!(path.ends_with("Goblin.png"));
        assert_eq!(ext, "png");
    }

    #[test]
    fn test_find_source_token_webp() {
        let temp = TempDir::new().unwrap();
        let img_dir = temp.path();

        // Create token directory structure
        let token_dir = img_dir.join("bestiary/tokens/MM");
        fs::create_dir_all(&token_dir).unwrap();
        fs::write(token_dir.join("Goblin.webp"), "fake webp").unwrap();

        let result = find_source_token(img_dir, "MM", "Goblin");
        assert!(result.is_some());
        let (_, ext) = result.unwrap();
        assert_eq!(ext, "webp");
    }

    #[test]
    fn test_find_source_token_not_found() {
        let temp = TempDir::new().unwrap();
        let img_dir = temp.path();

        let result = find_source_token(img_dir, "MM", "NonExistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_copy_token() {
        let temp_src = TempDir::new().unwrap();
        let temp_dest = TempDir::new().unwrap();

        // Create source token
        let token_dir = temp_src.path().join("bestiary/tokens/MM");
        fs::create_dir_all(&token_dir).unwrap();
        fs::write(token_dir.join("Goblin.png"), "fake png content").unwrap();

        let result = copy_token(
            temp_src.path(),
            temp_dest.path(),
            "MM",
            "Goblin",
        ).unwrap();

        assert!(result.is_some());
        let rel_path = result.unwrap();
        assert_eq!(rel_path, "tokens/MM/goblin.png");

        // Verify file was copied
        let dest_file = temp_dest.path().join(&rel_path);
        assert!(dest_file.exists());
        assert_eq!(fs::read_to_string(&dest_file).unwrap(), "fake png content");
    }

    #[test]
    fn test_copy_token_not_found() {
        let temp_src = TempDir::new().unwrap();
        let temp_dest = TempDir::new().unwrap();

        let result = copy_token(
            temp_src.path(),
            temp_dest.path(),
            "MM",
            "NonExistent",
        ).unwrap();

        assert!(result.is_none());
    }
}
