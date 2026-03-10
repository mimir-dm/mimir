#![allow(dead_code)]
//! Shared test helpers for mimir-core integration tests.
//!
//! Provides `setup_srd_db()` which creates an in-memory SQLite database
//! pre-seeded with SRD fixture data from `tests/fixtures/*.json`.
//!
//! SRD content is published under the OGL and safe to commit.

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde_json::Value;

use mimir_core::dal::catalog;
use mimir_core::models::catalog::{
    NewBackground, NewClass, NewClassFeature, NewItem, NewMonster, NewRace, NewSpell, NewSubclass,
    NewSubclassFeature, NewCatalogSource,
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Path to the fixtures directory relative to the crate root.
const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

/// Create an in-memory database with migrations applied.
fn create_db() -> SqliteConnection {
    let mut conn =
        SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    conn
}

/// Load a JSON fixture file and parse it as a Vec<Value>.
fn load_fixture(filename: &str) -> Vec<Value> {
    let path = format!("{}/{}", FIXTURES_DIR, filename);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path, e))
}

/// Set up an in-memory database seeded with SRD fixture data.
///
/// This includes:
/// - PHB catalog source
/// - All 12 SRD classes
/// - All 12 SRD subclasses
/// - 170 class features
/// - 89 subclass features
/// - 7 backgrounds
/// - 17 races
/// - ~44 items
/// - ~43 spells
/// - ~17 monsters
pub fn setup_srd_db() -> SqliteConnection {
    let mut conn = create_db();

    // Collect all unique sources from fixtures and insert them
    seed_sources(&mut conn);

    seed_classes(&mut conn);
    seed_subclasses(&mut conn);
    seed_class_features(&mut conn);
    seed_subclass_features(&mut conn);
    seed_backgrounds(&mut conn);
    seed_races(&mut conn);
    seed_items(&mut conn);
    seed_spells(&mut conn);
    seed_monsters(&mut conn);

    conn
}

/// Set up the SRD database plus additional sources for broader testing.
pub fn setup_srd_db_with_extra_sources() -> SqliteConnection {
    let mut conn = setup_srd_db();

    let extra_sources = [
        NewCatalogSource::new("MM", "Monster Manual", true, "2024-01-20T12:00:00Z"),
        NewCatalogSource::new("DMG", "Dungeon Master's Guide", true, "2024-01-20T12:00:00Z"),
        NewCatalogSource::new("XGE", "Xanathar's Guide to Everything", true, "2024-01-20T12:00:00Z"),
    ];
    for source in &extra_sources {
        catalog::insert_source(&mut conn, source).expect("Failed to insert source");
    }

    conn
}

// ─── Seed functions ──────────────────────────────────────────────────────────

/// Scan all fixture files for unique source codes and insert them.
fn seed_sources(conn: &mut SqliteConnection) {
    use std::collections::HashSet;

    let fixture_files = [
        "srd_classes.json",
        "srd_subclasses.json",
        "srd_class_features.json",
        "srd_subclass_features.json",
        "srd_backgrounds.json",
        "srd_races.json",
        "srd_items.json",
        "srd_spells.json",
        "srd_monsters.json",
    ];

    let mut source_codes = HashSet::new();
    for filename in &fixture_files {
        let fixtures = load_fixture(filename);
        for item in &fixtures {
            if let Some(source) = item["source"].as_str() {
                source_codes.insert(source.to_string());
            }
            // Also check classSource, subclassSource for feature tables
            if let Some(cs) = item.get("classSource").and_then(|v| v.as_str()) {
                source_codes.insert(cs.to_string());
            }
            if let Some(ss) = item.get("subclassSource").and_then(|v| v.as_str()) {
                source_codes.insert(ss.to_string());
            }
        }
    }

    let source_name_map: std::collections::HashMap<&str, &str> = [
        ("PHB", "Player's Handbook"),
        ("MM", "Monster Manual"),
        ("DMG", "Dungeon Master's Guide"),
        ("XGE", "Xanathar's Guide to Everything"),
        ("TCE", "Tasha's Cauldron of Everything"),
        ("XDMG", "Dungeon Master's Guide (2024)"),
        ("VGM", "Volo's Guide to Monsters"),
        ("MPMM", "Mordenkainen Presents: Monsters of the Multiverse"),
    ].into_iter().collect();

    for code in &source_codes {
        let default_name = code.as_str();
        let name = source_name_map.get(code.as_str()).copied().unwrap_or(default_name);
        let source = NewCatalogSource::new(code, name, true, "2024-01-20T12:00:00Z");
        catalog::insert_source(conn, &source)
            .unwrap_or_else(|e| panic!("Failed to insert source {}: {}", code, e));
    }
}

