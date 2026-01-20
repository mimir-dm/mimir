//! Monster catalog models

use super::types::{
    AlignmentValue, ArmorClassValue, ChallengeRatingValue, CreatureTypeValue, Entry,
    HitPointsValue, Image, LegendaryGroup, SpeedValue, SrdValue,
};
use crate::schema::catalog_monsters;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A D&D 5e monster/creature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monster {
    pub name: String,
    pub source: String,
    pub page: Option<u32>,
    pub size: Option<Vec<String>>, // S, M, L, etc.
    #[serde(rename = "type")]
    pub creature_type: Option<CreatureTypeValue>,
    pub alignment: Option<AlignmentValue>,
    pub ac: Option<ArmorClassValue>,
    pub hp: Option<HitPointsValue>,
    pub speed: Option<Speed>,

    // Ability scores
    pub str: Option<u8>,
    pub dex: Option<u8>,
    pub con: Option<u8>,
    pub int: Option<u8>,
    pub wis: Option<u8>,
    pub cha: Option<u8>,

    // Saves and skills
    pub save: Option<Saves>,
    pub skill: Option<Skills>,

    // Resistances and immunities
    pub damage_vulnerabilities: Option<Vec<String>>,
    pub damage_resistances: Option<Vec<String>>,
    pub damage_immunities: Option<Vec<String>>,
    pub condition_immunities: Option<Vec<String>>,

    // Senses
    pub senses: Option<Vec<String>>,
    pub passive: Option<u8>,
    pub languages: Option<Vec<String>>,

    // Challenge rating
    pub cr: Option<ChallengeRatingValue>,

    // Traits, actions, etc. - These are {name, entries} objects (not general Entry objects)
    #[serde(rename = "trait")]
    pub trait_entries: Option<Vec<MonsterAction>>,
    pub action: Option<Vec<MonsterAction>>,
    pub bonus: Option<Vec<MonsterAction>>,
    pub reaction: Option<Vec<MonsterAction>>,
    pub legendary: Option<Vec<MonsterAction>>,
    pub legendary_group: Option<LegendaryGroup>,
    pub mythic: Option<Vec<MonsterAction>>,

    // Environment
    pub environment: Option<Vec<String>>,

    // Flags
    pub srd: Option<SrdValue>,
    pub basic_rules: Option<bool>,

    // Token image
    #[serde(rename = "hasToken")]
    pub has_token: Option<bool>,
}

// Note: CreatureType, ArmorClass, HitPoints types are now in types.rs
// These local type aliases are kept for backwards compatibility in re-exports
pub type CreatureType = CreatureTypeValue;
pub type ArmorClass = ArmorClassValue;
pub type HitPoints = HitPointsValue;

