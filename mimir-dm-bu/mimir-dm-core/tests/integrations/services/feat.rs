//! Integration tests for FeatService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::FeatFilters;
use mimir_dm_core::services::FeatService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_feat_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_feat_data(conn: &mut SqliteConnection) {
    let feats = vec![
        (
            "Alert",
            "PHB",
            None::<&str>,
            "Always on the lookout for danger",
            r#"{"name":"Alert","source":"PHB","entries":["Always on the lookout for danger, you gain the following benefits:"]}"#,
        ),
        (
            "Athlete",
            "PHB",
            None,
            "You have undergone extensive physical training",
            r#"{"name":"Athlete","source":"PHB","entries":["You have undergone extensive physical training"]}"#,
        ),
        (
            "Actor",
            "PHB",
            None,
            "Skilled at mimicry and dramatics",
            r#"{"name":"Actor","source":"PHB","entries":["Skilled at mimicry and dramatics"]}"#,
        ),
        (
            "Charger",
            "PHB",
            None,
            "When you use your action to Dash",
            r#"{"name":"Charger","source":"PHB","entries":["When you use your action to Dash"]}"#,
        ),
        (
            "Crossbow Expert",
            "PHB",
            None,
            "You have mastered crossbows",
            r#"{"name":"Crossbow Expert","source":"PHB","entries":["You have mastered crossbows"]}"#,
        ),
        (
            "Defensive Duelist",
            "PHB",
            Some("Dexterity 13 or higher"),
            "When wielding a finesse weapon",
            r#"{"name":"Defensive Duelist","source":"PHB","prerequisite":[{"ability":[{"dex":13}]}],"entries":["When wielding a finesse weapon"]}"#,
        ),
        (
            "Dual Wielder",
            "PHB",
            None,
            "You master fighting with two weapons",
            r#"{"name":"Dual Wielder","source":"PHB","entries":["You master fighting with two weapons"]}"#,
        ),
        (
            "Dungeon Delver",
            "PHB",
            None,
            "You have honed your senses in the dark",
            r#"{"name":"Dungeon Delver","source":"PHB","entries":["You have honed your senses in the dark"]}"#,
        ),
        (
            "Durable",
            "PHB",
            None,
            "Hardy and resilient",
            r#"{"name":"Durable","source":"PHB","entries":["Hardy and resilient"]}"#,
        ),
        (
            "Elemental Adept",
            "PHB",
            Some("Spellcasting or Pact Magic feature"),
            "You have mastery over elements",
            r#"{"name":"Elemental Adept","source":"PHB","prerequisite":[{"spellcasting":true}],"entries":["You have mastery over elements"]}"#,
        ),
        (
            "Grappler",
            "PHB",
            Some("Strength 13 or higher"),
            "You are an expert at grappling",
            r#"{"name":"Grappler","source":"PHB","prerequisite":[{"ability":[{"str":13}]}],"entries":["You are an expert at grappling"]}"#,
        ),
        (
            "Great Weapon Master",
            "PHB",
            None,
            "You have learned to maximize your heavy weapons",
            r#"{"name":"Great Weapon Master","source":"PHB","entries":["You have learned to maximize your heavy weapons"]}"#,
        ),
        (
            "Healer",
            "PHB",
            None,
            "You are an able physician",
            r#"{"name":"Healer","source":"PHB","entries":["You are an able physician"]}"#,
        ),
        (
            "Heavy Armor Master",
            "PHB",
            Some("Proficiency with heavy armor"),
            "You can use heavy armor",
            r#"{"name":"Heavy Armor Master","source":"PHB","prerequisite":[{"proficiency":[{"armor":"heavy"}]}],"entries":["You can use heavy armor"]}"#,
        ),
        (
            "Lucky",
            "PHB",
            None,
            "You have inexplicable luck",
            r#"{"name":"Lucky","source":"PHB","entries":["You have inexplicable luck"]}"#,
        ),
        (
            "Fey Touched",
            "TCE",
            None,
            "Your exposure to the Feywild's magic",
            r#"{"name":"Fey Touched","source":"TCE","entries":["Your exposure to the Feywild's magic"]}"#,
        ),
        (
            "Shadow Touched",
            "TCE",
            None,
            "Your exposure to the Shadowfell's magic",
            r#"{"name":"Shadow Touched","source":"TCE","entries":["Your exposure to the Shadowfell's magic"]}"#,
        ),
        (
            "Telekinetic",
            "TCE",
            None,
            "You learn to move things with your mind",
            r#"{"name":"Telekinetic","source":"TCE","entries":["You learn to move things with your mind"]}"#,
        ),
    ];

    for (name, source, prerequisites, brief, json) in feats {
        diesel::sql_query(
            "INSERT INTO catalog_feats (name, source, prerequisites, brief, full_feat_json) VALUES (?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(prerequisites)
        .bind::<diesel::sql_types::Text, _>(brief)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_feats_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = FeatFilters {
        search_pattern: None,
        sources: None,
        has_prerequisites: None,
    };
    let results = FeatService::new(&mut conn).search_feats(filters).expect("Search should succeed");

    assert_eq!(results.len(), 18, "Should return all 18 seeded feats");
}

#[test]
fn test_search_feats_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = FeatFilters {
        search_pattern: Some("Lucky".to_string()),
        sources: None,
        has_prerequisites: None,
    };
    let results = FeatService::new(&mut conn).search_feats(filters).expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should return 1 feat matching 'Lucky'");
    assert_eq!(results[0].name, "Lucky");
}

#[test]
fn test_search_feats_by_brief_description() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = FeatFilters {
        search_pattern: Some("crossbow".to_string()),
        sources: None,
        has_prerequisites: None,
    };
    let results = FeatService::new(&mut conn).search_feats(filters).expect("Search should succeed");

    assert_eq!(
        results.len(),
        1,
        "Should return 1 feat with 'crossbow' in description"
    );
    assert_eq!(results[0].name, "Crossbow Expert");
}

