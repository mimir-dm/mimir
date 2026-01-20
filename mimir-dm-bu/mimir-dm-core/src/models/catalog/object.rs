use super::types::Entry;
use crate::schema::catalog_objects;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DamageType {
    Simple(String),
    Special { special: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArmorClass {
    Number(i32),
    Object {
        #[serde(default)]
        ac: Option<i32>,
        #[serde(default)]
        special: Option<String>,
        #[serde(default)]
        from: Option<Vec<String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HitPoints {
    Number(i32),
    Object {
        #[serde(default)]
        hp: Option<i32>,
        #[serde(default)]
        special: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackEntry {
    #[serde(rename = "attackType")]
    pub attack_type: Option<String>, // MW (Melee Weapon), RW (Ranged Weapon), etc.

    #[serde(rename = "attackEntries")]
    pub attack_entries: Option<Vec<String>>,

    #[serde(rename = "hitEntries")]
    pub hit_entries: Option<Vec<String>>,

    #[serde(rename = "type")]
    pub entry_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEntry {
    pub name: String,

    #[serde(rename = "type")]
    pub action_type: Option<String>,

    #[serde(default)]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DndObject {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,

    #[serde(rename = "objectType")]
    pub object_type: Option<String>, // SW (Siege Weapon), etc.

    pub size: Option<Vec<String>>,

    pub ac: Option<ArmorClass>,
    pub hp: Option<HitPoints>,

    pub immune: Option<Vec<DamageType>>,
    pub resist: Option<Vec<DamageType>>,
    pub vulnerable: Option<Vec<DamageType>>,

    #[serde(rename = "actionEntries")]
    pub action_entries: Option<Vec<ActionEntry>>,

    #[serde(default)]
    pub entries: Vec<Entry>,

    #[serde(rename = "hasToken")]
    pub has_token: Option<bool>,

    #[serde(rename = "tokenCredit")]
    pub token_credit: Option<String>,

    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,

    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
}

// Summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSummary {
    pub name: String,
    pub source: String,
    pub object_type: String,
    pub size: String,
    pub ac: String,
    pub hp: String,
}

impl From<&DndObject> for ObjectSummary {
    fn from(obj: &DndObject) -> Self {
        Self {
            name: obj.name.clone(),
            source: obj.source.clone(),
            object_type: format_object_type(&obj.object_type),
            size: format_size(&obj.size),
            ac: format_ac(&obj.ac),
            hp: format_hp(&obj.hp),
        }
    }
}

fn format_object_type(object_type: &Option<String>) -> String {
    match object_type.as_deref() {
        Some("SW") => "Siege Weapon".to_string(),
        Some("GEN") => "Generic".to_string(),
        Some(other) => other.to_string(),
        None => "Object".to_string(),
    }
}

fn format_size(size: &Option<Vec<String>>) -> String {
    if let Some(sizes) = size {
        sizes
            .iter()
            .map(|s| match s.as_str() {
                "T" => "Tiny",
                "S" => "Small",
                "M" => "Medium",
                "L" => "Large",
                "H" => "Huge",
                "G" => "Gargantuan",
                _ => s.as_str(),
            })
            .collect::<Vec<_>>()
            .join("/")
    } else {
        "—".to_string()
    }
}

fn format_ac(ac: &Option<ArmorClass>) -> String {
    match ac {
        Some(ArmorClass::Number(n)) => n.to_string(),
        Some(ArmorClass::Object { ac: Some(n), .. }) => n.to_string(),
        Some(ArmorClass::Object {
            special: Some(s), ..
        }) => s.clone(),
        _ => "—".to_string(),
    }
}

fn format_hp(hp: &Option<HitPoints>) -> String {
    match hp {
        Some(HitPoints::Number(n)) => n.to_string(),
        Some(HitPoints::Object { hp: Some(n), .. }) => n.to_string(),
        Some(HitPoints::Object {
            special: Some(s), ..
        }) => s.clone(),
        _ => "—".to_string(),
    }
}

// Container for JSON parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectData {
    pub object: Option<Vec<DndObject>>,
}

/// Database model for catalog_objects table
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = catalog_objects)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogObject {
    pub id: i32,
    pub name: String,
    pub object_type: Option<String>,
    pub size: Option<String>,
    pub ac: Option<String>,
    pub hp: Option<String>,
    pub source: String,
    pub full_object_json: String,
    pub created_at: Option<String>,
}

/// Model for inserting new objects into the database
#[derive(Insertable, Debug)]
#[diesel(table_name = catalog_objects)]
pub struct NewCatalogObject {
    pub name: String,
    pub object_type: Option<String>,
    pub size: Option<String>,
    pub ac: Option<String>,
    pub hp: Option<String>,
    pub source: String,
    pub full_object_json: String,
}

/// Filter parameters for object search
#[derive(Debug, Clone)]
pub struct ObjectFilters {
    pub search_pattern: Option<String>,
    pub sources: Option<Vec<String>>,
    pub object_types: Option<Vec<String>>,
    pub sizes: Option<Vec<String>>,
}

impl From<&CatalogObject> for ObjectSummary {
    fn from(obj: &CatalogObject) -> Self {
        ObjectSummary {
            name: obj.name.clone(),
            source: obj.source.clone(),
            object_type: obj
                .object_type
                .clone()
                .unwrap_or_else(|| "Object".to_string()),
            size: obj.size.clone().unwrap_or_else(|| "—".to_string()),
            ac: obj.ac.clone().unwrap_or_else(|| "—".to_string()),
            hp: obj.hp.clone().unwrap_or_else(|| "—".to_string()),
        }
    }
}

impl From<&DndObject> for NewCatalogObject {
    fn from(obj: &DndObject) -> Self {
        let object_summary = ObjectSummary::from(obj);

        NewCatalogObject {
            name: obj.name.clone(),
            object_type: Some(object_summary.object_type),
            size: Some(object_summary.size),
            ac: Some(object_summary.ac),
            hp: Some(object_summary.hp),
            source: obj.source.clone(),
            full_object_json: serde_json::to_string(obj).unwrap_or_default(),
        }
    }
}
