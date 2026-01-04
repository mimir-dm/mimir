//! Monster catalog models

use super::types::{
    AlignmentValue, ArmorClassValue, ChallengeRatingValue, CreatureTypeValue, Entry,
    HitPointsValue, Image, SpeedValue,
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

    // Traits, actions, etc.
    pub trait_entries: Option<Vec<Entry>>,
    pub action: Option<Vec<Entry>>,
    pub bonus: Option<Vec<Entry>>,
    pub reaction: Option<Vec<Entry>>,
    pub legendary: Option<Vec<Entry>>,
    pub legendary_group: Option<serde_json::Value>, // Keep as Value - complex structure
    pub mythic: Option<Vec<Entry>>,

    // Environment
    pub environment: Option<Vec<String>>,

    // Flags
    pub srd: Option<serde_json::Value>, // Can be true or "..."
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
                .and_then(|entry| entry.ac)
                .unwrap_or(10) as u8,
            None => 10,
        };

        // Extract environment
        let environment = monster.environment.clone().unwrap_or_default();

        // Get first trait or action for description
        let description = monster
            .trait_entries
            .as_ref()
            .and_then(|t| t.first())
            .map(|e| match e {
                Entry::Text(s) => s.chars().take(200).collect(),
                Entry::Object(_) => String::new(), // Complex entry - skip for now
            })
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
            Some(ArmorClassValue::Array(arr)) => arr.first().and_then(|entry| entry.ac),
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
