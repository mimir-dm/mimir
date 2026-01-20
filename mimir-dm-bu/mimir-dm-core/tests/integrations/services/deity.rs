//! Integration tests for DeityService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::deity::DeityFilters;
use mimir_dm_core::services::deity_service::DeityService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

/// Test deity data: (name, title, pantheon, alignment_parts, domains, symbol, source)
type DeityTestData<'a> = (
    &'a str,
    &'a str,
    &'a str,
    &'a [&'a str],
    &'a str,
    &'a str,
    &'a str,
);

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_deity_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_deity_data(conn: &mut SqliteConnection) {
    // Deities with (name, title, pantheon, alignment_parts, domains, symbol, source)
    // alignment_parts is serialized as JSON array (e.g., ["N", "G"])
    let deities: Vec<DeityTestData> = vec![
        (
            "Pelor",
            "The Shining One",
            "Dawn War",
            &["N", "G"],
            "Life, Light",
            "Sun",
            "PHB",
        ),
        (
            "Moradin",
            "The All-Father",
            "Dwarven",
            &["L", "G"],
            "Forge, Knowledge",
            "Hammer and anvil",
            "PHB",
        ),
        (
            "Corellon Larethian",
            "Creator of Elves",
            "Elven",
            &["C", "G"],
            "Arcana, Light",
            "Crescent moon",
            "PHB",
        ),
        (
            "Lolth",
            "Queen of Spiders",
            "Dark Seldarine",
            &["C", "E"],
            "Trickery, War",
            "Spider",
            "PHB",
        ),
        (
            "Tiamat",
            "Dragon Queen",
            "Draconic",
            &["L", "E"],
            "Trickery, War",
            "Five-headed dragon",
            "PHB",
        ),
        (
            "Bahamut",
            "The Platinum Dragon",
            "Draconic",
            &["L", "G"],
            "Life, War",
            "Dragon head in profile",
            "PHB",
        ),
        (
            "Odin",
            "The Allfather",
            "Norse",
            &["N", "G"],
            "Knowledge, War",
            "Watching blue eye",
            "MTF",
        ),
        (
            "Thor",
            "God of Thunder",
            "Norse",
            &["C", "G"],
            "Tempest, War",
            "Hammer",
            "MTF",
        ),
        (
            "Athena",
            "Goddess of Wisdom",
            "Greek",
            &["L", "G"],
            "Knowledge, War",
            "Owl",
            "MTF",
        ),
        (
            "Zeus",
            "King of the Gods",
            "Greek",
            &["N"],
            "Tempest",
            "Fist full of lightning",
            "MTF",
        ),
    ];

    for (name, title, pantheon, alignment_parts, domains, symbol, source) in deities {
        // Build alignment JSON array
        let alignment_json = alignment_parts
            .iter()
            .map(|a| format!("\"{}\"", a))
            .collect::<Vec<_>>()
            .join(",");

        // Build domains JSON array
        let domains_json = domains
            .split(", ")
            .map(|d| format!("\"{}\"", d))
            .collect::<Vec<_>>()
            .join(",");

        // Build alignment string for DB column (e.g., "NG", "CE")
        let alignment_str = alignment_parts.join("");

        let json = format!(
            r#"{{"name":"{}","title":"{}","pantheon":"{}","alignment":[{}],"domains":[{}],"symbol":"{}","source":"{}"}}"#,
            name, title, pantheon, alignment_json, domains_json, symbol, source
        );

        diesel::sql_query(
            "INSERT INTO catalog_deities (name, title, pantheon, alignment, domains, symbol, source, full_deity_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(title)
        .bind::<diesel::sql_types::Text, _>(pantheon)
        .bind::<diesel::sql_types::Text, _>(&alignment_str)
        .bind::<diesel::sql_types::Text, _>(domains)
        .bind::<diesel::sql_types::Text, _>(symbol)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(&json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_deities_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let filters = DeityFilters {
        name: None,
        sources: None,
        pantheons: None,
        domains: None,
        alignments: None,
    };
    let results = service
        .search_deities(filters)
        .expect("Should search deities");

    assert_eq!(results.len(), 10);
}

#[test]
fn test_search_deities_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let filters = DeityFilters {
        name: Some("pelor".to_string()),
        sources: None,
        pantheons: None,
        domains: None,
        alignments: None,
    };
    let results = service
        .search_deities(filters)
        .expect("Should search deities");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Pelor");
}

#[test]
fn test_search_deities_by_pantheon() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let filters = DeityFilters {
        name: None,
        sources: None,
        pantheons: Some(vec!["Norse".to_string()]),
        domains: None,
        alignments: None,
    };
    let results = service
        .search_deities(filters)
        .expect("Should search deities");

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|d| d.pantheon == "Norse"));
}

