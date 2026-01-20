//! Class catalog extraction types
//!
//! Types for deserializing 5etools class JSON data.

use super::types::SrdValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A D&D 5e character class from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    // Core class features
    #[serde(default)]
    pub hd: Option<HitDice>,
    #[serde(default)]
    pub proficiency: Option<Vec<String>>,
    #[serde(default)]
    pub starting_proficiencies: Option<StartingProficiencies>,
    #[serde(default)]
    pub starting_equipment: Option<StartingEquipment>,

    // Class progression
    #[serde(default)]
    pub class_features: Option<Vec<ClassFeatureRef>>,
    #[serde(default)]
    pub class_table_groups: Option<Vec<ClassTableGroup>>,
    #[serde(default)]
    pub subclass_title: Option<String>,

    // Spellcasting
    #[serde(default)]
    pub caster_progression: Option<String>,
    #[serde(default)]
    pub cantrip_progression: Option<Vec<u8>>,
    #[serde(default)]
    pub spells_known_progression: Option<Vec<u8>>,
    #[serde(default)]
    pub prepared_spells: Option<String>,
    #[serde(default)]
    pub spellcasting_ability: Option<String>,

    // Multiclassing
    #[serde(default)]
    pub multiclassing: Option<Multiclassing>,

    // Optional features (invocations, metamagic, etc.)
    #[serde(default)]
    pub optionalfeature_progression: Option<Vec<OptionalFeatureProgression>>,

    // Fluff
    #[serde(default)]
    pub fluff: Option<serde_json::Value>,

    // Flags
    #[serde(default)]
    pub srd: Option<SrdValue>,
    #[serde(default)]
    pub basic_rules: Option<bool>,

    // Copy directive
    #[serde(default, rename = "_copy")]
    pub copy: Option<serde_json::Value>,
}

/// Hit dice specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitDice {
    pub number: u8,
    pub faces: u8,
}

/// A class feature reference - can be string or object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClassFeatureRef {
    /// Simple string reference (e.g., "Spellcasting|Cleric||1")
    Simple(String),
    /// Object with metadata
    Object(ClassFeatureRefObject),
}

/// Class feature reference object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassFeatureRefObject {
    pub class_feature: String,
    #[serde(default)]
    pub gain_subclass_feature: Option<bool>,
    #[serde(default)]
    pub table_display_name: Option<String>,
}

/// Starting proficiencies for a class.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartingProficiencies {
    #[serde(default)]
    pub armor: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub weapons: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub skills: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub saving_throws: Option<Vec<String>>,
}

/// Starting equipment options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartingEquipment {
    #[serde(default)]
    pub additional_from_background: Option<bool>,
    #[serde(default)]
    pub default: Option<Vec<String>>,
    #[serde(default, rename = "defaultData")]
    pub default_data: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub gold_alternative: Option<String>,
}

/// Class table group for progression tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassTableGroup {
    #[serde(default)]
    pub col_labels: Option<Vec<String>>,
    #[serde(default)]
    pub col_styles: Option<Vec<String>>,
    #[serde(default)]
    pub rows: Option<Vec<Vec<serde_json::Value>>>,
    #[serde(default)]
    pub row_sub_headers: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub title: Option<String>,
}

/// Multiclassing requirements and benefits.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Multiclassing {
    #[serde(default)]
    pub requirements: Option<MulticlassingRequirements>,
    #[serde(default)]
    pub proficiencies_gained: Option<MulticlassingProficiencies>,
}

/// Multiclassing ability score requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MulticlassingRequirements {
    #[serde(default, rename = "str")]
    pub strength: Option<i32>,
    #[serde(default, rename = "dex")]
    pub dexterity: Option<i32>,
    #[serde(default, rename = "con")]
    pub constitution: Option<i32>,
    #[serde(default, rename = "int")]
    pub intelligence: Option<i32>,
    #[serde(default, rename = "wis")]
    pub wisdom: Option<i32>,
    #[serde(default, rename = "cha")]
    pub charisma: Option<i32>,
    #[serde(default)]
    pub or: Option<Vec<MulticlassingRequirements>>,
}

