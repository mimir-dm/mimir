//! D&D 5e Character Sheet Computation Helpers
//!
//! Helper functions for computing character statistics like hit points,
//! spell slots, and armor class for PDF character sheets.

use mimir_core::dal::catalog as catalog_dal;
use mimir_print::sections::{ClassInfo, InventoryItem};

/// Get the hit die size for a class (e.g. "Fighter" -> 10)
pub fn hit_die_for_class(class_name: &str) -> i32 {
    match class_name {
        "Barbarian" => 12,
        "Fighter" | "Paladin" | "Ranger" => 10,
        "Artificer" | "Bard" | "Cleric" | "Druid" | "Monk" | "Rogue" | "Warlock" => 8,
        "Sorcerer" | "Wizard" => 6,
        _ => 8, // default to d8
    }
}

/// Get the spellcasting ability abbreviation for a class, if it's a caster
pub fn spellcasting_ability_for_class(class_name: &str) -> Option<&'static str> {
    match class_name {
        "Artificer" | "Wizard" => Some("INT"),
        "Cleric" | "Druid" | "Ranger" => Some("WIS"),
        "Bard" | "Paladin" | "Sorcerer" | "Warlock" => Some("CHA"),
        _ => None,
    }
}

/// Get the caster level multiplier for multiclass spell slot calculation
pub fn caster_level_multiplier(class_name: &str) -> f64 {
    match class_name {
        "Bard" | "Cleric" | "Druid" | "Sorcerer" | "Wizard" => 1.0,
        "Artificer" | "Paladin" | "Ranger" => 0.5,
        // Third casters (subclass-based, but handle if they appear)
        "Eldritch Knight" | "Arcane Trickster" => 1.0 / 3.0,
        _ => 0.0,
    }
}

/// Standard 5e spell slot table indexed by caster level (1-20), returning slots for levels 1-9
pub fn spell_slots_for_caster_level(caster_level: i32) -> Vec<i32> {
    match caster_level {
        1  => vec![2, 0, 0, 0, 0, 0, 0, 0, 0],
        2  => vec![3, 0, 0, 0, 0, 0, 0, 0, 0],
        3  => vec![4, 2, 0, 0, 0, 0, 0, 0, 0],
        4  => vec![4, 3, 0, 0, 0, 0, 0, 0, 0],
        5  => vec![4, 3, 2, 0, 0, 0, 0, 0, 0],
        6  => vec![4, 3, 3, 0, 0, 0, 0, 0, 0],
        7  => vec![4, 3, 3, 1, 0, 0, 0, 0, 0],
        8  => vec![4, 3, 3, 2, 0, 0, 0, 0, 0],
        9  => vec![4, 3, 3, 3, 1, 0, 0, 0, 0],
        10 => vec![4, 3, 3, 3, 2, 0, 0, 0, 0],
        11 => vec![4, 3, 3, 3, 2, 1, 0, 0, 0],
        12 => vec![4, 3, 3, 3, 2, 1, 0, 0, 0],
        13 => vec![4, 3, 3, 3, 2, 1, 1, 0, 0],
        14 => vec![4, 3, 3, 3, 2, 1, 1, 0, 0],
        15 => vec![4, 3, 3, 3, 2, 1, 1, 1, 0],
        16 => vec![4, 3, 3, 3, 2, 1, 1, 1, 0],
        17 => vec![4, 3, 3, 3, 2, 1, 1, 1, 1],
        18 => vec![4, 3, 3, 3, 3, 1, 1, 1, 1],
        19 => vec![4, 3, 3, 3, 3, 2, 1, 1, 1],
        20 => vec![4, 3, 3, 3, 3, 2, 2, 1, 1],
        _ if caster_level < 1 => vec![0, 0, 0, 0, 0, 0, 0, 0, 0],
        _ => vec![4, 3, 3, 3, 3, 2, 2, 1, 1], // cap at 20
    }
}

/// Compute hit points max: level 1 = max die + CON mod, subsequent = avg die + CON mod
pub fn compute_hp_max(classes: &[ClassInfo], con_mod: i32) -> i32 {
    if classes.is_empty() {
        return 0;
    }

    let mut hp = 0;
    let mut is_first_level = true;

    // Find starting class for first-level HP
    let starting = classes.iter().find(|c| c.is_starting).unwrap_or(&classes[0]);
    let starting_die = hit_die_for_class(&starting.class_name);
    // Level 1: max die + CON mod
    hp += starting_die + con_mod;
    let starting_remaining = starting.level - 1;
    // Remaining levels of starting class: avg + CON mod
    hp += starting_remaining * (starting_die / 2 + 1 + con_mod);

    for class in classes {
        if class.is_starting || (!is_first_level && std::ptr::eq(class, &classes[0])) {
            is_first_level = false;
            continue;
        }
        is_first_level = false;
        let die = hit_die_for_class(&class.class_name);
        hp += class.level * (die / 2 + 1 + con_mod);
    }

    hp.max(1)
}

