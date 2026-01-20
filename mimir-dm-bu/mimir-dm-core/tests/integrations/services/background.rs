//! Integration tests for BackgroundService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::BackgroundFilters;
use mimir_dm_core::services::BackgroundService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_background_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_background_data(conn: &mut SqliteConnection) {
    // Note: tools column uses "None" string for backgrounds without tool proficiencies
    // This matches the filter logic in BackgroundService
    let backgrounds = vec![
        (
            "Acolyte",
            "PHB",
            "Insight, Religion",
            "None",
            "Shelter of the Faithful",
            r#"{"name":"Acolyte","source":"PHB","skillProficiencies":[{"insight":true,"religion":true}],"feature":{"name":"Shelter of the Faithful"}}"#,
        ),
        (
            "Criminal",
            "PHB",
            "Deception, Stealth",
            "Thieves' tools",
            "Criminal Contact",
            r#"{"name":"Criminal","source":"PHB","skillProficiencies":[{"deception":true,"stealth":true}],"toolProficiencies":[{"thieves' tools":true}],"feature":{"name":"Criminal Contact"}}"#,
        ),
        (
            "Folk Hero",
            "PHB",
            "Animal Handling, Survival",
            "Vehicles (land)",
            "Rustic Hospitality",
            r#"{"name":"Folk Hero","source":"PHB","skillProficiencies":[{"animal handling":true,"survival":true}],"toolProficiencies":[{"vehicles (land)":true}],"feature":{"name":"Rustic Hospitality"}}"#,
        ),
        (
            "Noble",
            "PHB",
            "History, Persuasion",
            "Gaming set",
            "Position of Privilege",
            r#"{"name":"Noble","source":"PHB","skillProficiencies":[{"history":true,"persuasion":true}],"toolProficiencies":[{"gaming set":true}],"feature":{"name":"Position of Privilege"}}"#,
        ),
        (
            "Sage",
            "PHB",
            "Arcana, History",
            "None",
            "Researcher",
            r#"{"name":"Sage","source":"PHB","skillProficiencies":[{"arcana":true,"history":true}],"feature":{"name":"Researcher"}}"#,
        ),
        (
            "Soldier",
            "PHB",
            "Athletics, Intimidation",
            "Vehicles (land)",
            "Military Rank",
            r#"{"name":"Soldier","source":"PHB","skillProficiencies":[{"athletics":true,"intimidation":true}],"toolProficiencies":[{"vehicles (land)":true}],"feature":{"name":"Military Rank"}}"#,
        ),
        (
            "Urchin",
            "PHB",
            "Sleight of Hand, Stealth",
            "Thieves' tools, Disguise kit",
            "City Secrets",
            r#"{"name":"Urchin","source":"PHB","skillProficiencies":[{"sleight of hand":true,"stealth":true}],"toolProficiencies":[{"thieves' tools":true,"disguise kit":true}],"feature":{"name":"City Secrets"}}"#,
        ),
        (
            "Haunted One",
            "VRGR",
            "Arcana, Investigation",
            "None",
            "Heart of Darkness",
            r#"{"name":"Haunted One","source":"VRGR","skillProficiencies":[{"arcana":true,"investigation":true}],"feature":{"name":"Heart of Darkness"}}"#,
        ),
        (
            "Far Traveler",
            "SCAG",
            "Insight, Perception",
            "Musical instrument",
            "All Eyes on You",
            r#"{"name":"Far Traveler","source":"SCAG","skillProficiencies":[{"insight":true,"perception":true}],"toolProficiencies":[{"musical instrument":true}],"feature":{"name":"All Eyes on You"}}"#,
        ),
        (
            "Outlander",
            "PHB",
            "Athletics, Survival",
            "Musical instrument",
            "Wanderer",
            r#"{"name":"Outlander","source":"PHB","skillProficiencies":[{"athletics":true,"survival":true}],"toolProficiencies":[{"musical instrument":true}],"feature":{"name":"Wanderer"}}"#,
        ),
    ];

    for (name, source, skills, tools, feature, json) in backgrounds {
        diesel::sql_query(
            "INSERT INTO catalog_backgrounds (name, source, skills, tools, feature, languages, full_background_json) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(skills)
        .bind::<diesel::sql_types::Text, _>(tools)
        .bind::<diesel::sql_types::Text, _>(feature)
        .bind::<diesel::sql_types::Text, _>("")
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_backgrounds_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = BackgroundFilters::default();
    let mut service = BackgroundService::new(&mut conn);
    let results = service
        .search_backgrounds(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 10, "Should return all 10 seeded backgrounds");
}

#[test]
fn test_search_backgrounds_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = BackgroundFilters {
        search_pattern: Some("Sage".to_string()),
        sources: None,
        has_tools: None,
    };
    let mut service = BackgroundService::new(&mut conn);
    let results = service
        .search_backgrounds(filters)
        .expect("Search should succeed");

    assert_eq!(
        results.len(),
        1,
        "Should return 1 background matching 'Sage'"
    );
    assert_eq!(results[0].name, "Sage");
}

#[test]
fn test_search_backgrounds_by_skill() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = BackgroundFilters {
        search_pattern: Some("stealth".to_string()),
        sources: None,
        has_tools: None,
    };
    let mut service = BackgroundService::new(&mut conn);
    let results = service
        .search_backgrounds(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should return 2 backgrounds with Stealth");
}

#[test]
fn test_search_backgrounds_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = BackgroundFilters {
        search_pattern: None,
        sources: Some(vec!["PHB".to_string()]),
        has_tools: None,
    };
    let mut service = BackgroundService::new(&mut conn);
    let results = service
        .search_backgrounds(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 8, "Should return 8 PHB backgrounds");
    assert!(results.iter().all(|b| b.source == "PHB"));
}