/// Proficiencies gained from multiclassing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MulticlassingProficiencies {
    #[serde(default)]
    pub armor: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub weapons: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub skills: Option<Vec<serde_json::Value>>,
}

/// Optional feature progression (invocations, metamagic, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalFeatureProgression {
    #[serde(default)]
    pub feature_type: Option<Vec<String>>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub progression: Option<OptionalFeatureProgressionValue>,
}

/// Progression value - array or level-keyed object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalFeatureProgressionValue {
    Array(Vec<i32>),
    Object(HashMap<String, i32>),
}

// =============================================================================
// Subclass Types
// =============================================================================

/// A character subclass.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subclass {
    pub name: String,
    #[serde(default)]
    pub short_name: Option<String>,
    pub source: String,
    pub class_name: String,
    pub class_source: String,
    #[serde(default)]
    pub page: Option<u32>,

    // Subclass features
    #[serde(default)]
    pub subclass_features: Option<Vec<String>>,
    #[serde(default)]
    pub subclass_table_groups: Option<Vec<ClassTableGroup>>,

    // Spellcasting (if different from base class)
    #[serde(default)]
    pub caster_progression: Option<String>,
    #[serde(default)]
    pub spellcasting_ability: Option<String>,
    #[serde(default)]
    pub cantrip_progression: Option<Vec<u8>>,
    #[serde(default)]
    pub spells_known_progression: Option<Vec<u8>>,
    #[serde(default)]
    pub additional_spells: Option<Vec<serde_json::Value>>,

    // Fluff
    #[serde(default)]
    pub fluff: Option<serde_json::Value>,

    // Flags
    #[serde(default)]
    pub srd: Option<SrdValue>,
    #[serde(default)]
    pub basic_rules: Option<bool>,
}

// =============================================================================
// Class Feature Types
// =============================================================================

/// A class feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassFeature {
    pub name: String,
    pub source: String,
    pub class_name: String,
    pub class_source: String,
    pub level: u8,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,
    #[serde(default)]
    pub srd: Option<SrdValue>,
    #[serde(default)]
    pub basic_rules: Option<bool>,
}

/// A subclass feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubclassFeature {
    pub name: String,
    pub source: String,
    pub class_name: String,
    pub class_source: String,
    #[serde(default)]
    pub subclass_short_name: Option<String>,
    pub subclass_source: String,
    pub level: u8,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,
    #[serde(default)]
    pub srd: Option<SrdValue>,
    #[serde(default)]
    pub basic_rules: Option<bool>,
}

// =============================================================================
// Container Types
// =============================================================================