fn seed_classes(conn: &mut SqliteConnection) {
    let fixtures = load_fixture("srd_classes.json");
    for item in &fixtures {
        let name = item["name"].as_str().unwrap();
        let source = item["source"].as_str().unwrap();
        // Reconstruct the data blob: everything except id, fluff
        let mut data = item.clone();
        if let Value::Object(ref mut map) = data {
            map.remove("id");
            map.remove("fluff");
        }
        let data_str = serde_json::to_string(&data).unwrap();
        let fluff = item.get("fluff").map(|f| serde_json::to_string(f).unwrap());
        let new_class = NewClass {
            name,
            source,
            data: &data_str,
            fluff: fluff.as_deref(),
        };
        catalog::insert_class(conn, &new_class)
            .unwrap_or_else(|e| panic!("Failed to insert class {}: {}", name, e));
    }
}

fn seed_subclasses(conn: &mut SqliteConnection) {
    let fixtures = load_fixture("srd_subclasses.json");
    for item in &fixtures {
        let name = item["name"].as_str().unwrap();
        let class_name = item["className"].as_str().unwrap();
        let source = item["source"].as_str().unwrap();
        let mut data = item.clone();
        if let Value::Object(ref mut map) = data {
            map.remove("id");
            map.remove("fluff");
            map.remove("className");
        }
        let data_str = serde_json::to_string(&data).unwrap();
        let fluff = item.get("fluff").map(|f| serde_json::to_string(f).unwrap());
        let new_sub = NewSubclass {
            name,
            class_name,
            source,
            data: &data_str,
            fluff: fluff.as_deref(),
        };
        catalog::insert_subclass(conn, &new_sub)
            .unwrap_or_else(|e| panic!("Failed to insert subclass {}: {}", name, e));
    }
}

fn seed_class_features(conn: &mut SqliteConnection) {
    let fixtures = load_fixture("srd_class_features.json");
    for item in &fixtures {
        let name = item["name"].as_str().unwrap();
        let source = item["source"].as_str().unwrap();
        let class_name = item["className"].as_str().unwrap();
        let class_source = item["classSource"].as_str().unwrap_or("PHB");
        let level = item["level"].as_i64().unwrap() as i32;
        let mut data = item.clone();
        if let Value::Object(ref mut map) = data {
            map.remove("id");
            map.remove("className");
            map.remove("classSource");
        }
        let data_str = serde_json::to_string(&data).unwrap();
        let new_feature = NewClassFeature {
            name,
            source,
            class_name,
            class_source,
            level,
            data: &data_str,
        };
        catalog::insert_class_feature(conn, &new_feature)
            .unwrap_or_else(|e| panic!("Failed to insert class feature {}: {}", name, e));
    }
}

fn seed_subclass_features(conn: &mut SqliteConnection) {
    let fixtures = load_fixture("srd_subclass_features.json");
    for item in &fixtures {
        let name = item["name"].as_str().unwrap();
        let source = item["source"].as_str().unwrap();
        let class_name = item["className"].as_str().unwrap();
        let class_source = item["classSource"].as_str().unwrap_or("PHB");
        let subclass_name = item["subclassName"].as_str().unwrap();
        let subclass_source = item["subclassSource"].as_str().unwrap_or("PHB");
        let level = item["level"].as_i64().unwrap() as i32;
        let mut data = item.clone();
        if let Value::Object(ref mut map) = data {
            map.remove("id");
            map.remove("className");
            map.remove("classSource");
            map.remove("subclassName");
            map.remove("subclassSource");
        }
        let data_str = serde_json::to_string(&data).unwrap();
        let new_feature = NewSubclassFeature {
            name,
            source,
            class_name,
            class_source,
            subclass_name,
            subclass_source,
            level,
            data: &data_str,
        };
        catalog::insert_subclass_feature(conn, &new_feature)
            .unwrap_or_else(|e| panic!("Failed to insert subclass feature {}: {}", name, e));
    }
}

fn seed_backgrounds(conn: &mut SqliteConnection) {
    let fixtures = load_fixture("srd_backgrounds.json");
    for item in &fixtures {
        let name = item["name"].as_str().unwrap();
        let source = item["source"].as_str().unwrap();
        let mut data = item.clone();
        if let Value::Object(ref mut map) = data {
            map.remove("id");
            map.remove("fluff");
        }
        let data_str = serde_json::to_string(&data).unwrap();
        let fluff = item.get("fluff").map(|f| serde_json::to_string(f).unwrap());
        let new_bg = NewBackground {
            name,
            source,
            data: &data_str,
            fluff: fluff.as_deref(),
        };
        catalog::insert_background(conn, &new_bg)
            .unwrap_or_else(|e| panic!("Failed to insert background {}: {}", name, e));
    }
}

