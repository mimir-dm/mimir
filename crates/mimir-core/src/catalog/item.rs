//! Item catalog extraction types
//!
//! Types for deserializing 5etools item JSON data.

use super::types::SrdValue;
use serde::{Deserialize, Deserializer, Serialize};

/// A D&D 5e item from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    // Item classification
    #[serde(default, rename = "type")]
    pub item_type: Option<String>,
    #[serde(default)]
    pub rarity: Option<ItemRarity>,
    #[serde(default)]
    pub tier: Option<String>,

    // Physical properties
    #[serde(default)]
    pub weight: Option<f32>,
    #[serde(default)]
    pub value: Option<f64>, // Value in copper pieces
    #[serde(default)]
    pub age: Option<String>,

    // Armor properties
    #[serde(default)]
    pub ac: Option<i32>,
    #[serde(default)]
    pub strength: Option<String>,
    #[serde(default)]
    pub stealth: Option<bool>,

    // Weapon properties
    #[serde(default)]
    pub dmg1: Option<String>,
    #[serde(default)]
    pub dmg2: Option<String>,
    #[serde(default)]
    pub dmg_type: Option<String>,
    #[serde(default)]
    pub property: Option<Vec<String>>,
    #[serde(default)]
    pub range: Option<String>,
    #[serde(default)]
    pub reload: Option<u8>,
    #[serde(default)]
    pub weapon_category: Option<String>,

    // Magic item properties
    #[serde(default, deserialize_with = "deserialize_attunement")]
    pub req_attune: Option<String>,
    #[serde(default)]
    pub req_attune_tags: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub bonus_weapon: Option<String>,
    #[serde(default)]
    pub bonus_ac: Option<String>,
    #[serde(default)]
    pub bonus_saving_throw: Option<String>,
    #[serde(default)]
    pub bonus_spell_attack: Option<String>,

    // Container properties
    #[serde(default)]
    pub container_capacity: Option<serde_json::Value>,

    // Description (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Flags
    #[serde(default)]
    pub srd: Option<SrdValue>,
    #[serde(default)]
    pub basic_rules: Option<bool>,
    #[serde(default)]
    pub wondrous: Option<bool>,
    #[serde(default)]
    pub tattoo: Option<bool>,
    #[serde(default)]
    pub sentient: Option<bool>,
    #[serde(default)]
    pub curse: Option<bool>,
    #[serde(default)]
    pub poison: Option<bool>,

    // Copy directive
    #[serde(default, rename = "_copy")]
    pub copy: Option<serde_json::Value>,
}

/// Item rarity - can be string or "none".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemRarity {
    None,
    Common,
    Uncommon,
    Rare,
    #[serde(rename = "very rare")]
    VeryRare,
    Legendary,
    Artifact,
    Unknown,
    Varies,
    #[serde(other)]
    Other,
}

impl ItemRarity {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::VeryRare => "very rare",
            Self::Legendary => "legendary",
            Self::Artifact => "artifact",
            Self::Unknown => "unknown",
            Self::Varies => "varies",
            Self::Other => "other",
        }
    }
}

/// Container for item data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    #[serde(default)]
    pub item: Vec<Item>,
    #[serde(default, rename = "itemGroup")]
    pub item_group: Option<Vec<serde_json::Value>>,
}

/// Base item data (weapons, armor without magic properties).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseItemData {
    #[serde(default, rename = "baseitem")]
    pub base_item: Vec<Item>,
}

/// Custom deserializer for reqAttune field that can be boolean or string.
fn deserialize_attunement<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value = Option::<serde_json::Value>::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(serde_json::Value::Bool(true)) => Ok(Some("true".to_string())),
        Some(serde_json::Value::Bool(false)) => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(other) => Err(D::Error::custom(format!(
            "Expected boolean or string for reqAttune, got {:?}",
            other
        ))),
    }
}

