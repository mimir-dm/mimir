//! Integration tests for RaceService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::RaceFilters;
use mimir_dm_core::services::RaceService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_race_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_race_data(conn: &mut SqliteConnection) {
    let races = vec![
        (
            "Human",
            "PHB",
            "M",
            30,
            "+1 to all",
            r#"{"name":"Human","source":"PHB","size":["M"],"speed":30,"ability":[{"str":1,"dex":1,"con":1,"int":1,"wis":1,"cha":1}]}"#,
        ),
        (
            "Elf",
            "PHB",
            "M",
            30,
            "+2 Dex",
            r#"{"name":"Elf","source":"PHB","size":["M"],"speed":30,"darkvision":60,"ability":[{"dex":2}]}"#,
        ),
        (
            "Dwarf",
            "PHB",
            "M",
            25,
            "+2 Con",
            r#"{"name":"Dwarf","source":"PHB","size":["M"],"speed":25,"darkvision":60,"ability":[{"con":2}]}"#,
        ),
        (
            "Halfling",
            "PHB",
            "S",
            25,
            "+2 Dex",
            r#"{"name":"Halfling","source":"PHB","size":["S"],"speed":25,"ability":[{"dex":2}]}"#,
        ),
        (
            "Dragonborn",
            "PHB",
            "M",
            30,
            "+2 Str, +1 Cha",
            r#"{"name":"Dragonborn","source":"PHB","size":["M"],"speed":30,"ability":[{"str":2,"cha":1}]}"#,
        ),
        (
            "Tiefling",
            "PHB",
            "M",
            30,
            "+1 Int, +2 Cha",
            r#"{"name":"Tiefling","source":"PHB","size":["M"],"speed":30,"darkvision":60,"ability":[{"int":1,"cha":2}]}"#,
        ),
        (
            "Aarakocra",
            "EEPC",
            "M",
            25,
            "+2 Dex, +1 Wis",
            r#"{"name":"Aarakocra","source":"EEPC","size":["M"],"speed":{"walk":25,"fly":50},"ability":[{"dex":2,"wis":1}]}"#,
        ),
        (
            "Gnome",
            "PHB",
            "S",
            25,
            "+2 Int",
            r#"{"name":"Gnome","source":"PHB","size":["S"],"speed":25,"darkvision":60,"ability":[{"int":2}]}"#,
        ),
        (
            "Fairy",
            "WBtW",
            "S",
            30,
            "+2 to any",
            r#"{"name":"Fairy","source":"WBtW","size":["S"],"speed":{"walk":30,"fly":30},"ability":[{"choose":{"from":["str","dex","con","int","wis","cha"],"count":2}}]}"#,
        ),
        (
            "Half-Orc",
            "PHB",
            "M",
            30,
            "+2 Str, +1 Con",
            r#"{"name":"Half-Orc","source":"PHB","size":["M"],"speed":30,"darkvision":60,"ability":[{"str":2,"con":1}]}"#,
        ),
    ];

    for (name, source, size, speed, ability_bonuses, json) in races {
        diesel::sql_query(
            "INSERT INTO catalog_races (name, source, size, speed, ability_bonuses, full_race_json) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(size)
        .bind::<diesel::sql_types::Integer, _>(speed)
        .bind::<diesel::sql_types::Text, _>(ability_bonuses)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_races_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RaceFilters {
        search_pattern: None,
        sources: None,
        sizes: None,
        has_darkvision: None,
        has_flight: None,
    };
    let results = RaceService::search_races(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 10, "Should return all 10 seeded races");
}

#[test]
fn test_search_races_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RaceFilters {
        search_pattern: Some("Elf".to_string()),
        sources: None,
        sizes: None,
        has_darkvision: None,
        has_flight: None,
    };
    let results = RaceService::search_races(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should return 1 race matching 'Elf'");
    assert_eq!(results[0].name, "Elf");
}

#[test]
fn test_search_races_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RaceFilters {
        search_pattern: None,
        sources: Some(vec!["PHB".to_string()]),
        sizes: None,
        has_darkvision: None,
        has_flight: None,
    };
    let results = RaceService::search_races(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 8, "Should return 8 PHB races");
    assert!(results.iter().all(|r| r.source == "PHB"));
}

#[test]
fn test_search_races_by_size() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RaceFilters {
        search_pattern: None,
        sources: None,
        sizes: Some(vec!["S".to_string()]),
        has_darkvision: None,
        has_flight: None,
    };
    let results = RaceService::search_races(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 3, "Should return 3 Small races");
    assert!(results.iter().all(|r| r.size == "S"));
}

