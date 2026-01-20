//! Integration tests for TableService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::table::TableFilters;
use mimir_dm_core::services::TableService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_table_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_table_data(conn: &mut SqliteConnection) {
    // Schema: name, caption, category, source, page, columns_count, rows_count, full_table_json
    let tables = vec![
        (
            "Wild Magic Surge",
            Some("Wild Magic Surge Effects"),
            "Wild Magic",
            "PHB",
            Some(104),
            2,
            50,
            r#"{"name":"Wild Magic Surge","source":"PHB","page":104,"caption":"Wild Magic Surge Effects","colLabels":["d100","Effect"],"rows":[["01-02","Roll on this table at the start of each of your turns for the next minute."]]}"#,
        ),
        (
            "Trinkets",
            Some("Trinkets Table"),
            "Trinkets",
            "PHB",
            Some(160),
            2,
            100,
            r#"{"name":"Trinkets","source":"PHB","page":160,"caption":"Trinkets Table","colLabels":["d100","Trinket"],"rows":[["01","A mummified goblin hand"]]}"#,
        ),
        (
            "Individual Treasure: Challenge 0-4",
            Some("Individual Treasure"),
            "Treasure",
            "DMG",
            Some(136),
            2,
            6,
            r#"{"name":"Individual Treasure: Challenge 0-4","source":"DMG","page":136,"caption":"Individual Treasure","colLabels":["d100","Treasure"],"rows":[["01-30","5d6 cp"]]}"#,
        ),
        (
            "Treasure Hoard: Challenge 0-4",
            Some("Treasure Hoard"),
            "Treasure",
            "DMG",
            Some(137),
            4,
            10,
            r#"{"name":"Treasure Hoard: Challenge 0-4","source":"DMG","page":137,"caption":"Treasure Hoard","colLabels":["d100","CP","SP","GP"],"rows":[["01-06","6d6 x 100","3d6 x 100","2d6 x 10"]]}"#,
        ),
        (
            "Random Encounter: Forest",
            Some("Forest Encounters"),
            "Encounters",
            "DMG",
            Some(87),
            2,
            20,
            r#"{"name":"Random Encounter: Forest","source":"DMG","page":87,"caption":"Forest Encounters","colLabels":["d100","Encounter"],"rows":[["01-05","1d4 + 1 pixies"]]}"#,
        ),
        (
            "Random Encounter: Desert",
            Some("Desert Encounters"),
            "Encounters",
            "DMG",
            Some(95),
            2,
            20,
            r#"{"name":"Random Encounter: Desert","source":"DMG","page":95,"caption":"Desert Encounters","colLabels":["d100","Encounter"],"rows":[["01-05","1d4 scorpions"]]}"#,
        ),
        (
            "Madness Effects",
            Some("Short-Term Madness"),
            "Madness",
            "DMG",
            Some(259),
            2,
            10,
            r#"{"name":"Madness Effects","source":"DMG","page":259,"caption":"Short-Term Madness","colLabels":["d100","Effect"],"rows":[["01-20","The character retreats into their mind"]]}"#,
        ),
        (
            "NPC Personality Traits",
            Some("NPC Traits"),
            "NPCs",
            "DMG",
            Some(89),
            2,
            12,
            r#"{"name":"NPC Personality Traits","source":"DMG","page":89,"caption":"NPC Traits","colLabels":["d12","Trait"],"rows":[["1","Argumentative"]]}"#,
        ),
        (
            "Lingering Injuries",
            Some("Lingering Injuries Table"),
            "Combat",
            "DMG",
            Some(272),
            2,
            12,
            r#"{"name":"Lingering Injuries","source":"DMG","page":272,"caption":"Lingering Injuries Table","colLabels":["d20","Injury"],"rows":[["1","Lose an Eye"]]}"#,
        ),
        (
            "System Shock",
            Some("System Shock Table"),
            "Combat",
            "DMG",
            Some(273),
            2,
            10,
            r#"{"name":"System Shock","source":"DMG","page":273,"caption":"System Shock Table","colLabels":["d10","Effect"],"rows":[["1","Creature drops to 0 hit points"]]}"#,
        ),
        (
            "Quest Hooks",
            Some("Adventure Quest Hooks"),
            "Adventures",
            "XGE",
            Some(85),
            2,
            20,
            r#"{"name":"Quest Hooks","source":"XGE","page":85,"caption":"Adventure Quest Hooks","colLabels":["d20","Hook"],"rows":[["1","A dying messenger delivers a cryptic message"]]}"#,
        ),
        (
            "Magic Item Tables",
            Some("Random Magic Items"),
            "Magic Items",
            "DMG",
            Some(144),
            2,
            100,
            r#"{"name":"Magic Item Tables","source":"DMG","page":144,"caption":"Random Magic Items","colLabels":["d100","Item"],"rows":[["01-50","Potion of healing"]]}"#,
        ),
    ];

    for (name, caption, category, source, page, columns, rows, json) in tables {
        diesel::sql_query(
            "INSERT INTO catalog_tables (name, caption, category, source, page, columns_count, rows_count, full_table_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(caption)
        .bind::<diesel::sql_types::Text, _>(category)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(page)
        .bind::<diesel::sql_types::Integer, _>(columns)
        .bind::<diesel::sql_types::Integer, _>(rows)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_tables_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = TableFilters::default();
    let results = TableService::new(&mut conn)
        .search_tables(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 12, "Should return all 12 seeded tables");
}

#[test]
fn test_search_tables_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = TableFilters {
        name: Some("Wild Magic".to_string()),
        categories: None,
        sources: None,
    };
    let results = TableService::new(&mut conn)
        .search_tables(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should return 1 table matching 'Wild Magic'");
    assert_eq!(results[0].name, "Wild Magic Surge");
}

#[test]
fn test_search_tables_by_category() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = TableFilters {
        name: None,
        categories: Some(vec!["Treasure".to_string()]),
        sources: None,
    };
    let results = TableService::new(&mut conn)
        .search_tables(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should return 2 Treasure tables");
    assert!(results.iter().all(|t| t.category == "Treasure"));
}

#[test]
fn test_search_tables_by_multiple_categories() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = TableFilters {
        name: None,
        categories: Some(vec!["Treasure".to_string(), "Encounters".to_string()]),
        sources: None,
    };
    let results = TableService::new(&mut conn)
        .search_tables(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 4, "Should return 4 tables (2 Treasure + 2 Encounters)");
    assert!(results.iter().all(|t| t.category == "Treasure" || t.category == "Encounters"));
}

#[test]
fn test_search_tables_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = TableFilters {
        name: None,
        categories: None,
        sources: Some(vec!["PHB".to_string()]),
    };
    let results = TableService::new(&mut conn)
        .search_tables(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should return 2 PHB tables");
    assert!(results.iter().all(|t| t.source == "PHB"));
}

#[test]
fn test_search_tables_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = TableFilters {
        name: Some("Encounter".to_string()),
        categories: Some(vec!["Encounters".to_string()]),
        sources: Some(vec!["DMG".to_string()]),
    };
    let results = TableService::new(&mut conn)
        .search_tables(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should return 2 DMG encounter tables");
    assert!(results.iter().all(|t| t.source == "DMG" && t.category == "Encounters"));
}

#[test]
fn test_search_tables_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = TableFilters {
        name: Some("Nonexistent Table".to_string()),
        categories: None,
        sources: None,
    };
    let results = TableService::new(&mut conn)
        .search_tables(filters)
        .expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_table_by_id() {
    let (mut conn, _temp_dir) = setup_test_db();

    // Get first table (ID 1 in fresh DB)
    let table = TableService::new(&mut conn)
        .get_table_by_id(1)
        .expect("Should get table");

    assert!(table.is_some(), "Should find table with ID 1");
    let table = table.unwrap();
    assert_eq!(table.name, "Wild Magic Surge");
    assert_eq!(table.source, "PHB");
}

#[test]
fn test_get_table_by_id_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let table = TableService::new(&mut conn)
        .get_table_by_id(99999)
        .expect("Should not error");

    assert!(table.is_none(), "Should return None for nonexistent ID");
}

