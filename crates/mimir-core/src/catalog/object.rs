//! Object catalog extraction types
//!
//! Types for deserializing 5etools object JSON data (siege weapons, etc.).

use serde::{Deserialize, Serialize};

/// Armor class can be a simple number or a complex object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmorClass {
    Number(i32),
    Object {
        #[serde(default)]
        ac: Option<i32>,
        #[serde(default)]
        special: Option<String>,
        #[serde(default)]
        from: Option<Vec<String>>,
    },
}

/// Hit points can be a simple number or a complex object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HitPoints {
    Number(i32),
    Object {
        #[serde(default)]
        hp: Option<i32>,
        #[serde(default)]
        special: Option<String>,
    },
}

/// Damage type can be simple string or special object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DamageType {
    Simple(String),
    Special { special: String },
}

/// A D&D 5e object from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DndObject {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Object type: SW (Siege Weapon), GEN (Generic), etc.
    #[serde(default)]
    pub object_type: Option<String>,

    /// Size codes
    #[serde(default)]
    pub size: Option<Vec<String>>,

    /// Armor class
    #[serde(default)]
    pub ac: Option<ArmorClass>,

    /// Hit points
    #[serde(default)]
    pub hp: Option<HitPoints>,

    /// Damage immunities
    #[serde(default)]
    pub immune: Option<Vec<DamageType>>,

    /// Damage resistances
    #[serde(default)]
    pub resist: Option<Vec<DamageType>>,

    /// Damage vulnerabilities
    #[serde(default)]
    pub vulnerable: Option<Vec<DamageType>>,

    /// Action entries (stored as JSON blob)
    #[serde(default)]
    pub action_entries: Option<Vec<serde_json::Value>>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Flags
    #[serde(default)]
    pub has_token: Option<bool>,
    #[serde(default)]
    pub token_credit: Option<String>,
    #[serde(default)]
    pub has_fluff: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,
}

/// Container for object data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectData {
    #[serde(default)]
    pub object: Vec<DndObject>,
}

/// Get human-readable name for object type code.
pub fn get_object_type_name(code: &str) -> &'static str {
    match code {
        "SW" => "Siege Weapon",
        "GEN" => "Generic",
        "U" => "Unknown",
        _ => "Object",
    }
}

/// Get human-readable name for size code.
pub fn get_size_name(code: &str) -> &'static str {
    match code {
        "T" => "Tiny",
        "S" => "Small",
        "M" => "Medium",
        "L" => "Large",
        "H" => "Huge",
        "G" => "Gargantuan",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_siege_weapon() {
        let json = json!({
            "name": "Ballista",
            "source": "DMG",
            "objectType": "SW",
            "size": ["L"],
            "ac": 15,
            "hp": 50,
            "entries": ["A ballista is a massive crossbow..."]
        });
        let obj: DndObject = serde_json::from_value(json).unwrap();
        assert_eq!(obj.name, "Ballista");
        assert_eq!(obj.object_type, Some("SW".to_string()));
        assert!(matches!(obj.ac, Some(ArmorClass::Number(15))));
    }

    #[test]
    fn test_complex_ac() {
        let json = json!({
            "name": "Test Object",
            "source": "DMG",
            "ac": {"ac": 18, "from": ["natural armor"]}
        });
        let obj: DndObject = serde_json::from_value(json).unwrap();
        if let Some(ArmorClass::Object { ac, from, .. }) = obj.ac {
            assert_eq!(ac, Some(18));
            assert!(from.is_some());
        } else {
            panic!("Expected complex AC");
        }
    }

    #[test]
    fn test_immunities() {
        let json = json!({
            "name": "Construct",
            "source": "DMG",
            "immune": ["poison", {"special": "nonmagical weapons"}]
        });
        let obj: DndObject = serde_json::from_value(json).unwrap();
        assert_eq!(obj.immune.unwrap().len(), 2);
    }

    #[test]
    fn test_object_data() {
        let json = json!({
            "object": [
                {"name": "Ballista", "source": "DMG"},
                {"name": "Trebuchet", "source": "DMG"}
            ]
        });
        let data: ObjectData = serde_json::from_value(json).unwrap();
        assert_eq!(data.object.len(), 2);
    }

    #[test]
    fn test_object_type_name() {
        assert_eq!(get_object_type_name("SW"), "Siege Weapon");
        assert_eq!(get_object_type_name("GEN"), "Generic");
    }

    #[test]
    fn test_size_name() {
        assert_eq!(get_size_name("L"), "Large");
        assert_eq!(get_size_name("H"), "Huge");
    }
}
