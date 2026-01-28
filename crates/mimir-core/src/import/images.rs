//! Image copying utilities for catalog import.
//!
//! Copies images from 5etools source directory to application data directory,
//! preserving the directory structure so paths in imported data work as-is.

use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Copy all images from source directory to destination, preserving structure.
///
/// # Arguments
/// * `source_dir` - Source directory containing images (e.g., 5etools `img/`)
/// * `dest_dir` - Destination directory for copied images
///
/// # Returns
/// Number of files copied (skips existing files with matching size)
pub fn copy_images(source_dir: &Path, dest_dir: &Path) -> Result<usize> {
    if !source_dir.exists() {
        warn!("Source image directory does not exist: {:?}", source_dir);
        return Ok(0);
    }

    let mut copied = 0;
    let mut skipped = 0;
    let mut errors = 0;

    info!("Copying images from {:?} to {:?}", source_dir, dest_dir);

    for entry in WalkDir::new(source_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories - we create them as needed
        if path.is_dir() {
            continue;
        }

        // Get relative path from source
        let rel_path = match path.strip_prefix(source_dir) {
            Ok(p) => p,
            Err(_) => {
                warn!("Failed to get relative path for {:?}", path);
                continue;
            }
        };

        let dest_path = dest_dir.join(rel_path);

        // Skip if destination exists and has same size (idempotent)
        if dest_path.exists() {
            if let (Ok(src_meta), Ok(dest_meta)) = (path.metadata(), dest_path.metadata()) {
                if src_meta.len() == dest_meta.len() {
                    skipped += 1;
                    continue;
                }
            }
        }

        // Create parent directories if needed
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    warn!("Failed to create directory {:?}: {}", parent, e);
                    errors += 1;
                    continue;
                }
            }
        }

        // Copy the file
        match fs::copy(path, &dest_path) {
            Ok(_) => {
                copied += 1;
                if copied % 1000 == 0 {
                    debug!("Copied {} images...", copied);
                }
            }
            Err(e) => {
                warn!("Failed to copy {:?} to {:?}: {}", path, dest_path, e);
                errors += 1;
            }
        }
    }

    info!(
        "Image copy complete: {} copied, {} skipped (existing), {} errors",
        copied, skipped, errors
    );

    Ok(copied)
}

/// Check if an image exists at the given path relative to the images directory.
pub fn image_exists(img_dir: &Path, relative_path: &str) -> bool {
    img_dir.join(relative_path).exists()
}

/// Get the expected token path for a monster.
///
/// Token paths follow the pattern: `bestiary/tokens/{source}/{name}.webp`
/// Falls back to `.png` if `.webp` doesn't exist.
pub fn get_token_path(img_dir: &Path, source: &str, monster_name: &str) -> Option<String> {
    let base_path = format!("bestiary/tokens/{}", source);

    // Try exact name first
    let webp_path = format!("{}/{}.webp", base_path, monster_name);
    if image_exists(img_dir, &webp_path) {
        return Some(webp_path);
    }

    let png_path = format!("{}/{}.png", base_path, monster_name);
    if image_exists(img_dir, &png_path) {
        return Some(png_path);
    }

    // Try slugified name
    let slug = slugify(monster_name);
    let webp_slug_path = format!("{}/{}.webp", base_path, slug);
    if image_exists(img_dir, &webp_slug_path) {
        return Some(webp_slug_path);
    }

    let png_slug_path = format!("{}/{}.png", base_path, slug);
    if image_exists(img_dir, &png_slug_path) {
        return Some(png_slug_path);
    }

    None
}

/// Convert a name to a URL-safe slug.
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Adult Red Dragon"), "adult-red-dragon");
        assert_eq!(slugify("Goblin"), "goblin");
        assert_eq!(slugify("Mind Flayer (Illithid)"), "mind-flayer-illithid");
        assert_eq!(slugify("Lich's Phylactery"), "lich-s-phylactery");
    }

    #[test]
    fn test_copy_images_empty_source() {
        let source = tempdir().unwrap();
        let dest = tempdir().unwrap();

        let copied = copy_images(source.path(), dest.path()).unwrap();
        assert_eq!(copied, 0);
    }

    #[test]
    fn test_copy_images_preserves_structure() {
        let source = tempdir().unwrap();
        let dest = tempdir().unwrap();

        // Create source structure: bestiary/MM/Goblin.webp
        let src_subdir = source.path().join("bestiary").join("MM");
        fs::create_dir_all(&src_subdir).unwrap();

        let src_file = src_subdir.join("Goblin.webp");
        let mut f = File::create(&src_file).unwrap();
        f.write_all(b"fake image data").unwrap();

        // Copy
        let copied = copy_images(source.path(), dest.path()).unwrap();
        assert_eq!(copied, 1);

        // Verify destination structure
        let dest_file = dest.path().join("bestiary").join("MM").join("Goblin.webp");
        assert!(dest_file.exists());
    }

    #[test]
    fn test_copy_images_skips_existing() {
        let source = tempdir().unwrap();
        let dest = tempdir().unwrap();

        // Create source file
        let src_file = source.path().join("test.webp");
        let mut f = File::create(&src_file).unwrap();
        f.write_all(b"fake image data").unwrap();

        // Create matching dest file
        let dest_file = dest.path().join("test.webp");
        let mut f = File::create(&dest_file).unwrap();
        f.write_all(b"fake image data").unwrap();

        // Copy - should skip
        let copied = copy_images(source.path(), dest.path()).unwrap();
        assert_eq!(copied, 0);
    }

    #[test]
    fn test_copy_images_nonexistent_source() {
        let dest = tempdir().unwrap();
        let nonexistent = Path::new("/nonexistent/path/to/images");

        let copied = copy_images(nonexistent, dest.path()).unwrap();
        assert_eq!(copied, 0);
    }

    #[test]
    fn test_get_token_path() {
        let dir = tempdir().unwrap();

        // Create token directory structure
        let token_dir = dir.path().join("bestiary").join("tokens").join("MM");
        fs::create_dir_all(&token_dir).unwrap();

        // Create a token file
        let token_file = token_dir.join("Goblin.webp");
        File::create(&token_file).unwrap();

        // Should find it
        let path = get_token_path(dir.path(), "MM", "Goblin");
        assert_eq!(path, Some("bestiary/tokens/MM/Goblin.webp".to_string()));

        // Should not find nonexistent
        let path = get_token_path(dir.path(), "MM", "Dragon");
        assert!(path.is_none());
    }

    #[test]
    fn test_get_token_path_slugified() {
        let dir = tempdir().unwrap();

        // Create token directory structure
        let token_dir = dir.path().join("bestiary").join("tokens").join("MM");
        fs::create_dir_all(&token_dir).unwrap();

        // Create a slugified token file
        let token_file = token_dir.join("adult-red-dragon.webp");
        File::create(&token_file).unwrap();

        // Should find via slugified name
        let path = get_token_path(dir.path(), "MM", "Adult Red Dragon");
        assert_eq!(
            path,
            Some("bestiary/tokens/MM/adult-red-dragon.webp".to_string())
        );
    }
}
