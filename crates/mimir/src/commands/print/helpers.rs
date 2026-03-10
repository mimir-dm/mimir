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
    campaign_id: Option<&str>,
) -> InventoryItem {
    let equipped = inv_item.is_equipped();
    let attuned = inv_item.is_attuned();

    // Parse item data from either homebrew or catalog
    let parse_item_data = |data: &serde_json::Value| {
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
    };

    // Try homebrew first if source is "HB", then fall back to catalog
    let (item_type, damage, damage_type, armor_ac, finesse) =
        if inv_item.item_source == "HB" {
            if let Some(cid) = campaign_id {
                match mimir_core::dal::campaign::get_campaign_homebrew_item_by_name(
                    db, cid, &inv_item.item_name,
                ) {
                    Ok(Some(hb_item)) => {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&hb_item.data) {
                            let (it, dmg, dt, ac, fin) = parse_item_data(&data);
                            // Fall back to DB item_type if JSON data has no type field
                            let item_type = it.or_else(|| {
                                hb_item.item_type.as_deref().and_then(|t| {
                                    super::character::homebrew_item_type_to_code(
                                        t,
                                        data.as_object().unwrap_or(&serde_json::Map::new()),
                                    )
                                })
                            });
                            (item_type, dmg, dt, ac, fin)
                        } else {
                            (None, None, None, None, false)
                        }
                    }
                    _ => (None, None, None, None, false),
                }
            } else {
                (None, None, None, None, false)
            }
        } else {
            match catalog_dal::get_item_by_name(db, &inv_item.item_name, &inv_item.item_source) {
                Ok(Some(catalog_item)) => {
                    if let Ok(data) = catalog_item.parse_data() {
                        parse_item_data(&data)
                    } else {
                        (None, None, None, None, false)
                    }
                }
                _ => (None, None, None, None, false),
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    // ─── hit_die_for_class ──────────────────────────────────────────────

    #[test]
    fn hit_die_barbarian_d12() {
        assert_eq!(hit_die_for_class("Barbarian"), 12);
    }

    #[test]
    fn hit_die_fighter_d10() {
        assert_eq!(hit_die_for_class("Fighter"), 10);
    }

    #[test]
    fn hit_die_paladin_d10() {
        assert_eq!(hit_die_for_class("Paladin"), 10);
    }

    #[test]
    fn hit_die_ranger_d10() {
        assert_eq!(hit_die_for_class("Ranger"), 10);
    }

    #[test]
    fn hit_die_d8_classes() {
        for class in &["Bard", "Cleric", "Druid", "Monk", "Rogue", "Warlock", "Artificer"] {
            assert_eq!(hit_die_for_class(class), 8, "Expected d8 for {}", class);
        }
    }

    #[test]
    fn hit_die_wizard_d6() {
        assert_eq!(hit_die_for_class("Wizard"), 6);
    }

    #[test]
    fn hit_die_sorcerer_d6() {
        assert_eq!(hit_die_for_class("Sorcerer"), 6);
    }

    #[test]
    fn hit_die_unknown_defaults_d8() {
        assert_eq!(hit_die_for_class("Blood Hunter"), 8);
    }

    // ─── spellcasting_ability_for_class ─────────────────────────────────

    #[test]
    fn spell_ability_int_casters() {
        assert_eq!(spellcasting_ability_for_class("Wizard"), Some("INT"));
        assert_eq!(spellcasting_ability_for_class("Artificer"), Some("INT"));
    }

    #[test]
    fn spell_ability_wis_casters() {
        assert_eq!(spellcasting_ability_for_class("Cleric"), Some("WIS"));
        assert_eq!(spellcasting_ability_for_class("Druid"), Some("WIS"));
        assert_eq!(spellcasting_ability_for_class("Ranger"), Some("WIS"));
    }

    #[test]
    fn spell_ability_cha_casters() {
        assert_eq!(spellcasting_ability_for_class("Bard"), Some("CHA"));
        assert_eq!(spellcasting_ability_for_class("Paladin"), Some("CHA"));
        assert_eq!(spellcasting_ability_for_class("Sorcerer"), Some("CHA"));
        assert_eq!(spellcasting_ability_for_class("Warlock"), Some("CHA"));
    }

    #[test]
    fn spell_ability_non_casters() {
        assert_eq!(spellcasting_ability_for_class("Fighter"), None);
        assert_eq!(spellcasting_ability_for_class("Rogue"), None);
        assert_eq!(spellcasting_ability_for_class("Barbarian"), None);
        assert_eq!(spellcasting_ability_for_class("Monk"), None);
    }

    // ─── caster_level_multiplier ────────────────────────────────────────

    #[test]
    fn caster_multiplier_full_casters() {
        for class in &["Bard", "Cleric", "Druid", "Sorcerer", "Wizard"] {
            assert_eq!(caster_level_multiplier(class), 1.0, "Expected 1.0 for {}", class);
        }
    }

    #[test]
    fn caster_multiplier_half_casters() {
        for class in &["Artificer", "Paladin", "Ranger"] {
            assert_eq!(caster_level_multiplier(class), 0.5, "Expected 0.5 for {}", class);
        }
    }

    #[test]
    fn caster_multiplier_third_casters() {
        let third = 1.0 / 3.0;
        assert!((caster_level_multiplier("Eldritch Knight") - third).abs() < 0.001);
        assert!((caster_level_multiplier("Arcane Trickster") - third).abs() < 0.001);
    }

    #[test]
    fn caster_multiplier_non_casters() {
        assert_eq!(caster_level_multiplier("Fighter"), 0.0);
        assert_eq!(caster_level_multiplier("Rogue"), 0.0);
        assert_eq!(caster_level_multiplier("Barbarian"), 0.0);
    }

    // ─── spell_slots_for_caster_level ───────────────────────────────────

    #[test]
    fn spell_slots_level_0() {
        assert_eq!(spell_slots_for_caster_level(0), vec![0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn spell_slots_level_1() {
        assert_eq!(spell_slots_for_caster_level(1), vec![2, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn spell_slots_level_5() {
        assert_eq!(spell_slots_for_caster_level(5), vec![4, 3, 2, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn spell_slots_level_9() {
        assert_eq!(spell_slots_for_caster_level(9), vec![4, 3, 3, 3, 1, 0, 0, 0, 0]);
    }

    #[test]
    fn spell_slots_level_20() {
        assert_eq!(spell_slots_for_caster_level(20), vec![4, 3, 3, 3, 3, 2, 2, 1, 1]);
    }

    #[test]
    fn spell_slots_above_20_caps() {
        assert_eq!(spell_slots_for_caster_level(25), vec![4, 3, 3, 3, 3, 2, 2, 1, 1]);
    }

    #[test]
    fn spell_slots_negative_level() {
        assert_eq!(spell_slots_for_caster_level(-1), vec![0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    // ─── compute_hp_max ─────────────────────────────────────────────────

    #[test]
    fn hp_single_class_level_1() {
        let classes = vec![ClassInfo {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            level: 1,
            subclass_name: None,
            is_starting: true,
        }];
        // Level 1: max die (10) + CON mod (2) = 12
        assert_eq!(compute_hp_max(&classes, 2), 12);
    }

    #[test]
    fn hp_single_class_level_5() {
        let classes = vec![ClassInfo {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            level: 5,
            subclass_name: None,
            is_starting: true,
        }];
        // Level 1: 10 + 2 = 12
        // Levels 2-5: 4 × (5 + 1 + 2) = 4 × 8 = 32
        // Total: 44
        assert_eq!(compute_hp_max(&classes, 2), 44);
    }

    #[test]
    fn hp_multiclass() {
        let classes = vec![
            ClassInfo {
                class_name: "Fighter".to_string(),
                class_source: "PHB".to_string(),
                level: 5,
                subclass_name: None,
                is_starting: true,
            },
            ClassInfo {
                class_name: "Rogue".to_string(),
                class_source: "PHB".to_string(),
                level: 3,
                subclass_name: None,
                is_starting: false,
            },
        ];
        // Fighter starting: 10 + 2 = 12 (level 1) + 4 × (5+1+2) = 32 → 44
        // Rogue: 3 × (4+1+2) = 21
        // Total: 65
        assert_eq!(compute_hp_max(&classes, 2), 65);
    }

    #[test]
    fn hp_wizard_low_con() {
        let classes = vec![ClassInfo {
            class_name: "Wizard".to_string(),
            class_source: "PHB".to_string(),
            level: 1,
            subclass_name: None,
            is_starting: true,
        }];
        // Level 1: 6 + (-1) = 5
        assert_eq!(compute_hp_max(&classes, -1), 5);
    }

    #[test]
    fn hp_empty_classes() {
        assert_eq!(compute_hp_max(&[], 2), 0);
    }

    #[test]
    fn hp_minimum_is_1() {
        let classes = vec![ClassInfo {
            class_name: "Wizard".to_string(),
            class_source: "PHB".to_string(),
            level: 1,
            subclass_name: None,
            is_starting: true,
        }];
        // Level 1: 6 + (-10) = -4, clamped to 1
        assert_eq!(compute_hp_max(&classes, -10), 1);
    }

    // ─── compute_hit_die_string ─────────────────────────────────────────

    #[test]
    fn hit_die_string_single_class() {
        let classes = vec![ClassInfo {
            class_name: "Fighter".to_string(),
            class_source: "PHB".to_string(),
            level: 5,
            subclass_name: None,
            is_starting: true,
        }];
        assert_eq!(compute_hit_die_string(&classes), "5d10");
    }

    #[test]
    fn hit_die_string_multiclass() {
        let classes = vec![
            ClassInfo {
                class_name: "Fighter".to_string(),
                class_source: "PHB".to_string(),
                level: 5,
                subclass_name: None,
                is_starting: true,
            },
            ClassInfo {
                class_name: "Rogue".to_string(),
                class_source: "PHB".to_string(),
                level: 3,
                subclass_name: None,
                is_starting: false,
            },
        ];
        assert_eq!(compute_hit_die_string(&classes), "5d10 + 3d8");
    }

    #[test]
    fn hit_die_string_empty() {
        assert_eq!(compute_hit_die_string(&[]), "");
    }

    // ─── compute_ac ─────────────────────────────────────────────────────

    #[test]
    fn ac_no_armor() {
        // No armor: 10 + DEX mod
        assert_eq!(compute_ac(&[], 3), 13);
    }

    #[test]
    fn ac_light_armor() {
        let inv = vec![InventoryItem {
            name: "Leather Armor".to_string(),
            quantity: 1,
            equipped: true,
            item_type: Some("LA".to_string()),
            armor_ac: Some(11),
            ..Default::default()
        }];
        // Light: base + full DEX mod
        assert_eq!(compute_ac(&inv, 3), 14);
    }

    #[test]
    fn ac_medium_armor_dex_capped() {
        let inv = vec![InventoryItem {
            name: "Breastplate".to_string(),
            quantity: 1,
            equipped: true,
            item_type: Some("MA".to_string()),
            armor_ac: Some(14),
            ..Default::default()
        }];
        // Medium: base + min(DEX, 2)
        assert_eq!(compute_ac(&inv, 4), 16); // DEX 4 capped at 2
    }

    #[test]
    fn ac_medium_armor_low_dex() {
        let inv = vec![InventoryItem {
            name: "Breastplate".to_string(),
            quantity: 1,
            equipped: true,
            item_type: Some("MA".to_string()),
            armor_ac: Some(14),
            ..Default::default()
        }];
        // Medium with low DEX: base + DEX (since DEX < 2)
        assert_eq!(compute_ac(&inv, 1), 15);
    }

    #[test]
    fn ac_heavy_armor_ignores_dex() {
        let inv = vec![InventoryItem {
            name: "Chain Mail".to_string(),
            quantity: 1,
            equipped: true,
            item_type: Some("HA".to_string()),
            armor_ac: Some(16),
            ..Default::default()
        }];
        // Heavy: flat AC, no DEX
        assert_eq!(compute_ac(&inv, 4), 16);
    }

    #[test]
    fn ac_shield_adds_bonus() {
        let inv = vec![
            InventoryItem {
                name: "Chain Mail".to_string(),
                quantity: 1,
                equipped: true,
                item_type: Some("HA".to_string()),
                armor_ac: Some(16),
                ..Default::default()
            },
            InventoryItem {
                name: "Shield".to_string(),
                quantity: 1,
                equipped: true,
                item_type: Some("S".to_string()),
                armor_ac: Some(2),
                ..Default::default()
            },
        ];
        assert_eq!(compute_ac(&inv, 0), 18);
    }

    #[test]
    fn ac_unequipped_armor_ignored() {
        let inv = vec![InventoryItem {
            name: "Plate Armor".to_string(),
            quantity: 1,
            equipped: false,
            item_type: Some("HA".to_string()),
            armor_ac: Some(18),
            ..Default::default()
        }];
        // Not equipped → uses default 10 + DEX
        assert_eq!(compute_ac(&inv, 2), 12);
    }

    // ─── max_spell_level_for_class ──────────────────────────────────────

    #[test]
    fn max_spell_level_wizard() {
        assert_eq!(max_spell_level_for_class("Wizard", 1), 1);
        assert_eq!(max_spell_level_for_class("Wizard", 3), 2);
        assert_eq!(max_spell_level_for_class("Wizard", 5), 3);
        assert_eq!(max_spell_level_for_class("Wizard", 9), 5);
        assert_eq!(max_spell_level_for_class("Wizard", 17), 9);
        assert_eq!(max_spell_level_for_class("Wizard", 20), 9); // capped at 9
    }

    #[test]
    fn max_spell_level_paladin() {
        assert_eq!(max_spell_level_for_class("Paladin", 1), 0); // No casting at level 1
        assert_eq!(max_spell_level_for_class("Paladin", 2), 1);
        assert_eq!(max_spell_level_for_class("Paladin", 5), 2);
        assert_eq!(max_spell_level_for_class("Paladin", 9), 3);
        assert_eq!(max_spell_level_for_class("Paladin", 17), 5);
    }

    #[test]
    fn max_spell_level_warlock() {
        assert_eq!(max_spell_level_for_class("Warlock", 1), 1);
        assert_eq!(max_spell_level_for_class("Warlock", 3), 2);
        assert_eq!(max_spell_level_for_class("Warlock", 5), 3);
        assert_eq!(max_spell_level_for_class("Warlock", 9), 5);
        assert_eq!(max_spell_level_for_class("Warlock", 20), 5); // Pact slots cap at 5
    }

    #[test]
    fn max_spell_level_artificer() {
        assert_eq!(max_spell_level_for_class("Artificer", 1), 1); // Starts at 1
        assert_eq!(max_spell_level_for_class("Artificer", 5), 2);
        assert_eq!(max_spell_level_for_class("Artificer", 17), 5);
    }

    #[test]
    fn max_spell_level_third_caster() {
        assert_eq!(max_spell_level_for_class("Eldritch Knight", 2), 0);
        assert_eq!(max_spell_level_for_class("Eldritch Knight", 3), 1);
        assert_eq!(max_spell_level_for_class("Eldritch Knight", 7), 2);
        assert_eq!(max_spell_level_for_class("Eldritch Knight", 19), 4);
    }

    #[test]
    fn max_spell_level_non_caster() {
        assert_eq!(max_spell_level_for_class("Fighter", 20), 0);
        assert_eq!(max_spell_level_for_class("Rogue", 20), 0);
        assert_eq!(max_spell_level_for_class("Barbarian", 20), 0);
    }
}
