//! Generate a minimal DEV book from existing book imports.
//!
//! This extracts monsters, items, and spells needed for the dev seeder.
//!
//! Usage:
//!   cargo run --bin gen_dev_book -- <books_dir> <output_dir>
//!
//! Example:
//!   cargo run --bin gen_dev_book -- ~/Library/Application\ Support/com.mimir.mimir-test/data/books /tmp/DEV_BOOK

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Monsters needed by the dev seeder
const REQUIRED_MONSTERS: &[&str] = &[
    "Goblin",
    "Goblin Boss",
    "Wolf",
    "Bugbear",
    "Adult Black Dragon",
    "Mage",
];

/// Items needed by the dev seeder
const REQUIRED_ITEMS: &[&str] = &[
    "Potion of Healing",
    "+1 Weapon",
    "Spell Scroll",
    "Longsword",
];

/// Spells for testing (common low-level spells)
const REQUIRED_SPELLS: &[&str] = &[
    "Fire Bolt",
    "Light",
    "Mage Hand",
    "Prestidigitation",
    "Magic Missile",
    "Shield",
    "Cure Wounds",
    "Healing Word",
    "Guiding Bolt",
    "Bless",
    "Detect Magic",
    "Identify",
];

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <books_dir> <output_dir>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!(
            "  {} ~/Library/Application\\ Support/com.mimir.mimir-test/data/books /tmp/DEV_BOOK",
            args[0]
        );
        std::process::exit(1);
    }

    let books_dir = Path::new(&args[1]);
    let output_dir = Path::new(&args[2]);

    if !books_dir.exists() {
        eprintln!("Error: Books directory does not exist: {:?}", books_dir);
        std::process::exit(1);
    }

    generate_dev_book(books_dir, output_dir)?;
    println!();
    println!("DEV book generated at {:?}", output_dir);
    println!();
    println!("To create a tarball:");
    println!("  cd {:?} && tar -cvf DEV_BOOK.tar DEV_BOOK/", output_dir.parent().unwrap_or(output_dir));

    Ok(())
}

fn generate_dev_book(books_dir: &Path, output_dir: &Path) -> Result<()> {
    let required_monsters: HashSet<&str> = REQUIRED_MONSTERS.iter().copied().collect();
    let required_items: HashSet<&str> = REQUIRED_ITEMS.iter().copied().collect();
    let required_spells: HashSet<&str> = REQUIRED_SPELLS.iter().copied().collect();

    // Create output directories
    fs::create_dir_all(output_dir.join("bestiary"))?;
    fs::create_dir_all(output_dir.join("items"))?;
    fs::create_dir_all(output_dir.join("spells"))?;
    fs::create_dir_all(output_dir.join("img/bestiary/tokens/DEV"))?;

    // Create metadata.json
    let metadata = json!({
        "name": "Dev Seeder Assets",
        "id": "dev-seeder",
        "source": "DEV",
        "group": "Development",
        "published": "2024-01-01",
        "author": "Mimir Dev Team",
    });
    fs::write(
        output_dir.join("metadata.json"),
        serde_json::to_string_pretty(&metadata)?,
    )?;
    println!("Created metadata.json");

    let mut extracted_monsters = Vec::new();
    let mut extracted_items = Vec::new();
    let mut extracted_spells = Vec::new();
    let mut token_sources: Vec<(String, std::path::PathBuf)> = Vec::new();

    // Scan all book directories
    for entry in fs::read_dir(books_dir)? {
        let entry = entry?;
        let book_path = entry.path();
        if !book_path.is_dir() {
            continue;
        }
        let book_name = entry.file_name().to_string_lossy().to_string();
        println!("Scanning book: {}", book_name);

        // Extract monsters
        let bestiary_dir = book_path.join("bestiary");
        if bestiary_dir.exists() {
            extract_from_json_dir(&bestiary_dir, "monster", &required_monsters, &mut extracted_monsters)?;
        }

        // Extract items
        let items_dir = book_path.join("items");
        if items_dir.exists() {
            extract_from_json_dir(&items_dir, "item", &required_items, &mut extracted_items)?;
        }
        // Also check items-base for basic equipment
        let items_base_dir = book_path.join("items-base");
        if items_base_dir.exists() {
            extract_from_json_dir(&items_base_dir, "baseitem", &required_items, &mut extracted_items)?;
        }

        // Extract spells
        let spells_dir = book_path.join("spells");
        if spells_dir.exists() {
            extract_from_json_dir(&spells_dir, "spell", &required_spells, &mut extracted_spells)?;
        }

        // Track token sources
        let tokens_dir = book_path.join("img/bestiary/tokens").join(&book_name);
        if tokens_dir.exists() {
            token_sources.push((book_name.clone(), tokens_dir));
        }
    }

    // Write extracted monsters
    if !extracted_monsters.is_empty() {
        let monsters_json = json!({ "monster": extracted_monsters });
        fs::write(
            output_dir.join("bestiary/dev.json"),
            serde_json::to_string_pretty(&monsters_json)?,
        )?;
        println!("Created bestiary/dev.json with {} monsters", extracted_monsters.len());
    }

    // Write extracted items
    if !extracted_items.is_empty() {
        let items_json = json!({ "item": extracted_items });
        fs::write(
            output_dir.join("items/dev.json"),
            serde_json::to_string_pretty(&items_json)?,
        )?;
        println!("Created items/dev.json with {} items", extracted_items.len());
    }

    // Write extracted spells
    if !extracted_spells.is_empty() {
        let spells_json = json!({ "spell": extracted_spells });
        fs::write(
            output_dir.join("spells/dev.json"),
            serde_json::to_string_pretty(&spells_json)?,
        )?;
        println!("Created spells/dev.json with {} spells", extracted_spells.len());
    }

    // Copy token images
    let tokens_dst = output_dir.join("img/bestiary/tokens/DEV");
    for monster_name in REQUIRED_MONSTERS {
        let mut found = false;
        for (book_name, tokens_src) in &token_sources {
            let src_file = tokens_src.join(format!("{}.webp", monster_name));
            if src_file.exists() {
                let dst_file = tokens_dst.join(format!("{}.webp", monster_name));
                fs::copy(&src_file, &dst_file)
                    .context(format!("Failed to copy token for {}", monster_name))?;
                println!("Copied token from {}: {}.webp", book_name, monster_name);
                found = true;
                break;
            }
        }
        if !found {
            eprintln!("Warning: Token not found for {}", monster_name);
        }
    }

    Ok(())
}

fn extract_from_json_dir(
    dir: &Path,
    key: &str,
    required: &HashSet<&str>,
    output: &mut Vec<Value>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if filename.starts_with("fluff-") || filename == "index.json" {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let data: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(items) = data.get(key).and_then(|m| m.as_array()) {
            for item in items {
                if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                    // Check if name matches or starts with any required name
                    let matches = required.iter().any(|r| name == *r || name.starts_with(r));
                    if matches {
                        let mut i = item.clone();
                        if let Some(obj) = i.as_object_mut() {
                            obj.insert("source".to_string(), json!("DEV"));
                        }
                        output.push(i);
                        println!("  Extracted {}: {}", key, name);
                    }
                }
            }
        }
    }
    Ok(())
}
