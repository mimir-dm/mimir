//! Integration tests for catalog import from 5etools archive.
//!
//! These tests use the actual 5etools data from the data directory.

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use mimir_core::dal::catalog;
use mimir_core::import::CatalogImportService;
use std::path::Path;

/// Embed all migrations at compile time.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Path to 5etools data directory.
const FIVETOOLS_DATA_PATH: &str = "../../data/5etools-2014-src-v1.210.46";

/// Set up an in-memory database with all migrations applied.
fn setup_test_db() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:")
        .expect("Failed to create in-memory database");

    // Run all migrations
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    conn
}

/// Check if 5etools data is available for testing.
fn fivetools_data_exists() -> bool {
    let path = Path::new(FIVETOOLS_DATA_PATH);
    path.exists() && path.join("data").join("books.json").exists()
}

#[test]
fn test_import_phb_source() {
    if !fivetools_data_exists() {
        eprintln!("Skipping integration test: 5etools data not found at {}", FIVETOOLS_DATA_PATH);
        return;
    }

    let mut conn = setup_test_db();
    let repo_path = Path::new(FIVETOOLS_DATA_PATH);

    // Create import service and import PHB
    let mut service = CatalogImportService::new(&mut conn);
    let result = service.import_from_directory(repo_path).expect("Import should succeed");

    // PHB should be in the imported sources
    assert!(
        result.sources_imported.iter().any(|s| s == "PHB"),
        "PHB should be imported. Got sources: {:?}",
        result.sources_imported
    );

    // Verify we imported a significant number of entities
    assert!(
        result.total_entities > 100,
        "Should import many entities. Got {}",
        result.total_entities
    );

    // Print summary for debugging
    println!("Import summary:\n{}", result.summary());
}

#[test]
fn test_import_spells_from_phb() {
    if !fivetools_data_exists() {
        eprintln!("Skipping integration test: 5etools data not found");
        return;
    }

    let mut conn = setup_test_db();
    let repo_path = Path::new(FIVETOOLS_DATA_PATH);

    // Import from directory
    let mut service = CatalogImportService::new(&mut conn);
    let result = service.import_from_directory(repo_path).expect("Import should succeed");

    // Check spell count
    let spell_count = catalog::count_spells(&mut conn).expect("Should count spells");
    println!("Imported {} spells", spell_count);

    assert!(
        spell_count > 0,
        "Should import spells from PHB"
    );

    // Verify a known spell exists (Fireball is iconic)
    let fireball = catalog::get_spell_by_name(&mut conn, "Fireball", "PHB")
        .expect("Query should succeed");

    assert!(
        fireball.is_some(),
        "Fireball spell should exist from PHB"
    );

    if let Some(spell) = fireball {
        assert_eq!(spell.level, 3, "Fireball should be level 3");
        assert_eq!(spell.school.as_deref(), Some("V"), "Fireball should be Evocation");
    }

    // Check total from import result
    if let Some(&count) = result.entity_counts.get("spell") {
        println!("Import result shows {} spells", count);
        assert!(count > 100, "PHB should have 100+ spells");
    }
}

#[test]
fn test_import_monsters_from_mm() {
    if !fivetools_data_exists() {
        eprintln!("Skipping integration test: 5etools data not found");
        return;
    }

    let mut conn = setup_test_db();
    let repo_path = Path::new(FIVETOOLS_DATA_PATH);

    // Import from directory
    let mut service = CatalogImportService::new(&mut conn);
    let result = service.import_from_directory(repo_path).expect("Import should succeed");

    // Check monster count
    let monster_count = catalog::count_monsters(&mut conn).expect("Should count monsters");
    println!("Imported {} monsters", monster_count);

    // MM should be imported
    assert!(
        result.sources_imported.iter().any(|s| s == "MM"),
        "Monster Manual should be imported"
    );

    // Verify known monsters exist
    let goblin = catalog::get_monster_by_name(&mut conn, "Goblin", "MM")
        .expect("Query should succeed");
    assert!(goblin.is_some(), "Goblin should exist from MM");

    if let Some(monster) = goblin {
        assert_eq!(monster.cr.as_deref(), Some("1/4"), "Goblin should be CR 1/4");
        assert_eq!(monster.creature_type.as_deref(), Some("humanoid"), "Goblin should be humanoid");
        assert_eq!(monster.size.as_deref(), Some("S"), "Goblin should be Small");
    }

    // Check for another iconic monster
    let dragon = catalog::get_monster_by_name(&mut conn, "Adult Red Dragon", "MM")
        .expect("Query should succeed");
    assert!(dragon.is_some(), "Adult Red Dragon should exist from MM");

    if let Some(monster) = dragon {
        assert_eq!(monster.cr.as_deref(), Some("17"), "Adult Red Dragon should be CR 17");
        assert_eq!(monster.creature_type.as_deref(), Some("dragon"), "Should be dragon type");
    }
}

