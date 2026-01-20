//! Monster catalog extraction types
//!
//! Types for deserializing 5etools bestiary JSON data.

use super::types::{
    AlignmentValue, ArmorClassValue, ChallengeRatingValue, CreatureTypeValue, HitPointsValue,
    LegendaryGroup, SpeedValue, SrdValue,
};
use serde::{Deserialize, Serialize};

/// A D&D 5e monster/creature from 5etools bestiary data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monster {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub page: Option<u32>,

    // Size and type
    #[serde(default)]
    pub size: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub creature_type: Option<CreatureTypeValue>,
    #[serde(default)]
    pub alignment: Option<AlignmentValue>,

    // Combat stats
    #[serde(default)]
    pub ac: Option<ArmorClassValue>,
    #[serde(default)]
    pub hp: Option<HitPointsValue>,
    #[serde(default)]
    pub speed: Option<MonsterSpeed>,

    // Ability scores
    #[serde(default, rename = "str")]
    pub strength: Option<u8>,
    #[serde(default, rename = "dex")]
    pub dexterity: Option<u8>,
    #[serde(default, rename = "con")]
    pub constitution: Option<u8>,
    #[serde(default, rename = "int")]
    pub intelligence: Option<u8>,
    #[serde(default, rename = "wis")]
    pub wisdom: Option<u8>,
    #[serde(default, rename = "cha")]
    pub charisma: Option<u8>,

    // Saves and skills
    #[serde(default)]
    pub save: Option<MonsterSaves>,
    #[serde(default)]
    pub skill: Option<MonsterSkills>,

    // Resistances and immunities
    #[serde(default)]
    pub vulnerable: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub resist: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub immune: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub condition_immune: Option<Vec<serde_json::Value>>,

    // Senses and languages
    #[serde(default)]
    pub senses: Option<Vec<String>>,
    #[serde(default)]
    pub passive: Option<u8>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,

    // Challenge rating
    #[serde(default)]
    pub cr: Option<ChallengeRatingValue>,

    // Actions and abilities (stored as JSON blobs)
    #[serde(default, rename = "trait")]
    pub traits: Option<Vec<MonsterAction>>,
    #[serde(default)]
    pub action: Option<Vec<MonsterAction>>,
    #[serde(default)]
    pub bonus: Option<Vec<MonsterAction>>,
    #[serde(default)]
    pub reaction: Option<Vec<MonsterAction>>,
    #[serde(default)]
    pub legendary: Option<Vec<MonsterAction>>,
    #[serde(default)]
    pub legendary_group: Option<LegendaryGroup>,
    #[serde(default)]
    pub mythic: Option<Vec<MonsterAction>>,

    // Spellcasting (stored as JSON blob)
    #[serde(default)]
    pub spellcasting: Option<Vec<serde_json::Value>>,

    // Environment
    #[serde(default)]
    pub environment: Option<Vec<String>>,

    // Flags
    #[serde(default)]
    pub srd: Option<SrdValue>,
    #[serde(default)]
    pub basic_rules: Option<bool>,
    #[serde(default)]
    pub has_token: Option<bool>,
    #[serde(default)]
    pub has_fluff: Option<bool>,
    #[serde(default)]
    pub has_fluff_images: Option<bool>,

    // Copy directive (for monsters that inherit from others)
    #[serde(default, rename = "_copy")]
    pub copy: Option<serde_json::Value>,
}

/// Monster speed object with multiple movement types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonsterSpeed {
    #[serde(default)]
    pub walk: Option<SpeedValue>,
    #[serde(default)]
    pub burrow: Option<SpeedValue>,
    #[serde(default)]
    pub climb: Option<SpeedValue>,
    #[serde(default)]
    pub fly: Option<SpeedValue>,
    #[serde(default)]
    pub swim: Option<SpeedValue>,
    #[serde(default)]
    pub can_hover: Option<bool>,
    #[serde(default)]
    pub hover: Option<bool>,
}

/// Monster saving throw bonuses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterSaves {
    #[serde(default, rename = "str")]
    pub strength: Option<String>,
    #[serde(default, rename = "dex")]
    pub dexterity: Option<String>,
    #[serde(default, rename = "con")]
    pub constitution: Option<String>,
    #[serde(default, rename = "int")]
    pub intelligence: Option<String>,
    #[serde(default, rename = "wis")]
    pub wisdom: Option<String>,
    #[serde(default, rename = "cha")]
    pub charisma: Option<String>,
}

