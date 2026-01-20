//! Integration tests for ItemService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::item::ItemFilters;
use mimir_dm_core::services::ItemService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_item_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_item_data(conn: &mut SqliteConnection) {
    // Schema: name, item_type, type_name, rarity, value, weight, ac, damage, requires_attunement, source, full_item_json
    let items = vec![
        (
            "Longsword",
            "M",
            "Martial Weapon",
            "none",
            1500.0,
            3.0,
            None::<i32>,
            Some("1d8 slashing"),
            None::<&str>,
            "PHB",
            r#"{"name":"Longsword","source":"PHB","type":"M","rarity":"none","value":1500}"#,
        ),
        (
            "Shortsword",
            "M",
            "Martial Weapon",
            "none",
            1000.0,
            2.0,
            None,
            Some("1d6 piercing"),
            None,
            "PHB",
            r#"{"name":"Shortsword","source":"PHB","type":"M","rarity":"none","value":1000}"#,
        ),
        (
            "Dagger",
            "M",
            "Simple Weapon",
            "none",
            200.0,
            1.0,
            None,
            Some("1d4 piercing"),
            None,
            "PHB",
            r#"{"name":"Dagger","source":"PHB","type":"M","rarity":"none","value":200}"#,
        ),
        (
            "Plate Armor",
            "HA",
            "Heavy Armor",
            "none",
            150000.0,
            65.0,
            Some(18),
            None,
            None,
            "PHB",
            r#"{"name":"Plate Armor","source":"PHB","type":"HA","rarity":"none","value":150000,"ac":18}"#,
        ),
        (
            "Chain Mail",
            "HA",
            "Heavy Armor",
            "none",
            7500.0,
            55.0,
            Some(16),
            None,
            None,
            "PHB",
            r#"{"name":"Chain Mail","source":"PHB","type":"HA","rarity":"none","value":7500,"ac":16}"#,
        ),
        (
            "Leather Armor",
            "LA",
            "Light Armor",
            "none",
            1000.0,
            10.0,
            Some(11),
            None,
            None,
            "PHB",
            r#"{"name":"Leather Armor","source":"PHB","type":"LA","rarity":"none","value":1000,"ac":11}"#,
        ),
        (
            "Shield",
            "S",
            "Shield",
            "none",
            1000.0,
            6.0,
            Some(2),
            None,
            None,
            "PHB",
            r#"{"name":"Shield","source":"PHB","type":"S","rarity":"none","value":1000,"ac":2}"#,
        ),
        (
            "Rope, Hempen",
            "G",
            "Adventuring Gear",
            "none",
            100.0,
            10.0,
            None,
            None,
            None,
            "PHB",
            r#"{"name":"Rope, Hempen","source":"PHB","type":"G","rarity":"none","value":100}"#,
        ),
        (
            "Torch",
            "G",
            "Adventuring Gear",
            "none",
            1.0,
            1.0,
            None,
            None,
            None,
            "PHB",
            r#"{"name":"Torch","source":"PHB","type":"G","rarity":"none","value":1}"#,
        ),
        (
            "+1 Longsword",
            "M",
            "Martial Weapon",
            "uncommon",
            100000.0,
            3.0,
            None,
            Some("1d8+1 slashing"),
            Some("true"),
            "DMG",
            r#"{"name":"+1 Longsword","source":"DMG","type":"M","rarity":"uncommon","value":100000,"reqAttune":true}"#,
        ),
        (
            "Flame Tongue",
            "M",
            "Martial Weapon",
            "rare",
            500000.0,
            3.0,
            None,
            Some("1d8 + 2d6 fire"),
            Some("true"),
            "DMG",
            r#"{"name":"Flame Tongue","source":"DMG","type":"M","rarity":"rare","value":500000,"reqAttune":true}"#,
        ),
        (
            "Vorpal Sword",
            "M",
            "Martial Weapon",
            "legendary",
            2400000.0,
            3.0,
            None,
            Some("1d8 slashing"),
            Some("true"),
            "DMG",
            r#"{"name":"Vorpal Sword","source":"DMG","type":"M","rarity":"legendary","value":2400000,"reqAttune":true}"#,
        ),
        (
            "Bag of Holding",
            "W",
            "Wondrous Item",
            "uncommon",
            400000.0,
            15.0,
            None,
            None,
            None,
            "DMG",
            r#"{"name":"Bag of Holding","source":"DMG","type":"W","rarity":"uncommon","value":400000}"#,
        ),
        (
            "Cloak of Protection",
            "W",
            "Wondrous Item",
            "uncommon",
            350000.0,
            1.0,
            None,
            None,
            Some("true"),
            "DMG",
            r#"{"name":"Cloak of Protection","source":"DMG","type":"W","rarity":"uncommon","value":350000,"reqAttune":true}"#,
        ),
        (
            "Ring of Spell Storing",
            "RG",
            "Ring",
            "rare",
            2400000.0,
            0.0,
            None,
            None,
            Some("true"),
            "DMG",
            r#"{"name":"Ring of Spell Storing","source":"DMG","type":"RG","rarity":"rare","value":2400000,"reqAttune":true}"#,
        ),
    ];

    for (name, item_type, type_name, rarity, value, weight, ac, damage, req_attune, source, json) in
        items
    {
        diesel::sql_query(
            "INSERT INTO catalog_items (name, item_type, type_name, rarity, value, weight, ac, damage, requires_attunement, source, full_item_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(item_type)
        .bind::<diesel::sql_types::Text, _>(type_name)
        .bind::<diesel::sql_types::Text, _>(rarity)
        .bind::<diesel::sql_types::Double, _>(value)
        .bind::<diesel::sql_types::Double, _>(weight)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(ac)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(damage)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req_attune)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_items_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let filters = ItemFilters {
        name: None,
        item_types: None,
        rarities: None,
        sources: None,
        min_value: None,
        max_value: None,
    };
    let results = service
        .search_items(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 15, "Should return all 15 seeded items");
}

#[test]
fn test_search_items_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let filters = ItemFilters {
        name: Some("Sword".to_string()),
        item_types: None,
        rarities: None,
        sources: None,
        min_value: None,
        max_value: None,
    };
    let results = service
        .search_items(filters)
        .expect("Search should succeed");

    // Longsword, Shortsword, +1 Longsword, Flame Tongue (sword), Vorpal Sword
    assert_eq!(results.len(), 4, "Should return 4 items matching 'Sword'");
}