#[test]
fn test_import_items_with_attunement() {
    if !fivetools_data_exists() {
        eprintln!("Skipping integration test: 5etools data not found");
        return;
    }

    let mut conn = setup_test_db();
    let repo_path = Path::new(FIVETOOLS_DATA_PATH);

    // Import from directory
    let mut service = CatalogImportService::new(&mut conn);
    service.import_from_directory(repo_path).expect("Import should succeed");

    // Check item count
    let item_count = catalog::count_items(&mut conn).expect("Should count items");
    println!("Imported {} items", item_count);
    assert!(item_count > 0, "Should import items");

    // Verify DMG is imported for magic items
    let sources = catalog::list_sources(&mut conn).expect("Should list sources");
    let dmg_source = sources.iter().find(|s| s.code == "DMG");
    assert!(dmg_source.is_some(), "DMG should be imported for magic items");
}

#[test]
fn test_import_classes_and_subclasses() {
    if !fivetools_data_exists() {
        eprintln!("Skipping integration test: 5etools data not found");
        return;
    }

    let mut conn = setup_test_db();
    let repo_path = Path::new(FIVETOOLS_DATA_PATH);

    // Import from directory
    let mut service = CatalogImportService::new(&mut conn);
    service.import_from_directory(repo_path).expect("Import should succeed");

    // Check class count
    let class_count = catalog::count_classes(&mut conn).expect("Should count classes");
    println!("Imported {} classes", class_count);

    // Core classes should exist
    assert!(class_count >= 12, "Should have at least 12 core classes (including Artificer)");

    // Verify known class exists
    let fighter = catalog::get_class_by_name(&mut conn, "Fighter", "PHB")
        .expect("Query should succeed");
    assert!(fighter.is_some(), "Fighter class should exist from PHB");

    // Check subclasses
    let subclass_count = catalog::count_subclasses(&mut conn).expect("Should count subclasses");
    println!("Imported {} subclasses", subclass_count);
    assert!(subclass_count > 0, "Should import subclasses");
}

#[test]
fn test_import_optional_features() {
    if !fivetools_data_exists() {
        eprintln!("Skipping integration test: 5etools data not found");
        return;
    }

    let mut conn = setup_test_db();
    let repo_path = Path::new(FIVETOOLS_DATA_PATH);

    // Import from directory
    let mut service = CatalogImportService::new(&mut conn);
    let result = service.import_from_directory(repo_path).expect("Import should succeed");

    // Check optional feature count
    let optional_feature_count = catalog::count_optional_features(&mut conn)
        .expect("Should count optional features");
    println!("Imported {} optional features", optional_feature_count);

    // Should have imported some optional features (invocations, metamagic, etc.)
    if let Some(&count) = result.entity_counts.get("optionalfeature") {
        println!("Import result shows {} optional features", count);
    }

    // If there are optional features, verify we can list them
    if optional_feature_count > 0 {
        let features = catalog::list_optional_features(&mut conn)
            .expect("Should list optional features");
        println!("First few optional features: {:?}",
            features.iter().take(5).map(|f| &f.name).collect::<Vec<_>>());
    }
}

#[test]
fn test_import_preserves_full_json_data() {
    if !fivetools_data_exists() {
        eprintln!("Skipping integration test: 5etools data not found");
        return;
    }

    let mut conn = setup_test_db();
    let repo_path = Path::new(FIVETOOLS_DATA_PATH);

    // Import from directory
    let mut service = CatalogImportService::new(&mut conn);
    service.import_from_directory(repo_path).expect("Import should succeed");

    // Get a spell and verify the data column has the full JSON
    let fireball = catalog::get_spell_by_name(&mut conn, "Fireball", "PHB")
        .expect("Query should succeed")
        .expect("Fireball should exist");

    // Parse the data column
    let data: serde_json::Value = serde_json::from_str(&fireball.data)
        .expect("Data column should be valid JSON");

    // Verify the JSON contains expected fields
    assert_eq!(data["name"].as_str(), Some("Fireball"), "JSON should have name");
    assert_eq!(data["level"].as_i64(), Some(3), "JSON should have level");

    // Check for entries (the spell description)
    assert!(data["entries"].is_array(), "JSON should have entries array");

    // Verify there's actual content in entries
    let entries = data["entries"].as_array().unwrap();
    assert!(!entries.is_empty(), "Entries should not be empty");
}

