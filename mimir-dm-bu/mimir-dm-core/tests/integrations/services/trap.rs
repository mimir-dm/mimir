//! Integration tests for TrapService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::TrapFilters;
use mimir_dm_core::services::TrapService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_trap_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_trap_data(conn: &mut SqliteConnection) {
    // Schema: name, category, trap_type, source, full_trap_json
    let traps = vec![
        // Traps
        (
            "Collapsing Roof",
            "Trap",
            "Mechanical",
            "DMG",
            r#"{"name":"Collapsing Roof","source":"DMG","trapHazType":"MECH"}"#,
        ),
        (
            "Falling Net",
            "Trap",
            "Mechanical",
            "DMG",
            r#"{"name":"Falling Net","source":"DMG","trapHazType":"MECH"}"#,
        ),
        (
            "Fire-Breathing Statue",
            "Trap",
            "Magic",
            "DMG",
            r#"{"name":"Fire-Breathing Statue","source":"DMG","trapHazType":"MAG"}"#,
        ),
        (
            "Pits",
            "Trap",
            "Mechanical",
            "DMG",
            r#"{"name":"Pits","source":"DMG","trapHazType":"MECH"}"#,
        ),
        (
            "Poison Darts",
            "Trap",
            "Mechanical",
            "DMG",
            r#"{"name":"Poison Darts","source":"DMG","trapHazType":"MECH"}"#,
        ),
        (
            "Poison Needle",
            "Trap",
            "Mechanical",
            "DMG",
            r#"{"name":"Poison Needle","source":"DMG","trapHazType":"MECH"}"#,
        ),
        (
            "Rolling Sphere",
            "Trap",
            "Mechanical",
            "DMG",
            r#"{"name":"Rolling Sphere","source":"DMG","trapHazType":"MECH"}"#,
        ),
        (
            "Sphere of Annihilation",
            "Trap",
            "Magic",
            "DMG",
            r#"{"name":"Sphere of Annihilation","source":"DMG","trapHazType":"MAG"}"#,
        ),
        // Hazards
        (
            "Brown Mold",
            "Hazard",
            "Environmental",
            "DMG",
            r#"{"name":"Brown Mold","source":"DMG","trapHazType":"ENV"}"#,
        ),
        (
            "Green Slime",
            "Hazard",
            "Environmental",
            "DMG",
            r#"{"name":"Green Slime","source":"DMG","trapHazType":"ENV"}"#,
        ),
        (
            "Yellow Mold",
            "Hazard",
            "Environmental",
            "DMG",
            r#"{"name":"Yellow Mold","source":"DMG","trapHazType":"ENV"}"#,
        ),
        (
            "Quicksand",
            "Hazard",
            "Wild",
            "DMG",
            r#"{"name":"Quicksand","source":"DMG","trapHazType":"WLD"}"#,
        ),
        (
            "Razorvine",
            "Hazard",
            "Wild",
            "DMG",
            r#"{"name":"Razorvine","source":"DMG","trapHazType":"WLD"}"#,
        ),
        (
            "Webs",
            "Hazard",
            "Wild",
            "DMG",
            r#"{"name":"Webs","source":"DMG","trapHazType":"WLD"}"#,
        ),
        (
            "Desecrated Ground",
            "Hazard",
            "Magic",
            "XGE",
            r#"{"name":"Desecrated Ground","source":"XGE","trapHazType":"MAG"}"#,
        ),
    ];

    for (name, category, trap_type, source, json) in traps {
        diesel::sql_query(
            "INSERT INTO catalog_traps (name, category, trap_type, source, full_trap_json) VALUES (?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(category)
        .bind::<diesel::sql_types::Text, _>(trap_type)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_traps_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let filters = TrapFilters {
        search: None,
        sources: None,
        categories: None,
        trap_types: None,
    };
    let results = service
        .search_traps(&mut conn, filters)
        .expect("Search should succeed");

    assert_eq!(
        results.len(),
        15,
        "Should return all 15 seeded traps/hazards"
    );
}

#[test]
fn test_search_traps_by_search() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let filters = TrapFilters {
        search: Some("Mold".to_string()),
        sources: None,
        categories: None,
        trap_types: None,
    };
    let results = service
        .search_traps(&mut conn, filters)
        .expect("Search should succeed");

    // Brown Mold, Yellow Mold
    assert_eq!(results.len(), 2, "Should return 2 mold hazards");
}

#[test]
fn test_search_traps_by_category_trap() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let filters = TrapFilters {
        search: None,
        sources: None,
        categories: Some(vec!["Trap".to_string()]),
        trap_types: None,
    };
    let results = service
        .search_traps(&mut conn, filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 8, "Should return 8 traps");
    assert!(results.iter().all(|t| t.category == "Trap"));
}

