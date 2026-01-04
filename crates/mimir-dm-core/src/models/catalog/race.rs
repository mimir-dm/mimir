//! D&D 5e Race models for catalog

use super::types::{AbilityBonus, DamageModifier, Entry, Image, ProficiencyItem};
use crate::schema::catalog_races;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A player character race
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Race {
    pub name: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<serde_json::Value>, // Can be number or Speed object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<Vec<AbilityBonus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<Age>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub darkvision: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trait_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weapon_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armor_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resist: Option<Vec<DamageModifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immune: Option<Vec<DamageModifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerable: Option<Vec<DamageModifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_immune: Option<Vec<String>>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "soundClip")]
    pub sound_clip: Option<SoundClip>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<serde_json::Value>, // Can be boolean true or string (source)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "heightAndWeight")]
    pub height_and_weight: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub srd: Option<bool>,
}

/// A subrace variant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subrace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    pub race_name: String,
    pub race_source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<Vec<AbilityBonus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<serde_json::Value>, // Can be number or Speed object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub darkvision: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resist: Option<Vec<DamageModifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trait_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weapon_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armor_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_proficiencies: Option<Vec<ProficiencyItem>>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<Overwrite>,
}

/// Speed values for movement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Speed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub walk: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fly: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swim: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub climb: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burrow: Option<i32>,
}

/// Ability score bonuses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityScores {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub str: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dex: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub con: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wis: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cha: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choose: Option<Choose>,
}

/// Choice for ability scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choose {
    pub from: Vec<String>,
    pub count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i32>,
}

/// Age information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Age {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mature: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i32>,
}

/// Sound clip reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundClip {
    #[serde(rename = "type")]
    pub clip_type: String,
    pub path: String,
}

/// Overwrite settings for subraces
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Overwrite {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_proficiencies: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_proficiencies: Option<bool>,
}

/// Container for race data from JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race: Option<Vec<Race>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subrace: Option<Vec<Subrace>>,
}

/// Race fluff/lore data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceFluff {
    pub name: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<Entry>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<Image>,
}

/// Container for race fluff data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceFluffData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race_fluff: Option<Vec<RaceFluff>>,
}

/// Summary of a race for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceSummary {
    pub name: String,
    pub source: String,
    pub size: String,
    pub speed: i32,
    pub ability_bonuses: String,
    pub traits_count: usize,
    pub is_subrace: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_race: Option<String>,
}

impl From<&Race> for RaceSummary {
    fn from(race: &Race) -> Self {
        let size = race
            .size
            .as_ref()
            .and_then(|s| s.first())
            .map(|s| match s.as_str() {
                "T" => "Tiny",
                "S" => "Small",
                "M" => "Medium",
                "L" => "Large",
                "H" => "Huge",
                "G" => "Gargantuan",
                _ => s.as_str(),
            })
            .unwrap_or("Medium")
            .to_string();

        let speed = match &race.speed {
            Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(30) as i32,
            Some(serde_json::Value::Object(obj)) => {
                obj.get("walk").and_then(|v| v.as_i64()).unwrap_or(30) as i32
            }
            _ => 30,
        };

        let ability_bonuses = format_ability_bonuses(race.ability.as_ref());

        Self {
            name: race.name.clone(),
            source: race.source.clone(),
            size,
            speed,
            ability_bonuses,
            traits_count: race.entries.len(),
            is_subrace: false,
            parent_race: None,
        }
    }
}

impl From<&Subrace> for RaceSummary {
    fn from(subrace: &Subrace) -> Self {
        let ability_bonuses = format_ability_bonuses(subrace.ability.as_ref());

        let speed = match &subrace.speed {
            Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(30) as i32,
            Some(serde_json::Value::Object(obj)) => {
                obj.get("walk").and_then(|v| v.as_i64()).unwrap_or(30) as i32
            }
            _ => 30,
        };

        // Format subrace name as "Subrace, Race" for better sorting
        let name = match &subrace.name {
            Some(n) if !n.is_empty() => {
                format!("{}, {}", n, subrace.race_name)
            }
            _ => format!("{} (Base)", subrace.race_name),
        };

        Self {
            name,
            source: subrace.source.clone(),
            size: "Varies".to_string(),
            speed,
            ability_bonuses,
            traits_count: subrace.entries.len(),
            is_subrace: true,
            parent_race: Some(subrace.race_name.clone()),
        }
    }
}

