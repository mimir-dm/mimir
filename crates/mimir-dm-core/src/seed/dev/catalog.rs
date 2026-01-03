//! DEV book seeding for dev data.
//!
//! Embeds a pre-built DEV book with monsters, items, and spells,
//! extracts it to data/books/DEV, and imports the catalog data.

use crate::connection::DbConnection;
use crate::error::Result;
use crate::models::catalog::NewUploadedBook;
use crate::schema::uploaded_books;
use crate::services::{ItemService, MonsterService, SpellService};
use chrono::Utc;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tracing::info;

// Embedded JSON files
const BESTIARY_JSON: &[u8] = include_bytes!("../assets/DEV_BOOK/bestiary/dev.json");
const ITEMS_JSON: &[u8] = include_bytes!("../assets/DEV_BOOK/items/dev.json");
const SPELLS_JSON: &[u8] = include_bytes!("../assets/DEV_BOOK/spells/spells-dev.json");
const METADATA_JSON: &[u8] = include_bytes!("../assets/DEV_BOOK/metadata.json");
const BOOK_JSON: &[u8] = include_bytes!("../assets/DEV_BOOK/book.json");

// Embedded token images
const TOKEN_GOBLIN: &[u8] = include_bytes!("../assets/DEV_BOOK/img/bestiary/tokens/DEV/Goblin.webp");
const TOKEN_GOBLIN_BOSS: &[u8] = include_bytes!("../assets/DEV_BOOK/img/bestiary/tokens/DEV/Goblin Boss.webp");
const TOKEN_WOLF: &[u8] = include_bytes!("../assets/DEV_BOOK/img/bestiary/tokens/DEV/Wolf.webp");
const TOKEN_BUGBEAR: &[u8] = include_bytes!("../assets/DEV_BOOK/img/bestiary/tokens/DEV/Bugbear.webp");
const TOKEN_MAGE: &[u8] = include_bytes!("../assets/DEV_BOOK/img/bestiary/tokens/DEV/Mage.webp");
const TOKEN_DRAGON: &[u8] = include_bytes!("../assets/DEV_BOOK/img/bestiary/tokens/DEV/Adult Black Dragon.webp");

/// Seed the DEV book catalog data.
///
/// Extracts the embedded DEV book to data/books/DEV and imports
/// monsters, items, and spells into the catalog tables.
pub fn seed(conn: &mut DbConnection, data_dir: &str) -> Result<()> {
    let books_dir = Path::new(data_dir).join("books");
    let dev_book_dir = books_dir.join("DEV");

    // Extract DEV book if not already present
    if !dev_book_dir.exists() {
        info!("Extracting embedded DEV book to {:?}", dev_book_dir);
        extract_dev_book(&dev_book_dir)?;
    }

    // Register the book in uploaded_books table
    register_dev_book(conn, &dev_book_dir)?;

    // Import catalog data from the extracted book
    import_from_dev_book(conn, &dev_book_dir)?;

    Ok(())
}

fn register_dev_book(conn: &mut DbConnection, dev_book_dir: &Path) -> Result<()> {
    let metadata = serde_json::json!({
        "name": "Dev Seeder Assets",
        "id": "dev-seeder",
        "source": "DEV",
        "group": "Development",
        "published": "2024-01-01",
        "author": "Mimir Dev Team"
    });

    let new_book = NewUploadedBook {
        id: "DEV".to_string(),
        name: "Dev Seeder Assets".to_string(),
        location: dev_book_dir.to_string_lossy().to_string(),
        archive_path: String::new(), // No archive for embedded book
        uploaded_at: Utc::now().to_rfc3339(),
        metadata_json: Some(metadata.to_string()),
    };

    diesel::insert_into(uploaded_books::table)
        .values(&new_book)
        .on_conflict(uploaded_books::id)
        .do_nothing()
        .execute(conn)?;

    info!("Registered DEV book in library");
    Ok(())
}

fn extract_dev_book(dev_book_dir: &Path) -> Result<()> {
    // Create directory structure
    fs::create_dir_all(dev_book_dir.join("bestiary"))?;
    fs::create_dir_all(dev_book_dir.join("items"))?;
    fs::create_dir_all(dev_book_dir.join("spells"))?;
    fs::create_dir_all(dev_book_dir.join("img/bestiary/tokens/DEV"))?;

    // Write JSON files
    fs::write(dev_book_dir.join("metadata.json"), METADATA_JSON)?;
    fs::write(dev_book_dir.join("book.json"), BOOK_JSON)?;
    fs::write(dev_book_dir.join("bestiary/dev.json"), BESTIARY_JSON)?;
    fs::write(dev_book_dir.join("items/dev.json"), ITEMS_JSON)?;
    fs::write(dev_book_dir.join("spells/spells-dev.json"), SPELLS_JSON)?;

    // Write token images
    let tokens_dir = dev_book_dir.join("img/bestiary/tokens/DEV");
    fs::write(tokens_dir.join("Goblin.webp"), TOKEN_GOBLIN)?;
    fs::write(tokens_dir.join("Goblin Boss.webp"), TOKEN_GOBLIN_BOSS)?;
    fs::write(tokens_dir.join("Wolf.webp"), TOKEN_WOLF)?;
    fs::write(tokens_dir.join("Bugbear.webp"), TOKEN_BUGBEAR)?;
    fs::write(tokens_dir.join("Mage.webp"), TOKEN_MAGE)?;
    fs::write(tokens_dir.join("Adult Black Dragon.webp"), TOKEN_DRAGON)?;

    info!("Extracted DEV book with 6 tokens");
    Ok(())
}

fn import_from_dev_book(conn: &mut DbConnection, book_dir: &Path) -> Result<()> {
    // Import monsters
    match MonsterService::import_monsters_from_book(conn, book_dir, "DEV") {
        Ok(count) => info!("Imported {} monsters from DEV book", count),
        Err(e) => info!("No monsters imported from DEV book: {}", e),
    }

    // Import items
    match ItemService::import_items_from_book(conn, book_dir, "DEV") {
        Ok(count) => info!("Imported {} items from DEV book", count),
        Err(e) => info!("No items imported from DEV book: {}", e),
    }

    // Import spells
    match SpellService::import_spells_from_book(conn, book_dir, "DEV") {
        Ok(count) => info!("Imported {} spells from DEV book", count),
        Err(e) => info!("No spells imported from DEV book: {}", e),
    }

    Ok(())
}
