//! Integration tests for LanguageService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::LanguageFilters;
use mimir_dm_core::services::language_service::LanguageService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_language_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_language_data(conn: &mut SqliteConnection) {
    // Languages with (name, language_type, script, typical_speakers_array, source)
    let languages: Vec<(&str, &str, &str, &[&str], &str)> = vec![
        (
            "Common",
            "Standard",
            "Common",
            &["Humans", "halflings", "most civilized races"],
            "PHB",
        ),
        ("Dwarvish", "Standard", "Dwarvish", &["Dwarves"], "PHB"),
        ("Elvish", "Standard", "Elvish", &["Elves"], "PHB"),
        ("Giant", "Standard", "Dwarvish", &["Giants", "ogres"], "PHB"),
        ("Gnomish", "Standard", "Dwarvish", &["Gnomes"], "PHB"),
        ("Goblin", "Standard", "Dwarvish", &["Goblinoids"], "PHB"),
        ("Halfling", "Standard", "Common", &["Halflings"], "PHB"),
        ("Orc", "Standard", "Dwarvish", &["Orcs"], "PHB"),
        ("Abyssal", "Exotic", "Infernal", &["Demons"], "PHB"),
        ("Celestial", "Exotic", "Celestial", &["Celestials"], "PHB"),
        (
            "Draconic",
            "Exotic",
            "Draconic",
            &["Dragons", "dragonborn"],
            "PHB",
        ),
        ("Deep Speech", "Exotic", "None", &["Aberrations"], "PHB"),
        ("Infernal", "Exotic", "Infernal", &["Devils"], "PHB"),
        ("Primordial", "Exotic", "Dwarvish", &["Elementals"], "PHB"),
        ("Sylvan", "Exotic", "Elvish", &["Fey creatures"], "PHB"),
        (
            "Undercommon",
            "Exotic",
            "Elvish",
            &["Underdark traders"],
            "PHB",
        ),
    ];

    for (name, language_type, script, typical_speakers, source) in languages {
        // Build typical_speakers JSON array
        let speakers_json = typical_speakers
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",");

        // Build typical_speakers string for DB column
        let speakers_str = typical_speakers.join(", ");

        let json = format!(
            r#"{{"name":"{}","type":"{}","script":"{}","typicalSpeakers":[{}],"source":"{}"}}"#,
            name, language_type, script, speakers_json, source
        );

        diesel::sql_query(
            "INSERT INTO catalog_languages (name, language_type, script, typical_speakers, source, full_language_json) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(language_type)
        .bind::<diesel::sql_types::Text, _>(script)
        .bind::<diesel::sql_types::Text, _>(&speakers_str)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(&json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_languages_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = LanguageFilters {
        name: None,
        search: None,
        language_types: None,
        scripts: None,
        sources: None,
    };
    let results =
        LanguageService::search_languages(&mut conn, filters).expect("Should search languages");

    assert_eq!(results.len(), 16);
}

#[test]
fn test_search_languages_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = LanguageFilters {
        name: Some("Elvish".to_string()),
        search: None,
        language_types: None,
        scripts: None,
        sources: None,
    };
    let results =
        LanguageService::search_languages(&mut conn, filters).expect("Should search languages");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Elvish");
}

#[test]
fn test_search_languages_by_search() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = LanguageFilters {
        name: None,
        search: Some("dragon".to_string()),
        language_types: None,
        scripts: None,
        sources: None,
    };
    let results =
        LanguageService::search_languages(&mut conn, filters).expect("Should search languages");

    // Should find Draconic (typical speakers include "dragon")
    assert!(!results.is_empty());
    assert!(results.iter().any(|l| l.name == "Draconic"));
}

#[test]
fn test_search_languages_by_type_standard() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = LanguageFilters {
        name: None,
        search: None,
        language_types: Some(vec!["Standard".to_string()]),
        scripts: None,
        sources: None,
    };
    let results =
        LanguageService::search_languages(&mut conn, filters).expect("Should search languages");

    assert_eq!(results.len(), 8);
    assert!(results.iter().all(|l| l.language_type == "Standard"));
}

