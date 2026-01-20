use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Find all image files referenced by a book
pub fn find_book_images(repo_path: &Path, source: &str) -> Result<Vec<PathBuf>> {
    let mut images = Vec::new();
    let img_dir = repo_path.join("img");

    if !img_dir.exists() {
        return Ok(images);
    }

    // Common image locations for books
    let locations = [
        img_dir.join("book").join(source),
        img_dir.join("adventure").join(source),
        img_dir.join("bestiary").join(source),
        img_dir.join("items").join(source),
        img_dir.join("backgrounds").join(source),
        img_dir.join("races").join(source),
        img_dir.join("classes").join(source),
        img_dir.join("covers").join(format!("{}.webp", source)),
    ];

    for location in &locations {
        if location.is_file() {
            images.push(location.clone());
        } else if location.is_dir() {
            for entry in WalkDir::new(location)
                .follow_links(true)
                .into_iter()
                .flatten()
            {
                if entry.file_type().is_file() {
                    images.push(entry.path().to_path_buf());
                }
            }
        }
    }

    Ok(images)
}

/// Extract image references from JSON content
pub fn extract_image_refs(content: &serde_json::Value) -> HashSet<String> {
    let mut refs = HashSet::new();
    extract_refs_recursive(content, &mut refs);
    refs
}

fn extract_refs_recursive(value: &serde_json::Value, refs: &mut HashSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            // Look for common image reference fields
            if let Some(img_path) = map.get("path").and_then(|v| v.as_str()) {
                refs.insert(img_path.to_string());
            }
            if let Some(img_href) = map.get("href").and_then(|v| v.as_object()) {
                if let Some(path) = img_href.get("path").and_then(|v| v.as_str()) {
                    refs.insert(path.to_string());
                }
            }

            // Recurse into all values
            for v in map.values() {
                extract_refs_recursive(v, refs);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                extract_refs_recursive(v, refs);
            }
        }
        serde_json::Value::String(s) => {
            // Check if string looks like an image path
            if (s.contains(".jpg")
                || s.contains(".png")
                || s.contains(".webp")
                || s.contains(".gif"))
                && (s.starts_with("img/") || s.starts_with("covers/"))
            {
                refs.insert(s.to_string());
            }
        }
        _ => {}
    }
}