fn format_ability_bonuses(ability: Option<&Vec<AbilityBonus>>) -> String {
    use super::types::AbilityBonus;

    match ability {
        Some(scores) => {
            scores
                .iter()
                .filter_map(|val| {
                    match val {
                        AbilityBonus::Fixed(bonuses) => {
                            let mut parts = Vec::new();

                            // Check each ability score
                            if let Some(v) = bonuses.get("str") {
                                parts.push(format!("STR +{}", v));
                            }
                            if let Some(v) = bonuses.get("dex") {
                                parts.push(format!("DEX +{}", v));
                            }
                            if let Some(v) = bonuses.get("con") {
                                parts.push(format!("CON +{}", v));
                            }
                            if let Some(v) = bonuses.get("int") {
                                parts.push(format!("INT +{}", v));
                            }
                            if let Some(v) = bonuses.get("wis") {
                                parts.push(format!("WIS +{}", v));
                            }
                            if let Some(v) = bonuses.get("cha") {
                                parts.push(format!("CHA +{}", v));
                            }

                            if !parts.is_empty() {
                                Some(parts.join(", "))
                            } else {
                                None
                            }
                        }
                        AbilityBonus::Choice(choice) => {
                            // Handle choose option
                            let count = choice.count.unwrap_or(1);
                            let from = choice
                                .from
                                .as_ref()
                                .map(|arr| arr.join("/"))
                                .or_else(|| {
                                    choice.choose.as_ref().and_then(|c| {
                                        c.from.as_ref().map(|arr| arr.join("/"))
                                    })
                                })
                                .unwrap_or_default();
                            Some(format!("Choose {} from {}", count, from))
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join("; ")
        }
        None => "None".to_string(),
    }
}

/// Database model for catalog_races table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = catalog_races)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogRace {
    pub id: i32,
    pub name: String,
    pub size: Option<String>,
    pub speed: Option<i32>,
    pub ability_bonuses: Option<String>,
    pub traits_count: i32,
    pub source: String,
    pub full_race_json: String,
    pub created_at: Option<String>,
}

/// Model for inserting new races into the database
#[derive(Insertable, Debug)]
#[diesel(table_name = catalog_races)]
pub struct NewCatalogRace {
    pub name: String,
    pub size: Option<String>,
    pub speed: Option<i32>,
    pub ability_bonuses: Option<String>,
    pub traits_count: i32,
    pub source: String,
    pub full_race_json: String,
}

/// Filter parameters for race search
#[derive(Debug, Clone)]
pub struct RaceFilters {
    pub search_pattern: Option<String>,
    pub sources: Option<Vec<String>>,
    pub sizes: Option<Vec<String>>,
    pub has_darkvision: Option<bool>,
    pub has_flight: Option<bool>,
}

impl From<&CatalogRace> for RaceSummary {
    fn from(race: &CatalogRace) -> Self {
        RaceSummary {
            name: race.name.clone(),
            source: race.source.clone(),
            size: race.size.clone().unwrap_or("Medium".to_string()),
            speed: race.speed.unwrap_or(30),
            ability_bonuses: race.ability_bonuses.clone().unwrap_or("None".to_string()),
            traits_count: race.traits_count as usize,
            is_subrace: race.name.contains(", "), // Detect subraces by name format
            parent_race: if race.name.contains(", ") {
                Some(race.name.split(", ").nth(1).unwrap_or("").to_string())
            } else {
                None
            },
        }
    }
}

impl From<&Race> for NewCatalogRace {
    fn from(race: &Race) -> Self {
        let race_summary = RaceSummary::from(race);

        NewCatalogRace {
            name: race.name.clone(),
            size: Some(race_summary.size),
            speed: Some(race_summary.speed),
            ability_bonuses: Some(race_summary.ability_bonuses),
            traits_count: race_summary.traits_count as i32,
            source: race.source.clone(),
            full_race_json: serde_json::to_string(race).unwrap_or_default(),
        }
    }
}

impl From<&Subrace> for NewCatalogRace {
    fn from(subrace: &Subrace) -> Self {
        let race_summary = RaceSummary::from(subrace);

        NewCatalogRace {
            name: race_summary.name, // Already formatted as "Subrace, Race"
            size: Some(race_summary.size),
            speed: Some(race_summary.speed),
            ability_bonuses: Some(race_summary.ability_bonuses),
            traits_count: race_summary.traits_count as i32,
            source: subrace.source.clone(),
            full_race_json: serde_json::to_string(subrace).unwrap_or_default(),
        }
    }
}