#[test]
fn test_search_languages_by_type_exotic() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = LanguageFilters {
        name: None,
        search: None,
        language_types: Some(vec!["Exotic".to_string()]),
        scripts: None,
        sources: None,
    };
    let results =
        LanguageService::search_languages(&mut conn, filters).expect("Should search languages");

    assert_eq!(results.len(), 8);
    assert!(results.iter().all(|l| l.language_type == "Exotic"));
}

#[test]
fn test_search_languages_by_script() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = LanguageFilters {
        name: None,
        search: None,
        language_types: None,
        scripts: Some(vec!["Dwarvish".to_string()]),
        sources: None,
    };
    let results =
        LanguageService::search_languages(&mut conn, filters).expect("Should search languages");

    // Dwarvish, Giant, Gnomish, Goblin, Orc, Primordial
    assert_eq!(results.len(), 6);
}

#[test]
fn test_search_languages_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = LanguageFilters {
        name: None,
        search: None,
        language_types: None,
        scripts: None,
        sources: Some(vec!["PHB".to_string()]),
    };
    let results =
        LanguageService::search_languages(&mut conn, filters).expect("Should search languages");

    assert_eq!(results.len(), 16);
}

#[test]
fn test_search_languages_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = LanguageFilters {
        name: None,
        search: None,
        language_types: Some(vec!["Exotic".to_string()]),
        scripts: Some(vec!["Elvish".to_string()]),
        sources: None,
    };
    let results =
        LanguageService::search_languages(&mut conn, filters).expect("Should search languages");

    // Sylvan and Undercommon use Elvish script and are Exotic
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_languages_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = LanguageFilters {
        name: Some("NonexistentLanguage".to_string()),
        search: None,
        language_types: None,
        scripts: None,
        sources: None,
    };
    let results =
        LanguageService::search_languages(&mut conn, filters).expect("Should search languages");

    assert!(results.is_empty());
}

#[test]
fn test_get_language_by_id() {
    let (mut conn, _temp_dir) = setup_test_db();

    let result = LanguageService::get_language_by_id(&mut conn, 1).expect("Should get language");

    assert!(result.is_some());
}

#[test]
fn test_get_language_by_id_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let result =
        LanguageService::get_language_by_id(&mut conn, 9999).expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_language_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let result = LanguageService::get_language_by_name_and_source(&mut conn, "Elvish", "PHB")
        .expect("Should get language");

    assert!(result.is_some());
    let language = result.unwrap();
    assert_eq!(language.name, "Elvish");
}

#[test]
fn test_get_language_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let result = LanguageService::get_language_by_name_and_source(&mut conn, "Nonexistent", "PHB")
        .expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_language_types() {
    let (mut conn, _temp_dir) = setup_test_db();

    let types = LanguageService::get_language_types(&mut conn).expect("Should get types");

    assert!(types.contains(&"Standard".to_string()));
    assert!(types.contains(&"Exotic".to_string()));
}

#[test]
fn test_get_scripts() {
    let (mut conn, _temp_dir) = setup_test_db();

    let scripts = LanguageService::get_scripts(&mut conn).expect("Should get scripts");

    assert!(scripts.contains(&"Common".to_string()));
    assert!(scripts.contains(&"Dwarvish".to_string()));
    assert!(scripts.contains(&"Elvish".to_string()));
}

#[test]
fn test_get_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = LanguageService::get_sources(&mut conn).expect("Should get sources");

    assert!(sources.contains(&"PHB".to_string()));
}

#[test]
fn test_get_language_count() {
    let (mut conn, _temp_dir) = setup_test_db();

    let count = LanguageService::get_language_count(&mut conn).expect("Should get count");

    assert_eq!(count, 16);
}
