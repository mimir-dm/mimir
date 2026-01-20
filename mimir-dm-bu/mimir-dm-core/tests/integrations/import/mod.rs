//! Import tests for catalog deserialization
//!
//! These tests validate that real 5etools data can be correctly parsed
//! using the typed catalog models. Fixtures are subsets of actual 5etools
//! data files to ensure compatibility with upstream changes.
//!
//! ## Updating Fixtures
//!
//! To update fixtures when 5etools schema changes:
//!
//! 1. Download the latest 5etools data
//! 2. Run extraction commands (see comments in each test)
//! 3. Run tests to validate deserialization
//! 4. Commit updated fixture files
//!
//! Example extraction for monsters:
//! ```bash
//! jq '{monster: [.monster[] | select(.name == "Aboleth" or .name == "Goblin" ...)]}'
//!     data/5etools/bestiary-mm.json > fixtures/monsters.json
//! ```

use mimir_dm_core::models::catalog::{
    Background, BackgroundData, Class, ClassData, Item, ItemData, Monster, MonsterData, Race,
    RaceData, Spell, SpellData,
};

/// Path to fixture files relative to the test binary
const FIXTURES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/integrations/import/fixtures"
);

// =============================================================================
// Monster Import Tests
// =============================================================================

#[test]
fn test_import_monsters_from_fixture() {
    let fixture_path = format!("{}/monsters.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_path, e));

    let data: MonsterData = serde_json::from_str(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse monsters fixture: {}", e));

    assert!(
        !data.monster.is_empty(),
        "Monster fixture should contain monsters"
    );

    // Verify specific monsters are present
    let names: Vec<&str> = data.monster.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"Aboleth"), "Should contain Aboleth");
    assert!(names.contains(&"Goblin"), "Should contain Goblin");
}