/// A monster action, trait, reaction, or legendary action.
///
/// These are `{name, entries}` objects distinct from the general Entry system.
///
/// # Why entries uses `serde_json::Value`
///
/// We deliberately keep entries as `Value` rather than using the typed `Entry` enum because:
/// 1. The frontend handles 5etools tag processing (e.g., `{@atk mw}`, `{@damage 1d8}`)
/// 2. Using the typed Entry enum would lose information for unknown entry types
/// 3. This is a pass-through field - Rust doesn't need to process the content
///
/// See `types.rs` for the typed `Entry` enum when Rust-side processing is needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterAction {
    pub name: Option<String>,
    pub entries: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speed {
    pub walk: Option<SpeedValue>,
    pub burrow: Option<SpeedValue>,
    pub climb: Option<SpeedValue>,
    pub fly: Option<SpeedValue>,
    pub hover: Option<bool>,
    pub swim: Option<SpeedValue>,
    #[serde(rename = "canHover")]
    pub can_hover: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Saves {
    pub str: Option<String>,
    pub dex: Option<String>,
    pub con: Option<String>,
    pub int: Option<String>,
    pub wis: Option<String>,
    pub cha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub acrobatics: Option<String>,
    pub arcana: Option<String>,
    pub athletics: Option<String>,
    pub deception: Option<String>,
    pub history: Option<String>,
    pub insight: Option<String>,
    pub intimidation: Option<String>,
    pub investigation: Option<String>,
    pub medicine: Option<String>,
    pub nature: Option<String>,
    pub perception: Option<String>,
    pub performance: Option<String>,
    pub persuasion: Option<String>,
    pub religion: Option<String>,
    pub sleight_of_hand: Option<String>,
    pub stealth: Option<String>,
    pub survival: Option<String>,
}

/// Container for monster data from 5etools JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterData {
    pub monster: Vec<Monster>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterFluff {
    pub name: String,
    pub source: String,
    pub entries: Option<Vec<Entry>>,
    pub images: Option<Vec<Image>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterFluffData {
    #[serde(rename = "monsterFluff")]
    pub monster_fluff: Vec<MonsterFluff>,
}

/// Simplified monster for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterSummary {
    pub name: String,
    pub source: String,
    pub size: String,
    pub creature_type: String,
    pub alignment: String,
    pub cr: String,
    pub cr_numeric: f32, // For sorting
    pub hp: u32,
    pub ac: u8,
    pub environment: Vec<String>,
    pub description: String,
}

impl From<&Monster> for MonsterSummary {
    fn from(monster: &Monster) -> Self {
        use super::types::{AlignmentComponent, AlignmentValue, ChallengeRatingValue};

        // Extract size
        let size = monster
            .size
            .as_ref()
            .and_then(|s| s.first())
            .cloned()
            .unwrap_or_else(|| "Medium".to_string());

        // Extract creature type
        let creature_type = match &monster.creature_type {
            Some(CreatureTypeValue::Simple(s)) => s.clone(),
            Some(CreatureTypeValue::Complex { base_type, .. }) => base_type.clone(),
            None => "Unknown".to_string(),
        };

        // Extract alignment
        let alignment = match &monster.alignment {
            Some(AlignmentValue::Array(arr)) => {
                arr.first()
                    .map(|c| match c {
                        AlignmentComponent::Abbr(s) => s.clone(),
                        AlignmentComponent::Special { special } => special.clone(),
                        AlignmentComponent::Choice { alignment } => {
                            alignment.first().cloned().unwrap_or_else(|| "N".to_string())
                        }
                    })
                    .unwrap_or_else(|| "unaligned".to_string())
            }
            Some(AlignmentValue::Single(c)) => match c {
                AlignmentComponent::Abbr(s) => s.clone(),
                AlignmentComponent::Special { special } => special.clone(),
                AlignmentComponent::Choice { alignment } => {
                    alignment.first().cloned().unwrap_or_else(|| "N".to_string())
                }
            },
            None => "unaligned".to_string(),
        };

        // Helper to parse CR string to numeric
        fn cr_to_numeric(cr_str: &str) -> f32 {
            match cr_str {
                "1/8" => 0.125,
                "1/4" => 0.25,
                "1/2" => 0.5,
                _ => cr_str.parse().unwrap_or(0.0),
            }
        }

        // Extract CR
        let (cr, cr_numeric) = match &monster.cr {
            Some(ChallengeRatingValue::Simple(s)) => (s.clone(), cr_to_numeric(s)),
            Some(ChallengeRatingValue::Complex { cr, .. }) => (cr.clone(), cr_to_numeric(cr)),
            None => ("0".to_string(), 0.0),
        };

        // Extract HP
        let hp = match &monster.hp {
            Some(HitPointsValue::Standard { average, .. }) => *average as u32,
            Some(HitPointsValue::Number(n)) => *n as u32,
            Some(HitPointsValue::Special { .. }) => 1, // Special HP like "see entry"
            None => 1,
        };

        // Extract AC
        let ac = match &monster.ac {
            Some(ArmorClassValue::Number(n)) => *n as u8,
            Some(ArmorClassValue::Array(arr)) => arr
                .first()
                .and_then(|entry| entry.ac())
                .unwrap_or(10) as u8,
            None => 10,
        };

        // Extract environment
        let environment = monster.environment.clone().unwrap_or_default();

        // Get first trait's first entry for description
        let description = monster
            .trait_entries
            .as_ref()
            .and_then(|traits| traits.first())
            .and_then(|action| action.entries.as_ref())
            .and_then(|entries| entries.first())
            .and_then(|e| e.as_str())
            .map(|s| s.chars().take(200).collect())
            .unwrap_or_default();

        MonsterSummary {
            name: monster.name.clone(),
            source: monster.source.clone(),
            size,
            creature_type,
            alignment,
            cr,
            cr_numeric,
            hp,
            ac,
            environment,
            description,
        }
    }
}

// Database models
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = catalog_monsters)]
pub struct CatalogMonster {
    pub id: i32,
    pub name: String,
    pub size: Option<String>,
    pub creature_type: Option<String>,
    pub alignment: Option<String>,
    pub cr: Option<String>,
    pub cr_numeric: Option<f64>,
    pub hp: Option<i32>,
    pub ac: Option<i32>,
    pub source: String,
    pub page: Option<i32>,
    pub full_monster_json: String,
    pub fluff_json: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub token_image_path: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = catalog_monsters)]
