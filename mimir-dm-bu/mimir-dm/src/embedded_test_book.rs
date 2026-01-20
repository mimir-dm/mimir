//! Dev test books - loaded from disk at runtime (not embedded in binary)
//!
//! For development, books are loaded from the repo's assets/dev directory.
//! Set MIMIR_DEV_ASSETS env var to override the path.

use flate2::read::GzDecoder;
use std::path::PathBuf;
use tar::Archive;
use tracing::{error, info, warn};

/// Structure to hold test book data loaded from disk
pub struct DevTestBook {
    pub name: String,
    pub data: Vec<u8>,
}

/// Check if we're in development mode
pub fn is_dev_build() -> bool {
    cfg!(debug_assertions) || std::env::var("MIMIR_DEV").is_ok()
}

/// Get the dev assets directory path
fn get_dev_assets_dir() -> Option<PathBuf> {
    // 1. Check env var override
    if let Ok(path) = std::env::var("MIMIR_DEV_ASSETS") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
        warn!("MIMIR_DEV_ASSETS path does not exist: {:?}", p);
    }

    // 2. Try to find repo root from executable location (for dev builds)
    if let Ok(exe) = std::env::current_exe() {
        // Walk up from executable to find Cargo.toml (repo root indicator)
        let mut current = exe.parent();
        while let Some(dir) = current {
            let cargo_toml = dir.join("Cargo.toml");
            if cargo_toml.exists() {
                let assets_dir = dir.join("crates/mimir-dm/assets/dev");
                if assets_dir.exists() {
                    return Some(assets_dir);
                }
            }
            current = dir.parent();
        }
    }

    // 3. Try relative to current working directory
    let cwd_assets = PathBuf::from("crates/mimir-dm/assets/dev");
    if cwd_assets.exists() {
        return Some(cwd_assets);
    }

    None
}

/// Get all dev test books by loading from disk
pub fn get_dev_test_books() -> Vec<DevTestBook> {
    let Some(assets_dir) = get_dev_assets_dir() else {
        if is_dev_build() {
            warn!("Dev assets directory not found. Set MIMIR_DEV_ASSETS env var or run from repo root.");
        }
        return Vec::new();
    };

    let book_files = [
        ("PHB", "phb.tar.gz"),
        ("MM", "mm.tar.gz"),
        ("DMG", "dmg.tar.gz"),
    ];

    let mut books = Vec::new();
    for (name, filename) in book_files {
        let path = assets_dir.join(filename);
        if path.exists() {
            match std::fs::read(&path) {
                Ok(data) => {
                    info!("Loaded dev book {} ({} bytes)", name, data.len());
                    books.push(DevTestBook {
                        name: name.to_string(),
                        data,
                    });
                }
                Err(e) => {
                    warn!("Failed to read dev book {}: {}", name, e);
                }
            }
        }
    }

    books
}

/// Backwards compatibility alias
pub fn get_embedded_test_books() -> Vec<DevTestBook> {
    get_dev_test_books()
}

/// Extract all dev test book archives to target directory
pub fn extract_all_test_books(
    target_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_books = get_dev_test_books();

    if test_books.is_empty() {
        info!("No dev test books found to extract");
        return Ok(());
    }

    info!(
        "Extracting {} dev test books to {:?}",
        test_books.len(),
        target_dir
    );

    for book in test_books {
        match extract_single_book(&book, target_dir) {
            Ok(_) => info!("Successfully extracted test book: {}", book.name),
            Err(e) => error!("Failed to extract test book {}: {}", book.name, e),
        }
    }

    Ok(())
}

/// Extract a single test book archive
fn extract_single_book(
    book: &DevTestBook,
    target_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let decoder = GzDecoder::new(book.data.as_slice());
    let mut archive = Archive::new(decoder);
    archive.unpack(target_dir)?;
    Ok(())
}
