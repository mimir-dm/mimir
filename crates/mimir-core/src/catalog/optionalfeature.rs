//! Optional Feature catalog extraction types
//!
//! Types for deserializing 5etools optional feature JSON data.
//! Includes Eldritch Invocations, Metamagic, Fighting Styles, etc.

use super::types::SrdValue;
use serde::{Deserialize, Serialize};

/// A D&D 5e optional feature from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalFeature {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    /// Feature type codes: EI (Eldritch Invocation), MM (Metamagic), FS:F (Fighting Style: Fighter), etc.
    #[serde(default)]
    pub feature_type: Vec<String>,

    /// Prerequisites (stored as JSON blob due to complex structure)
    #[serde(default)]
    pub prerequisite: Option<Vec<serde_json::Value>>,

    /// Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    /// Is this a class feature variant?
    #[serde(default)]
    pub is_class_feature_variant: Option<bool>,

    /// Resource consumption
    #[serde(default)]
    pub consumes: Option<Consumes>,

    /// Additional spells granted (stored as JSON blob)
    #[serde(default)]
    pub additional_spells: Option<Vec<serde_json::Value>>,

    // Fluff flags
    #[serde(default)]
    pub has_fluff_images: Option<bool>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
}

/// Resource consumption for optional features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumes {
    pub name: String,
    #[serde(default)]
    pub amount: Option<i32>,
}

/// Container for optional feature data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionalFeatureData {
    #[serde(rename = "optionalfeature", default)]
    pub optional_features: Vec<OptionalFeature>,
}

/// Get human-readable name for feature type code.
pub fn get_feature_type_name(code: &str) -> &'static str {
    match code {
        "AI" => "Artificer Infusion",
        "ED" => "Elemental Discipline",
        "EI" => "Eldritch Invocation",
        "MM" => "Metamagic",
        "MV" => "Maneuver",
        "MV:B" => "Maneuver (Battle Master)",
        "MV:C2-UA" => "Maneuver (Cavalier V2 UA)",
        "AS:V1-UA" => "Arcane Shot (V1 UA)",
        "AS:V2-UA" => "Arcane Shot (V2 UA)",
        "AS" => "Arcane Shot",
        "OTH" => "Other",
        "FS:F" => "Fighting Style (Fighter)",
        "FS:B" => "Fighting Style (Bard)",
        "FS:P" => "Fighting Style (Paladin)",
        "FS:R" => "Fighting Style (Ranger)",
        "PB" => "Pact Boon",
        "OR" => "Onomancy Resonant",
        "RN" => "Rune Knight Rune",
        "AF" => "Alchemical Formula",
        "TT" => "Traveler's Trick",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_eldritch_invocation() {
        let json = json!({
            "name": "Agonizing Blast",
            "source": "PHB",
            "featureType": ["EI"],
            "prerequisite": [{"spell": ["eldritch blast#c"]}],
            "entries": ["When you cast eldritch blast, add your Charisma modifier to the damage."]
        });
        let opt: OptionalFeature = serde_json::from_value(json).unwrap();
        assert_eq!(opt.name, "Agonizing Blast");
        assert_eq!(opt.feature_type, vec!["EI"]);
        assert!(opt.prerequisite.is_some());
    }

    #[test]
    fn test_metamagic() {
        let json = json!({
            "name": "Quickened Spell",
            "source": "PHB",
            "featureType": ["MM"],
            "consumes": {"name": "Sorcery Points", "amount": 2},
            "entries": ["When you cast a spell that has a casting time of 1 action..."]
        });
        let opt: OptionalFeature = serde_json::from_value(json).unwrap();
        assert_eq!(opt.name, "Quickened Spell");
        assert!(opt.consumes.is_some());
        assert_eq!(opt.consumes.unwrap().amount, Some(2));
    }

    #[test]
    fn test_fighting_style() {
        let json = json!({
            "name": "Defense",
            "source": "PHB",
            "featureType": ["FS:F", "FS:P", "FS:R"],
            "entries": ["While you are wearing armor, you gain a +1 bonus to AC."]
        });
        let opt: OptionalFeature = serde_json::from_value(json).unwrap();
        assert_eq!(opt.feature_type.len(), 3);
    }

    #[test]
    fn test_with_additional_spells() {
        let json = json!({
            "name": "Armor of Shadows",
            "source": "PHB",
            "featureType": ["EI"],
            "additionalSpells": [{"innate": {"_": ["mage armor"]}}],
            "entries": ["You can cast mage armor on yourself at will."]
        });
        let opt: OptionalFeature = serde_json::from_value(json).unwrap();
        assert!(opt.additional_spells.is_some());
    }

    #[test]
    fn test_feature_type_name() {
        assert_eq!(get_feature_type_name("EI"), "Eldritch Invocation");
        assert_eq!(get_feature_type_name("MM"), "Metamagic");
        assert_eq!(get_feature_type_name("FS:F"), "Fighting Style (Fighter)");
    }

    #[test]
    fn test_data_container() {
        let json = json!({
            "optionalfeature": [
                {"name": "Test1", "source": "PHB", "featureType": ["EI"]},
                {"name": "Test2", "source": "PHB", "featureType": ["MM"]}
            ]
        });
        let data: OptionalFeatureData = serde_json::from_value(json).unwrap();
        assert_eq!(data.optional_features.len(), 2);
    }
}