pub struct NewCatalogMonster {
    pub name: String,
    pub size: Option<String>,
    pub creature_type: Option<String>,
    pub alignment: Option<String>,
    pub cr: Option<String>,
    pub cr_numeric: Option<f64>,
    pub hp: Option<i32>,
    pub ac: Option<i32>,
    pub source: String,
    pub page: Option<i32>,
    pub full_monster_json: String,
    pub fluff_json: Option<String>,
    pub token_image_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonsterFilters {
    pub name: Option<String>,
    pub sizes: Option<Vec<String>>,
    pub creature_types: Option<Vec<String>>,
    pub alignments: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
    pub min_cr: Option<f64>,
    pub max_cr: Option<f64>,
    pub min_hp: Option<i32>,
    pub max_hp: Option<i32>,
    pub environment: Option<Vec<String>>,
}

impl From<&CatalogMonster> for MonsterSummary {
    fn from(catalog: &CatalogMonster) -> Self {
        Self {
            name: catalog.name.clone(),
            source: catalog.source.clone(),
            size: catalog.size.clone().unwrap_or_else(|| "Medium".to_string()),
            creature_type: catalog
                .creature_type
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            alignment: catalog
                .alignment
                .clone()
                .unwrap_or_else(|| "unaligned".to_string()),
            cr: catalog.cr.clone().unwrap_or_else(|| "0".to_string()),
            cr_numeric: catalog.cr_numeric.unwrap_or(0.0) as f32,
            hp: catalog.hp.unwrap_or(1) as u32,
            ac: catalog.ac.unwrap_or(10) as u8,
            environment: vec![],        // Will be populated from JSON if needed
            description: String::new(), // Will be populated from JSON if needed
        }
    }
}

impl From<&Monster> for NewCatalogMonster {
    fn from(monster: &Monster) -> Self {
        use super::types::{AlignmentComponent, AlignmentValue, ChallengeRatingValue};

        // Extract simplified size (first one)
        let size = monster.size.as_ref().and_then(|s| s.first()).cloned();

        // Extract simplified creature type
        let creature_type = match &monster.creature_type {
            Some(CreatureTypeValue::Simple(s)) => Some(s.clone()),
            Some(CreatureTypeValue::Complex { base_type, .. }) => Some(base_type.clone()),
            None => None,
        };

        // Extract simplified alignment (first one)
        let alignment = match &monster.alignment {
            Some(AlignmentValue::Array(arr)) => arr.first().map(|c| match c {
                AlignmentComponent::Abbr(s) => s.clone(),
                AlignmentComponent::Special { special } => special.clone(),
                AlignmentComponent::Choice { alignment } => {
                    alignment.first().cloned().unwrap_or_else(|| "N".to_string())
                }
            }),
            Some(AlignmentValue::Single(c)) => Some(match c {
                AlignmentComponent::Abbr(s) => s.clone(),
                AlignmentComponent::Special { special } => special.clone(),
                AlignmentComponent::Choice { alignment } => {
                    alignment.first().cloned().unwrap_or_else(|| "N".to_string())
                }
            }),
            None => None,
        };

        // Helper to parse CR string to numeric
        fn cr_to_numeric(cr_str: &str) -> f64 {
            match cr_str {
                "1/8" => 0.125,
                "1/4" => 0.25,
                "1/2" => 0.5,
                _ => cr_str.parse().unwrap_or(0.0),
            }
        }

        // Extract CR and CR numeric
        let (cr, cr_numeric) = match &monster.cr {
            Some(ChallengeRatingValue::Simple(s)) => (Some(s.clone()), Some(cr_to_numeric(s))),
            Some(ChallengeRatingValue::Complex { cr, .. }) => {
                (Some(cr.clone()), Some(cr_to_numeric(cr)))
            }
            None => (None, None),
        };

        // Extract HP
        let hp = match &monster.hp {
            Some(HitPointsValue::Standard { average, .. }) => Some(*average),
            Some(HitPointsValue::Number(n)) => Some(*n),
            Some(HitPointsValue::Special { .. }) => None,
            None => None,
        };

        // Extract AC
        let ac = match &monster.ac {
            Some(ArmorClassValue::Number(n)) => Some(*n),
            Some(ArmorClassValue::Array(arr)) => arr.first().and_then(|entry| entry.ac()),
            None => None,
        };

        // Build token image path if monster has a token
        let token_image_path = if monster.has_token.unwrap_or(false) {
            Some(format!(
                "img/bestiary/tokens/{}/{}.webp",
                monster.source, monster.name
            ))
        } else {
            None
        };

        Self {
            name: monster.name.clone(),
            size,
            creature_type,
            alignment,
            cr,
            cr_numeric,
            hp,
            ac,
            source: monster.source.clone(),
            page: monster.page.map(|p| p as i32),
            full_monster_json: serde_json::to_string(monster).unwrap_or_default(),
            fluff_json: None, // Fluff data will be set separately during import
            token_image_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_legendary_group_deserialization() {
        let json = json!({
            "name": "Aboleth",
            "source": "MM"
        });

        let group: LegendaryGroup = serde_json::from_value(json).unwrap();
        assert_eq!(group.name, "Aboleth");
        assert_eq!(group.source, "MM");
    }

    #[test]
    fn test_srd_value_boolean() {
        let json = json!(true);
        let srd: SrdValue = serde_json::from_value(json).unwrap();
        assert!(matches!(srd, SrdValue::Flag(true)));

        let json = json!(false);
        let srd: SrdValue = serde_json::from_value(json).unwrap();
        assert!(matches!(srd, SrdValue::Flag(false)));
    }

    #[test]
    fn test_srd_value_string() {
        let json = json!("Apparatus of the Crab");
        let srd: SrdValue = serde_json::from_value(json).unwrap();
        assert!(matches!(srd, SrdValue::Name(ref s) if s == "Apparatus of the Crab"));
    }

    #[test]
    fn test_monster_with_legendary_group() {
        let json = json!({
            "name": "Adult Black Dragon",
            "source": "MM",
            "legendaryGroup": {
                "name": "Black Dragon",
                "source": "MM"
            },
            "srd": true
        });

        let monster: Monster = serde_json::from_value(json).unwrap();
        assert_eq!(monster.name, "Adult Black Dragon");
        assert!(monster.legendary_group.is_some());
        let lg = monster.legendary_group.unwrap();
        assert_eq!(lg.name, "Black Dragon");
        assert_eq!(lg.source, "MM");
        assert!(matches!(monster.srd, Some(SrdValue::Flag(true))));
    }

    #[test]
    fn test_monster_without_legendary_group() {
        let json = json!({
            "name": "Goblin",
            "source": "MM",
            "srd": true
        });

        let monster: Monster = serde_json::from_value(json).unwrap();
        assert_eq!(monster.name, "Goblin");
        assert!(monster.legendary_group.is_none());
    }

    #[test]
    fn test_monster_action_with_entries() {
        let json = json!({
            "name": "Multiattack",
            "entries": [
                "The dragon makes three attacks: one with its bite and two with its claws."
            ]
        });

        let action: MonsterAction = serde_json::from_value(json).unwrap();
        assert_eq!(action.name.as_deref(), Some("Multiattack"));
        assert!(action.entries.is_some());
        let entries = action.entries.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].as_str().unwrap(), "The dragon makes three attacks: one with its bite and two with its claws.");
    }