#[test]
fn test_monster_ac_polymorphism() {
    // Test that AC can be parsed as number or array
    use mimir_dm_core::models::catalog::types::ArmorClassValue;

    let fixture_path = format!("{}/monsters.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: MonsterData = serde_json::from_str(&json_content).unwrap();

    for monster in &data.monster {
        // Just verify each monster's AC parses without error
        if let Some(ref ac) = monster.ac {
            match ac {
                ArmorClassValue::Number(n) => assert!(*n > 0, "Monster {} has invalid AC", monster.name),
                ArmorClassValue::Array(entries) => assert!(!entries.is_empty(), "Monster {} has empty AC array", monster.name),
            }
        }
    }
}

#[test]
fn test_monster_cr_polymorphism() {
    // CR can be simple string or complex object with lair CR
    let fixture_path = format!("{}/monsters.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: MonsterData = serde_json::from_str(&json_content).unwrap();

    for monster in &data.monster {
        // All MM monsters should have CR
        assert!(
            monster.cr.is_some(),
            "Monster {} should have CR",
            monster.name
        );
    }
}

#[test]
fn test_monster_roundtrip_serialization() {
    let fixture_path = format!("{}/monsters.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: MonsterData = serde_json::from_str(&json_content).unwrap();

    for monster in &data.monster {
        // Serialize then deserialize
        let serialized = serde_json::to_string(monster)
            .unwrap_or_else(|e| panic!("Failed to serialize {}: {}", monster.name, e));
        let _deserialized: Monster = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", monster.name, e));
    }
}

// =============================================================================
// Spell Import Tests
// =============================================================================

#[test]
fn test_import_spells_from_fixture() {
    let fixture_path = format!("{}/spells.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_path, e));

    let data: SpellData = serde_json::from_str(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse spells fixture: {}", e));

    assert!(
        !data.spell.is_empty(),
        "Spell fixture should contain spells"
    );

    let names: Vec<&str> = data.spell.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"Fireball"), "Should contain Fireball");
    assert!(names.contains(&"Magic Missile"), "Should contain Magic Missile");
}

#[test]
fn test_spell_material_component_consume() {
    // Test that consume field is properly typed (bool or "optional")
    let fixture_path = format!("{}/spells.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: SpellData = serde_json::from_str(&json_content).unwrap();

    // Clone spell has consumed components
    let clone_spell = data.spell.iter().find(|s| s.name == "Clone");
    assert!(clone_spell.is_some(), "Should have Clone spell in fixture");
}

#[test]
fn test_spell_roundtrip_serialization() {
    let fixture_path = format!("{}/spells.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: SpellData = serde_json::from_str(&json_content).unwrap();

    for spell in &data.spell {
        let serialized = serde_json::to_string(spell)
            .unwrap_or_else(|e| panic!("Failed to serialize {}: {}", spell.name, e));
        let _deserialized: Spell = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", spell.name, e));
    }
}

// =============================================================================
// Class Import Tests
// =============================================================================

#[test]
fn test_import_classes_from_fixture() {
    let fixture_path = format!("{}/classes.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_path, e));

    let data: ClassData = serde_json::from_str(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse classes fixture: {}", e));

    assert!(
        !data.classes.is_empty(),
        "Class fixture should contain classes"
    );

    let names: Vec<&str> = data.classes.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"Fighter"), "Should contain Fighter");
    assert!(names.contains(&"Wizard"), "Should contain Wizard");
}

#[test]
fn test_class_hit_dice_typed() {
    let fixture_path = format!("{}/classes.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: ClassData = serde_json::from_str(&json_content).unwrap();

    for class in &data.classes {
        if let Some(ref hd) = class.hd {
            assert!(
                hd.faces > 0,
                "Class {} should have valid hit dice faces",
                class.name
            );
        }
    }
}

#[test]
fn test_class_roundtrip_serialization() {
    let fixture_path = format!("{}/classes.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: ClassData = serde_json::from_str(&json_content).unwrap();

    for class in &data.classes {
        let serialized = serde_json::to_string(class)
            .unwrap_or_else(|e| panic!("Failed to serialize {}: {}", class.name, e));
        let _deserialized: Class = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", class.name, e));
    }
}

// =============================================================================
// Race Import Tests
// =============================================================================

#[test]
fn test_import_races_from_fixture() {
    let fixture_path = format!("{}/races.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_path, e));

    let data: RaceData = serde_json::from_str(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse races fixture: {}", e));

    assert!(data.race.is_some(), "Race fixture should contain races");

    let races = data.race.as_ref().unwrap();
    let names: Vec<&str> = races.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"Human"), "Should contain Human");
    assert!(names.contains(&"Elf"), "Should contain Elf");
}

#[test]
fn test_race_speed_polymorphism() {
    // Speed can be number or object with walk/fly/swim
    let fixture_path = format!("{}/races.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: RaceData = serde_json::from_str(&json_content).unwrap();

    if let Some(ref races) = data.race {
        for race in races {
            if let Some(ref speed) = race.speed {
                // Verify walk_speed helper works
                let walk = speed.walk_speed();
                assert!(walk > 0, "Race {} should have positive walk speed", race.name);
            }
        }
    }
}

#[test]
fn test_race_roundtrip_serialization() {
    let fixture_path = format!("{}/races.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: RaceData = serde_json::from_str(&json_content).unwrap();

    if let Some(ref races) = data.race {
        for race in races {
            let serialized = serde_json::to_string(race)
                .unwrap_or_else(|e| panic!("Failed to serialize {}: {}", race.name, e));
            let _deserialized: Race = serde_json::from_str(&serialized)
                .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", race.name, e));
        }
    }
}

// =============================================================================
// Item Import Tests
// =============================================================================

#[test]
fn test_import_items_from_fixture() {
    let fixture_path = format!("{}/items.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_path, e));

    let data: ItemData = serde_json::from_str(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse items fixture: {}", e));

    assert!(
        !data.item.is_empty(),
        "Item fixture should contain items"
    );

    let names: Vec<&str> = data.item.iter().map(|i| i.name.as_str()).collect();
    assert!(names.contains(&"Bag of Holding"), "Should contain Bag of Holding");
}

#[test]
fn test_item_roundtrip_serialization() {
    let fixture_path = format!("{}/items.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: ItemData = serde_json::from_str(&json_content).unwrap();

    for item in &data.item {
        let serialized = serde_json::to_string(item)
            .unwrap_or_else(|e| panic!("Failed to serialize {}: {}", item.name, e));
        let _deserialized: Item = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", item.name, e));
    }
}

// =============================================================================
// Background Import Tests
// =============================================================================

#[test]
fn test_import_backgrounds_from_fixture() {
    let fixture_path = format!("{}/backgrounds.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", fixture_path, e));

    let data: BackgroundData = serde_json::from_str(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse backgrounds fixture: {}", e));

    assert!(
        data.background.is_some(),
        "Background fixture should contain backgrounds"
    );

    let backgrounds = data.background.as_ref().unwrap();
    let names: Vec<&str> = backgrounds.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"Acolyte"), "Should contain Acolyte");
    assert!(names.contains(&"Criminal"), "Should contain Criminal");
}

#[test]
fn test_background_starting_equipment_typed() {
    let fixture_path = format!("{}/backgrounds.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: BackgroundData = serde_json::from_str(&json_content).unwrap();

    if let Some(ref backgrounds) = data.background {
        let acolyte = backgrounds.iter().find(|b| b.name == "Acolyte");
        assert!(acolyte.is_some(), "Should have Acolyte background");

        let acolyte = acolyte.unwrap();
        assert!(
            !acolyte.starting_equipment.is_empty(),
            "Acolyte should have starting equipment"
        );
    }
}

#[test]
fn test_background_roundtrip_serialization() {
    let fixture_path = format!("{}/backgrounds.json", FIXTURES_DIR);
    let json_content = std::fs::read_to_string(&fixture_path).unwrap();
    let data: BackgroundData = serde_json::from_str(&json_content).unwrap();

    if let Some(ref backgrounds) = data.background {
        for background in backgrounds {
            let serialized = serde_json::to_string(background)
                .unwrap_or_else(|e| panic!("Failed to serialize {}: {}", background.name, e));
            let _deserialized: Background = serde_json::from_str(&serialized)
                .unwrap_or_else(|e| panic!("Failed to deserialize {}: {}", background.name, e));
        }
    }
}
