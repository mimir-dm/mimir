//! Integration tests for OptionalFeatureService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::optionalfeature::OptionalFeatureFilters;
use mimir_dm_core::services::optional_feature_service::OptionalFeatureService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_optional_feature_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_optional_feature_data(conn: &mut SqliteConnection) {
    let features = vec![
        // Eldritch Invocations
        (
            "Agonizing Blast",
            "[\"EI\"]",
            "Eldritch Invocation",
            Some("eldritch blast cantrip"),
            false,
            "PHB",
        ),
        (
            "Armor of Shadows",
            "[\"EI\"]",
            "Eldritch Invocation",
            None,
            false,
            "PHB",
        ),
        (
            "Beast Speech",
            "[\"EI\"]",
            "Eldritch Invocation",
            None,
            true,
            "PHB",
        ),
        (
            "Devil's Sight",
            "[\"EI\"]",
            "Eldritch Invocation",
            None,
            false,
            "PHB",
        ),
        (
            "Eldritch Sight",
            "[\"EI\"]",
            "Eldritch Invocation",
            None,
            true,
            "PHB",
        ),
        // Fighting Styles
        (
            "Archery",
            "[\"FS:F\",\"FS:R\"]",
            "Fighting Style",
            None,
            false,
            "PHB",
        ),
        (
            "Defense",
            "[\"FS:F\",\"FS:P\",\"FS:R\"]",
            "Fighting Style",
            None,
            false,
            "PHB",
        ),
        (
            "Dueling",
            "[\"FS:F\",\"FS:P\",\"FS:R\"]",
            "Fighting Style",
            None,
            false,
            "PHB",
        ),
        (
            "Great Weapon Fighting",
            "[\"FS:F\",\"FS:P\"]",
            "Fighting Style",
            None,
            false,
            "PHB",
        ),
        (
            "Protection",
            "[\"FS:F\",\"FS:P\"]",
            "Fighting Style",
            None,
            false,
            "PHB",
        ),
        // Metamagic
        ("Careful Spell", "[\"MM\"]", "Metamagic", None, false, "PHB"),
        ("Distant Spell", "[\"MM\"]", "Metamagic", None, false, "PHB"),
        (
            "Empowered Spell",
            "[\"MM\"]",
            "Metamagic",
            None,
            false,
            "PHB",
        ),
        (
            "Extended Spell",
            "[\"MM\"]",
            "Metamagic",
            None,
            false,
            "PHB",
        ),
        (
            "Quickened Spell",
            "[\"MM\"]",
            "Metamagic",
            None,
            false,
            "PHB",
        ),
        ("Subtle Spell", "[\"MM\"]", "Metamagic", None, false, "PHB"),
    ];

    for (name, feature_types, feature_type_full, prereq, grants_spells, source) in features {
        // JSON must include entries field (required by OptionalFeature struct)
        let json = format!(
            r#"{{"name":"{}","featureType":{},"source":"{}","entries":["Test entry for {}"]}}"#,
            name, feature_types, source, name
        );

        let grants_spells_val: i32 = if grants_spells { 1 } else { 0 };

        if let Some(prereq_text) = prereq {
            diesel::sql_query(
                "INSERT INTO catalog_optional_features (name, feature_types, feature_type_full, prerequisite_text, grants_spells, source, full_optional_feature_json) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind::<diesel::sql_types::Text, _>(name)
            .bind::<diesel::sql_types::Text, _>(feature_types)
            .bind::<diesel::sql_types::Text, _>(feature_type_full)
            .bind::<diesel::sql_types::Text, _>(prereq_text)
            .bind::<diesel::sql_types::Integer, _>(grants_spells_val)
            .bind::<diesel::sql_types::Text, _>(source)
            .bind::<diesel::sql_types::Text, _>(&json)
            .execute(conn)
            .ok();
        } else {
            diesel::sql_query(
                "INSERT INTO catalog_optional_features (name, feature_types, feature_type_full, prerequisite_text, grants_spells, source, full_optional_feature_json) VALUES (?, ?, ?, NULL, ?, ?, ?)"
            )
            .bind::<diesel::sql_types::Text, _>(name)
            .bind::<diesel::sql_types::Text, _>(feature_types)
            .bind::<diesel::sql_types::Text, _>(feature_type_full)
            .bind::<diesel::sql_types::Integer, _>(grants_spells_val)
            .bind::<diesel::sql_types::Text, _>(source)
            .bind::<diesel::sql_types::Text, _>(&json)
            .execute(conn)
            .ok();
        }
    }
}

#[test]
fn test_search_optional_features_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: None,
        feature_types: None,
        sources: None,
        grants_spells: None,
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    assert_eq!(results.len(), 16);
}

