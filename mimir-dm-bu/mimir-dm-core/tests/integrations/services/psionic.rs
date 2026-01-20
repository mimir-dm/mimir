//! Integration tests for PsionicService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::PsionicFilters;
use mimir_dm_core::services::psionic_service::PsionicService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_psionic_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_psionic_data(conn: &mut SqliteConnection) {
    // Disciplines (D) with orders
    let disciplines = vec![
        ("Adaptive Body", "D", "Immortal", "UAMystic"),
        ("Aura Sight", "D", "Awakened", "UAMystic"),
        ("Bestial Form", "D", "Immortal", "UAMystic"),
        ("Brute Force", "D", "Avatar", "UAMystic"),
        ("Celerity", "D", "Immortal", "UAMystic"),
        ("Corrosive Metabolism", "D", "Immortal", "UAMystic"),
        ("Crown of Disgust", "D", "Avatar", "UAMystic"),
        ("Crown of Rage", "D", "Avatar", "UAMystic"),
    ];

    for (name, psionic_type, order, source) in disciplines {
        let json = format!(
            r#"{{"name":"{}","type":"{}","order":"{}","source":"{}"}}"#,
            name, psionic_type, order, source
        );

        diesel::sql_query(
            "INSERT INTO catalog_psionics (name, psionic_type, psionic_order, source, full_psionic_json) VALUES (?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(psionic_type)
        .bind::<diesel::sql_types::Text, _>(order)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(&json)
        .execute(conn)
        .ok();
    }

    // Talents (T) without orders
    let talents = vec![
        ("Beacon", "T", "UAMystic"),
        ("Blade Meld", "T", "UAMystic"),
        ("Blind Spot", "T", "UAMystic"),
        ("Delusion", "T", "UAMystic"),
        ("Energy Beam", "T", "UAMystic"),
        ("Light Step", "T", "UAMystic"),
        ("Mind Meld", "T", "UAMystic"),
        ("Mind Slam", "T", "UAMystic"),
    ];

    for (name, psionic_type, source) in talents {
        let json = format!(
            r#"{{"name":"{}","type":"{}","source":"{}"}}"#,
            name, psionic_type, source
        );

        diesel::sql_query(
            "INSERT INTO catalog_psionics (name, psionic_type, psionic_order, source, full_psionic_json) VALUES (?, ?, NULL, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(psionic_type)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(&json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_psionics_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = PsionicFilters {
        name: None,
        psionic_types: None,
        orders: None,
        sources: None,
    };
    let results =
        PsionicService::search_psionics(&mut conn, filters).expect("Should search psionics");

    assert_eq!(results.len(), 16);
}

#[test]
fn test_search_psionics_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = PsionicFilters {
        name: Some("Crown".to_string()),
        psionic_types: None,
        orders: None,
        sources: None,
    };
    let results =
        PsionicService::search_psionics(&mut conn, filters).expect("Should search psionics");

    assert_eq!(results.len(), 2); // Crown of Disgust, Crown of Rage
}

#[test]
fn test_search_psionics_by_type_discipline() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = PsionicFilters {
        name: None,
        psionic_types: Some(vec!["D".to_string()]),
        orders: None,
        sources: None,
    };
    let results =
        PsionicService::search_psionics(&mut conn, filters).expect("Should search psionics");

    assert_eq!(results.len(), 8);
    assert!(results.iter().all(|p| p.psionic_type == "D"));
}

#[test]
fn test_search_psionics_by_type_talent() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = PsionicFilters {
        name: None,
        psionic_types: Some(vec!["T".to_string()]),
        orders: None,
        sources: None,
    };
    let results =
        PsionicService::search_psionics(&mut conn, filters).expect("Should search psionics");

    assert_eq!(results.len(), 8);
    assert!(results.iter().all(|p| p.psionic_type == "T"));
}

#[test]
fn test_search_psionics_by_order() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = PsionicFilters {
        name: None,
        psionic_types: None,
        orders: Some(vec!["Avatar".to_string()]),
        sources: None,
    };
    let results =
        PsionicService::search_psionics(&mut conn, filters).expect("Should search psionics");

    assert_eq!(results.len(), 3); // Brute Force, Crown of Disgust, Crown of Rage
}

#[test]
fn test_search_psionics_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = PsionicFilters {
        name: None,
        psionic_types: None,
        orders: None,
        sources: Some(vec!["UAMystic".to_string()]),
    };
    let results =
        PsionicService::search_psionics(&mut conn, filters).expect("Should search psionics");

    assert_eq!(results.len(), 16);
}

#[test]
fn test_search_psionics_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = PsionicFilters {
        name: None,
        psionic_types: Some(vec!["D".to_string()]),
        orders: Some(vec!["Immortal".to_string()]),
        sources: None,
    };
    let results =
        PsionicService::search_psionics(&mut conn, filters).expect("Should search psionics");

    // Adaptive Body, Bestial Form, Celerity, Corrosive Metabolism
    assert_eq!(results.len(), 4);
}

#[test]
fn test_search_psionics_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = PsionicFilters {
        name: Some("NonexistentPsionic".to_string()),
        psionic_types: None,
        orders: None,
        sources: None,
    };
    let results =
        PsionicService::search_psionics(&mut conn, filters).expect("Should search psionics");

    assert!(results.is_empty());
}

#[test]
fn test_get_psionic_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let result =
        PsionicService::get_psionic_by_name_and_source(&mut conn, "Aura Sight", "UAMystic")
            .expect("Should get psionic");

    assert!(result.is_some());
    let psionic = result.unwrap();
    assert_eq!(psionic.name, "Aura Sight");
}

#[test]
fn test_get_psionic_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let result =
        PsionicService::get_psionic_by_name_and_source(&mut conn, "Nonexistent", "UAMystic")
            .expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_psionic_by_id() {
    let (mut conn, _temp_dir) = setup_test_db();

    let result = PsionicService::get_psionic_by_id(&mut conn, 1).expect("Should get psionic");

    assert!(result.is_some());
}

#[test]
fn test_get_psionic_by_id_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let result =
        PsionicService::get_psionic_by_id(&mut conn, 9999).expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_all_psionic_types() {
    let (mut conn, _temp_dir) = setup_test_db();

    let types = PsionicService::get_all_psionic_types(&mut conn).expect("Should get types");

    assert!(types.contains(&"D".to_string()));
    assert!(types.contains(&"T".to_string()));
}

#[test]
fn test_get_all_psionic_orders() {
    let (mut conn, _temp_dir) = setup_test_db();

    let orders = PsionicService::get_all_psionic_orders(&mut conn).expect("Should get orders");

    assert!(orders.contains(&"Avatar".to_string()));
    assert!(orders.contains(&"Awakened".to_string()));
    assert!(orders.contains(&"Immortal".to_string()));
}

#[test]
fn test_get_all_psionic_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = PsionicService::get_all_psionic_sources(&mut conn).expect("Should get sources");

    assert!(sources.contains(&"UAMystic".to_string()));
}