#[test]
fn test_search_deities_by_domain() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let filters = DeityFilters {
        name: None,
        sources: None,
        pantheons: None,
        domains: Some(vec!["War".to_string()]),
        alignments: None,
    };
    let results = service
        .search_deities(filters)
        .expect("Should search deities");

    // Lolth, Tiamat, Bahamut, Odin, Thor, Athena all have War domain
    assert_eq!(results.len(), 6);
}

#[test]
fn test_search_deities_by_alignment() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let filters = DeityFilters {
        name: None,
        sources: None,
        pantheons: None,
        domains: None,
        alignments: Some(vec!["LG".to_string()]),
    };
    let results = service
        .search_deities(filters)
        .expect("Should search deities");

    assert_eq!(results.len(), 3); // Moradin, Bahamut, Athena
}

#[test]
fn test_search_deities_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let filters = DeityFilters {
        name: None,
        sources: Some(vec!["MTF".to_string()]),
        pantheons: None,
        domains: None,
        alignments: None,
    };
    let results = service
        .search_deities(filters)
        .expect("Should search deities");

    assert_eq!(results.len(), 4);
}

#[test]
fn test_search_deities_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let filters = DeityFilters {
        name: None,
        sources: None,
        pantheons: Some(vec!["Draconic".to_string()]),
        domains: None,
        alignments: Some(vec!["LG".to_string()]),
    };
    let results = service
        .search_deities(filters)
        .expect("Should search deities");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Bahamut");
}

#[test]
fn test_search_deities_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let filters = DeityFilters {
        name: Some("NonexistentDeity".to_string()),
        sources: None,
        pantheons: None,
        domains: None,
        alignments: None,
    };
    let results = service
        .search_deities(filters)
        .expect("Should search deities");

    assert!(results.is_empty());
}

#[test]
fn test_get_deity_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let result = service
        .get_deity_by_name_and_source("Pelor", "PHB")
        .expect("Should get deity");

    assert!(result.is_some());
    let deity = result.unwrap();
    assert_eq!(deity.name, "Pelor");
}

#[test]
fn test_get_deity_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let result = service
        .get_deity_by_name_and_source("Nonexistent", "PHB")
        .expect("Should handle not found");

    assert!(result.is_none());
}

#[test]
fn test_get_all_pantheons() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let pantheons = service.get_all_pantheons().expect("Should get pantheons");

    assert!(pantheons.contains(&"Norse".to_string()));
    assert!(pantheons.contains(&"Greek".to_string()));
    assert!(pantheons.contains(&"Draconic".to_string()));
}

#[test]
fn test_get_all_domains() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let domains = service.get_all_domains().expect("Should get domains");

    assert!(domains.contains(&"War".to_string()));
    assert!(domains.contains(&"Life".to_string()));
    assert!(domains.contains(&"Knowledge".to_string()));
}

#[test]
fn test_get_all_alignments() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let alignments = service.get_all_alignments().expect("Should get alignments");

    assert!(alignments.contains(&"LG".to_string()));
    assert!(alignments.contains(&"NG".to_string()));
    assert!(alignments.contains(&"CE".to_string()));
}

#[test]
fn test_get_deity_count_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = DeityService::new(&mut conn);

    let counts = service
        .get_deity_count_by_source()
        .expect("Should get counts");

    let phb_count = counts.iter().find(|(s, _)| s == "PHB").map(|(_, c)| *c);
    let mtf_count = counts.iter().find(|(s, _)| s == "MTF").map(|(_, c)| *c);

    assert_eq!(phb_count, Some(6));
    assert_eq!(mtf_count, Some(4));
}