fn seed_races(conn: &mut SqliteConnection) {
    let fixtures = load_fixture("srd_races.json");
    for item in &fixtures {
        let name = item["name"].as_str().unwrap();
        let source = item["source"].as_str().unwrap();
        let mut data = item.clone();
        if let Value::Object(ref mut map) = data {
            map.remove("id");
            map.remove("fluff");
        }
        let data_str = serde_json::to_string(&data).unwrap();
        let fluff = item.get("fluff").map(|f| serde_json::to_string(f).unwrap());
        let new_race = NewRace {
            name,
            source,
            data: &data_str,
            fluff: fluff.as_deref(),
        };
        catalog::insert_race(conn, &new_race)
            .unwrap_or_else(|e| panic!("Failed to insert race {}: {}", name, e));
    }
}

fn seed_items(conn: &mut SqliteConnection) {
    let fixtures = load_fixture("srd_items.json");
    for item in &fixtures {
        let name = item["name"].as_str().unwrap();
        let source = item["source"].as_str().unwrap();
        let item_type = item.get("_itemType").and_then(|v| v.as_str());
        let rarity = item.get("_rarity").and_then(|v| v.as_str());
        let mut data = item.clone();
        if let Value::Object(ref mut map) = data {
            map.remove("id");
            map.remove("fluff");
            map.remove("_itemType");
            map.remove("_rarity");
        }
        let data_str = serde_json::to_string(&data).unwrap();
        let fluff = item.get("fluff").map(|f| serde_json::to_string(f).unwrap());
        let new_item = NewItem {
            name,
            source,
            item_type,
            rarity,
            data: &data_str,
            fluff: fluff.as_deref(),
        };
        catalog::insert_item(conn, &new_item)
            .unwrap_or_else(|e| panic!("Failed to insert item {}: {}", name, e));
    }
}

fn seed_spells(conn: &mut SqliteConnection) {
    let fixtures = load_fixture("srd_spells.json");
    for item in &fixtures {
        let name = item["name"].as_str().unwrap();
        let source = item["source"].as_str().unwrap();
        let level = item.get("_level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let school = item.get("_school").and_then(|v| v.as_str());
        let ritual = if item.get("_ritual").and_then(|v| v.as_bool()).unwrap_or(false) { 1 } else { 0 };
        let concentration = if item.get("_concentration").and_then(|v| v.as_bool()).unwrap_or(false) { 1 } else { 0 };
        let mut data = item.clone();
        if let Value::Object(ref mut map) = data {
            map.remove("id");
            map.remove("fluff");
            map.remove("_level");
            map.remove("_school");
            map.remove("_ritual");
            map.remove("_concentration");
        }
        let data_str = serde_json::to_string(&data).unwrap();
        let fluff = item.get("fluff").map(|f| serde_json::to_string(f).unwrap());
        let new_spell = NewSpell {
            name,
            source,
            level,
            school,
            ritual,
            concentration,
            data: &data_str,
            fluff: fluff.as_deref(),
        };
        catalog::insert_spell(conn, &new_spell)
            .unwrap_or_else(|e| panic!("Failed to insert spell {}: {}", name, e));
    }
}

fn seed_monsters(conn: &mut SqliteConnection) {
    let fixtures = load_fixture("srd_monsters.json");
    for item in &fixtures {
        let name = item["name"].as_str().unwrap();
        let source = item["source"].as_str().unwrap();
        let cr = item.get("_cr").and_then(|v| v.as_str());
        let creature_type = item.get("_creatureType").and_then(|v| v.as_str());
        let size = item.get("_size").and_then(|v| v.as_str());
        let mut data = item.clone();
        if let Value::Object(ref mut map) = data {
            map.remove("id");
            map.remove("fluff");
            map.remove("_cr");
            map.remove("_creatureType");
            map.remove("_size");
        }
        let data_str = serde_json::to_string(&data).unwrap();
        let fluff = item.get("fluff").map(|f| serde_json::to_string(f).unwrap());
        let new_monster = NewMonster {
            name,
            source,
            cr,
            creature_type,
            size,
            token_image_path: None,
            data: &data_str,
            fluff: fluff.as_deref(),
        };
        catalog::insert_monster(conn, &new_monster)
            .unwrap_or_else(|e| panic!("Failed to insert monster {}: {}", name, e));
    }
}
