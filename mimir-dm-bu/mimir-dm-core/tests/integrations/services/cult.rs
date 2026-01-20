//! Integration tests for CultService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::cult::CultFilters;
use mimir_dm_core::services::cult_service::CultService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_cult_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_cult_data(conn: &mut SqliteConnection) {
    let cults = vec![
        // Cults
        ("Cult of Asmodeus", "cult", "Diabolical", "MTF"),
        ("Cult of Baphomet", "cult", "Demonic", "MTF"),
        ("Cult of Demogorgon", "cult", "Demonic", "MTF"),
        ("Cult of Orcus", "cult", "Demonic", "MTF"),
        ("Cult of Tiamat", "cult", "Draconic", "RoT"),
        ("Cult of the Dragon", "cult", "Draconic", "RoT"),
        // Boons
        ("Dark One's Blessing", "boon", "Diabolical", "MTF"),
        ("Demonic Vigor", "boon", "Demonic", "MTF"),
        ("Blessing of the Pit", "boon", "Diabolical", "MTF"),
        ("Elder Influence", "boon", "Elder Evil", "MTF"),
    ];

    for (name, category, cult_type, source) in cults {
        let json = format!(
            r#"{{"name":"{}","category":"{}","cultType":"{}","source":"{}"}}"#,
            name, category, cult_type, source
        );

        diesel::sql_query(
            "INSERT INTO catalog_cults (name, category, cult_type, source, full_cult_json) VALUES (?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(category)
        .bind::<diesel::sql_types::Text, _>(cult_type)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(&json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_cults_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let filters = CultFilters {
        name: None,
        category: None,
        cult_type: None,
        source: None,
    };
    let results = service
        .search_cults(&mut conn, filters)
        .expect("Should search cults");

    assert_eq!(results.len(), 10);
}

#[test]
fn test_search_cults_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let filters = CultFilters {
        name: Some("Asmodeus".to_string()),
        category: None,
        cult_type: None,
        source: None,
    };
    let results = service
        .search_cults(&mut conn, filters)
        .expect("Should search cults");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Cult of Asmodeus");
}

#[test]
fn test_search_cults_by_category_cult() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let filters = CultFilters {
        name: None,
        category: Some(vec!["cult".to_string()]),
        cult_type: None,
        source: None,
    };
    let results = service
        .search_cults(&mut conn, filters)
        .expect("Should search cults");

    assert_eq!(results.len(), 6);
    assert!(results.iter().all(|c| c.item_type == "cult"));
}

#[test]
fn test_search_cults_by_category_boon() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let filters = CultFilters {
        name: None,
        category: Some(vec!["boon".to_string()]),
        cult_type: None,
        source: None,
    };
    let results = service
        .search_cults(&mut conn, filters)
        .expect("Should search cults");

    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|c| c.item_type == "boon"));
}

#[test]
fn test_search_cults_by_cult_type() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let filters = CultFilters {
        name: None,
        category: None,
        cult_type: Some(vec!["Demonic".to_string()]),
        source: None,
    };
    let results = service
        .search_cults(&mut conn, filters)
        .expect("Should search cults");

    assert_eq!(results.len(), 4); // 3 cults + 1 boon
}

#[test]
fn test_search_cults_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let filters = CultFilters {
        name: None,
        category: None,
        cult_type: None,
        source: Some(vec!["RoT".to_string()]),
    };
    let results = service
        .search_cults(&mut conn, filters)
        .expect("Should search cults");

    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_cults_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let filters = CultFilters {
        name: None,
        category: Some(vec!["boon".to_string()]),
        cult_type: Some(vec!["Diabolical".to_string()]),
        source: None,
    };
    let results = service
        .search_cults(&mut conn, filters)
        .expect("Should search cults");

    assert_eq!(results.len(), 2); // Dark One's Blessing, Blessing of the Pit
}

#[test]
fn test_search_cults_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let filters = CultFilters {
        name: Some("NonexistentCult".to_string()),
        category: None,
        cult_type: None,
        source: None,
    };
    let results = service
        .search_cults(&mut conn, filters)
        .expect("Should search cults");

    assert!(results.is_empty());
}

#[test]
fn test_get_cult_details() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let result = service
        .get_cult_details(&mut conn, "Cult of Asmodeus".to_string(), "MTF".to_string())
        .expect("Should get cult details");

    assert!(result.is_some());
    let cult = result.unwrap();
    assert_eq!(cult.name, "Cult of Asmodeus");
}

#[test]
fn test_get_cult_details_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let result = service
        .get_cult_details(&mut conn, "Nonexistent".to_string(), "PHB".to_string())
        .expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_cult_sources() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let sources = service
        .get_cult_sources(&mut conn)
        .expect("Should get sources");

    assert!(sources.contains(&"MTF".to_string()));
    assert!(sources.contains(&"RoT".to_string()));
}

#[test]
fn test_get_cult_count() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let count = service.get_cult_count(&mut conn).expect("Should get count");

    assert_eq!(count, 10);
}

#[test]
fn test_get_cult_types() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let types = service.get_cult_types(&mut conn).expect("Should get types");

    assert!(types.contains(&"Diabolical".to_string()));
    assert!(types.contains(&"Demonic".to_string()));
    assert!(types.contains(&"Draconic".to_string()));
}

#[test]
fn test_get_cult_categories() {
    let (mut conn, _temp_dir) = setup_test_db();
    let service = CultService;

    let categories = service
        .get_cult_categories(&mut conn)
        .expect("Should get categories");

    assert!(categories.contains(&"cult".to_string()));
    assert!(categories.contains(&"boon".to_string()));
}
