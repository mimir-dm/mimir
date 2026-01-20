//! Integration tests for ObjectService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::ObjectFilters;
use mimir_dm_core::services::ObjectService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_object_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_object_data(conn: &mut SqliteConnection) {
    // Schema: name, object_type, size, ac, hp, source, full_object_json
    let objects = vec![
        (
            "Ballista",
            "SW",
            "L",
            "15",
            "50",
            "DMG",
            r#"{"name":"Ballista","source":"DMG","objectType":"SW","size":["L"],"ac":15,"hp":50}"#,
        ),
        (
            "Cannon",
            "SW",
            "L",
            "19",
            "75",
            "DMG",
            r#"{"name":"Cannon","source":"DMG","objectType":"SW","size":["L"],"ac":19,"hp":75}"#,
        ),
        (
            "Mangonel",
            "SW",
            "L",
            "15",
            "100",
            "DMG",
            r#"{"name":"Mangonel","source":"DMG","objectType":"SW","size":["L"],"ac":15,"hp":100}"#,
        ),
        (
            "Trebuchet",
            "SW",
            "H",
            "15",
            "150",
            "DMG",
            r#"{"name":"Trebuchet","source":"DMG","objectType":"SW","size":["H"],"ac":15,"hp":150}"#,
        ),
        (
            "Ram",
            "SW",
            "L",
            "15",
            "100",
            "DMG",
            r#"{"name":"Ram","source":"DMG","objectType":"SW","size":["L"],"ac":15,"hp":100}"#,
        ),
        (
            "Siege Tower",
            "SW",
            "G",
            "15",
            "200",
            "DMG",
            r#"{"name":"Siege Tower","source":"DMG","objectType":"SW","size":["G"],"ac":15,"hp":200}"#,
        ),
        (
            "Cauldron of Oil",
            "GS",
            "M",
            "19",
            "10",
            "DMG",
            r#"{"name":"Cauldron of Oil","source":"DMG","objectType":"GS","size":["M"],"ac":19,"hp":10}"#,
        ),
        (
            "Suspended Cauldron",
            "GS",
            "M",
            "19",
            "5",
            "DMG",
            r#"{"name":"Suspended Cauldron","source":"DMG","objectType":"GS","size":["M"],"ac":19,"hp":5}"#,
        ),
        (
            "Wooden Door",
            "U",
            "L",
            "15",
            "18",
            "PHB",
            r#"{"name":"Wooden Door","source":"PHB","objectType":"U","size":["L"],"ac":15,"hp":18}"#,
        ),
        (
            "Iron Door",
            "U",
            "L",
            "19",
            "27",
            "PHB",
            r#"{"name":"Iron Door","source":"PHB","objectType":"U","size":["L"],"ac":19,"hp":27}"#,
        ),
        (
            "Stone Wall",
            "U",
            "H",
            "17",
            "50",
            "PHB",
            r#"{"name":"Stone Wall","source":"PHB","objectType":"U","size":["H"],"ac":17,"hp":50}"#,
        ),
        (
            "Wooden Chest",
            "U",
            "M",
            "15",
            "10",
            "PHB",
            r#"{"name":"Wooden Chest","source":"PHB","objectType":"U","size":["M"],"ac":15,"hp":10}"#,
        ),
    ];

    for (name, object_type, size, ac, hp, source, json) in objects {
        diesel::sql_query(
            "INSERT INTO catalog_objects (name, object_type, size, ac, hp, source, full_object_json) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(object_type)
        .bind::<diesel::sql_types::Text, _>(size)
        .bind::<diesel::sql_types::Text, _>(ac)
        .bind::<diesel::sql_types::Text, _>(hp)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_objects_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ObjectFilters {
        search_pattern: None,
        sources: None,
        object_types: None,
        sizes: None,
    };
    let results = ObjectService::search_objects(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 12, "Should return all 12 seeded objects");
}

#[test]
fn test_search_objects_by_search_pattern() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ObjectFilters {
        search_pattern: Some("Door".to_string()),
        sources: None,
        object_types: None,
        sizes: None,
    };
    let results = ObjectService::search_objects(&mut conn, filters).expect("Search should succeed");

    // Wooden Door, Iron Door
    assert_eq!(results.len(), 2, "Should return 2 door objects");
}

#[test]
fn test_search_objects_by_object_type() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ObjectFilters {
        search_pattern: None,
        sources: None,
        object_types: Some(vec!["SW".to_string()]),
        sizes: None,
    };
    let results = ObjectService::search_objects(&mut conn, filters).expect("Search should succeed");

    // Ballista, Cannon, Mangonel, Trebuchet, Ram, Siege Tower
    assert_eq!(results.len(), 6, "Should return 6 siege weapons");
}