/// Container for class data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassData {
    #[serde(default, rename = "class")]
    pub classes: Vec<Class>,
    #[serde(default)]
    pub subclass: Option<Vec<Subclass>>,
    #[serde(default, rename = "classFeature")]
    pub class_features: Option<Vec<ClassFeature>>,
    #[serde(default, rename = "subclassFeature")]
    pub subclass_features: Option<Vec<SubclassFeature>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hit_dice() {
        let json = json!({"number": 1, "faces": 10});
        let hd: HitDice = serde_json::from_value(json).unwrap();
        assert_eq!(hd.number, 1);
        assert_eq!(hd.faces, 10);
    }

    #[test]
    fn test_class_feature_ref_simple() {
        let json = json!("Spellcasting|Cleric||1");
        let feature: ClassFeatureRef = serde_json::from_value(json).unwrap();
        assert!(matches!(feature, ClassFeatureRef::Simple(s) if s.contains("Spellcasting")));
    }

    #[test]
    fn test_class_feature_ref_object() {
        let json = json!({
            "classFeature": "Divine Domain|Cleric||1",
            "gainSubclassFeature": true
        });
        let feature: ClassFeatureRef = serde_json::from_value(json).unwrap();
        if let ClassFeatureRef::Object(obj) = feature {
            assert!(obj.class_feature.contains("Divine Domain"));
            assert_eq!(obj.gain_subclass_feature, Some(true));
        } else {
            panic!("Expected Object variant");
        }
    }

    #[test]
    fn test_multiclassing_requirements() {
        let json = json!({"str": 13, "cha": 13});
        let reqs: MulticlassingRequirements = serde_json::from_value(json).unwrap();
        assert_eq!(reqs.strength, Some(13));
        assert_eq!(reqs.charisma, Some(13));
        assert_eq!(reqs.dexterity, None);
    }

    #[test]
    fn test_optional_feature_progression_array() {
        let json = json!({
            "featureType": ["EI"],
            "name": "Eldritch Invocations",
            "progression": [0, 2, 2, 2, 3]
        });
        let prog: OptionalFeatureProgression = serde_json::from_value(json).unwrap();
        assert_eq!(prog.name, Some("Eldritch Invocations".to_string()));
        assert!(matches!(prog.progression, Some(OptionalFeatureProgressionValue::Array(_))));
    }

    #[test]
    fn test_optional_feature_progression_object() {
        let json = json!({
            "featureType": ["PB"],
            "name": "Pact Boon",
            "progression": {"3": 1}
        });
        let prog: OptionalFeatureProgression = serde_json::from_value(json).unwrap();
        if let Some(OptionalFeatureProgressionValue::Object(map)) = prog.progression {
            assert_eq!(map.get("3"), Some(&1));
        } else {
            panic!("Expected Object progression");
        }
    }

    #[test]
    fn test_minimal_class() {
        let json = json!({
            "name": "Fighter",
            "source": "PHB",
            "hd": {"number": 1, "faces": 10},
            "proficiency": ["str", "con"],
            "srd": true
        });
        let class: Class = serde_json::from_value(json).unwrap();
        assert_eq!(class.name, "Fighter");
        assert_eq!(class.source, "PHB");
        assert!(class.hd.is_some());
        assert!(matches!(class.srd, Some(SrdValue::Flag(true))));
    }

    #[test]
    fn test_class_with_features() {
        let json = json!({
            "name": "Cleric",
            "source": "PHB",
            "classFeatures": [
                "Spellcasting|Cleric||1",
                {"classFeature": "Divine Domain|Cleric||1", "gainSubclassFeature": true}
            ],
            "srd": true
        });
        let class: Class = serde_json::from_value(json).unwrap();
        let features = class.class_features.unwrap();
        assert_eq!(features.len(), 2);
    }

    #[test]
    fn test_subclass() {
        let json = json!({
            "name": "Knowledge Domain",
            "source": "PHB",
            "className": "Cleric",
            "classSource": "PHB",
            "subclassFeatures": [
                "Knowledge Domain|Cleric||Knowledge||1"
            ],
            "srd": true
        });
        let subclass: Subclass = serde_json::from_value(json).unwrap();
        assert_eq!(subclass.name, "Knowledge Domain");
        assert_eq!(subclass.class_name, "Cleric");
        assert!(matches!(subclass.srd, Some(SrdValue::Flag(true))));
    }

    #[test]
    fn test_class_feature() {
        let json = json!({
            "name": "Spellcasting",
            "source": "PHB",
            "className": "Cleric",
            "classSource": "PHB",
            "level": 1,
            "entries": ["You can cast spells..."],
            "srd": true
        });
        let feature: ClassFeature = serde_json::from_value(json).unwrap();
        assert_eq!(feature.name, "Spellcasting");
        assert_eq!(feature.level, 1);
        assert!(matches!(feature.srd, Some(SrdValue::Flag(true))));
    }

    #[test]
    fn test_class_data_container() {
        let json = json!({
            "class": [
                {"name": "Fighter", "source": "PHB"}
            ],
            "subclass": [
                {"name": "Champion", "source": "PHB", "className": "Fighter", "classSource": "PHB"}
            ]
        });
        let data: ClassData = serde_json::from_value(json).unwrap();
        assert_eq!(data.classes.len(), 1);
        assert_eq!(data.subclass.unwrap().len(), 1);
    }
}
