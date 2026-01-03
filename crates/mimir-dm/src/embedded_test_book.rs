//! Embedded test books for development builds
//! Automatically loads all .tar.gz files from assets/dev/

use flate2::read::GzDecoder;
use tar::Archive;
use tracing::{error, info};

/// Structure to hold embedded test book data
#[allow(dead_code)]
pub struct EmbeddedTestBook {
    pub name: String,
    #[allow(dead_code)]
    pub data: &'static [u8],
}

/// Macro to include all test books from the assets/dev directory
/// Only includes files that exist (checked by build.rs setting cfg flags)
macro_rules! include_test_books {
    () => {{
        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut books = Vec::new();

        // PHB - only included if build.rs detected the file exists
        #[cfg(has_dev_phb)]
        books.push(EmbeddedTestBook {
            name: "PHB".to_string(),
            data: include_bytes!("../assets/dev/phb.tar.gz"),
        });

        // MM - only included if build.rs detected the file exists
        #[cfg(has_dev_mm)]
        books.push(EmbeddedTestBook {
            name: "MM".to_string(),
            data: include_bytes!("../assets/dev/mm.tar.gz"),
        });

        // DMG - only included if build.rs detected the file exists
        #[cfg(has_dev_dmg)]
        books.push(EmbeddedTestBook {
            name: "DMG".to_string(),
            data: include_bytes!("../assets/dev/dmg.tar.gz"),
        });

        // Dev tokens - minimal token images for seeder
        #[cfg(has_dev_tokens)]
        books.push(EmbeddedTestBook {
            name: "DEV_TOKENS".to_string(),
            data: include_bytes!("../assets/dev/dev-tokens.tar.gz"),
        });

        books
    }};
}

/// Check if we're in development mode
pub fn is_dev_build() -> bool {
    cfg!(debug_assertions) || std::env::var("MIMIR_DEV").is_ok()
}

/// Get all embedded test books
#[allow(clippy::vec_init_then_push)]
pub fn get_embedded_test_books() -> Vec<EmbeddedTestBook> {
    include_test_books!()
}

/// Extract all embedded test book archives
#[allow(dead_code)]
pub fn extract_all_test_books(
    target_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_books = get_embedded_test_books();

    info!(
        "Extracting {} embedded test books to {:?}",
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
#[allow(dead_code)]
fn extract_single_book(
    book: &EmbeddedTestBook,
    target_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a decoder from the embedded bytes
    let decoder = GzDecoder::new(book.data);
    let mut archive = Archive::new(decoder);

    // Extract the archive
    archive.unpack(target_dir)?;

    Ok(())
}