#[test]
fn test_search_objects_by_size() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ObjectFilters {
        search_pattern: None,
        sources: None,
        object_types: None,
        sizes: Some(vec!["M".to_string()]),
    };
    let results = ObjectService::search_objects(&mut conn, filters).expect("Search should succeed");

    // Cauldron of Oil, Suspended Cauldron, Wooden Chest
    assert_eq!(results.len(), 3, "Should return 3 Medium objects");
}

#[test]
fn test_search_objects_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ObjectFilters {
        search_pattern: None,
        sources: Some(vec!["DMG".to_string()]),
        object_types: None,
        sizes: None,
    };
    let results = ObjectService::search_objects(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 8, "Should return 8 DMG objects");
    assert!(results.iter().all(|o| o.source == "DMG"));
}

#[test]
fn test_search_objects_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ObjectFilters {
        search_pattern: None,
        sources: Some(vec!["DMG".to_string()]),
        object_types: Some(vec!["SW".to_string()]),
        sizes: Some(vec!["L".to_string()]),
    };
    let results = ObjectService::search_objects(&mut conn, filters).expect("Search should succeed");

    // Ballista, Cannon, Mangonel, Ram (all DMG, SW, Large)
    assert_eq!(results.len(), 4, "Should return 4 DMG Large siege weapons");
}

#[test]
fn test_search_objects_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ObjectFilters {
        search_pattern: Some("Nonexistent".to_string()),
        sources: None,
        object_types: None,
        sizes: None,
    };
    let results = ObjectService::search_objects(&mut conn, filters).expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_object_details() {
    let (mut conn, _temp_dir) = setup_test_db();

    let object =
        ObjectService::get_object_details(&mut conn, "Ballista", "DMG").expect("Should get object");

    assert!(object.is_some());
    let json = object.unwrap();
    assert!(json.contains("Ballista"));
}

#[test]
fn test_get_object_details_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let object = ObjectService::get_object_details(&mut conn, "Nonexistent", "DMG")
        .expect("Should not error");

    assert!(object.is_none());
}

#[test]
fn test_get_object_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = ObjectService::get_object_sources(&mut conn).expect("Should get sources");

    assert_eq!(sources.len(), 2, "Should have 2 sources");
    assert!(sources.contains(&"DMG".to_string()));
    assert!(sources.contains(&"PHB".to_string()));
}

#[test]
fn test_get_object_count() {
    let (mut conn, _temp_dir) = setup_test_db();

    let count = ObjectService::get_object_count(&mut conn).expect("Should get count");

    assert_eq!(count, 12, "Should have 12 objects");
}

#[test]
fn test_get_object_types() {
    let (mut conn, _temp_dir) = setup_test_db();

    let types = ObjectService::get_object_types(&mut conn).expect("Should get object types");

    // GS, SW, U
    assert_eq!(types.len(), 3, "Should have 3 object types");
    assert!(types.contains(&"SW".to_string()));
    assert!(types.contains(&"GS".to_string()));
    assert!(types.contains(&"U".to_string()));
}

#[test]
fn test_get_object_sizes() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sizes = ObjectService::get_object_sizes(&mut conn).expect("Should get sizes");

    // G, H, L, M
    assert_eq!(sizes.len(), 4, "Should have 4 sizes");
    assert!(sizes.contains(&"L".to_string()));
    assert!(sizes.contains(&"M".to_string()));
    assert!(sizes.contains(&"H".to_string()));
    assert!(sizes.contains(&"G".to_string()));
}