/// Monster skill bonuses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterSkills {
    #[serde(default)]
    pub acrobatics: Option<String>,
    #[serde(default, rename = "animal handling")]
    pub animal_handling: Option<String>,
    #[serde(default)]
    pub arcana: Option<String>,
    #[serde(default)]
    pub athletics: Option<String>,
    #[serde(default)]
    pub deception: Option<String>,
    #[serde(default)]
    pub history: Option<String>,
    #[serde(default)]
    pub insight: Option<String>,
    #[serde(default)]
    pub intimidation: Option<String>,
    #[serde(default)]
    pub investigation: Option<String>,
    #[serde(default)]
    pub medicine: Option<String>,
    #[serde(default)]
    pub nature: Option<String>,
    #[serde(default)]
    pub perception: Option<String>,
    #[serde(default)]
    pub performance: Option<String>,
    #[serde(default)]
    pub persuasion: Option<String>,
    #[serde(default)]
    pub religion: Option<String>,
    #[serde(default, rename = "sleight of hand")]
    pub sleight_of_hand: Option<String>,
    #[serde(default)]
    pub stealth: Option<String>,
    #[serde(default)]
    pub survival: Option<String>,
}

/// A monster action, trait, reaction, or legendary action.
///
/// We keep `entries` as `serde_json::Value` because:
/// 1. The frontend handles 5etools tag processing (e.g., `{@atk mw}`)
/// 2. This is pass-through data - Rust doesn't need to process the content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterAction {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
}

/// Container for monster data from 5etools bestiary JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterData {
    #[serde(default)]
    pub monster: Vec<Monster>,
}

/// Monster fluff (lore/description) data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub images: Option<Vec<serde_json::Value>>,
}

/// Container for monster fluff data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterFluffData {
    #[serde(rename = "monsterFluff", default)]
    pub monster_fluff: Vec<MonsterFluff>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_monster() {
        let json = json!({
            "name": "Goblin",
            "source": "MM",
            "page": 166,
            "size": ["S"],
            "type": "humanoid",
            "alignment": ["N", "E"],
            "ac": [{"ac": 15, "from": ["leather armor", "shield"]}],
            "hp": {"average": 7, "formula": "2d6"},
            "speed": {"walk": 30},
            "str": 8,
            "dex": 14,
            "con": 10,
            "int": 10,
            "wis": 8,
            "cha": 8,
            "skill": {"stealth": "+6"},
            "senses": ["darkvision 60 ft."],
            "passive": 9,
            "languages": ["Common", "Goblin"],
            "cr": "1/4",
            "srd": true
        });

        let monster: Monster = serde_json::from_value(json).unwrap();
        assert_eq!(monster.name, "Goblin");
        assert_eq!(monster.source, "MM");
        assert_eq!(monster.page, Some(166));
        assert_eq!(monster.strength, Some(8));
        assert_eq!(monster.dexterity, Some(14));
    }

    #[test]
    fn test_monster_with_legendary_group() {
        let json = json!({
            "name": "Adult Black Dragon",
            "source": "MM",
            "legendaryGroup": {
                "name": "Black Dragon",
                "source": "MM"
            }
        });

        let monster: Monster = serde_json::from_value(json).unwrap();
        assert!(monster.legendary_group.is_some());
        let lg = monster.legendary_group.unwrap();
        assert_eq!(lg.name, "Black Dragon");
    }

    #[test]
    fn test_full_monster() {
        let json = json!({
            "name": "Aboleth",
            "source": "MM",
            "page": 13,
            "size": ["L"],
            "type": "aberration",
            "alignment": ["L", "E"],
            "ac": [{"ac": 17, "from": ["natural armor"]}],
            "hp": {"average": 135, "formula": "18d10 + 36"},
            "speed": {"walk": 10, "swim": 40},
            "str": 21,
            "dex": 9,
            "con": 15,
            "int": 18,
            "wis": 15,
            "cha": 18,
            "save": {"con": "+6", "int": "+8", "wis": "+6"},
            "skill": {"history": "+12", "perception": "+10"},
            "senses": ["darkvision 120 ft."],
            "passive": 20,
            "languages": ["Deep Speech", "telepathy 120 ft."],
            "cr": "10",
            "trait": [{"name": "Amphibious", "entries": ["The aboleth can breathe air and water."]}],
            "action": [{"name": "Multiattack", "entries": ["The aboleth makes three tentacle attacks."]}],
            "legendary": [{"name": "Detect", "entries": ["The aboleth makes a Wisdom (Perception) check."]}],
            "legendaryGroup": {"name": "Aboleth", "source": "MM"},
            "environment": ["underdark"],
            "srd": true,
            "hasToken": true
        });

        let monster: Monster = serde_json::from_value(json).unwrap();

        assert_eq!(monster.name, "Aboleth");
        assert_eq!(monster.strength, Some(21));
        assert_eq!(monster.intelligence, Some(18));
        assert!(monster.legendary_group.is_some());
        assert!(monster.traits.is_some());
        assert!(monster.action.is_some());
        assert!(monster.legendary.is_some());
        assert_eq!(monster.environment, Some(vec!["underdark".to_string()]));
    }

    #[test]
    fn test_monster_data_container() {
        let json = json!({
            "monster": [
                {"name": "Goblin", "source": "MM"},
                {"name": "Orc", "source": "MM"}
            ]
        });

        let data: MonsterData = serde_json::from_value(json).unwrap();
        assert_eq!(data.monster.len(), 2);
        assert_eq!(data.monster[0].name, "Goblin");
        assert_eq!(data.monster[1].name, "Orc");
    }
}
