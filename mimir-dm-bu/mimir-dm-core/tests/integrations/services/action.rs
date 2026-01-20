//! Integration tests for ActionService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::ActionFilters;
use mimir_dm_core::services::ActionService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_action_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_action_data(conn: &mut SqliteConnection) {
    // Schema: name, time_type, description, see_also, source, full_action_json
    let actions = vec![
        (
            "Attack",
            "Action",
            "Make a melee or ranged attack",
            None::<&str>,
            "PHB",
            r#"{"name":"Attack","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Make a melee or ranged attack"]}"#,
        ),
        (
            "Cast a Spell",
            "Action",
            "Cast a spell with a casting time of 1 action",
            None,
            "PHB",
            r#"{"name":"Cast a Spell","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Cast a spell with a casting time of 1 action"]}"#,
        ),
        (
            "Dash",
            "Action",
            "Gain extra movement equal to your speed",
            None,
            "PHB",
            r#"{"name":"Dash","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Gain extra movement equal to your speed"]}"#,
        ),
        (
            "Disengage",
            "Action",
            "Your movement doesn't provoke opportunity attacks",
            None,
            "PHB",
            r#"{"name":"Disengage","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Your movement doesn't provoke opportunity attacks"]}"#,
        ),
        (
            "Dodge",
            "Action",
            "Focus on avoiding attacks",
            None,
            "PHB",
            r#"{"name":"Dodge","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Focus on avoiding attacks"]}"#,
        ),
        (
            "Help",
            "Action",
            "Aid another creature in a task",
            None,
            "PHB",
            r#"{"name":"Help","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Aid another creature in a task"]}"#,
        ),
        (
            "Hide",
            "Action",
            "Attempt to hide from enemies",
            None,
            "PHB",
            r#"{"name":"Hide","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Attempt to hide from enemies"]}"#,
        ),
        (
            "Ready",
            "Action",
            "Prepare to take an action in response to a trigger",
            None,
            "PHB",
            r#"{"name":"Ready","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Prepare to take an action in response to a trigger"]}"#,
        ),
        (
            "Search",
            "Action",
            "Look for something",
            None,
            "PHB",
            r#"{"name":"Search","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Look for something"]}"#,
        ),
        (
            "Use an Object",
            "Action",
            "Interact with an object that requires your action",
            None,
            "PHB",
            r#"{"name":"Use an Object","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Interact with an object that requires your action"]}"#,
        ),
        (
            "Opportunity Attack",
            "Reaction",
            "Make a melee attack against a creature leaving your reach",
            None,
            "PHB",
            r#"{"name":"Opportunity Attack","source":"PHB","time":[{"number":1,"unit":"reaction"}],"entries":["Make a melee attack against a creature leaving your reach"]}"#,
        ),
        (
            "Two-Weapon Fighting",
            "Bonus Action",
            "Make an attack with a light melee weapon",
            None,
            "PHB",
            r#"{"name":"Two-Weapon Fighting","source":"PHB","time":[{"number":1,"unit":"bonus"}],"entries":["Make an attack with a light melee weapon"]}"#,
        ),
        (
            "Grapple",
            "Action",
            "Attempt to grab a creature",
            Some("Shove"),
            "PHB",
            r#"{"name":"Grapple","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Attempt to grab a creature"],"seeAlsoAction":["Shove"]}"#,
        ),
        (
            "Shove",
            "Action",
            "Attempt to push a creature",
            Some("Grapple"),
            "PHB",
            r#"{"name":"Shove","source":"PHB","time":[{"number":1,"unit":"action"}],"entries":["Attempt to push a creature"],"seeAlsoAction":["Grapple"]}"#,
        ),
        (
            "Activate Magic Item",
            "Action",
            "Activate a magic item that requires an action",
            None,
            "DMG",
            r#"{"name":"Activate Magic Item","source":"DMG","time":[{"number":1,"unit":"action"}],"entries":["Activate a magic item that requires an action"]}"#,
        ),
    ];

    for (name, time_type, description, see_also, source, json) in actions {
        diesel::sql_query(
            "INSERT INTO catalog_actions (name, time_type, description, see_also, source, full_action_json) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(time_type)
        .bind::<diesel::sql_types::Text, _>(description)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(see_also)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_actions_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ActionFilters {
        name: None,
        time_types: None,
        sources: None,
        search: None,
    };
    let results = ActionService::new(&mut conn).search_actions(filters).expect("Search should succeed");

    assert_eq!(results.len(), 15, "Should return all 15 seeded actions");
}

#[test]
fn test_search_actions_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ActionFilters {
        name: Some("Attack".to_string()),
        time_types: None,
        sources: None,
        search: None,
    };
    let results = ActionService::new(&mut conn).search_actions(filters).expect("Search should succeed");

    // Attack, Opportunity Attack
    assert_eq!(
        results.len(),
        2,
        "Should return 2 actions matching 'Attack'"
    );
}

#[test]
fn test_search_actions_by_time_type() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ActionFilters {
        name: None,
        time_types: Some(vec!["Reaction".to_string()]),
        sources: None,
        search: None,
    };
    let results = ActionService::new(&mut conn).search_actions(filters).expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should return 1 reaction");
    assert_eq!(results[0].name, "Opportunity Attack");
}

