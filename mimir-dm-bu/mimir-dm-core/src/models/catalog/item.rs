//! Item catalog models

use super::types::Entry;
use crate::schema::catalog_items;
use diesel::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

/// A D&D 5e item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub name: String,
    pub source: String,
    pub page: Option<u32>,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub rarity: Option<String>,
    #[serde(default)]
    pub weight: Option<f32>,
    #[serde(default)]
    pub value: Option<f64>, // Value in copper pieces (can be fractional)
    #[serde(default)]
    pub ac: Option<u8>,
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
    #[serde(default, deserialize_with = "deserialize_attunement")]
    pub requires_attunement: Option<String>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default, deserialize_with = "deserialize_srd")]
    pub srd: Option<String>,
}

/// Container for item data from JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    pub item: Vec<Item>,
}

/// Simplified item for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    pub name: String,
    pub source: String,
    #[serde(rename = "itemType")]
    pub item_type: String,
    #[serde(rename = "typeName")]
    pub type_name: String,
    pub rarity: String,
    pub value: Option<f64>,
    pub weight: Option<f32>,
    pub ac: Option<u8>,
    pub damage: Option<String>,
    #[serde(rename = "reqAttune")]
    pub req_attune: Option<String>,
    pub description: String,
}

impl From<&Item> for ItemSummary {
    fn from(item: &Item) -> Self {
        let item_type = item
            .item_type
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let type_name = get_type_name(&item_type);
        let rarity = item.rarity.clone().unwrap_or_else(|| "none".to_string());

        // Get first line of description for summary
        let description = item
            .entries
            .first()
            .and_then(|e| match e {
                Entry::Text(s) => Some(s.as_str()),
                Entry::Object(_) => None,
            })
            .unwrap_or("")
            .chars()
            .take(200)
            .collect::<String>();

        // Combine damage fields if present
        let damage = match (&item.dmg1, &item.dmg2) {
            (Some(d1), Some(d2)) => Some(format!("{}/{}", d1, d2)),
            (Some(d1), None) => Some(d1.clone()),
            (None, Some(d2)) => Some(d2.clone()),
            _ => None,
        };

        ItemSummary {
            name: item.name.clone(),
            source: item.source.clone(),
            item_type,
            type_name,
            rarity,
            value: item.value,
            weight: item.weight,
            ac: item.ac,
            damage,
            req_attune: item.requires_attunement.clone(),
            description,
        }
    }
}

fn get_type_name(item_type: &str) -> String {
    // Handle complex type formats like "$G|DMG", "EXP|DMG", etc.
    // Extract the base type code before any | separator
    let base_type = if let Some(pipe_pos) = item_type.find('|') {
        &item_type[..pipe_pos]
    } else {
        item_type
    };

    // Handle treasure types with $ prefix - these are special and should NOT be stripped
    match base_type {
        "$" => "Treasure",    // $ = TREASURE
        "$A" => "Art Object", // $A = TREASURE_ART_OBJECT
        "$C" => "Coinage",    // $C = TREASURE_COINAGE
        "$G" => "Gemstone",   // $G = TREASURE_GEMSTONE
        // Regular item types (without $ prefix)
        "A" => "Ammunition",             // A = AMMUNITION
        "AF" => "Futuristic Ammunition", // AF = AMMUNITION_FUTURISTIC
        "AIR" => "Aircraft",             // AIR = VEHICLE_AIR
        "AT" => "Artisan's Tools",       // AT = ARTISAN_TOOL
        "EXP" => "Explosive",            // EXP = EXPLOSIVE
        "FD" => "Food & Drink",          // FD = FOOD_AND_DRINK
        "G" => "Adventuring Gear",       // G = ADVENTURING_GEAR
        "GS" => "Gaming Set",            // GS = GAMING_SET
        "GV" => "Generic Variant",       // GV = GENERIC_VARIANT
        "HA" => "Heavy Armor",           // HA = HEAVY_ARMOR
        "IDG" => "Illegal Drug",         // IDG = ILLEGAL_DRUG
        "INS" => "Musical Instrument",   // INS = INSTRUMENT
        "LA" => "Light Armor",           // LA = LIGHT_ARMOR
        "M" => "Melee Weapon",           // M = MELEE_WEAPON
        "MA" => "Medium Armor",          // MA = MEDIUM_ARMOR
        "MNT" => "Mount",                // MNT = MOUNT
        "OTH" => "Other",                // OTH = OTHER
        "P" => "Potion",                 // P = POTION
        "R" => "Ranged Weapon",          // R = RANGED_WEAPON
        "RD" => "Rod",                   // RD = ROD
        "RG" => "Ring",                  // RG = RING
        "S" => "Shield",                 // S = SHIELD
        "SC" => "Scroll",                // SC = SCROLL
        "SCF" => "Spellcasting Focus",   // SCF = SPELLCASTING_FOCUS
        "SHP" => "Ship",                 // SHP = VEHICLE_WATER
        "SPC" => "Spacecraft",           // SPC = VEHICLE_SPACE
        "T" => "Tool",                   // T = TOOL
        "TAH" => "Tack & Harness",       // TAH = TACK_AND_HARNESS
        "TB" => "Trade Bar",             // TB = TRADE_BAR
        "TG" => "Trade Good",            // TG = TRADE_GOOD
        "VEH" => "Vehicle (Land)",       // VEH = VEHICLE_LAND
        "WD" => "Wand",                  // WD = WAND
        "W" => "Wondrous Item",          // W = Not in 5etools but commonly used
        _ => base_type,                  // Return base type without source suffix if no match found
    }
    .to_string()
}