#[test]
fn test_search_items_by_item_type() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let filters = ItemFilters {
        name: None,
        item_types: Some(vec!["HA".to_string()]),
        rarities: None,
        sources: None,
        min_value: None,
        max_value: None,
    };
    let results = service
        .search_items(filters)
        .expect("Search should succeed");

    // Plate Armor, Chain Mail
    assert_eq!(results.len(), 2, "Should return 2 heavy armor items");
}

#[test]
fn test_search_items_by_rarity() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let filters = ItemFilters {
        name: None,
        item_types: None,
        rarities: Some(vec!["uncommon".to_string()]),
        sources: None,
        min_value: None,
        max_value: None,
    };
    let results = service
        .search_items(filters)
        .expect("Search should succeed");

    // +1 Longsword, Bag of Holding, Cloak of Protection
    assert_eq!(results.len(), 3, "Should return 3 uncommon items");
}

#[test]
fn test_search_items_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let filters = ItemFilters {
        name: None,
        item_types: None,
        rarities: None,
        sources: Some(vec!["PHB".to_string()]),
        min_value: None,
        max_value: None,
    };
    let results = service
        .search_items(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 9, "Should return 9 PHB items");
    assert!(results.iter().all(|i| i.source == "PHB"));
}

#[test]
fn test_search_items_by_value_range() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let filters = ItemFilters {
        name: None,
        item_types: None,
        rarities: None,
        sources: None,
        min_value: Some(100000.0),
        max_value: Some(500000.0),
    };
    let results = service
        .search_items(filters)
        .expect("Search should succeed");

    // +1 Longsword (100000), Plate Armor (150000), Cloak of Protection (350000), Bag of Holding (400000), Flame Tongue (500000)
    assert_eq!(
        results.len(),
        5,
        "Should return 5 items in value range 100000-500000"
    );
}

#[test]
fn test_search_items_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let filters = ItemFilters {
        name: None,
        item_types: Some(vec!["M".to_string()]),
        rarities: Some(vec!["rare".to_string(), "legendary".to_string()]),
        sources: Some(vec!["DMG".to_string()]),
        min_value: None,
        max_value: None,
    };
    let results = service
        .search_items(filters)
        .expect("Search should succeed");

    // Flame Tongue (rare), Vorpal Sword (legendary)
    assert_eq!(
        results.len(),
        2,
        "Should return 2 rare/legendary martial weapons from DMG"
    );
}

#[test]
fn test_search_items_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let filters = ItemFilters {
        name: Some("Nonexistent".to_string()),
        item_types: None,
        rarities: None,
        sources: None,
        min_value: None,
        max_value: None,
    };
    let results = service
        .search_items(filters)
        .expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_item_by_id() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    // First item in fresh DB should be ID 1
    let item = service.get_item_by_id(1).expect("Should get item");

    assert!(item.is_some());
}

#[test]
fn test_get_item_by_id_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let item = service.get_item_by_id(99999).expect("Should not error");

    assert!(item.is_none());
}

#[test]
fn test_get_item_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let item = service
        .get_item_by_name_and_source("Longsword", "PHB")
        .expect("Should get item");

    assert!(item.is_some());
    let item = item.unwrap();
    assert_eq!(item.name, "Longsword");
    assert_eq!(item.source, "PHB");
}

#[test]
fn test_get_item_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let item = service
        .get_item_by_name_and_source("Nonexistent", "PHB")
        .expect("Should not error");

    assert!(item.is_none());
}

#[test]
fn test_get_item_types() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let types = service.get_item_types().expect("Should get item types");

    // G, HA, LA, M, RG, S, W
    assert_eq!(types.len(), 7, "Should have 7 item types");
    assert!(types.contains(&"M".to_string()));
    assert!(types.contains(&"HA".to_string()));
    assert!(types.contains(&"W".to_string()));
}

#[test]
fn test_get_item_rarities() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let rarities = service.get_item_rarities().expect("Should get rarities");

    // legendary, none, rare, uncommon
    assert_eq!(rarities.len(), 4, "Should have 4 rarities");
    assert!(rarities.contains(&"none".to_string()));
    assert!(rarities.contains(&"uncommon".to_string()));
    assert!(rarities.contains(&"rare".to_string()));
    assert!(rarities.contains(&"legendary".to_string()));
}

#[test]
fn test_get_item_sources() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ItemService::new(&mut conn);

    let sources = service.get_item_sources().expect("Should get sources");

    assert_eq!(sources.len(), 2, "Should have 2 sources");
    assert!(sources.contains(&"PHB".to_string()));
    assert!(sources.contains(&"DMG".to_string()));
}