#[test]
fn test_search_traps_by_category_hazard() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let filters = TrapFilters {
        search: None,
        sources: None,
        categories: Some(vec!["Hazard".to_string()]),
        trap_types: None,
    };
    let results = service
        .search_traps(&mut conn, filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 7, "Should return 7 hazards");
    assert!(results.iter().all(|t| t.category == "Hazard"));
}

#[test]
fn test_search_traps_by_trap_type() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let filters = TrapFilters {
        search: None,
        sources: None,
        categories: None,
        trap_types: Some(vec!["Mechanical".to_string()]),
    };
    let results = service
        .search_traps(&mut conn, filters)
        .expect("Search should succeed");

    // Collapsing Roof, Falling Net, Pits, Poison Darts, Poison Needle, Rolling Sphere
    assert_eq!(results.len(), 6, "Should return 6 mechanical traps");
}

#[test]
fn test_search_traps_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let filters = TrapFilters {
        search: None,
        sources: Some(vec!["XGE".to_string()]),
        categories: None,
        trap_types: None,
    };
    let results = service
        .search_traps(&mut conn, filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should return 1 XGE hazard");
    assert_eq!(results[0].source, "XGE");
}

#[test]
fn test_search_traps_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let filters = TrapFilters {
        search: None,
        sources: Some(vec!["DMG".to_string()]),
        categories: Some(vec!["Hazard".to_string()]),
        trap_types: Some(vec!["Environmental".to_string()]),
    };
    let results = service
        .search_traps(&mut conn, filters)
        .expect("Search should succeed");

    // Brown Mold, Green Slime, Yellow Mold
    assert_eq!(
        results.len(),
        3,
        "Should return 3 DMG environmental hazards"
    );
}

#[test]
fn test_search_traps_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let filters = TrapFilters {
        search: Some("Nonexistent".to_string()),
        sources: None,
        categories: None,
        trap_types: None,
    };
    let results = service
        .search_traps(&mut conn, filters)
        .expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_trap_details() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let trap = service
        .get_trap_details(&mut conn, "Collapsing Roof".to_string(), "DMG".to_string())
        .expect("Should get trap");

    assert!(trap.is_some());
    let trap = trap.unwrap();
    assert_eq!(trap.name, "Collapsing Roof");
    assert_eq!(trap.source, "DMG");
}

#[test]
fn test_get_trap_details_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let trap = service
        .get_trap_details(&mut conn, "Nonexistent".to_string(), "DMG".to_string())
        .expect("Should not error");

    assert!(trap.is_none());
}

#[test]
fn test_get_trap_sources() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let sources = service
        .get_trap_sources(&mut conn)
        .expect("Should get sources");

    assert_eq!(sources.len(), 2, "Should have 2 sources");
    assert!(sources.contains(&"DMG".to_string()));
    assert!(sources.contains(&"XGE".to_string()));
}

#[test]
fn test_get_trap_count() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let count = service.get_trap_count(&mut conn).expect("Should get count");

    assert_eq!(count, 15, "Should have 15 traps/hazards");
}

#[test]
fn test_get_trap_types() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let types = service
        .get_trap_types(&mut conn)
        .expect("Should get trap types");

    // Environmental, Magic, Mechanical, Wild
    assert_eq!(types.len(), 4, "Should have 4 trap types");
    assert!(types.contains(&"Mechanical".to_string()));
    assert!(types.contains(&"Magic".to_string()));
    assert!(types.contains(&"Environmental".to_string()));
    assert!(types.contains(&"Wild".to_string()));
}

#[test]
fn test_get_trap_categories() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = TrapService;

    let categories = service
        .get_trap_categories(&mut conn)
        .expect("Should get categories");

    assert_eq!(categories.len(), 2, "Should have 2 categories");
    assert!(categories.contains(&"Trap".to_string()));
    assert!(categories.contains(&"Hazard".to_string()));
}