#[test]
fn test_search_feats_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = FeatFilters {
        search_pattern: None,
        sources: Some(vec!["PHB".to_string()]),
        has_prerequisites: None,
    };
    let results = FeatService::new(&mut conn).search_feats(filters).expect("Search should succeed");

    assert_eq!(results.len(), 15, "Should return 15 PHB feats");
    assert!(results.iter().all(|f| f.source == "PHB"));
}

#[test]
fn test_search_feats_by_multiple_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = FeatFilters {
        search_pattern: None,
        sources: Some(vec!["TCE".to_string()]),
        has_prerequisites: None,
    };
    let results = FeatService::new(&mut conn).search_feats(filters).expect("Search should succeed");

    assert_eq!(results.len(), 3, "Should return 3 TCE feats");
}

#[test]
fn test_search_feats_with_prerequisites() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = FeatFilters {
        search_pattern: None,
        sources: None,
        has_prerequisites: Some(true),
    };
    let results = FeatService::new(&mut conn).search_feats(filters).expect("Search should succeed");

    // Feats with prerequisites: Defensive Duelist, Elemental Adept, Grappler, Heavy Armor Master
    assert_eq!(results.len(), 4, "Should return 4 feats with prerequisites");
    assert!(results.iter().all(|f| f.prerequisites.is_some()));
}

#[test]
fn test_search_feats_without_prerequisites() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = FeatFilters {
        search_pattern: None,
        sources: None,
        has_prerequisites: Some(false),
    };
    let results = FeatService::new(&mut conn).search_feats(filters).expect("Search should succeed");

    assert_eq!(
        results.len(),
        14,
        "Should return 14 feats without prerequisites"
    );
    assert!(results.iter().all(|f| f.prerequisites.is_none()));
}

#[test]
fn test_search_feats_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = FeatFilters {
        search_pattern: None,
        sources: Some(vec!["PHB".to_string()]),
        has_prerequisites: Some(true),
    };
    let results = FeatService::new(&mut conn).search_feats(filters).expect("Search should succeed");

    // PHB feats with prerequisites
    assert_eq!(
        results.len(),
        4,
        "Should return PHB feats with prerequisites"
    );
    assert!(results
        .iter()
        .all(|f| f.source == "PHB" && f.prerequisites.is_some()));
}

#[test]
fn test_search_feats_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = FeatFilters {
        search_pattern: Some("Nonexistent".to_string()),
        sources: None,
        has_prerequisites: None,
    };
    let results = FeatService::new(&mut conn).search_feats(filters).expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_feat_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let feat = FeatService::new(&mut conn).get_feat_by_name_and_source("Lucky", "PHB")
        .expect("Should get feat")
        .expect("Should find feat");

    assert_eq!(feat.name, "Lucky");
    assert_eq!(feat.source, "PHB");
}

#[test]
fn test_get_feat_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let result = FeatService::new(&mut conn).get_feat_by_name_and_source("Nonexistent", "PHB")
        .expect("Query should succeed");

    assert!(result.is_none(), "Should return None for nonexistent feat");
}

#[test]
fn test_get_feat_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = FeatService::new(&mut conn).get_feat_sources().expect("Should get sources");

    assert_eq!(sources.len(), 2, "Should have 2 sources");
    assert!(sources.contains(&"PHB".to_string()));
    assert!(sources.contains(&"TCE".to_string()));
}

#[test]
fn test_get_feat_count() {
    let (mut conn, _temp_dir) = setup_test_db();

    let count = FeatService::new(&mut conn).get_feat_count().expect("Should get count");

    assert_eq!(count, 18, "Should have 18 feats");
}
