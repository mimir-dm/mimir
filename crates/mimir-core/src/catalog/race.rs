//! Race catalog extraction types
//!
//! Types for deserializing 5etools race JSON data.

use super::types::{
    AbilityBonus, DamageModifier, HeightAndWeight, Lineage, ProficiencyItem, RaceSpeed, SrdValue,
};
use serde::{Deserialize, Serialize};

/// A D&D 5e player race from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Race {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    // Size and speed
    #[serde(default)]
    pub size: Option<Vec<String>>,
    #[serde(default)]
    pub speed: Option<RaceSpeed>,

    // Ability scores
    #[serde(default)]
    pub ability: Option<Vec<AbilityBonus>>,

    // Vision and senses
    #[serde(default)]
    pub darkvision: Option<i32>,

    // Age info
    #[serde(default)]
    pub age: Option<Age>,

    // Trait tags (e.g., "Amphibious", "Natural Armor")
    #[serde(default)]
    pub trait_tags: Option<Vec<String>>,

    // Proficiencies
    #[serde(default)]
    pub language_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub skill_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub weapon_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub armor_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub tool_proficiencies: Option<Vec<ProficiencyItem>>,

    // Resistances and immunities
    #[serde(default)]
    pub resist: Option<Vec<DamageModifier>>,
    #[serde(default)]
    pub immune: Option<Vec<DamageModifier>>,
    #[serde(default)]
    pub vulnerable: Option<Vec<DamageModifier>>,
    #[serde(default)]
    pub condition_immune: Option<Vec<String>>,

    // Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Sound clip reference
    #[serde(default)]
    pub sound_clip: Option<SoundClip>,

    // Lineage information
    #[serde(default)]
    pub lineage: Option<Lineage>,

    // Base race info (for subraces published separately)
    #[serde(default)]
    pub race_name: Option<String>,
    #[serde(default)]
    pub race_source: Option<String>,

    // Physical characteristics
    #[serde(default)]
    pub height_and_weight: Option<HeightAndWeight>,

    // Additional spells (stored as JSON blob)
    #[serde(default)]
    pub additional_spells: Option<Vec<serde_json::Value>>,

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

/// A subrace variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subrace {
    #[serde(default)]
    pub name: Option<String>,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    // Parent race reference
    pub race_name: String,
    pub race_source: String,

    // Overrides from parent
    #[serde(default)]
    pub ability: Option<Vec<AbilityBonus>>,
    #[serde(default)]
    pub speed: Option<RaceSpeed>,
    #[serde(default)]
    pub darkvision: Option<i32>,
    #[serde(default)]
    pub size: Option<Vec<String>>,

    // Resistances
    #[serde(default)]
    pub resist: Option<Vec<DamageModifier>>,

    // Trait tags
    #[serde(default)]
    pub trait_tags: Option<Vec<String>>,

    // Proficiencies
    #[serde(default)]
    pub language_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub skill_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub weapon_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub armor_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub tool_proficiencies: Option<Vec<ProficiencyItem>>,

    // Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Override settings
    #[serde(default)]
    pub overwrite: Option<Overwrite>,

    // Additional spells (stored as JSON blob)
    #[serde(default)]
    pub additional_spells: Option<Vec<serde_json::Value>>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
}

/// Age information for a race.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Age {
    /// Age at which race members are considered mature
    #[serde(default)]
    pub mature: Option<i32>,
    /// Maximum typical lifespan
    #[serde(default)]
    pub max: Option<i32>,
}

/// Sound clip reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundClip {
    #[serde(rename = "type")]
    pub clip_type: String,
    pub path: String,
}

/// Overwrite settings for subraces.
///
/// When true, the subrace's value completely replaces the parent race's value
/// instead of augmenting it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Overwrite {
    #[serde(default)]
    pub ability: Option<bool>,
    #[serde(default)]
    pub language_proficiencies: Option<bool>,
    #[serde(default)]
    pub skill_proficiencies: Option<bool>,
}

/// Container for race data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceData {
    #[serde(default)]
    pub race: Vec<Race>,
    #[serde(default)]
    pub subrace: Option<Vec<Subrace>>,
}

/// Race fluff (lore/description) data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub images: Option<Vec<serde_json::Value>>,
}

