//! Integration tests for MonsterService

use diesel::prelude::*;
use mimir_dm_core::models::catalog::monster::MonsterFilters;
use mimir_dm_core::services::MonsterService;
use mimir_dm_core::{establish_connection, run_migrations};
use tempfile::TempDir;

fn setup_test_db() -> (SqliteConnection, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let mut conn = establish_connection(db_path.to_str().unwrap()).unwrap();
    run_migrations(&mut conn).expect("Failed to run migrations");
    seed_test_monster_data(&mut conn);
    (conn, temp_dir)
}

fn seed_test_monster_data(conn: &mut SqliteConnection) {
    // Schema: name, size, creature_type, alignment, cr, cr_numeric, hp, ac, source, page, full_monster_json
    let monsters = vec![
        (
            "Goblin",
            "S",
            "Humanoid",
            "Neutral Evil",
            "1/4",
            0.25,
            7,
            15,
            "MM",
            r#"{"name":"Goblin","source":"MM","size":["S"],"type":"humanoid","alignment":["N","E"],"ac":[{"ac":15}],"hp":{"average":7},"cr":"1/4"}"#,
        ),
        (
            "Orc",
            "M",
            "Humanoid",
            "Chaotic Evil",
            "1/2",
            0.5,
            15,
            13,
            "MM",
            r#"{"name":"Orc","source":"MM","size":["M"],"type":"humanoid","alignment":["C","E"],"ac":[{"ac":13}],"hp":{"average":15},"cr":"1/2"}"#,
        ),
        (
            "Skeleton",
            "M",
            "Undead",
            "Lawful Evil",
            "1/4",
            0.25,
            13,
            13,
            "MM",
            r#"{"name":"Skeleton","source":"MM","size":["M"],"type":"undead","alignment":["L","E"],"ac":[{"ac":13}],"hp":{"average":13},"cr":"1/4"}"#,
        ),
        (
            "Zombie",
            "M",
            "Undead",
            "Neutral Evil",
            "1/4",
            0.25,
            22,
            8,
            "MM",
            r#"{"name":"Zombie","source":"MM","size":["M"],"type":"undead","alignment":["N","E"],"ac":[{"ac":8}],"hp":{"average":22},"cr":"1/4"}"#,
        ),
        (
            "Owlbear",
            "L",
            "Monstrosity",
            "Unaligned",
            "3",
            3.0,
            59,
            13,
            "MM",
            r#"{"name":"Owlbear","source":"MM","size":["L"],"type":"monstrosity","alignment":["U"],"ac":[{"ac":13}],"hp":{"average":59},"cr":"3"}"#,
        ),
        (
            "Troll",
            "L",
            "Giant",
            "Chaotic Evil",
            "5",
            5.0,
            84,
            15,
            "MM",
            r#"{"name":"Troll","source":"MM","size":["L"],"type":"giant","alignment":["C","E"],"ac":[{"ac":15}],"hp":{"average":84},"cr":"5"}"#,
        ),
        (
            "Hill Giant",
            "H",
            "Giant",
            "Chaotic Evil",
            "5",
            5.0,
            105,
            13,
            "MM",
            r#"{"name":"Hill Giant","source":"MM","size":["H"],"type":"giant","alignment":["C","E"],"ac":[{"ac":13}],"hp":{"average":105},"cr":"5"}"#,
        ),
        (
            "Young Red Dragon",
            "L",
            "Dragon",
            "Chaotic Evil",
            "10",
            10.0,
            178,
            18,
            "MM",
            r#"{"name":"Young Red Dragon","source":"MM","size":["L"],"type":"dragon","alignment":["C","E"],"ac":[{"ac":18}],"hp":{"average":178},"cr":"10"}"#,
        ),
        (
            "Adult Red Dragon",
            "H",
            "Dragon",
            "Chaotic Evil",
            "17",
            17.0,
            256,
            19,
            "MM",
            r#"{"name":"Adult Red Dragon","source":"MM","size":["H"],"type":"dragon","alignment":["C","E"],"ac":[{"ac":19}],"hp":{"average":256},"cr":"17"}"#,
        ),
        (
            "Lich",
            "M",
            "Undead",
            "Any Evil",
            "21",
            21.0,
            135,
            17,
            "MM",
            r#"{"name":"Lich","source":"MM","size":["M"],"type":"undead","alignment":["A","E"],"ac":[{"ac":17}],"hp":{"average":135},"cr":"21"}"#,
        ),
        (
            "Bandit",
            "M",
            "Humanoid",
            "Any Non-lawful",
            "1/8",
            0.125,
            11,
            12,
            "VRGR",
            r#"{"name":"Bandit","source":"VRGR","size":["M"],"type":"humanoid","alignment":["A","NX","L"],"ac":[{"ac":12}],"hp":{"average":11},"cr":"1/8"}"#,
        ),
        (
            "Vampire Spawn",
            "M",
            "Undead",
            "Neutral Evil",
            "5",
            5.0,
            82,
            15,
            "VRGR",
            r#"{"name":"Vampire Spawn","source":"VRGR","size":["M"],"type":"undead","alignment":["N","E"],"ac":[{"ac":15}],"hp":{"average":82},"cr":"5"}"#,
        ),
    ];

    for (name, size, creature_type, alignment, cr, cr_numeric, hp, ac, source, json) in monsters {
        diesel::sql_query(
            "INSERT INTO catalog_monsters (name, size, creature_type, alignment, cr, cr_numeric, hp, ac, source, full_monster_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(size)
        .bind::<diesel::sql_types::Text, _>(creature_type)
        .bind::<diesel::sql_types::Text, _>(alignment)
        .bind::<diesel::sql_types::Text, _>(cr)
        .bind::<diesel::sql_types::Double, _>(cr_numeric)
        .bind::<diesel::sql_types::Integer, _>(hp)
        .bind::<diesel::sql_types::Integer, _>(ac)
        .bind::<diesel::sql_types::Text, _>(source)
        .bind::<diesel::sql_types::Text, _>(json)
        .execute(conn)
        .ok();
    }
}

#[test]
fn test_search_monsters_no_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let filters = MonsterFilters {
        name: None,
        sizes: None,
        creature_types: None,
        alignments: None,
        sources: None,
        min_cr: None,
        max_cr: None,
        min_hp: None,
        max_hp: None,
        environment: None,
    };
    let results = service
        .search_monsters(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 12, "Should return all 12 seeded monsters");
}