#[test]
fn test_search_actions_by_bonus_action() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ActionFilters {
        name: None,
        time_types: Some(vec!["Bonus Action".to_string()]),
        sources: None,
        search: None,
    };
    let results = ActionService::new(&mut conn).search_actions(filters).expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should return 1 bonus action");
    assert_eq!(results[0].name, "Two-Weapon Fighting");
}

#[test]
fn test_search_actions_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ActionFilters {
        name: None,
        time_types: None,
        sources: Some(vec!["PHB".to_string()]),
        search: None,
    };
    let results = ActionService::new(&mut conn).search_actions(filters).expect("Search should succeed");

    assert_eq!(results.len(), 14, "Should return 14 PHB actions");
    assert!(results.iter().all(|a| a.source == "PHB"));
}

#[test]
fn test_search_actions_by_description() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ActionFilters {
        name: None,
        time_types: None,
        sources: None,
        search: Some("melee".to_string()),
    };
    let results = ActionService::new(&mut conn).search_actions(filters).expect("Search should succeed");

    // Attack, Opportunity Attack, Two-Weapon Fighting all mention melee
    assert!(results.len() >= 2, "Should return actions mentioning melee");
}

#[test]
fn test_search_actions_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ActionFilters {
        name: None,
        time_types: Some(vec!["Action".to_string()]),
        sources: Some(vec!["PHB".to_string()]),
        search: None,
    };
    let results = ActionService::new(&mut conn).search_actions(filters).expect("Search should succeed");

    // All PHB actions that are "Action" type
    assert!(results.len() >= 10, "Should return PHB action-type actions");
}

#[test]
fn test_search_actions_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = ActionFilters {
        name: Some("Nonexistent".to_string()),
        time_types: None,
        sources: None,
        search: None,
    };
    let results = ActionService::new(&mut conn).search_actions(filters).expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_action_by_id() {
    let (mut conn, _temp_dir) = setup_test_db();

    // First, get an action to find its ID
    let filters = ActionFilters {
        name: Some("Attack".to_string()),
        time_types: None,
        sources: None,
        search: None,
    };
    let results = ActionService::new(&mut conn).search_actions(filters).expect("Search should succeed");

    assert!(!results.is_empty());
    let action_id = results[0].id;

    let action = ActionService::new(&mut conn).get_action_by_id(action_id).expect("Should get action");

    assert!(action.is_some());
    let action = action.unwrap();
    assert_eq!(action.name, "Attack");
}

#[test]
fn test_get_action_by_id_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let action = ActionService::new(&mut conn).get_action_by_id(99999).expect("Should not error");

    assert!(action.is_none());
}

#[test]
fn test_get_time_types() {
    let (mut conn, _temp_dir) = setup_test_db();

    let types = ActionService::new(&mut conn).get_time_types().expect("Should get time types");

    assert_eq!(types.len(), 3, "Should have 3 time types");
    assert!(types.contains(&"Action".to_string()));
    assert!(types.contains(&"Reaction".to_string()));
    assert!(types.contains(&"Bonus Action".to_string()));
}

#[test]
fn test_get_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = ActionService::new(&mut conn).get_action_sources().expect("Should get sources");

    assert_eq!(sources.len(), 2, "Should have 2 sources");
    assert!(sources.contains(&"PHB".to_string()));
    assert!(sources.contains(&"DMG".to_string()));
}

#[test]
fn test_get_action_count() {
    let (mut conn, _temp_dir) = setup_test_db();

    let count = ActionService::new(&mut conn).get_action_count().expect("Should get count");

    assert_eq!(count, 15, "Should have 15 actions");
}