/// Get human-readable type name from 5etools type code.
pub fn get_item_type_name(item_type: &str) -> &'static str {
    // Handle complex type formats like "$G|DMG", "EXP|DMG", etc.
    let base_type = item_type.split('|').next().unwrap_or(item_type);

    match base_type {
        // Treasure types ($ prefix)
        "$" => "Treasure",
        "$A" => "Art Object",
        "$C" => "Coinage",
        "$G" => "Gemstone",
        // Regular item types
        "A" => "Ammunition",
        "AF" => "Futuristic Ammunition",
        "AIR" => "Aircraft",
        "AT" => "Artisan's Tools",
        "EXP" => "Explosive",
        "FD" => "Food & Drink",
        "G" => "Adventuring Gear",
        "GS" => "Gaming Set",
        "GV" => "Generic Variant",
        "HA" => "Heavy Armor",
        "IDG" => "Illegal Drug",
        "INS" => "Musical Instrument",
        "LA" => "Light Armor",
        "M" => "Melee Weapon",
        "MA" => "Medium Armor",
        "MNT" => "Mount",
        "OTH" => "Other",
        "P" => "Potion",
        "R" => "Ranged Weapon",
        "RD" => "Rod",
        "RG" => "Ring",
        "S" => "Shield",
        "SC" => "Scroll",
        "SCF" => "Spellcasting Focus",
        "SHP" => "Ship",
        "SPC" => "Spacecraft",
        "T" => "Tool",
        "TAH" => "Tack & Harness",
        "TB" => "Trade Bar",
        "TG" => "Trade Good",
        "VEH" => "Vehicle (Land)",
        "WD" => "Wand",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_weapon() {
        let json = json!({
            "name": "Longsword",
            "source": "PHB",
            "page": 149,
            "type": "M",
            "rarity": "none",
            "weight": 3,
            "value": 1500,
            "dmg1": "1d8",
            "dmg2": "1d10",
            "dmgType": "S",
            "property": ["V"],
            "srd": true
        });

        let item: Item = serde_json::from_value(json).unwrap();
        assert_eq!(item.name, "Longsword");
        assert_eq!(item.dmg1, Some("1d8".to_string()));
        assert_eq!(item.dmg2, Some("1d10".to_string()));
    }

    #[test]
    fn test_magic_item_with_attunement() {
        let json = json!({
            "name": "Staff of Power",
            "source": "DMG",
            "type": "ST",
            "rarity": "very rare",
            "reqAttune": "by a sorcerer, warlock, or wizard",
            "weight": 4,
            "bonusSpellAttack": "+2",
            "entries": ["This staff can be wielded as a magic quarterstaff..."]
        });

        let item: Item = serde_json::from_value(json).unwrap();
        assert_eq!(item.name, "Staff of Power");
        assert_eq!(item.rarity, Some(ItemRarity::VeryRare));
        assert_eq!(
            item.req_attune,
            Some("by a sorcerer, warlock, or wizard".to_string())
        );
    }

    #[test]
    fn test_attunement_boolean() {
        let json = json!({
            "name": "Ring of Protection",
            "source": "DMG",
            "type": "RG",
            "rarity": "rare",
            "reqAttune": true
        });

        let item: Item = serde_json::from_value(json).unwrap();
        assert_eq!(item.req_attune, Some("true".to_string()));
    }

    #[test]
    fn test_armor() {
        let json = json!({
            "name": "Plate Armor",
            "source": "PHB",
            "type": "HA",
            "rarity": "none",
            "ac": 18,
            "strength": "15",
            "stealth": true,
            "weight": 65,
            "value": 150000
        });

        let item: Item = serde_json::from_value(json).unwrap();
        assert_eq!(item.ac, Some(18));
        assert_eq!(item.strength, Some("15".to_string()));
        assert_eq!(item.stealth, Some(true));
    }

    #[test]
    fn test_item_type_name() {
        assert_eq!(get_item_type_name("M"), "Melee Weapon");
        assert_eq!(get_item_type_name("R"), "Ranged Weapon");
        assert_eq!(get_item_type_name("LA"), "Light Armor");
        assert_eq!(get_item_type_name("$G|DMG"), "Gemstone");
        assert_eq!(get_item_type_name("P"), "Potion");
    }

    #[test]
    fn test_srd_values() {
        // Boolean true
        let json = json!({"name": "Sword", "source": "PHB", "srd": true});
        let item: Item = serde_json::from_value(json).unwrap();
        assert!(matches!(item.srd, Some(SrdValue::Flag(true))));

        // String (alternate name)
        let json = json!({"name": "Apparatus of Kwalish", "source": "DMG", "srd": "Apparatus of the Crab"});
        let item: Item = serde_json::from_value(json).unwrap();
        assert!(matches!(item.srd, Some(SrdValue::Name(ref s)) if s == "Apparatus of the Crab"));
    }

    #[test]
    fn test_item_data_container() {
        let json = json!({
            "item": [
                {"name": "Sword", "source": "PHB"},
                {"name": "Shield", "source": "PHB"}
            ]
        });

        let data: ItemData = serde_json::from_value(json).unwrap();
        assert_eq!(data.item.len(), 2);
    }
}
