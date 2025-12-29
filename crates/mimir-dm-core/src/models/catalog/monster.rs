//! Monster catalog models

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
    pub creature_type: Option<serde_json::Value>, // Can be string or object
    pub alignment: Option<serde_json::Value>, // Can be array of strings or array of objects
    pub ac: Option<serde_json::Value>, // Can be number or array of AC objects
    pub hp: Option<serde_json::Value>, // Can be object or number
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
    pub cr: Option<serde_json::Value>, // Can be string or object

    // Traits, actions, etc.
    pub trait_entries: Option<Vec<serde_json::Value>>,
    pub action: Option<Vec<serde_json::Value>>,
    pub bonus: Option<Vec<serde_json::Value>>,
    pub reaction: Option<Vec<serde_json::Value>>,
    pub legendary: Option<Vec<serde_json::Value>>,
    pub legendary_group: Option<serde_json::Value>,
    pub mythic: Option<Vec<serde_json::Value>>,

    // Environment
    pub environment: Option<Vec<String>>,

    // Flags
    pub srd: Option<serde_json::Value>,
    pub basic_rules: Option<bool>,

    // Token image
    #[serde(rename = "hasToken")]
    pub has_token: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureType {
    #[serde(rename = "type")]
    pub base_type: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorClass {
    pub ac: u8,
    pub from: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitPoints {
    pub average: u32,
    pub formula: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speed {
    pub walk: Option<serde_json::Value>, // Can be number or object
    pub burrow: Option<serde_json::Value>,
    pub climb: Option<serde_json::Value>,
    pub fly: Option<serde_json::Value>, // Can be number or object with condition
    pub hover: Option<bool>,
    pub swim: Option<serde_json::Value>,
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
    pub entries: Option<Vec<serde_json::Value>>,
    pub images: Option<Vec<serde_json::Value>>,
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
        // Extract size
        let size = monster
            .size
            .as_ref()
            .and_then(|s| s.first())
            .cloned()
            .unwrap_or_else(|| "Medium".to_string());

        // Extract creature type
        let creature_type = if let Some(ct) = &monster.creature_type {
            match ct {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Object(obj) => obj
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                _ => "Unknown".to_string(),
            }
        } else {
            "Unknown".to_string()
        };

        // Extract alignment
        let alignment = if let Some(al) = &monster.alignment {
            match al {
                serde_json::Value::Array(arr) => arr
                    .first()
                    .and_then(|v| v.as_str())
                    .unwrap_or("unaligned")
                    .to_string(),
                _ => "unaligned".to_string(),
            }
        } else {
            "unaligned".to_string()
        };

        // Extract CR
        let (cr, cr_numeric) = if let Some(cr_val) = &monster.cr {
            match cr_val {
                serde_json::Value::String(s) => {
                    let numeric = match s.as_str() {
                        "1/8" => 0.125,
                        "1/4" => 0.25,
                        "1/2" => 0.5,
                        _ => s.parse().unwrap_or(0.0),
                    };
                    (s.clone(), numeric as f32)
                }
                serde_json::Value::Object(obj) => {
                    let cr_str = obj
                        .get("cr")
                        .and_then(|c| c.as_str())
                        .unwrap_or("0")
                        .to_string();
                    let numeric = match cr_str.as_str() {
                        "1/8" => 0.125,
                        "1/4" => 0.25,
                        "1/2" => 0.5,
                        _ => cr_str.parse().unwrap_or(0.0),
                    };
                    (cr_str, numeric as f32)
                }
                _ => ("0".to_string(), 0.0),
            }
        } else {
            ("0".to_string(), 0.0)
        };

        // Extract HP
        let hp = if let Some(hp_val) = &monster.hp {
            match hp_val {
                serde_json::Value::Number(n) => n.as_u64().unwrap_or(1) as u32,
                serde_json::Value::Object(obj) => {
                    obj.get("average").and_then(|a| a.as_u64()).unwrap_or(1) as u32
                }
                _ => 1,
            }
        } else {
            1
        };

        // Extract AC
        let ac = if let Some(ac_val) = &monster.ac {
            match ac_val {
                serde_json::Value::Number(n) => n.as_u64().unwrap_or(10) as u8,
                serde_json::Value::Array(arr) => arr
                    .first()
                    .and_then(|v| v.get("ac"))
                    .and_then(|a| a.as_u64())
                    .unwrap_or(10) as u8,
                _ => 10,
            }
        } else {
            10
        };

        // Extract environment
        let environment = monster.environment.clone().unwrap_or_default();

        // Get first trait or action for description
        let description = monster
            .trait_entries
            .as_ref()
            .and_then(|t| t.first())
            .and_then(|e| e.get("entries"))
            .and_then(|entries| entries.get(0))
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .chars()
            .take(200)
            .collect::<String>();

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
        // Extract simplified size (first one)
        let size = monster.size.as_ref().and_then(|s| s.first()).cloned();

        // Extract simplified creature type
        let creature_type = if let Some(ct) = &monster.creature_type {
            match ct {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Object(obj) => obj
                    .get("type")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string()),
                _ => None,
            }
        } else {
            None
        };

        // Extract simplified alignment (first one)
        let alignment = if let Some(al) = &monster.alignment {
            match al {
                serde_json::Value::Array(arr) => {
                    arr.first().and_then(|v| v.as_str()).map(|s| s.to_string())
                }
                serde_json::Value::String(s) => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        };

        // Extract CR and CR numeric
        let (cr, cr_numeric) = if let Some(cr_val) = &monster.cr {
            match cr_val {
                serde_json::Value::String(s) => {
                    let numeric = match s.as_str() {
                        "1/8" => 0.125,
                        "1/4" => 0.25,
                        "1/2" => 0.5,
                        _ => s.parse().unwrap_or(0.0),
                    };
                    (Some(s.clone()), Some(numeric))
                }
                serde_json::Value::Object(obj) => {
                    let cr_str = obj
                        .get("cr")
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string());
                    let numeric = cr_str.as_ref().and_then(|s| match s.as_str() {
                        "1/8" => Some(0.125),
                        "1/4" => Some(0.25),
                        "1/2" => Some(0.5),
                        _ => s.parse().ok(),
                    });
                    (cr_str, numeric)
                }
                _ => (None, None),
            }
        } else {
            (None, None)
        };

        // Extract HP
        let hp = if let Some(hp_val) = &monster.hp {
            match hp_val {
                serde_json::Value::Number(n) => n.as_u64().map(|h| h as i32),
                serde_json::Value::Object(obj) => obj
                    .get("average")
                    .and_then(|a| a.as_u64())
                    .map(|h| h as i32),
                _ => None,
            }
        } else {
            None
        };

        // Extract AC
        let ac = if let Some(ac_val) = &monster.ac {
            match ac_val {
                serde_json::Value::Number(n) => n.as_u64().map(|a| a as i32),
                serde_json::Value::Array(arr) => arr
                    .first()
                    .and_then(|v| v.get("ac"))
                    .and_then(|a| a.as_u64())
                    .map(|a| a as i32),
                _ => None,
            }
        } else {
            None
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
