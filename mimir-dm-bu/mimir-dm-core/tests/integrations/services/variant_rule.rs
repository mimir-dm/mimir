//! Integration tests for VariantRuleService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::variant_rule::VariantRuleFilters;
use mimir_dm_core::services::variant_rule_service::VariantRuleService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_variant_rule_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_variant_rule_data(conn: &mut SqliteConnection) {
    // Rules with types
    let typed_rules = vec![
        ("Ability Score Options", "Character Creation", "DMG"),
        ("Cleaving Through Creatures", "Combat", "DMG"),
        ("Climbing onto a Bigger Creature", "Combat", "DMG"),
        ("Customizing Your Origin", "Character Creation", "TCoE"),
        ("Disarm", "Combat", "DMG"),
        ("Flanking", "Combat", "DMG"),
        ("Gritty Realism", "Rest", "DMG"),
        ("Hitting Cover", "Combat", "DMG"),
        ("Massive Damage", "Combat", "DMG"),
        ("Morale", "Combat", "DMG"),
        ("Optional Class Features", "Class Features", "TCoE"),
        ("Slow Natural Healing", "Rest", "DMG"),
    ];

    for (name, rule_type, source) in typed_rules {
        let json = format!(
            r#"{{"name":"{}","ruleType":"{}","source":"{}"}}"#,
            name, rule_type, source
        );

        diesel::sql_query(
            "INSERT INTO catalog_variant_rules (name, rule_type, source, full_variant_rule_json) VALUES (?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(rule_type)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(&json)
        .execute(conn)
        .ok();
    }

    // Rules without type (General)
    let general_rules = vec![
        ("Proficiency Dice", "DMG"),
        ("Training to Gain Levels", "DMG"),
    ];

    for (name, source) in general_rules {
        let json = format!(r#"{{"name":"{}","source":"{}"}}"#, name, source);

        diesel::sql_query(
            "INSERT INTO catalog_variant_rules (name, rule_type, source, full_variant_rule_json) VALUES (?, NULL, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(&json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_variant_rules_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let filters = VariantRuleFilters {
        name: None,
        rule_types: None,
        sources: None,
    };
    let results = service
        .search_variant_rules(filters)
        .expect("Should search rules");

    assert_eq!(results.len(), 14);
}

#[test]
fn test_search_variant_rules_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let filters = VariantRuleFilters {
        name: Some("Flanking".to_string()),
        rule_types: None,
        sources: None,
    };
    let results = service
        .search_variant_rules(filters)
        .expect("Should search rules");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Flanking");
}

#[test]
fn test_search_variant_rules_by_rule_type() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let filters = VariantRuleFilters {
        name: None,
        rule_types: Some(vec!["Combat".to_string()]),
        sources: None,
    };
    let results = service
        .search_variant_rules(filters)
        .expect("Should search rules");

    assert_eq!(results.len(), 7); // All combat-related rules
}

#[test]
fn test_search_variant_rules_by_general_type() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let filters = VariantRuleFilters {
        name: None,
        rule_types: Some(vec!["General".to_string()]),
        sources: None,
    };
    let results = service
        .search_variant_rules(filters)
        .expect("Should search rules");

    // Rules with NULL rule_type are categorized as "General"
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_variant_rules_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let filters = VariantRuleFilters {
        name: None,
        rule_types: None,
        sources: Some(vec!["TCoE".to_string()]),
    };
    let results = service
        .search_variant_rules(filters)
        .expect("Should search rules");

    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_variant_rules_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let filters = VariantRuleFilters {
        name: None,
        rule_types: Some(vec!["Rest".to_string()]),
        sources: Some(vec!["DMG".to_string()]),
    };
    let results = service
        .search_variant_rules(filters)
        .expect("Should search rules");

    // Gritty Realism and Slow Natural Healing
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_variant_rules_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let filters = VariantRuleFilters {
        name: Some("NonexistentRule".to_string()),
        rule_types: None,
        sources: None,
    };
    let results = service
        .search_variant_rules(filters)
        .expect("Should search rules");

    assert!(results.is_empty());
}

#[test]
fn test_get_variant_rule_by_id() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let result = service.get_variant_rule_by_id(1).expect("Should get rule");

    assert!(result.is_some());
}

#[test]
fn test_get_variant_rule_by_id_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let result = service
        .get_variant_rule_by_id(9999)
        .expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_variant_rule_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let result = service
        .get_variant_rule_by_name_and_source("Flanking", "DMG")
        .expect("Should get rule");

    assert!(result.is_some());
    let rule = result.unwrap();
    assert_eq!(rule.name, "Flanking");
}

#[test]
fn test_get_variant_rule_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let result = service
        .get_variant_rule_by_name_and_source("Nonexistent", "DMG")
        .expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_variant_rule_types() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let types = service.get_variant_rule_types().expect("Should get types");

    assert!(types.contains(&"Combat".to_string()));
    assert!(types.contains(&"Rest".to_string()));
    assert!(types.contains(&"Character Creation".to_string()));
    // NULL types become "General"
    assert!(types.contains(&"General".to_string()));
}

#[test]
fn test_get_variant_rule_sources() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = VariantRuleService::new(&mut conn);

    let sources = service
        .get_variant_rule_sources()
        .expect("Should get sources");

    assert!(sources.contains(&"DMG".to_string()));
    assert!(sources.contains(&"TCoE".to_string()));
}