/// Build hit die string like "5d10 + 3d8"
pub fn compute_hit_die_string(classes: &[ClassInfo]) -> String {
    classes
        .iter()
        .map(|c| format!("{}d{}", c.level, hit_die_for_class(&c.class_name)))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Enrich an inventory item with catalog data (weapon stats, armor AC, etc.)
pub fn enrich_inventory_item(
    db: &mut diesel::SqliteConnection,
    inv_item: &mimir_core::models::CharacterInventory,
) -> InventoryItem {
    let equipped = inv_item.is_equipped();
    let attuned = inv_item.is_attuned();

    // Try to look up catalog item for weapon/armor stats
    let (item_type, damage, damage_type, armor_ac, finesse) =
        match catalog_dal::get_item_by_name(db, &inv_item.item_name, &inv_item.item_source) {
            Ok(Some(catalog_item)) => {
                if let Ok(data) = catalog_item.parse_data() {
                    let it = data.get("type")
                        .or_else(|| data.get("item_type"))
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let dmg = data.get("dmg1").and_then(|v| v.as_str()).map(String::from);
                    let dt = data.get("dmg_type")
                        .or_else(|| data.get("dmgType"))
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let ac = data.get("ac").and_then(|v| v.as_i64()).map(|v| v as i32);
                    let fin = data.get("property")
                        .and_then(|v| v.as_array())
                        .map_or(false, |arr| arr.iter().any(|p| p.as_str() == Some("F")));
                    (it, dmg, dt, ac, fin)
                } else {
                    (None, None, None, None, false)
                }
            }
            _ => (None, None, None, None, false),
        };

    InventoryItem {
        name: inv_item.item_name.clone(),
        quantity: inv_item.quantity,
        equipped,
        attuned,
        item_type,
        damage,
        damage_type,
        armor_ac,
        finesse,
    }
}

/// Compute AC from equipped armor and DEX modifier
pub fn compute_ac(inventory: &[InventoryItem], dex_mod: i32) -> i32 {
    let mut base_ac = 10 + dex_mod; // Default: no armor
    let mut shield_bonus = 0;

    for item in inventory {
        if !item.equipped {
            continue;
        }
        match item.item_type.as_deref() {
            Some("LA") => {
                // Light armor: base AC + DEX mod
                if let Some(ac) = item.armor_ac {
                    base_ac = ac + dex_mod;
                }
            }
            Some("MA") => {
                // Medium armor: base AC + DEX mod (max 2)
                if let Some(ac) = item.armor_ac {
                    base_ac = ac + dex_mod.min(2);
                }
            }
            Some("HA") => {
                // Heavy armor: flat AC, no DEX
                if let Some(ac) = item.armor_ac {
                    base_ac = ac;
                }
            }
            Some("S") => {
                // Shield: +2 (or whatever the AC value is)
                shield_bonus = item.armor_ac.unwrap_or(2);
            }
            _ => {}
        }
    }

    base_ac + shield_bonus
}

/// Calculate the maximum spell level a class can cast at a given class level.
/// Returns 0 if the class has no spellcasting at that level.
pub fn max_spell_level_for_class(class_name: &str, class_level: i32) -> i32 {
    match class_name {
        // Full casters: spell level = (class_level + 1) / 2, max 9
        "Bard" | "Cleric" | "Druid" | "Sorcerer" | "Wizard" => {
            ((class_level + 1) / 2).min(9)
        }
        // Warlock: uses pact magic, different progression but similar max
        "Warlock" => {
            match class_level {
                1 => 1,
                2 => 1,
                3..=4 => 2,
                5..=6 => 3,
                7..=8 => 4,
                9..=10 => 5,
                11..=16 => 5, // Mystic Arcanum gives 6-9 but as 1/day, slots stay at 5
                17..=20 => 5,
                _ => 0,
            }
        }
        // Half casters: start at 2, spell level = (class_level + 1) / 4 + some offset
        "Paladin" | "Ranger" => {
            if class_level < 2 {
                0
            } else {
                match class_level {
                    2..=4 => 1,
                    5..=8 => 2,
                    9..=12 => 3,
                    13..=16 => 4,
                    17..=20 => 5,
                    _ => 0,
                }
            }
        }
        // Artificer: half caster but starts at 1
        "Artificer" => {
            match class_level {
                1..=4 => 1,
                5..=8 => 2,
                9..=12 => 3,
                13..=16 => 4,
                17..=20 => 5,
                _ => 0,
            }
        }
        // Third casters (subclasses, but if they appear as class names)
        "Eldritch Knight" | "Arcane Trickster" => {
            if class_level < 3 {
                0
            } else {
                match class_level {
                    3..=6 => 1,
                    7..=12 => 2,
                    13..=18 => 3,
                    19..=20 => 4,
                    _ => 0,
                }
            }
        }
        // Non-casters or unknown classes
        _ => 0,
    }
}