#[test]
fn test_search_monsters_by_name() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let filters = MonsterFilters {
        name: Some("Dragon".to_string()),
        sizes: None,
        creature_types: None,
        alignments: None,
        sources: None,
        min_cr: None,
        max_cr: None,
        min_hp: None,
        max_hp: None,
        environment: None,
    };
    let results = service
        .search_monsters(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 2, "Should return 2 dragons");
    assert!(results.iter().all(|m| m.name.contains("Dragon")));
}

#[test]
fn test_search_monsters_by_cr_range() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let filters = MonsterFilters {
        name: None,
        sizes: None,
        creature_types: None,
        alignments: None,
        sources: None,
        min_cr: Some(1.0),
        max_cr: Some(5.0),
        min_hp: None,
        max_hp: None,
        environment: None,
    };
    let results = service
        .search_monsters(filters)
        .expect("Search should succeed");

    // CR 1-5: Owlbear (3), Troll (5), Hill Giant (5), Vampire Spawn (5)
    assert_eq!(results.len(), 4, "Should return 4 monsters in CR 1-5 range");
}

#[test]
fn test_search_monsters_by_creature_type() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let filters = MonsterFilters {
        name: None,
        sizes: None,
        creature_types: Some(vec!["Undead".to_string()]),
        alignments: None,
        sources: None,
        min_cr: None,
        max_cr: None,
        min_hp: None,
        max_hp: None,
        environment: None,
    };
    let results = service
        .search_monsters(filters)
        .expect("Search should succeed");

    // Undead: Skeleton, Zombie, Lich, Vampire Spawn
    assert_eq!(results.len(), 4, "Should return 4 undead monsters");
    assert!(results.iter().all(|m| m.creature_type == "Undead"));
}

#[test]
fn test_search_monsters_by_size() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let filters = MonsterFilters {
        name: None,
        sizes: Some(vec!["L".to_string()]),
        creature_types: None,
        alignments: None,
        sources: None,
        min_cr: None,
        max_cr: None,
        min_hp: None,
        max_hp: None,
        environment: None,
    };
    let results = service
        .search_monsters(filters)
        .expect("Search should succeed");

    // Large: Owlbear, Troll, Young Red Dragon
    assert_eq!(results.len(), 3, "Should return 3 Large monsters");
}