    #[test]
    fn test_full_monster_deserialization() {
        // Test with realistic MM monster data
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
            "trait": [
                {
                    "name": "Amphibious",
                    "entries": ["The aboleth can breathe air and water."]
                }
            ],
            "action": [
                {
                    "name": "Multiattack",
                    "entries": ["The aboleth makes three tentacle attacks."]
                }
            ],
            "legendary": [
                {
                    "name": "Detect",
                    "entries": ["The aboleth makes a Wisdom (Perception) check."]
                }
            ],
            "legendaryGroup": {
                "name": "Aboleth",
                "source": "MM"
            },
            "environment": ["underdark"],
            "srd": true,
            "hasToken": true
        });

        let monster: Monster = serde_json::from_value(json).unwrap();

        // Core fields
        assert_eq!(monster.name, "Aboleth");
        assert_eq!(monster.source, "MM");
        assert_eq!(monster.page, Some(13));

        // Size and type
        assert_eq!(monster.size, Some(vec!["L".to_string()]));

        // Ability scores
        assert_eq!(monster.str, Some(21));
        assert_eq!(monster.dex, Some(9));
        assert_eq!(monster.con, Some(15));
        assert_eq!(monster.int, Some(18));
        assert_eq!(monster.wis, Some(15));
        assert_eq!(monster.cha, Some(18));

        // Legendary group
        assert!(monster.legendary_group.is_some());
        let lg = monster.legendary_group.unwrap();
        assert_eq!(lg.name, "Aboleth");
        assert_eq!(lg.source, "MM");

        // SRD
        assert!(matches!(monster.srd, Some(SrdValue::Flag(true))));

        // Actions
        assert!(monster.action.is_some());
        assert!(monster.legendary.is_some());
        assert!(monster.trait_entries.is_some());
    }
}