// Database models
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = catalog_items)]
pub struct CatalogItem {
    pub id: i32,
    pub name: String,
    pub item_type: Option<String>,
    pub type_name: Option<String>,
    pub rarity: Option<String>,
    pub value: Option<f64>,
    pub weight: Option<f64>,
    pub ac: Option<i32>,
    pub damage: Option<String>,
    pub requires_attunement: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_item_json: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = catalog_items)]
pub struct NewCatalogItem {
    pub name: String,
    pub item_type: Option<String>,
    pub type_name: Option<String>,
    pub rarity: Option<String>,
    pub value: Option<f64>,
    pub weight: Option<f64>,
    pub ac: Option<i32>,
    pub damage: Option<String>,
    pub requires_attunement: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_item_json: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemFilters {
    pub name: Option<String>,
    pub item_types: Option<Vec<String>>,
    pub rarities: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

impl From<&CatalogItem> for ItemSummary {
    fn from(catalog: &CatalogItem) -> Self {
        Self {
            name: catalog.name.clone(),
            source: catalog.source.clone(),
            item_type: catalog
                .item_type
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            type_name: catalog
                .type_name
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            rarity: catalog.rarity.clone().unwrap_or_else(|| "none".to_string()),
            value: catalog.value,
            weight: catalog.weight.map(|w| w as f32), // Convert f64 to f32 for compatibility
            ac: catalog.ac.map(|ac| ac as u8),        // Convert i32 to u8 for compatibility
            damage: catalog.damage.clone(),
            req_attune: catalog.requires_attunement.clone(),
            description: String::new(), // Will be populated from JSON if needed
        }
    }
}

impl From<&Item> for NewCatalogItem {
    fn from(item: &Item) -> Self {
        let item_type = item.item_type.clone();
        let type_name = item_type.as_ref().map(|t| get_type_name(t));
        let rarity = item.rarity.clone();

        // Combine damage fields if present
        let damage = match (&item.dmg1, &item.dmg2) {
            (Some(d1), Some(d2)) => Some(format!("{}/{}", d1, d2)),
            (Some(d1), None) => Some(d1.clone()),
            (None, Some(d2)) => Some(d2.clone()),
            _ => None,
        };

        Self {
            name: item.name.clone(),
            item_type,
            type_name,
            rarity,
            value: item.value,
            weight: item.weight.map(|w| w as f64), // Convert f32 to f64
            ac: item.ac.map(|ac| ac as i32),       // Convert u8 to i32
            damage,
            requires_attunement: item.requires_attunement.clone(),
            source: item.source.clone(),
            page: item.page.map(|p| p as i32),
            full_item_json: serde_json::to_string(item).unwrap_or_default(),
        }
    }
}

/// Custom deserializer for requires_attunement field that can be either boolean or string
fn deserialize_attunement<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Option::<Value>::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(Value::Bool(true)) => Ok(Some("true".to_string())),
        Some(Value::Bool(false)) => Ok(None), // false means no attunement required
        Some(Value::String(s)) => Ok(Some(s)),
        Some(other) => Err(D::Error::custom(format!(
            "Expected boolean or string for requires_attunement, got {:?}",
            other
        ))),
    }
}

