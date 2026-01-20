//! Integration tests for ClassService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::class::ClassFilters;
use mimir_dm_core::services::ClassService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_class_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_class_data(conn: &mut SqliteConnection) {
    // Insert base classes
    let classes = vec![
        (
            "Wizard",
            "PHB",
            "d6",
            Some("int"),
            Some("Intelligence"),
            r#"{"name":"Wizard","source":"PHB","hd":{"number":1,"faces":6},"casterProgression":"full","spellcastingAbility":"int"}"#,
        ),
        (
            "Fighter",
            "PHB",
            "d10",
            None,
            Some("Strength or Dexterity"),
            r#"{"name":"Fighter","source":"PHB","hd":{"number":1,"faces":10}}"#,
        ),
        (
            "Cleric",
            "PHB",
            "d8",
            Some("wis"),
            Some("Wisdom"),
            r#"{"name":"Cleric","source":"PHB","hd":{"number":1,"faces":8},"casterProgression":"full","spellcastingAbility":"wis"}"#,
        ),
        (
            "Rogue",
            "PHB",
            "d8",
            None,
            Some("Dexterity"),
            r#"{"name":"Rogue","source":"PHB","hd":{"number":1,"faces":8}}"#,
        ),
        (
            "Paladin",
            "PHB",
            "d10",
            Some("cha"),
            Some("Strength and Charisma"),
            r#"{"name":"Paladin","source":"PHB","hd":{"number":1,"faces":10},"casterProgression":"half","spellcastingAbility":"cha"}"#,
        ),
        (
            "Barbarian",
            "PHB",
            "d12",
            None,
            Some("Strength"),
            r#"{"name":"Barbarian","source":"PHB","hd":{"number":1,"faces":12}}"#,
        ),
        (
            "Blood Hunter",
            "TDCSR",
            "d10",
            None,
            Some("Strength or Dexterity"),
            r#"{"name":"Blood Hunter","source":"TDCSR","hd":{"number":1,"faces":10}}"#,
        ),
    ];

    for (name, source, hit_dice, spellcasting_ability, primary_ability, json) in classes {
        diesel::sql_query(
            "INSERT INTO catalog_classes (name, source, hit_dice, spellcasting_ability, primary_ability, full_class_json) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(hit_dice)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(spellcasting_ability)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(primary_ability)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }

    // Insert subclasses
    let subclasses = vec![
        (
            "School of Evocation",
            "Wizard",
            "PHB",
            "PHB",
            Some("int"),
            r#"{"name":"School of Evocation","shortName":"Evocation","source":"PHB","className":"Wizard","classSource":"PHB"}"#,
        ),
        (
            "School of Necromancy",
            "Wizard",
            "PHB",
            "PHB",
            Some("int"),
            r#"{"name":"School of Necromancy","shortName":"Necromancy","source":"PHB","className":"Wizard","classSource":"PHB"}"#,
        ),
        (
            "Champion",
            "Fighter",
            "PHB",
            "PHB",
            None,
            r#"{"name":"Champion","shortName":"Champion","source":"PHB","className":"Fighter","classSource":"PHB"}"#,
        ),
        (
            "Battle Master",
            "Fighter",
            "PHB",
            "PHB",
            None,
            r#"{"name":"Battle Master","shortName":"Battle Master","source":"PHB","className":"Fighter","classSource":"PHB"}"#,
        ),
        (
            "Life Domain",
            "Cleric",
            "PHB",
            "PHB",
            Some("wis"),
            r#"{"name":"Life Domain","shortName":"Life","source":"PHB","className":"Cleric","classSource":"PHB"}"#,
        ),
        (
            "Thief",
            "Rogue",
            "PHB",
            "PHB",
            None,
            r#"{"name":"Thief","shortName":"Thief","source":"PHB","className":"Rogue","classSource":"PHB"}"#,
        ),
        (
            "Echo Knight",
            "Fighter",
            "PHB",
            "EGW",
            None,
            r#"{"name":"Echo Knight","shortName":"Echo Knight","source":"EGW","className":"Fighter","classSource":"PHB"}"#,
        ),
    ];

    for (name, class_name, class_source, source, spellcasting_ability, json) in subclasses {
        diesel::sql_query(
            "INSERT INTO catalog_subclasses (name, class_name, class_source, source, spellcasting_ability, full_subclass_json) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(class_name)
        .bind::<diesel::sql_types::Text, _>(class_source)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(spellcasting_ability)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_classes_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let filters = ClassFilters {
        name: None,
        sources: None,
        has_spellcasting: None,
        primary_abilities: None,
    };
    let results = service
        .search_classes(filters)
        .expect("Search should succeed");

    // Should return base classes + subclasses
    assert!(results.len() >= 7, "Should return at least 7 base classes");
}

#[test]
fn test_search_classes_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let filters = ClassFilters {
        name: Some("Wizard".to_string()),
        sources: None,
        has_spellcasting: None,
        primary_abilities: None,
    };
    let results = service
        .search_classes(filters)
        .expect("Search should succeed");

    // Should match Wizard base class and its subclasses
    assert!(!results.is_empty(), "Should find Wizard");
    assert!(results.iter().any(|c| c.name == "Wizard"));
}

#[test]
fn test_search_classes_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let filters = ClassFilters {
        name: None,
        sources: Some(vec!["PHB".to_string()]),
        has_spellcasting: None,
        primary_abilities: None,
    };
    let results = service
        .search_classes(filters)
        .expect("Search should succeed");

    // Returns base classes + subclasses from PHB source
    assert!(results.len() >= 6, "Should return at least 6 PHB results");
    assert!(results.iter().all(|c| c.source == "PHB"));
}

#[test]
fn test_search_classes_with_spellcasting() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let filters = ClassFilters {
        name: None,
        sources: None,
        has_spellcasting: Some(true),
        primary_abilities: None,
    };
    let results = service
        .search_classes(filters)
        .expect("Search should succeed");

    // Should return Wizard, Cleric, Paladin (spellcasters)
    assert!(
        results.len() >= 3,
        "Should return at least 3 spellcasting classes"
    );
}

#[test]
fn test_search_classes_without_spellcasting() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let filters = ClassFilters {
        name: None,
        sources: None,
        has_spellcasting: Some(false),
        primary_abilities: None,
    };
    let results = service
        .search_classes(filters)
        .expect("Search should succeed");

    // Should return Fighter, Rogue, Barbarian, Blood Hunter (non-spellcasters)
    assert!(
        results.len() >= 4,
        "Should return at least 4 non-spellcasting classes"
    );
}

