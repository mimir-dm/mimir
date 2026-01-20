//! Integration tests for SpellService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::SpellFilters;
use mimir_dm_core::services::SpellService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_spell_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_spell_data(conn: &mut SqliteConnection) {
    // Insert test spells with various properties
    // Schema: name, level, school, cast_time, range, components, tags, source, full_spell_json
    let spells = vec![
        (
            "Fireball",
            "PHB",
            3,
            "Evocation",
            "1 action",
            "150 feet",
            "V, S, M",
            r#"{"name":"Fireball","source":"PHB","level":3,"school":"V","time":[{"number":1,"unit":"action"}],"range":{"type":"point","distance":{"type":"feet","amount":150}},"components":{"v":true,"s":true,"m":"a tiny ball of bat guano"},"duration":[{"type":"instant"}],"entries":["A bright streak flashes from your pointing finger."]}"#,
        ),
        (
            "Magic Missile",
            "PHB",
            1,
            "Evocation",
            "1 action",
            "120 feet",
            "V, S",
            r#"{"name":"Magic Missile","source":"PHB","level":1,"school":"V","time":[{"number":1,"unit":"action"}],"range":{"type":"point","distance":{"type":"feet","amount":120}},"components":{"v":true,"s":true},"duration":[{"type":"instant"}],"entries":["You create three glowing darts of magical force."]}"#,
        ),
        (
            "Shield",
            "PHB",
            1,
            "Abjuration",
            "1 reaction",
            "Self",
            "V, S",
            r#"{"name":"Shield","source":"PHB","level":1,"school":"A","time":[{"number":1,"unit":"reaction"}],"range":{"type":"point","distance":{"type":"self"}},"components":{"v":true,"s":true},"duration":[{"type":"timed","duration":{"type":"round","amount":1}}],"entries":["An invisible barrier of magical force appears."]}"#,
        ),
        (
            "Counterspell",
            "PHB",
            3,
            "Abjuration",
            "1 reaction",
            "60 feet",
            "S",
            r#"{"name":"Counterspell","source":"PHB","level":3,"school":"A","time":[{"number":1,"unit":"reaction"}],"range":{"type":"point","distance":{"type":"feet","amount":60}},"components":{"s":true},"duration":[{"type":"instant"}],"entries":["You attempt to interrupt a creature in the process of casting a spell."]}"#,
        ),
        (
            "Detect Magic",
            "PHB",
            1,
            "Divination",
            "1 action",
            "Self",
            "V, S",
            r#"{"name":"Detect Magic","source":"PHB","level":1,"school":"D","time":[{"number":1,"unit":"action"}],"range":{"type":"point","distance":{"type":"self"}},"components":{"v":true,"s":true},"duration":[{"type":"timed","duration":{"type":"minute","amount":10},"concentration":true}],"meta":{"ritual":true},"entries":["For the duration, you sense the presence of magic."]}"#,
        ),
        (
            "Prestidigitation",
            "PHB",
            0,
            "Transmutation",
            "1 action",
            "10 feet",
            "V, S",
            r#"{"name":"Prestidigitation","source":"PHB","level":0,"school":"T","time":[{"number":1,"unit":"action"}],"range":{"type":"point","distance":{"type":"feet","amount":10}},"components":{"v":true,"s":true},"duration":[{"type":"timed","duration":{"type":"hour","amount":1}}],"entries":["This spell is a minor magical trick."]}"#,
        ),
        (
            "Eldritch Blast",
            "PHB",
            0,
            "Evocation",
            "1 action",
            "120 feet",
            "V, S",
            r#"{"name":"Eldritch Blast","source":"PHB","level":0,"school":"V","time":[{"number":1,"unit":"action"}],"range":{"type":"point","distance":{"type":"feet","amount":120}},"components":{"v":true,"s":true},"duration":[{"type":"instant"}],"entries":["A beam of crackling energy streaks toward a creature."]}"#,
        ),
        (
            "Wish",
            "PHB",
            9,
            "Conjuration",
            "1 action",
            "Self",
            "V",
            r#"{"name":"Wish","source":"PHB","level":9,"school":"C","time":[{"number":1,"unit":"action"}],"range":{"type":"point","distance":{"type":"self"}},"components":{"v":true},"duration":[{"type":"instant"}],"entries":["Wish is the mightiest spell a mortal creature can cast."]}"#,
        ),
        (
            "Chaos Bolt",
            "XGE",
            1,
            "Evocation",
            "1 action",
            "120 feet",
            "V, S",
            r#"{"name":"Chaos Bolt","source":"XGE","level":1,"school":"V","time":[{"number":1,"unit":"action"}],"range":{"type":"point","distance":{"type":"feet","amount":120}},"components":{"v":true,"s":true},"duration":[{"type":"instant"}],"entries":["You hurl an undulating, warbling mass of chaotic energy."]}"#,
        ),
        (
            "Silvery Barbs",
            "SCC",
            1,
            "Enchantment",
            "1 reaction",
            "60 feet",
            "V",
            r#"{"name":"Silvery Barbs","source":"SCC","level":1,"school":"E","time":[{"number":1,"unit":"reaction"}],"range":{"type":"point","distance":{"type":"feet","amount":60}},"components":{"v":true},"duration":[{"type":"instant"}],"entries":["You magically distract the triggering creature."]}"#,
        ),
    ];

    for (name, source, level, school, cast_time, range, components, json) in spells {
        diesel::sql_query(
            "INSERT INTO catalog_spells (name, level, school, cast_time, range, components, tags, source, full_spell_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Integer, _>(level)
        .bind::<diesel::sql_types::Text, _>(school)
        .bind::<diesel::sql_types::Text, _>(cast_time)
        .bind::<diesel::sql_types::Text, _>(range)
        .bind::<diesel::sql_types::Text, _>(components)
        .bind::<diesel::sql_types::Text, _>("[]")
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_spells_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = SpellFilters::default();
    let results = SpellService::search_spells(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 10, "Should return all 10 seeded spells");
}

#[test]
fn test_search_spells_by_level() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = SpellFilters {
        levels: vec![1],
        ..Default::default()
    };
    let results = SpellService::search_spells(&mut conn, filters).expect("Search should succeed");

    // Level 1 spells: Magic Missile, Shield, Detect Magic, Chaos Bolt, Silvery Barbs
    assert_eq!(results.len(), 5, "Should return 5 level 1 spells");
    assert!(results.iter().all(|s| s.level == 1));
}

#[test]
fn test_search_spells_by_multiple_levels() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = SpellFilters {
        levels: vec![0, 9],
        ..Default::default()
    };
    let results = SpellService::search_spells(&mut conn, filters).expect("Search should succeed");

    assert_eq!(
        results.len(),
        3,
        "Should return cantrips and 9th level spells"
    );
    assert!(results.iter().all(|s| s.level == 0 || s.level == 9));
}