#[test]
fn test_search_backgrounds_with_tools() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = BackgroundFilters {
        search_pattern: None,
        sources: None,
        has_tools: Some(true),
    };
    let mut service = BackgroundService::new(&mut conn);
    let results = service
        .search_backgrounds(filters)
        .expect("Search should succeed");

    // Backgrounds with tools: Criminal, Folk Hero, Noble, Soldier, Urchin, Far Traveler, Outlander
    assert!(results.len() >= 7, "Should return backgrounds with tools");
}

#[test]
fn test_search_backgrounds_without_tools() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = BackgroundFilters {
        search_pattern: None,
        sources: None,
        has_tools: Some(false),
    };
    let mut service = BackgroundService::new(&mut conn);
    let results = service
        .search_backgrounds(filters)
        .expect("Search should succeed");

    // Backgrounds without tools: Acolyte, Sage, Haunted One
    assert!(
        results.len() >= 3,
        "Should return backgrounds without tools"
    );
}

#[test]
fn test_search_backgrounds_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = BackgroundFilters {
        search_pattern: None,
        sources: Some(vec!["PHB".to_string()]),
        has_tools: Some(true),
    };
    let mut service = BackgroundService::new(&mut conn);
    let results = service
        .search_backgrounds(filters)
        .expect("Search should succeed");

    // PHB backgrounds with tools
    assert!(
        results.len() >= 5,
        "Should return PHB backgrounds with tools"
    );
    assert!(results.iter().all(|b| b.source == "PHB"));
}

#[test]
fn test_search_backgrounds_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = BackgroundFilters {
        search_pattern: Some("Nonexistent".to_string()),
        sources: None,
        has_tools: None,
    };
    let mut service = BackgroundService::new(&mut conn);
    let results = service
        .search_backgrounds(filters)
        .expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_background_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let mut service = BackgroundService::new(&mut conn);
    let background = service
        .get_background_by_name_and_source("Sage", "PHB")
        .expect("Should get background")
        .expect("Background should exist");

    assert_eq!(background.name, "Sage");
    assert_eq!(background.source, "PHB");
}

#[test]
fn test_get_background_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let mut service = BackgroundService::new(&mut conn);
    let result = service
        .get_background_by_name_and_source("Nonexistent", "PHB")
        .expect("Query should succeed");

    assert!(
        result.is_none(),
        "Should return None for nonexistent background"
    );
}

#[test]
fn test_get_background_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let mut service = BackgroundService::new(&mut conn);
    let sources = service
        .get_background_sources()
        .expect("Should get sources");

    assert!(sources.len() >= 3, "Should have multiple sources");
    assert!(sources.contains(&"PHB".to_string()));
}

#[test]
fn test_get_background_count() {
    let (mut conn, _temp_dir) = setup_test_db();

    let mut service = BackgroundService::new(&mut conn);
    let count = service.get_background_count().expect("Should get count");

    assert_eq!(count, 10, "Should have 10 backgrounds");
}