#[test]
fn test_search_classes_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let filters = ClassFilters {
        name: Some("Nonexistent".to_string()),
        sources: None,
        has_spellcasting: None,
        primary_abilities: None,
    };
    let results = service
        .search_classes(filters)
        .expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_class_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let class = service
        .get_class_by_name_and_source("Wizard", "PHB")
        .expect("Should get class");

    assert!(class.is_some());
    let class = class.unwrap();
    assert_eq!(class.name, "Wizard");
    assert_eq!(class.source, "PHB");
}

#[test]
fn test_get_class_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let class = service
        .get_class_by_name_and_source("Nonexistent", "PHB")
        .expect("Should not error");

    assert!(class.is_none());
}

#[test]
fn test_get_subclasses_for_class() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let subclasses = service
        .get_subclasses_for_class("Fighter", "PHB")
        .expect("Should get subclasses");

    // We seeded 3 Fighter subclasses in test data
    assert!(
        subclasses.len() >= 3,
        "Fighter should have at least 3 subclasses"
    );
}

#[test]
fn test_get_class_sources() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let sources = service.get_class_sources().expect("Should get sources");

    assert!(sources.len() >= 2, "Should have multiple sources");
    assert!(sources.contains(&"PHB".to_string()));
}

#[test]
fn test_get_primary_abilities() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let abilities = service
        .get_primary_abilities()
        .expect("Should get abilities");

    assert!(!abilities.is_empty(), "Should have primary abilities");
    assert!(abilities.iter().any(|a| a.contains("Strength")));
}

#[test]
fn test_get_class_count_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = ClassService::new(&mut conn);

    let counts = service
        .get_class_count_by_source()
        .expect("Should get counts");

    let phb_count = counts.iter().find(|(s, _)| s == "PHB").map(|(_, c)| *c);
    assert_eq!(phb_count, Some(6), "PHB should have 6 classes");
}
