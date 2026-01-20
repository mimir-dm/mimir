//! Integration tests for ConditionService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::ConditionFilters;
use mimir_dm_core::services::ConditionService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_condition_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_condition_data(conn: &mut SqliteConnection) {
    // Schema: name, item_type, description, source, full_condition_json
    let conditions = vec![
        ("Blinded", "Condition", "A blinded creature can't see and automatically fails any ability check that requires sight.", "PHB", r#"{"name":"Blinded","source":"PHB","entries":["A blinded creature can't see and automatically fails any ability check that requires sight."]}"#),
        ("Charmed", "Condition", "A charmed creature can't attack the charmer.", "PHB", r#"{"name":"Charmed","source":"PHB","entries":["A charmed creature can't attack the charmer."]}"#),
        ("Deafened", "Condition", "A deafened creature can't hear and automatically fails any ability check that requires hearing.", "PHB", r#"{"name":"Deafened","source":"PHB","entries":["A deafened creature can't hear and automatically fails any ability check that requires hearing."]}"#),
        ("Frightened", "Condition", "A frightened creature has disadvantage on ability checks and attack rolls while the source of its fear is within line of sight.", "PHB", r#"{"name":"Frightened","source":"PHB","entries":["A frightened creature has disadvantage on ability checks and attack rolls while the source of its fear is within line of sight."]}"#),
        ("Grappled", "Condition", "A grappled creature's speed becomes 0.", "PHB", r#"{"name":"Grappled","source":"PHB","entries":["A grappled creature's speed becomes 0."]}"#),
        ("Incapacitated", "Condition", "An incapacitated creature can't take actions or reactions.", "PHB", r#"{"name":"Incapacitated","source":"PHB","entries":["An incapacitated creature can't take actions or reactions."]}"#),
        ("Invisible", "Condition", "An invisible creature is impossible to see without the aid of magic or a special sense.", "PHB", r#"{"name":"Invisible","source":"PHB","entries":["An invisible creature is impossible to see without the aid of magic or a special sense."]}"#),
        ("Paralyzed", "Condition", "A paralyzed creature is incapacitated and can't move or speak.", "PHB", r#"{"name":"Paralyzed","source":"PHB","entries":["A paralyzed creature is incapacitated and can't move or speak."]}"#),
        ("Petrified", "Condition", "A petrified creature is transformed into a solid inanimate substance.", "PHB", r#"{"name":"Petrified","source":"PHB","entries":["A petrified creature is transformed into a solid inanimate substance."]}"#),
        ("Poisoned", "Condition", "A poisoned creature has disadvantage on attack rolls and ability checks.", "PHB", r#"{"name":"Poisoned","source":"PHB","entries":["A poisoned creature has disadvantage on attack rolls and ability checks."]}"#),
        ("Prone", "Condition", "A prone creature's only movement option is to crawl.", "PHB", r#"{"name":"Prone","source":"PHB","entries":["A prone creature's only movement option is to crawl."]}"#),
        ("Restrained", "Condition", "A restrained creature's speed becomes 0.", "PHB", r#"{"name":"Restrained","source":"PHB","entries":["A restrained creature's speed becomes 0."]}"#),
        ("Stunned", "Condition", "A stunned creature is incapacitated, can't move, and can speak only falteringly.", "PHB", r#"{"name":"Stunned","source":"PHB","entries":["A stunned creature is incapacitated, can't move, and can speak only falteringly."]}"#),
        ("Unconscious", "Condition", "An unconscious creature is incapacitated, can't move or speak, and is unaware of its surroundings.", "PHB", r#"{"name":"Unconscious","source":"PHB","entries":["An unconscious creature is incapacitated, can't move or speak, and is unaware of its surroundings."]}"#),
        ("Exhaustion", "Condition", "Exhaustion is measured in six levels.", "PHB", r#"{"name":"Exhaustion","source":"PHB","entries":["Exhaustion is measured in six levels."]}"#),
        ("Cackle Fever", "Disease", "This disease targets humanoids, although gnomes are strangely immune.", "DMG", r#"{"name":"Cackle Fever","source":"DMG","entries":["This disease targets humanoids, although gnomes are strangely immune."]}"#),
        ("Sewer Plague", "Disease", "Sewer plague is a generic term for a broad category of illnesses.", "DMG", r#"{"name":"Sewer Plague","source":"DMG","entries":["Sewer plague is a generic term for a broad category of illnesses."]}"#),
        ("Sight Rot", "Disease", "This painful infection causes bleeding from the eyes.", "DMG", r#"{"name":"Sight Rot","source":"DMG","entries":["This painful infection causes bleeding from the eyes."]}"#),
    ];

    for (name, item_type, description, source, json) in conditions {
        diesel::sql_query(
            "INSERT INTO catalog_conditions (name, item_type, description, source, full_condition_json) VALUES (?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(item_type)
        .bind::<diesel::sql_types::Text, _>(description)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_conditions_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ConditionFilters {
        name: None,
        item_types: None,
        sources: None,
        search: None,
    };
    let results =
        ConditionService::new(&mut conn).search_conditions(filters).expect("Search should succeed");

    assert_eq!(
        results.len(),
        18,
        "Should return all 18 seeded conditions/diseases"
    );
}

#[test]
fn test_search_conditions_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ConditionFilters {
        name: Some("Blind".to_string()),
        item_types: None,
        sources: None,
        search: None,
    };
    let results =
        ConditionService::new(&mut conn).search_conditions(filters).expect("Search should succeed");

    assert_eq!(
        results.len(),
        1,
        "Should return 1 condition matching 'Blind'"
    );
    assert_eq!(results[0].name, "Blinded");
}

#[test]
fn test_search_conditions_by_item_type_condition() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ConditionFilters {
        name: None,
        item_types: Some(vec!["Condition".to_string()]),
        sources: None,
        search: None,
    };
    let results =
        ConditionService::new(&mut conn).search_conditions(filters).expect("Search should succeed");

    assert_eq!(results.len(), 15, "Should return 15 conditions");
    assert!(results.iter().all(|c| c.item_type == "Condition"));
}

#[test]
fn test_search_conditions_by_item_type_disease() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ConditionFilters {
        name: None,
        item_types: Some(vec!["Disease".to_string()]),
        sources: None,
        search: None,
    };
    let results =
        ConditionService::new(&mut conn).search_conditions(filters).expect("Search should succeed");

    assert_eq!(results.len(), 3, "Should return 3 diseases");
    assert!(results.iter().all(|c| c.item_type == "Disease"));
}

#[test]
fn test_search_conditions_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ConditionFilters {
        name: None,
        item_types: None,
        sources: Some(vec!["PHB".to_string()]),
        search: None,
    };
    let results =
        ConditionService::new(&mut conn).search_conditions(filters).expect("Search should succeed");

    assert_eq!(results.len(), 15, "Should return 15 PHB conditions");
    assert!(results.iter().all(|c| c.source == "PHB"));
}

#[test]
fn test_search_conditions_by_description() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ConditionFilters {
        name: None,
        item_types: None,
        sources: None,
        search: Some("incapacitated".to_string()),
    };
    let results =
        ConditionService::new(&mut conn).search_conditions(filters).expect("Search should succeed");

    // Incapacitated, Paralyzed, Stunned, Unconscious all mention incapacitated
    assert!(
        !results.is_empty(),
        "Should return conditions mentioning incapacitated"
    );
}

#[test]
fn test_search_conditions_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ConditionFilters {
        name: None,
        item_types: Some(vec!["Disease".to_string()]),
        sources: Some(vec!["DMG".to_string()]),
        search: None,
    };
    let results =
        ConditionService::new(&mut conn).search_conditions(filters).expect("Search should succeed");

    assert_eq!(results.len(), 3, "Should return 3 DMG diseases");
}

#[test]
fn test_search_conditions_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ConditionFilters {
        name: Some("Nonexistent".to_string()),
        item_types: None,
        sources: None,
        search: None,
    };
    let results =
        ConditionService::new(&mut conn).search_conditions(filters).expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_condition_by_id() {
    let (mut conn, _temp_dir) = setup_test_db();

    // Get first condition (ID 1 in fresh DB)
    let condition =
        ConditionService::new(&mut conn).get_condition_by_id(1).expect("Should get condition");

    assert!(condition.is_some());
}

#[test]
fn test_get_condition_by_id_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let condition =
        ConditionService::new(&mut conn).get_condition_by_id(99999).expect("Should not error");

    assert!(condition.is_none());
}

#[test]
fn test_get_item_types() {
    let (mut conn, _temp_dir) = setup_test_db();

    let types = ConditionService::new(&mut conn).get_item_types().expect("Should get item types");

    assert_eq!(types.len(), 2, "Should have 2 item types");
    assert!(types.contains(&"Condition".to_string()));
    assert!(types.contains(&"Disease".to_string()));
}

#[test]
fn test_get_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = ConditionService::new(&mut conn).get_condition_sources().expect("Should get sources");

    assert_eq!(sources.len(), 2, "Should have 2 sources");
    assert!(sources.contains(&"PHB".to_string()));
    assert!(sources.contains(&"DMG".to_string()));
}

#[test]
fn test_get_condition_count() {
    let (mut conn, _temp_dir) = setup_test_db();

    let count = ConditionService::new(&mut conn).get_condition_count().expect("Should get count");

    assert_eq!(count, 18, "Should have 18 conditions/diseases");
}