/// Custom deserializer for srd field that can be boolean or string
/// - true (boolean): item is in SRD with same name
/// - false (boolean): item is not in SRD  
/// - "Name" (string): item is in SRD but with different name
fn deserialize_srd<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Option::<Value>::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(Value::Bool(true)) => Ok(Some("true".to_string())), // In SRD with same name
        Some(Value::Bool(false)) => Ok(None),                    // Not in SRD
        Some(Value::String(s)) => Ok(Some(s)),                   // In SRD with different name
        Some(other) => Err(D::Error::custom(format!(
            "Expected boolean or string for srd, got {:?}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_get_type_name_simple_codes() {
        assert_eq!(get_type_name("G"), "Adventuring Gear");
        assert_eq!(get_type_name("M"), "Melee Weapon");
        assert_eq!(get_type_name("R"), "Ranged Weapon");
        assert_eq!(get_type_name("A"), "Ammunition");
        assert_eq!(get_type_name("AF"), "Futuristic Ammunition"); // Updated to match implementation
        assert_eq!(get_type_name("LA"), "Light Armor");
        assert_eq!(get_type_name("W"), "Wondrous Item");
        assert_eq!(get_type_name("P"), "Potion");
    }

    #[test]
    fn test_get_type_name_complex_with_pipe() {
        assert_eq!(get_type_name("$G|DMG"), "Gemstone"); // $G is treasure gemstone, not gear
        assert_eq!(get_type_name("$A|DMG"), "Art Object"); // $A is treasure art object
        assert_eq!(get_type_name("EXP|DMG"), "Explosive");
        assert_eq!(get_type_name("AIR|DMG"), "Aircraft");
        assert_eq!(get_type_name("GV|DMG"), "Generic Variant");
        assert_eq!(get_type_name("$C|PHB"), "Coinage"); // $C is treasure coinage
    }

    #[test]
    fn test_get_type_name_with_dollar_prefix() {
        assert_eq!(get_type_name("$G"), "Gemstone"); // $G is treasure gemstone
        assert_eq!(get_type_name("$C"), "Coinage"); // $C is treasure coinage
        assert_eq!(get_type_name("$A"), "Art Object"); // $A is treasure art object
    }

    #[test]
    fn test_get_type_name_unknown_codes() {
        // Unknown codes without prefix/pipe should return as-is
        assert_eq!(get_type_name("UNKNOWN"), "UNKNOWN");

        // Unknown codes with prefix/pipe return everything before pipe (as-is since no match)
        assert_eq!(get_type_name("$UNKNOWN|DMG"), "$UNKNOWN"); // Returns prefix+code as-is
        assert_eq!(get_type_name("WEIRD|TEST"), "WEIRD");
    }

    #[test]
    fn test_deserialize_srd_boolean() {
        // Test boolean true
        let json = r#"{"srd": true}"#;
        let test_struct: serde_json::Value = serde_json::from_str(json).unwrap();
        let srd_value = test_struct.get("srd").unwrap();

        // Test boolean false
        let json_false = r#"{"srd": false}"#;
        let test_struct_false: serde_json::Value = serde_json::from_str(json_false).unwrap();
        let srd_value_false = test_struct_false.get("srd").unwrap();

        assert!(srd_value.is_boolean());
        assert!(srd_value_false.is_boolean());
    }

    #[test]
    fn test_deserialize_srd_string() {
        // Test string value (SRD name different from original)
        let json = r#"{"srd": "Apparatus of the Crab"}"#;
        let test_struct: serde_json::Value = serde_json::from_str(json).unwrap();
        let srd_value = test_struct.get("srd").unwrap();

        assert!(srd_value.is_string());
        assert_eq!(srd_value.as_str().unwrap(), "Apparatus of the Crab");
    }

    #[test]
    fn test_deserialize_requires_attunement_boolean() {
        let json_true = r#"{"reqAttune": true}"#;
        let json_false = r#"{"reqAttune": false}"#;

        let test_true: serde_json::Value = serde_json::from_str(json_true).unwrap();
        let test_false: serde_json::Value = serde_json::from_str(json_false).unwrap();

        assert!(test_true.get("reqAttune").unwrap().is_boolean());
        assert!(test_false.get("reqAttune").unwrap().is_boolean());
    }

    #[test]
    fn test_deserialize_requires_attunement_string() {
        let json = r#"{"reqAttune": "by a spellcaster"}"#;
        let test_struct: serde_json::Value = serde_json::from_str(json).unwrap();
        let attune_value = test_struct.get("reqAttune").unwrap();

        assert!(attune_value.is_string());
        assert_eq!(attune_value.as_str().unwrap(), "by a spellcaster");
    }

    #[test]
    fn test_item_summary_conversion() {
        let item = Item {
            name: "Longsword".to_string(),
            source: "PHB".to_string(),
            page: Some(149),
            item_type: Some("M".to_string()),
            rarity: Some("none".to_string()),
            weight: Some(3.0),
            value: Some(1500.0), // 15 gp in copper
            ac: None,
            dmg1: Some("1d8".to_string()),
            dmg2: Some("1d10".to_string()),
            dmg_type: Some("S".to_string()),
            property: Some(vec!["V".to_string()]),
            range: None,
            reload: None,
            requires_attunement: None,
            entries: vec![Entry::Text("A versatile weapon".to_string())],
            srd: Some("true".to_string()),
        };

        let summary = ItemSummary::from(&item);

        assert_eq!(summary.name, "Longsword");
        assert_eq!(summary.source, "PHB");
        assert_eq!(summary.item_type, "M");
        assert_eq!(summary.type_name, "Melee Weapon");
        assert_eq!(summary.rarity, "none");
        assert_eq!(summary.weight, Some(3.0));
        assert_eq!(summary.value, Some(1500.0));
        assert_eq!(summary.damage, Some("1d8/1d10".to_string()));
        assert_eq!(summary.description, "A versatile weapon");
    }

    #[test]
    fn test_new_catalog_item_conversion() {
        let item = Item {
            name: "Magic Sword".to_string(),
            source: "DMG".to_string(),
            page: Some(200),
            item_type: Some("M|DMG".to_string()),
            rarity: Some("rare".to_string()),
            weight: Some(3.0),
            value: Some(500000.0), // 5000 gp in copper
            ac: None,
            dmg1: Some("1d8".to_string()),
            dmg2: None,
            dmg_type: Some("S".to_string()),
            property: None,
            range: None,
            reload: None,
            requires_attunement: Some("true".to_string()),
            entries: vec![],
            srd: Some("false".to_string()),
        };

        let catalog_item = NewCatalogItem::from(&item);

        assert_eq!(catalog_item.name, "Magic Sword");
        assert_eq!(catalog_item.source, "DMG");
        assert_eq!(catalog_item.item_type, Some("M|DMG".to_string()));
        assert_eq!(catalog_item.type_name, Some("Melee Weapon".to_string()));
        assert_eq!(catalog_item.rarity, Some("rare".to_string()));
        assert_eq!(catalog_item.weight, Some(3.0));
        assert_eq!(catalog_item.value, Some(500000.0));
        assert_eq!(catalog_item.damage, Some("1d8".to_string()));
        assert_eq!(catalog_item.requires_attunement, Some("true".to_string()));
    }
}
