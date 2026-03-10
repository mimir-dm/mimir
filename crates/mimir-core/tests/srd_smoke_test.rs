//! Smoke tests for the SRD-seeded test database.
//!
//! Verifies that the test harness correctly loads all SRD fixture data
//! and that basic queries work against the seeded database.

mod common;

use mimir_core::dal::catalog;

#[test]
fn test_srd_db_has_all_classes() {
    let mut conn = common::setup_srd_db();

    let classes = catalog::list_classes(&mut conn).expect("Failed to list classes");
    assert_eq!(classes.len(), 12, "Expected 12 SRD classes, got {}", classes.len());

    // Verify specific classes exist
    let fighter = catalog::get_class_by_name(&mut conn, "Fighter", "PHB")
        .expect("Query failed")
        .expect("Fighter should exist");
    assert_eq!(fighter.name, "Fighter");
    assert_eq!(fighter.source, "PHB");

    // Verify data blob contains expected fields
    let data: serde_json::Value = serde_json::from_str(&fighter.data).unwrap();
    assert!(data.get("hd").is_some(), "Fighter data should contain hit dice");
}

#[test]
fn test_srd_db_has_subclasses() {
    let mut conn = common::setup_srd_db();

    let champion = catalog::get_subclass_by_name(&mut conn, "Champion", "Fighter", "PHB")
        .expect("Query failed")
        .expect("Champion should exist");
    assert_eq!(champion.class_name, "Fighter");

    let thief = catalog::get_subclass_by_name(&mut conn, "Thief", "Rogue", "PHB")
        .expect("Query failed")
        .expect("Thief should exist");
    assert_eq!(thief.class_name, "Rogue");

    let evocation = catalog::get_subclass_by_name(&mut conn, "School of Evocation", "Wizard", "PHB")
        .expect("Query failed")
        .expect("School of Evocation should exist");
    assert_eq!(evocation.class_name, "Wizard");

    let life = catalog::get_subclass_by_name(&mut conn, "Life Domain", "Cleric", "PHB")
        .expect("Query failed")
        .expect("Life Domain should exist");
    assert_eq!(life.class_name, "Cleric");
}

#[test]
fn test_srd_db_has_class_features() {
    let mut conn = common::setup_srd_db();

    // Fighter should have features
    let features = catalog::list_class_features_by_class(&mut conn, "Fighter", "PHB")
        .expect("Failed to list Fighter features");
    assert!(!features.is_empty(), "Fighter should have class features");

    // Check that features span multiple levels
    let levels: Vec<i32> = features.iter().map(|f| f.level).collect();
    assert!(levels.contains(&1), "Fighter should have level 1 features");
    assert!(levels.contains(&2), "Fighter should have level 2 features");
}

#[test]
fn test_srd_db_has_subclass_features_with_children() {
    let mut conn = common::setup_srd_db();

    // Thief should have child features like Fast Hands
    let features = catalog::list_subclass_features_by_subclass(&mut conn, "Thief", "PHB")
        .expect("Failed to list Thief features");
    let names: Vec<&str> = features.iter().map(|f| f.name.as_str()).collect();

    assert!(names.contains(&"Fast Hands"), "Thief should have Fast Hands feature");
    assert!(
        names.contains(&"Second-Story Work"),
        "Thief should have Second-Story Work feature"
    );

    // Champion should have Improved Critical
    let champ_features = catalog::list_subclass_features_by_subclass(&mut conn, "Champion", "PHB")
        .expect("Failed to list Champion features");
    let champ_names: Vec<&str> = champ_features.iter().map(|f| f.name.as_str()).collect();
    assert!(
        champ_names.contains(&"Improved Critical"),
        "Champion should have Improved Critical feature"
    );
}

#[test]
fn test_srd_db_has_backgrounds() {
    let mut conn = common::setup_srd_db();

    let acolyte = catalog::get_background_by_name(&mut conn, "Acolyte", "PHB")
        .expect("Query failed")
        .expect("Acolyte should exist");
    assert_eq!(acolyte.name, "Acolyte");

    let soldier = catalog::get_background_by_name(&mut conn, "Soldier", "PHB")
        .expect("Query failed")
        .expect("Soldier should exist");
    assert_eq!(soldier.name, "Soldier");
}

#[test]
fn test_srd_db_has_races() {
    let mut conn = common::setup_srd_db();

    let human = catalog::get_race_by_name(&mut conn, "Human", "PHB")
        .expect("Query failed")
        .expect("Human should exist");
    assert_eq!(human.name, "Human");

    let elf = catalog::get_race_by_name(&mut conn, "Elf", "PHB")
        .expect("Query failed")
        .expect("Elf should exist");
    assert_eq!(elf.name, "Elf");
}

#[test]
fn test_srd_db_has_spells() {
    let mut conn = common::setup_srd_db();

    let spells = catalog::list_spells(&mut conn).expect("Failed to list spells");
    assert!(spells.len() >= 20, "Expected at least 20 SRD spells, got {}", spells.len());

    // Check specific spells
    let fireball = catalog::get_spell_by_name(&mut conn, "Fireball", "PHB")
        .expect("Query failed")
        .expect("Fireball should exist");
    assert_eq!(fireball.level, 3);

    // Check cantrip
    let firebolt = catalog::get_spell_by_name(&mut conn, "Fire Bolt", "PHB")
        .expect("Query failed")
        .expect("Fire Bolt should exist");
    assert_eq!(firebolt.level, 0);
}

#[test]
fn test_srd_db_has_items() {
    let mut conn = common::setup_srd_db();

    let longsword = catalog::get_item_by_name(&mut conn, "Longsword", "PHB")
        .expect("Query failed")
        .expect("Longsword should exist");
    assert_eq!(longsword.name, "Longsword");
}

#[test]
fn test_srd_db_has_monsters() {
    let mut conn = common::setup_srd_db();

    let monsters = catalog::list_monsters(&mut conn).expect("Failed to list monsters");
    assert!(
        monsters.len() >= 10,
        "Expected at least 10 SRD monsters, got {}",
        monsters.len()
    );

    // Goblin is sourced from MM in the SRD fixtures
    let goblin = catalog::get_monster_by_name(&mut conn, "Goblin", "MM")
        .expect("Query failed");
    assert!(goblin.is_some(), "Goblin should exist in the DB");
}

#[test]
fn test_srd_db_data_blobs_are_valid_json() {
    let mut conn = common::setup_srd_db();

    // Verify all class data blobs parse as valid JSON
    let classes = catalog::list_classes(&mut conn).expect("Failed to list classes");
    for class in &classes {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&class.data);
        assert!(
            parsed.is_ok(),
            "Class {} has invalid JSON data: {}",
            class.name,
            parsed.unwrap_err()
        );
    }

    // Verify spell data blobs
    let spells = catalog::list_spells(&mut conn).expect("Failed to list spells");
    for spell in &spells {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&spell.data);
        assert!(
            parsed.is_ok(),
            "Spell {} has invalid JSON data: {}",
            spell.name,
            parsed.unwrap_err()
        );
    }
}

#[test]
fn test_srd_db_entity_to_json_roundtrip() {
    let mut conn = common::setup_srd_db();

    // The data blob should contain name and source from extraction
    let fighter = catalog::get_class_by_name(&mut conn, "Fighter", "PHB")
        .expect("Query failed")
        .expect("Fighter should exist");
    let data: serde_json::Value = serde_json::from_str(&fighter.data).unwrap();

    assert_eq!(data["name"].as_str(), Some("Fighter"));
    assert_eq!(data["source"].as_str(), Some("PHB"));
}