/// Container for race fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceFluffData {
    #[serde(default)]
    pub race_fluff: Vec<RaceFluff>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_minimal_race() {
        let json = json!({"name": "Test Race", "source": "TEST"});
        let race: Race = serde_json::from_value(json).unwrap();
        assert_eq!(race.name, "Test Race");
        assert_eq!(race.source, "TEST");
        assert!(race.speed.is_none());
        assert!(race.lineage.is_none());
        assert!(race.height_and_weight.is_none());
    }

    #[test]
    fn test_race_speed_number() {
        let json = json!({"name": "Human", "source": "PHB", "speed": 30});
        let race: Race = serde_json::from_value(json).unwrap();
        assert_eq!(race.name, "Human");
        let speed = race.speed.unwrap();
        assert_eq!(speed.walk_speed(), 30);
    }

    #[test]
    fn test_race_speed_object() {
        let json = json!({"name": "Aarakocra", "source": "EEPC", "speed": {"walk": 25, "fly": 50}});
        let race: Race = serde_json::from_value(json).unwrap();
        let speed = race.speed.unwrap();
        assert_eq!(speed.walk_speed(), 25);
        match speed {
            RaceSpeed::Object(obj) => {
                assert_eq!(obj.fly.unwrap().as_number(), 50);
            }
            _ => panic!("Expected Object variant"),
        }
    }

    #[test]
    fn test_race_speed_with_swim_true() {
        // Some races have "swim": true meaning swim speed equals walk speed
        let json = json!({"name": "Giff", "source": "AAG", "speed": {"walk": 30, "swim": true}});
        let race: Race = serde_json::from_value(json).unwrap();
        let speed = race.speed.unwrap();
        match speed {
            RaceSpeed::Object(obj) => {
                assert!(obj.swim.as_ref().unwrap().is_equal_to_walk());
            }
            _ => panic!("Expected Object variant"),
        }
    }

    #[test]
    fn test_lineage_string() {
        let json = json!({"name": "Custom Lineage", "source": "TCE", "lineage": "VRGR"});
        let race: Race = serde_json::from_value(json).unwrap();
        match race.lineage.unwrap() {
            Lineage::Source(s) => assert_eq!(s, "VRGR"),
            _ => panic!("Expected Source variant"),
        }
    }

    #[test]
    fn test_lineage_bool() {
        let json = json!({"name": "Variant Human", "source": "PHB", "lineage": true});
        let race: Race = serde_json::from_value(json).unwrap();
        match race.lineage.unwrap() {
            Lineage::Flag(b) => assert!(b),
            _ => panic!("Expected Flag variant"),
        }
    }

    #[test]
    fn test_height_and_weight() {
        let json = json!({
            "name": "Dragonborn",
            "source": "PHB",
            "heightAndWeight": {
                "baseHeight": 66,
                "baseWeight": 175,
                "heightMod": "2d8",
                "weightMod": "2d6"
            }
        });
        let race: Race = serde_json::from_value(json).unwrap();
        let hw = race.height_and_weight.unwrap();
        assert_eq!(hw.base_height, Some(66));
        assert_eq!(hw.base_weight, Some(175));
        assert_eq!(hw.height_mod, Some("2d8".to_string()));
        assert_eq!(hw.weight_mod, Some("2d6".to_string()));
    }

    #[test]
    fn test_full_race() {
        let json = json!({
            "name": "Elf",
            "source": "PHB",
            "page": 21,
            "size": ["M"],
            "speed": 30,
            "ability": [{"dex": 2}],
            "darkvision": 60,
            "traitTags": ["Fey Ancestry", "Trance"],
            "languageProficiencies": [{"common": true}, {"elvish": true}],
            "entries": ["Elves are a magical people of otherworldly grace..."],
            "srd": true
        });
        let race: Race = serde_json::from_value(json).unwrap();
        assert_eq!(race.name, "Elf");
        assert_eq!(race.size, Some(vec!["M".to_string()]));
        assert_eq!(race.darkvision, Some(60));
        assert!(race.ability.is_some());
        assert!(race.trait_tags.is_some());
        assert!(matches!(race.srd, Some(SrdValue::Flag(true))));
    }

    #[test]
    fn test_subrace() {
        let json = json!({
            "name": "Wood Elf",
            "source": "PHB",
            "raceName": "Elf",
            "raceSource": "PHB",
            "speed": 35,
            "ability": [{"wis": 1}],
            "entries": ["As a wood elf, you have keen senses and intuition..."]
        });
        let subrace: Subrace = serde_json::from_value(json).unwrap();
        assert_eq!(subrace.name, Some("Wood Elf".to_string()));
        assert_eq!(subrace.race_name, "Elf");
        let speed = subrace.speed.unwrap();
        assert_eq!(speed.walk_speed(), 35);
    }

    #[test]
    fn test_subrace_overwrite() {
        let json = json!({
            "name": "Variant Tiefling",
            "source": "SCAG",
            "raceName": "Tiefling",
            "raceSource": "PHB",
            "ability": [{"int": 1, "cha": 2}],
            "overwrite": {
                "ability": true
            },
            "entries": []
        });
        let subrace: Subrace = serde_json::from_value(json).unwrap();
        let overwrite = subrace.overwrite.unwrap();
        assert_eq!(overwrite.ability, Some(true));
    }

    #[test]
    fn test_race_data_container() {
        let json = json!({
            "race": [
                {"name": "Human", "source": "PHB", "speed": 30},
                {"name": "Elf", "source": "PHB", "speed": 30}
            ],
            "subrace": [
                {"name": "High Elf", "source": "PHB", "raceName": "Elf", "raceSource": "PHB"}
            ]
        });
        let data: RaceData = serde_json::from_value(json).unwrap();
        assert_eq!(data.race.len(), 2);
        assert_eq!(data.subrace.unwrap().len(), 1);
    }

    #[test]
    fn test_race_with_srd_alternate_name() {
        let json = json!({
            "name": "Halfling",
            "source": "PHB",
            "srd": "Lightfoot Halfling"
        });
        let race: Race = serde_json::from_value(json).unwrap();
        assert!(
            matches!(race.srd, Some(SrdValue::Name(ref s)) if s == "Lightfoot Halfling")
        );
    }

    #[test]
    fn test_age() {
        let json = json!({
            "name": "Dwarf",
            "source": "PHB",
            "age": {"mature": 50, "max": 350}
        });
        let race: Race = serde_json::from_value(json).unwrap();
        let age = race.age.unwrap();
        assert_eq!(age.mature, Some(50));
        assert_eq!(age.max, Some(350));
    }
}
