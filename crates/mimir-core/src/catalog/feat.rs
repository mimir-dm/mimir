//! Feat catalog extraction types
//!
//! Types for deserializing 5etools feat JSON data.

use super::types::{AbilityBonus, DamageModifier, OtherSource, ProficiencyItem, SrdValue};
use serde::{Deserialize, Serialize};

/// A D&D 5e feat from 5etools data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feat {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    // Entries (stored as JSON blob)
    #[serde(default)]
    pub entries: Vec<serde_json::Value>,

    // Prerequisites (stored as JSON blob due to complex polymorphic structure)
    #[serde(default)]
    pub prerequisite: Option<Vec<serde_json::Value>>,

    // Ability score increases
    #[serde(default)]
    pub ability: Option<Vec<AbilityBonus>>,

    // Proficiencies gained
    #[serde(default)]
    pub skill_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub language_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub tool_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub weapon_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub armor_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub saving_throw_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub expertise: Option<Vec<ProficiencyItem>>,

    // Resistances and immunities
    #[serde(default)]
    pub resist: Option<Vec<DamageModifier>>,
    #[serde(default)]
    pub immune: Option<Vec<DamageModifier>>,

    // Senses
    #[serde(default)]
    pub senses: Option<Vec<ProficiencyItem>>,

    // Additional spells (stored as JSON blob)
    #[serde(default)]
    pub additional_spells: Option<Vec<serde_json::Value>>,

    // Other sources
    #[serde(default)]
    pub other_sources: Option<Vec<OtherSource>>,

    // Category (e.g., "General", "Racial")
    #[serde(default)]
    pub category: Option<String>,

    // Repeatable flag
    #[serde(default)]
    pub repeatable: Option<bool>,
    #[serde(default)]
    pub repeatable_note: Option<String>,

    // SRD status
    #[serde(default)]
    pub srd: Option<SrdValue>,
    #[serde(default)]
    pub basic_rules: Option<bool>,

    // Copy directive
    #[serde(default, rename = "_copy")]
    pub copy: Option<serde_json::Value>,
}

/// Container for feat data from 5etools JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatData {
    #[serde(default)]
    pub feat: Vec<Feat>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_minimal_feat() {
        let json = json!({
            "name": "Test Feat",
            "source": "TEST"
        });
        let feat: Feat = serde_json::from_value(json).unwrap();
        assert_eq!(feat.name, "Test Feat");
        assert_eq!(feat.source, "TEST");
        assert!(feat.prerequisite.is_none());
    }

    #[test]
    fn test_feat_with_ability_bonus() {
        let json = json!({
            "name": "Actor",
            "source": "PHB",
            "page": 165,
            "ability": [
                {"cha": 1}
            ],
            "entries": [
                "Skilled at mimicry and dramatics, you gain the following benefits:",
                {"type": "list", "items": ["Increase your Charisma score by 1..."]}
            ],
            "srd": true
        });
        let feat: Feat = serde_json::from_value(json).unwrap();
        assert_eq!(feat.name, "Actor");
        assert!(feat.ability.is_some());
        assert!(matches!(feat.srd, Some(SrdValue::Flag(true))));
    }

    #[test]
    fn test_feat_with_prerequisites() {
        let json = json!({
            "name": "Ritual Caster",
            "source": "PHB",
            "prerequisite": [
                {"ability": [{"int": 13}, {"wis": 13}]}
            ],
            "entries": ["You have learned a number of spells that you can cast as rituals..."]
        });
        let feat: Feat = serde_json::from_value(json).unwrap();
        assert_eq!(feat.name, "Ritual Caster");
        assert!(feat.prerequisite.is_some());
        assert_eq!(feat.prerequisite.unwrap().len(), 1);
    }

    #[test]
    fn test_feat_with_proficiencies() {
        let json = json!({
            "name": "Weapon Master",
            "source": "PHB",
            "ability": [
                {"choose": {"from": ["str", "dex"], "count": 1}}
            ],
            "weaponProficiencies": [
                {"choose": {"count": 4}}
            ],
            "entries": ["You have practiced extensively with a variety of weapons..."]
        });
        let feat: Feat = serde_json::from_value(json).unwrap();
        assert_eq!(feat.name, "Weapon Master");
        assert!(feat.weapon_proficiencies.is_some());
    }

    #[test]
    fn test_feat_with_additional_spells() {
        let json = json!({
            "name": "Magic Initiate",
            "source": "PHB",
            "additionalSpells": [
                {
                    "innate": {
                        "_": {"daily": {"1": [{"choose": "level=1|class=Bard"}]}}
                    },
                    "ability": {"choose": ["int", "wis", "cha"]}
                }
            ],
            "entries": ["Choose a class: bard, cleric, druid, sorcerer, warlock, or wizard..."]
        });
        let feat: Feat = serde_json::from_value(json).unwrap();
        assert_eq!(feat.name, "Magic Initiate");
        assert!(feat.additional_spells.is_some());
    }

    #[test]
    fn test_feat_with_resistance() {
        let json = json!({
            "name": "Dragon Fear",
            "source": "XGE",
            "prerequisite": [
                {"race": [{"name": "dragonborn"}]}
            ],
            "ability": [
                {"choose": {"from": ["str", "con", "cha"], "count": 1}}
            ],
            "resist": [
                {"choose": {"from": ["acid", "cold", "fire", "lightning", "poison"]}}
            ],
            "entries": ["When angered, you radiate menace..."]
        });
        let feat: Feat = serde_json::from_value(json).unwrap();
        assert_eq!(feat.name, "Dragon Fear");
        assert!(feat.resist.is_some());
    }

    #[test]
    fn test_feat_repeatable() {
        let json = json!({
            "name": "Elemental Adept",
            "source": "PHB",
            "prerequisite": [
                {"spellcasting": true}
            ],
            "repeatable": true,
            "repeatableNote": "Choose a different damage type each time",
            "entries": ["When you gain this feat, choose one of the following damage types..."]
        });
        let feat: Feat = serde_json::from_value(json).unwrap();
        assert_eq!(feat.name, "Elemental Adept");
        assert_eq!(feat.repeatable, Some(true));
        assert!(feat.repeatable_note.is_some());
    }

    #[test]
    fn test_feat_data_container() {
        let json = json!({
            "feat": [
                {"name": "Alert", "source": "PHB"},
                {"name": "Athlete", "source": "PHB"},
                {"name": "Actor", "source": "PHB"}
            ]
        });
        let data: FeatData = serde_json::from_value(json).unwrap();
        assert_eq!(data.feat.len(), 3);
    }

    #[test]
    fn test_feat_with_category() {
        let json = json!({
            "name": "Elven Accuracy",
            "source": "XGE",
            "category": "Racial",
            "prerequisite": [
                {"race": [{"name": "elf"}, {"name": "half-elf"}]}
            ],
            "entries": ["The accuracy of elves is legendary..."]
        });
        let feat: Feat = serde_json::from_value(json).unwrap();
        assert_eq!(feat.category, Some("Racial".to_string()));
    }
}