#[test]
fn test_search_races_with_darkvision() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RaceFilters {
        search_pattern: None,
        sources: None,
        sizes: None,
        has_darkvision: Some(true),
        has_flight: None,
    };
    let results = RaceService::search_races(&mut conn, filters).expect("Search should succeed");

    assert!(results.len() >= 5, "Should return races with darkvision");
}

#[test]
fn test_search_races_without_darkvision() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RaceFilters {
        search_pattern: None,
        sources: None,
        sizes: None,
        has_darkvision: Some(false),
        has_flight: None,
    };
    let results = RaceService::search_races(&mut conn, filters).expect("Search should succeed");

    assert!(results.len() >= 3, "Should return races without darkvision");
}

#[test]
fn test_search_races_with_flight() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RaceFilters {
        search_pattern: None,
        sources: None,
        sizes: None,
        has_darkvision: None,
        has_flight: Some(true),
    };
    let results = RaceService::search_races(&mut conn, filters).expect("Search should succeed");

    assert_eq!(
        results.len(),
        2,
        "Should return 2 flying races (Aarakocra, Fairy)"
    );
}

#[test]
fn test_search_races_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RaceFilters {
        search_pattern: None,
        sources: Some(vec!["PHB".to_string()]),
        sizes: Some(vec!["M".to_string()]),
        has_darkvision: Some(true),
        has_flight: None,
    };
    let results = RaceService::search_races(&mut conn, filters).expect("Search should succeed");

    // PHB, Medium, with darkvision
    assert!(
        results.len() >= 3,
        "Should return PHB Medium races with darkvision"
    );
}

#[test]
fn test_search_races_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = RaceFilters {
        search_pattern: Some("Nonexistent".to_string()),
        sources: None,
        sizes: None,
        has_darkvision: None,
        has_flight: None,
    };
    let results = RaceService::search_races(&mut conn, filters).expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_race_details() {
    let (mut conn, _temp_dir) = setup_test_db();

    let race_json =
        RaceService::get_race_details(&mut conn, "Human", "PHB").expect("Should get race details");

    assert!(race_json.is_some());
    let json = race_json.unwrap();
    assert!(json.contains("Human"));
}

#[test]
fn test_get_race_details_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let race_json =
        RaceService::get_race_details(&mut conn, "Nonexistent", "PHB").expect("Should not error");

    assert!(race_json.is_none());
}

#[test]
fn test_get_race_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = RaceService::get_race_sources(&mut conn).expect("Should get sources");

    assert!(sources.len() >= 3, "Should have multiple sources");
    assert!(sources.contains(&"PHB".to_string()));
}

#[test]
fn test_get_race_sizes() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sizes = RaceService::get_race_sizes(&mut conn).expect("Should get sizes");

    assert_eq!(sizes.len(), 2, "Should have M and S sizes");
    assert!(sizes.contains(&"M".to_string()));
    assert!(sizes.contains(&"S".to_string()));
}

#[test]
fn test_get_race_count() {
    let (mut conn, _temp_dir) = setup_test_db();

    let count = RaceService::get_race_count(&mut conn).expect("Should get count");

    assert_eq!(count, 10, "Should have 10 races");
}
