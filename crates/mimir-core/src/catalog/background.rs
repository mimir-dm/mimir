//! Background catalog extraction types
//!
//! Types for deserializing 5etools background JSON data.

use super::types::{ProficiencyItem, SrdValue, StartingEquipmentEntry};
use serde::{Deserialize, Serialize};

/// A D&D 5e character background from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Background {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    // Proficiencies
    #[serde(default)]
    pub skill_proficiencies: Vec<ProficiencyItem>,
    #[serde(default)]
    pub language_proficiencies: Vec<ProficiencyItem>,
    #[serde(default)]
    pub tool_proficiencies: Vec<ProficiencyItem>,

    // Starting equipment
    #[serde(default)]
    pub starting_equipment: Vec<StartingEquipmentEntry>,

    // Additional spells (stored as JSON blob)
    #[serde(default)]
    pub additional_spells: Option<Vec<serde_json::Value>>,

    // Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Fluff flags
    #[serde(default)]
    pub has_fluff: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
    #[serde(default)]
    pub basic_rules: Option<bool>,

    // Copy directive
    #[serde(default, rename = "_copy")]
    pub copy: Option<serde_json::Value>,
}

/// Container for background data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundData {
    #[serde(default)]
    pub background: Vec<Background>,
}

/// Background fluff (lore/description) data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub images: Option<Vec<serde_json::Value>>,
}

/// Container for background fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundFluffData {
    #[serde(default)]
    pub background_fluff: Vec<BackgroundFluff>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_minimal_background() {
        let json = json!({
            "name": "Custom",
            "source": "TEST"
        });
        let bg: Background = serde_json::from_value(json).unwrap();
        assert_eq!(bg.name, "Custom");
        assert_eq!(bg.source, "TEST");
        assert!(bg.starting_equipment.is_empty());
    }

    #[test]
    fn test_background_with_proficiencies() {
        let json = json!({
            "name": "Acolyte",
            "source": "PHB",
            "page": 127,
            "skillProficiencies": [
                {"insight": true, "religion": true}
            ],
            "languageProficiencies": [
                {"anyStandard": 2}
            ],
            "entries": ["You have spent your life in the service of a temple..."],
            "srd": true
        });
        let bg: Background = serde_json::from_value(json).unwrap();
        assert_eq!(bg.name, "Acolyte");
        assert_eq!(bg.skill_proficiencies.len(), 1);
        assert_eq!(bg.language_proficiencies.len(), 1);
        assert!(matches!(bg.srd, Some(SrdValue::Flag(true))));
    }

    #[test]
    fn test_background_with_tool_proficiencies() {
        let json = json!({
            "name": "Entertainer",
            "source": "PHB",
            "skillProficiencies": [
                {"acrobatics": true, "performance": true}
            ],
            "toolProficiencies": [
                {"disguise kit": true},
                {"choose": {"from": ["musical instrument"]}}
            ],
            "entries": []
        });
        let bg: Background = serde_json::from_value(json).unwrap();
        assert_eq!(bg.name, "Entertainer");
        assert_eq!(bg.tool_proficiencies.len(), 2);
    }

    #[test]
    fn test_starting_equipment_with_items() {
        let json = json!({
            "name": "Acolyte",
            "source": "PHB",
            "startingEquipment": [
                {
                    "_": [
                        {
                            "item": "holy symbol|phb",
                            "displayName": "holy symbol (a gift)"
                        },
                        {
                            "special": "sticks of incense",
                            "quantity": 5
                        },
                        "common clothes|phb",
                        {
                            "item": "pouch|phb",
                            "containsValue": 1500
                        }
                    ]
                }
            ],
            "entries": []
        });

        let bg: Background = serde_json::from_value(json).unwrap();
        assert_eq!(bg.name, "Acolyte");
        assert_eq!(bg.starting_equipment.len(), 1);
        assert_eq!(bg.starting_equipment[0].default_items.len(), 4);
    }

    #[test]
    fn test_starting_equipment_with_choices() {
        let json = json!({
            "name": "Acolyte",
            "source": "PHB",
            "startingEquipment": [
                {
                    "_": ["common clothes|phb"],
                    "a": [{"item": "book|phb", "displayName": "prayer book"}],
                    "b": [{"special": "prayer wheel"}]
                }
            ],
            "entries": []
        });

        let bg: Background = serde_json::from_value(json).unwrap();
        assert_eq!(bg.name, "Acolyte");
        assert_eq!(bg.starting_equipment.len(), 1);
        assert!(bg.starting_equipment[0].a.is_some());
        assert!(bg.starting_equipment[0].b.is_some());
    }

    #[test]
    fn test_background_data_container() {
        let json = json!({
            "background": [
                {"name": "Acolyte", "source": "PHB"},
                {"name": "Sage", "source": "PHB"}
            ]
        });
        let data: BackgroundData = serde_json::from_value(json).unwrap();
        assert_eq!(data.background.len(), 2);
    }

    #[test]
    fn test_background_fluff() {
        let json = json!({
            "name": "Acolyte",
            "source": "PHB",
            "entries": ["Background lore text..."]
        });
        let fluff: BackgroundFluff = serde_json::from_value(json).unwrap();
        assert_eq!(fluff.name, "Acolyte");
        assert!(fluff.entries.is_some());
    }
}