#[test]
fn test_search_spells_by_school() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = SpellFilters {
        schools: vec!["Evocation".to_string()],
        ..Default::default()
    };
    let results = SpellService::search_spells(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 4, "Should return 4 Evocation spells");
    assert!(results.iter().all(|s| s.school == "Evocation"));
}

#[test]
fn test_search_spells_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = SpellFilters {
        sources: vec!["PHB".to_string()],
        ..Default::default()
    };
    let results = SpellService::search_spells(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 8, "Should return 8 PHB spells");
    assert!(results.iter().all(|s| s.source == "PHB"));
}

#[test]
fn test_search_spells_by_name_query() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = SpellFilters {
        query: Some("Fire".to_string()),
        ..Default::default()
    };
    let results = SpellService::search_spells(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 1, "Should return 1 spell containing 'Fire'");
    assert_eq!(results[0].name, "Fireball");
}

#[test]
fn test_search_spells_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = SpellFilters {
        levels: vec![1],
        schools: vec!["Evocation".to_string()],
        sources: vec!["PHB".to_string()],
        ..Default::default()
    };
    let results = SpellService::search_spells(&mut conn, filters).expect("Search should succeed");

    assert_eq!(
        results.len(),
        1,
        "Should return 1 level 1 Evocation spell from PHB"
    );
    assert_eq!(results[0].name, "Magic Missile");
}

#[test]
fn test_search_spells_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = SpellFilters {
        levels: vec![5],
        ..Default::default()
    };
    let results = SpellService::search_spells(&mut conn, filters).expect("Search should succeed");

    assert!(
        results.is_empty(),
        "Should return empty results for level 5 spells"
    );
}

#[test]
fn test_search_spells_with_pagination() {
    let (mut conn, _temp_dir) = setup_test_db();

    let filters = SpellFilters {
        limit: Some(3),
        offset: Some(0),
        ..Default::default()
    };
    let results = SpellService::search_spells(&mut conn, filters).expect("Search should succeed");

    assert_eq!(results.len(), 3, "Should return 3 spells with limit");
}

#[test]
fn test_get_spell_details() {
    let (mut conn, _temp_dir) = setup_test_db();

    let spell = SpellService::get_spell_details(&mut conn, "Fireball", "PHB")
        .expect("Should get spell details");

    assert!(spell.is_some());
    let spell = spell.unwrap();
    assert_eq!(spell.name, "Fireball");
    assert_eq!(spell.source, "PHB");
    assert_eq!(spell.level, 3);
}

#[test]
fn test_get_spell_details_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();

    let spell = SpellService::get_spell_details(&mut conn, "Nonexistent Spell", "PHB")
        .expect("Should not error for missing spell");

    assert!(spell.is_none());
}

#[test]
fn test_get_spell_sources() {
    let (mut conn, _temp_dir) = setup_test_db();

    let sources = SpellService::get_spell_sources(&mut conn).expect("Should get spell sources");

    assert_eq!(sources.len(), 3, "Should have 3 unique sources");
    assert!(sources.contains(&"PHB".to_string()));
    assert!(sources.contains(&"XGE".to_string()));
    assert!(sources.contains(&"SCC".to_string()));
}

#[test]
fn test_get_spell_schools() {
    let (mut conn, _temp_dir) = setup_test_db();

    let schools = SpellService::get_spell_schools(&mut conn).expect("Should get spell schools");

    assert!(schools.len() >= 5, "Should have multiple schools");
    assert!(schools.contains(&"Evocation".to_string()));
    assert!(schools.contains(&"Abjuration".to_string()));
}

#[test]
fn test_get_total_spell_count() {
    let (mut conn, _temp_dir) = setup_test_db();

    let count = SpellService::get_total_spell_count(&mut conn).expect("Should get spell count");

    assert_eq!(count, 10, "Should have 10 total spells");
}

#[test]
fn test_get_spell_count_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();

    let counts = SpellService::get_spell_count_by_source(&mut conn)
        .expect("Should get spell counts by source");

    let phb_count = counts.iter().find(|(s, _)| s == "PHB").map(|(_, c)| *c);
    assert_eq!(phb_count, Some(8), "PHB should have 8 spells");
}