#[test]
fn test_get_table_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let table = TableService::new(&mut conn)
        .get_table_by_name_and_source("Trinkets", "PHB")
        .expect("Should get table");

    assert!(table.is_some(), "Should find table by name and source");
    let table = table.unwrap();
    assert_eq!(table.name, "Trinkets");
    assert_eq!(table.source, "PHB");
}

#[test]
fn test_get_table_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let table = TableService::new(&mut conn)
        .get_table_by_name_and_source("Nonexistent", "PHB")
        .expect("Should not error");

    assert!(table.is_none(), "Should return None for nonexistent table");
}

#[test]
fn test_get_table_by_name_and_source_wrong_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    // Trinkets exists in PHB, not DMG
    let table = TableService::new(&mut conn)
        .get_table_by_name_and_source("Trinkets", "DMG")
        .expect("Should not error");

    assert!(table.is_none(), "Should return None for wrong source");
}

#[test]
fn test_get_table_categories() {
    let (mut conn, _temp_dir) = setup_test_db();

    let categories = TableService::new(&mut conn)
        .get_table_categories()
        .expect("Should get categories");

    // We seeded: Wild Magic, Trinkets, Treasure, Encounters, Madness, NPCs, Combat, Adventures, Magic Items
    assert_eq!(categories.len(), 9, "Should have 9 unique categories");
    assert!(categories.contains(&"Wild Magic".to_string()));
    assert!(categories.contains(&"Treasure".to_string()));
    assert!(categories.contains(&"Encounters".to_string()));
    assert!(categories.contains(&"Combat".to_string()));
}

#[test]
fn test_get_table_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = TableService::new(&mut conn)
        .get_table_sources()
        .expect("Should get sources");

    assert_eq!(sources.len(), 3, "Should have 3 unique sources");
    assert!(sources.contains(&"PHB".to_string()));
    assert!(sources.contains(&"DMG".to_string()));
    assert!(sources.contains(&"XGE".to_string()));
}

#[test]
fn test_table_summary_fields() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = TableFilters {
        name: Some("Wild Magic Surge".to_string()),
        categories: None,
        sources: None,
    };
    let results = TableService::new(&mut conn)
        .search_tables(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 1);
    let summary = &results[0];

    assert_eq!(summary.name, "Wild Magic Surge");
    assert_eq!(summary.source, "PHB");
    assert_eq!(summary.caption, "Wild Magic Surge Effects");
    assert_eq!(summary.category, "Wild Magic");
    assert_eq!(summary.columns, 2);
    assert_eq!(summary.rows, 50);
}