#[test]
fn test_search_optional_features_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: Some("Agonizing".to_string()),
        feature_types: None,
        sources: None,
        grants_spells: None,
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Agonizing Blast");
}

#[test]
fn test_search_optional_features_by_type_eldritch_invocation() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: None,
        feature_types: Some(vec!["EI".to_string()]),
        sources: None,
        grants_spells: None,
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    assert_eq!(results.len(), 5);
}

#[test]
fn test_search_optional_features_by_type_fighting_style() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: None,
        feature_types: Some(vec!["FS:F".to_string()]),
        sources: None,
        grants_spells: None,
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    assert_eq!(results.len(), 5); // All fighting styles have FS:F
}

#[test]
fn test_search_optional_features_by_type_metamagic() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: None,
        feature_types: Some(vec!["MM".to_string()]),
        sources: None,
        grants_spells: None,
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    assert_eq!(results.len(), 6);
}

#[test]
fn test_search_optional_features_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: None,
        feature_types: None,
        sources: Some(vec!["PHB".to_string()]),
        grants_spells: None,
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    assert_eq!(results.len(), 16);
}

#[test]
fn test_search_optional_features_grants_spells_true() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: None,
        feature_types: None,
        sources: None,
        grants_spells: Some(true),
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    // Beast Speech and Eldritch Sight grant spells
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_optional_features_grants_spells_false() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: None,
        feature_types: None,
        sources: None,
        grants_spells: Some(false),
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    assert_eq!(results.len(), 14);
}

#[test]
fn test_search_optional_features_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: None,
        feature_types: Some(vec!["EI".to_string()]),
        sources: None,
        grants_spells: Some(true),
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    // Beast Speech and Eldritch Sight
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_optional_features_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let filters = OptionalFeatureFilters {
        name: Some("NonexistentFeature".to_string()),
        feature_types: None,
        sources: None,
        grants_spells: None,
    };
    let results = service
        .search_optional_features(filters)
        .expect("Should search features");

    assert!(results.is_empty());
}

#[test]
fn test_get_optional_feature_by_id() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let result = service
        .get_optional_feature_by_id(1)
        .expect("Should get feature");

    assert!(result.is_some());
}

#[test]
fn test_get_optional_feature_by_id_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let result = service
        .get_optional_feature_by_id(9999)
        .expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_optional_feature_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let result = service
        .get_optional_feature_by_name_and_source("Agonizing Blast", "PHB")
        .expect("Should get feature");

    assert!(result.is_some());
    let feature = result.unwrap();
    assert_eq!(feature.name, "Agonizing Blast");
}

#[test]
fn test_get_optional_feature_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let result = service
        .get_optional_feature_by_name_and_source("Nonexistent", "PHB")
        .expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_optional_feature_types() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let types = service
        .get_optional_feature_types()
        .expect("Should get types");

    assert!(types.contains(&"EI".to_string()));
    assert!(types.contains(&"MM".to_string()));
    // Fighting styles have multiple types
    assert!(types.contains(&"FS:F".to_string()));
}

#[test]
fn test_get_optional_feature_sources() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = OptionalFeatureService::new(&mut conn);

    let sources = service
        .get_optional_feature_sources()
        .expect("Should get sources");

    assert!(sources.contains(&"PHB".to_string()));
}