#[test]
fn test_search_monsters_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let filters = MonsterFilters {
        name: None,
        sizes: None,
        creature_types: None,
        alignments: None,
        sources: Some(vec!["MM".to_string()]),
        min_cr: None,
        max_cr: None,
        min_hp: None,
        max_hp: None,
        environment: None,
    };
    let results = service
        .search_monsters(filters)
        .expect("Search should succeed");

    assert_eq!(results.len(), 10, "Should return 10 MM monsters");
    assert!(results.iter().all(|m| m.source == "MM"));
}

#[test]
fn test_search_monsters_by_hp_range() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let filters = MonsterFilters {
        name: None,
        sizes: None,
        creature_types: None,
        alignments: None,
        sources: None,
        min_cr: None,
        max_cr: None,
        min_hp: Some(100),
        max_hp: None,
        environment: None,
    };
    let results = service
        .search_monsters(filters)
        .expect("Search should succeed");

    // HP >= 100: Hill Giant (105), Young Red Dragon (178), Adult Red Dragon (256), Lich (135)
    assert_eq!(results.len(), 4, "Should return 4 monsters with HP >= 100");
}

#[test]
fn test_search_monsters_combined_filters() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let filters = MonsterFilters {
        name: None,
        sizes: None,
        creature_types: Some(vec!["Dragon".to_string()]),
        alignments: None,
        sources: Some(vec!["MM".to_string()]),
        min_cr: Some(5.0),
        max_cr: Some(15.0),
        min_hp: None,
        max_hp: None,
        environment: None,
    };
    let results = service
        .search_monsters(filters)
        .expect("Search should succeed");

    // Dragons from MM with CR 5-15: Young Red Dragon (CR 10)
    assert_eq!(results.len(), 1, "Should return 1 dragon in CR 5-15 range");
    assert_eq!(results[0].name, "Young Red Dragon");
}

#[test]
fn test_search_monsters_empty_results() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let filters = MonsterFilters {
        name: Some("Nonexistent".to_string()),
        sizes: None,
        creature_types: None,
        alignments: None,
        sources: None,
        min_cr: None,
        max_cr: None,
        min_hp: None,
        max_hp: None,
        environment: None,
    };
    let results = service
        .search_monsters(filters)
        .expect("Search should succeed");

    assert!(results.is_empty(), "Should return empty results");
}

#[test]
fn test_get_monster_by_name_and_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let monster = service
        .get_monster_by_name_and_source("Goblin", "MM")
        .expect("Should get monster");

    assert!(monster.is_some());
    let monster = monster.unwrap();
    assert_eq!(monster.name, "Goblin");
    assert_eq!(monster.source, "MM");
}

#[test]
fn test_get_monster_by_name_and_source_not_found() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let monster = service
        .get_monster_by_name_and_source("Nonexistent", "MM")
        .expect("Should not error");

    assert!(monster.is_none());
}

#[test]
fn test_get_all_sizes() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let sizes = service.get_all_sizes().expect("Should get sizes");

    assert_eq!(sizes.len(), 4, "Should have 4 sizes (S, M, L, H)");
    assert!(sizes.contains(&"S".to_string()));
    assert!(sizes.contains(&"M".to_string()));
    assert!(sizes.contains(&"L".to_string()));
    assert!(sizes.contains(&"H".to_string()));
}

#[test]
fn test_get_all_creature_types() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let types = service
        .get_all_creature_types()
        .expect("Should get creature types");

    assert!(types.len() >= 5, "Should have multiple creature types");
    assert!(types.contains(&"Undead".to_string()));
    assert!(types.contains(&"Dragon".to_string()));
}

#[test]
fn test_get_cr_range() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let (min_cr, max_cr) = service.get_cr_range().expect("Should get CR range");

    assert!((min_cr - 0.125).abs() < 0.01, "Min CR should be 1/8");
    assert!((max_cr - 21.0).abs() < 0.01, "Max CR should be 21");
}

#[test]
fn test_get_monster_count_by_source() {
    let (mut conn, _temp_dir) = setup_test_db();
    let mut service = MonsterService::new(&mut conn);

    let counts = service
        .get_monster_count_by_source()
        .expect("Should get counts");

    let mm_count = counts.iter().find(|(s, _)| s == "MM").map(|(_, c)| *c);
    assert_eq!(mm_count, Some(10), "MM should have 10 monsters");
}