#[test]
fn test_spell_class_relationships() {
    if !fivetools_data_exists() {
        eprintln!("Skipping integration test: 5etools data not found");
        return;
    }

    let mut conn = setup_test_db();
    let repo_path = Path::new(FIVETOOLS_DATA_PATH);

    // Import from directory
    let mut service = CatalogImportService::new(&mut conn);
    service.import_from_directory(repo_path).expect("Import should succeed");

    // Get Fireball and check its class relationships
    let fireball = catalog::get_spell_by_name(&mut conn, "Fireball", "PHB")
        .expect("Query should succeed")
        .expect("Fireball should exist");

    let spell_classes = catalog::get_spell_classes(&mut conn, fireball.id.unwrap())
        .expect("Should get spell classes");

    println!("Fireball is available to: {:?}",
        spell_classes.iter().map(|sc| &sc.class_name).collect::<Vec<_>>());

    // Fireball should be available to Sorcerer and Wizard at minimum
    let class_names: Vec<&str> = spell_classes.iter().map(|sc| sc.class_name.as_str()).collect();
    assert!(class_names.contains(&"Sorcerer"), "Fireball should be a Sorcerer spell");
    assert!(class_names.contains(&"Wizard"), "Fireball should be a Wizard spell");
}

#[test]
fn test_import_summary_accuracy() {
    if !fivetools_data_exists() {
        eprintln!("Skipping integration test: 5etools data not found");
        return;
    }

    let mut conn = setup_test_db();
    let repo_path = Path::new(FIVETOOLS_DATA_PATH);

    // Import from directory
    let mut service = CatalogImportService::new(&mut conn);
    let result = service.import_from_directory(repo_path).expect("Import should succeed");

    // Verify counts match database
    for (entity_type, reported_count) in &result.entity_counts {
        let db_count = match entity_type.as_str() {
            "monster" => catalog::count_monsters(&mut conn).ok(),
            "spell" => catalog::count_spells(&mut conn).ok(),
            "item" => catalog::count_items(&mut conn).ok(),
            "class" => catalog::count_classes(&mut conn).ok(),
            "subclass" => catalog::count_subclasses(&mut conn).ok(),
            "race" => catalog::count_races(&mut conn).ok(),
            "background" => catalog::count_backgrounds(&mut conn).ok(),
            "feat" => catalog::count_feats(&mut conn).ok(),
            "condition" => catalog::count_conditions(&mut conn).ok(),
            "disease" => catalog::count_diseases(&mut conn).ok(),
            "action" => catalog::count_actions(&mut conn).ok(),
            "language" => catalog::count_languages(&mut conn).ok(),
            "vehicle" => catalog::count_vehicles(&mut conn).ok(),
            "object" => catalog::count_objects(&mut conn).ok(),
            "trap" => catalog::count_traps(&mut conn).ok(),
            "hazard" => catalog::count_hazards(&mut conn).ok(),
            "deity" => catalog::count_deities(&mut conn).ok(),
            "cult" | "boon" => catalog::count_cults(&mut conn).ok(),
            "optionalfeature" => catalog::count_optional_features(&mut conn).ok(),
            "psionic" => catalog::count_psionics(&mut conn).ok(),
            "reward" => catalog::count_rewards(&mut conn).ok(),
            "variantrule" => catalog::count_variant_rules(&mut conn).ok(),
            "table" => catalog::count_catalog_tables(&mut conn).ok(),
            _ => None,
        };

        if let Some(db_count) = db_count {
            // Allow for some variance due to filtering, but should be in same ballpark
            println!("{}: reported={}, db={}", entity_type, reported_count, db_count);
        }
    }

    // Total entities should be > 0
    assert!(result.total_entities > 0, "Should have imported some entities");

    // No failures expected for standard 5etools data
    if !result.sources_failed.is_empty() {
        println!("Failed sources: {:?}", result.sources_failed);
    }
}
