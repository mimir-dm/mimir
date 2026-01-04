//! Spell catalog models

use super::types::Entry;
use crate::schema::catalog_spells;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A D&D 5e spell
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spell {
    pub name: String,
    pub source: String,
    pub page: Option<u32>,
    pub level: u8,
    pub school: SpellSchool,
    pub time: Vec<CastingTime>,
    pub range: SpellRange,
    pub components: Components,
    pub duration: Vec<Duration>,
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub classes: Option<Classes>,
    #[serde(default)]
    pub scaling_level_dice: Option<ScalingLevelDice>,
    #[serde(default)]
    pub damage_inflict: Option<Vec<String>>,
    #[serde(default)]
    pub saving_throw: Option<Vec<String>>,
    #[serde(default)]
    pub meta: Option<SpellMeta>,
}

/// Spell metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellMeta {
    #[serde(default)]
    pub ritual: bool,
}

/// Spell school codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpellSchool {
    #[serde(rename = "A")]
    Abjuration,
    #[serde(rename = "C")]
    Conjuration,
    #[serde(rename = "D")]
    Divination,
    #[serde(rename = "E")]
    Enchantment,
    #[serde(rename = "V")]
    Evocation,
    #[serde(rename = "I")]
    Illusion,
    #[serde(rename = "N")]
    Necromancy,
    #[serde(rename = "T")]
    Transmutation,
}

impl SpellSchool {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Abjuration => "Abjuration",
            Self::Conjuration => "Conjuration",
            Self::Divination => "Divination",
            Self::Enchantment => "Enchantment",
            Self::Evocation => "Evocation",
            Self::Illusion => "Illusion",
            Self::Necromancy => "Necromancy",
            Self::Transmutation => "Transmutation",
        }
    }
}

/// Casting time specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastingTime {
    pub number: u32,
    pub unit: String,
    #[serde(default)]
    pub condition: Option<String>,
}

/// Spell range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpellRange {
    Point {
        #[serde(rename = "type")]
        range_type: String,
        distance: Distance,
    },
    Special {
        #[serde(rename = "type")]
        range_type: String,
    },
}

/// Distance specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Distance {
    #[serde(rename = "type")]
    pub distance_type: String,
    #[serde(default)]
    pub amount: Option<u32>,
}

/// Spell components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    #[serde(default)]
    pub v: Option<bool>,
    #[serde(default)]
    pub s: Option<bool>,
    #[serde(default)]
    pub m: Option<MaterialComponent>,
    #[serde(default)]
    pub r: Option<bool>,
}

/// Material component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaterialComponent {
    Text(String),
    Object {
        text: String,
        #[serde(default)]
        cost: Option<u32>,
        #[serde(default)]
        consume: Option<serde_json::Value>, // Can be bool or "optional"
    },
    Bool(bool), // Sometimes it's just true/false
}

/// Spell duration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duration {
    #[serde(rename = "type")]
    pub duration_type: String,
    #[serde(default)]
    pub duration: Option<DurationValue>,
    #[serde(default)]
    pub concentration: Option<bool>,
    #[serde(default)]
    pub ends: Option<Vec<String>>,
}

/// Duration value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(default)]
    pub amount: Option<u32>,
    #[serde(default)]
    pub up_to: Option<bool>,
}

/// Classes that can cast the spell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classes {
    #[serde(rename = "fromClassList")]
    pub from_class_list: Option<Vec<ClassReference>>,
    #[serde(rename = "fromSubclass")]
    pub from_subclass: Option<Vec<SubclassReference>>,
}

/// Class reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassReference {
    pub name: String,
    pub source: String,
}

/// Subclass reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassReference {
    pub class: ClassReference,
    pub subclass: SubclassReference2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassReference2 {
    pub name: String,
    pub source: String,
}

/// Scaling level dice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingLevelDice {
    pub label: String,
    pub scaling: std::collections::HashMap<String, String>,
}

/// Container for spell data from JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellData {
    pub spell: Vec<Spell>,
}

/// Simplified spell for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellSummary {
    pub name: String,
    pub level: u8,
    pub school: String,
    pub source: String,
    pub concentration: bool,
    pub ritual: bool,
    pub casting_time: String,
    pub range: String,
    pub components: String,
    pub classes: Vec<String>,
    pub description: String,
}

impl From<&Spell> for SpellSummary {
    fn from(spell: &Spell) -> Self {
        // Format casting time
        let casting_time = spell
            .time
            .first()
            .map(|t| format!("{} {}", t.number, t.unit))
            .unwrap_or_else(|| "Unknown".to_string());

        // Format range
        let range = match &spell.range {
            SpellRange::Point { distance, .. } => {
                if let Some(amount) = distance.amount {
                    format!("{} {}", amount, distance.distance_type)
                } else {
                    distance.distance_type.clone()
                }
            }
            SpellRange::Special { range_type } => range_type.clone(),
        };

        // Format components
        let mut comp_parts = vec![];
        if spell.components.v.unwrap_or(false) {
            comp_parts.push("V");
        }
        if spell.components.s.unwrap_or(false) {
            comp_parts.push("S");
        }
        if spell.components.m.is_some() {
            comp_parts.push("M");
        }
        let components = comp_parts.join(", ");

        // Extract classes
        let classes = spell
            .classes
            .as_ref()
            .and_then(|c| c.from_class_list.as_ref())
            .map(|list| list.iter().map(|c| c.name.clone()).collect())
            .unwrap_or_default();

        // Check for concentration and ritual
        let concentration = spell
            .duration
            .iter()
            .any(|d| d.concentration.unwrap_or(false));

        let ritual = spell.meta.as_ref().map(|m| m.ritual).unwrap_or(false);

        // Get first line of description for summary
        let description = spell
            .entries
            .first()
            .and_then(|e| match e {
                Entry::Text(s) => Some(s.as_str()),
                Entry::Object(_) => None,
            })
            .unwrap_or("")
            .chars()
            .take(200)
            .collect::<String>();

        SpellSummary {
            name: spell.name.clone(),
            level: spell.level,
            school: spell.school.as_str().to_string(),
            source: spell.source.clone(),
            concentration,
            ritual,
            casting_time,
            range,
            components,
            classes,
            description,
        }
    }
}

/// Database model for a spell in the catalog_spells table
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = catalog_spells)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogSpell {
    pub id: i32,
    pub name: String,
    pub level: i32,
    pub school: String,
    pub cast_time: String,
    pub range: String,
    pub components: String,
    pub tags: String, // JSON string of tags array
    pub source: String,
    pub full_spell_json: String, // Complete spell JSON for modal display
}

/// Insertable model for new spells in the database
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = catalog_spells)]
pub struct NewCatalogSpell {
    pub name: String,
    pub level: i32,
    pub school: String,
    pub cast_time: String,
    pub range: String,
    pub components: String,
    pub tags: String,
    pub source: String,
    pub full_spell_json: String,
}

impl CatalogSpell {
    /// Convert database spell to frontend SpellSummary
    pub fn to_summary(&self) -> SpellSummary {
        // Parse tags to extract boolean flags
        let tags: Vec<String> = serde_json::from_str(&self.tags).unwrap_or_default();
        let concentration = tags.contains(&"Concentration".to_string());
        let ritual = tags.contains(&"Ritual".to_string());

        // Parse full spell JSON to extract classes and description
        let full_spell: serde_json::Value =
            serde_json::from_str(&self.full_spell_json).unwrap_or_default();

        let classes = full_spell
            .get("classes")
            .and_then(|c| c.get("fromClassList"))
            .and_then(|list| list.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|class| class.get("name")?.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        let description = full_spell
            .get("entries")
            .and_then(|entries| entries.as_array())
            .and_then(|arr| arr.first())
            .and_then(|entry| entry.as_str())
            .unwrap_or("")
            .chars()
            .take(200)
            .collect::<String>();

        SpellSummary {
            name: self.name.clone(),
            level: self.level as u8,
            school: self.school.clone(),
            source: self.source.clone(),
            concentration,
            ritual,
            casting_time: self.cast_time.clone(),
            range: self.range.clone(),
            components: self.components.clone(),
            classes,
            description,
        }
    }
}

/// Spell filter parameters for database queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpellFilters {
    pub query: Option<String>,   // Name search
    pub levels: Vec<i32>,        // Filter by level
    pub schools: Vec<String>,    // Filter by school
    pub sources: Vec<String>,    // Filter by book
    pub tags: Vec<String>,       // Filter by tags (concentration, ritual, etc.)
    pub classes: Vec<String>,    // Filter by class (Wizard, Cleric, etc.)
    pub limit: Option<i32>,      // Pagination
    pub offset: Option<i32>,
}

impl NewCatalogSpell {
    /// Transform a JSON spell into a database-ready format
    pub fn from_spell(spell: Spell, source: &str) -> Self {
        // Format casting time
        let cast_time = spell
            .time
            .first()
            .map(|t| {
                let base = format!("{} {}", t.number, t.unit);
                if let Some(condition) = &t.condition {
                    format!("{} ({})", base, condition)
                } else {
                    base
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        // Format range
        let range = match &spell.range {
            SpellRange::Point { distance, .. } => {
                if let Some(amount) = distance.amount {
                    format!("{} {}", amount, distance.distance_type)
                } else {
                    distance.distance_type.clone()
                }
            }
            SpellRange::Special { range_type } => range_type.clone(),
        };

        // Format components
        let mut comp_parts = Vec::new();
        if spell.components.v.unwrap_or(false) {
            comp_parts.push("V".to_string());
        }
        if spell.components.s.unwrap_or(false) {
            comp_parts.push("S".to_string());
        }
        if spell.components.m.is_some() {
            comp_parts.push("M".to_string());
        }
        let components = comp_parts.join(", ");

        // Build tags array
        let mut tags = Vec::new();

        // Check for concentration
        if spell
            .duration
            .iter()
            .any(|d| d.concentration.unwrap_or(false))
        {
            tags.push("Concentration".to_string());
        }

        // Check for ritual
        if spell.meta.as_ref().map(|m| m.ritual).unwrap_or(false) {
            tags.push("Ritual".to_string());
        }

        // Note: SRD and Basic Rules metadata would be found at spell root level,
        // not in entries. Entries contain descriptive text only.

        Self {
            name: spell.name.clone(),
            level: spell.level as i32,
            school: spell.school.as_str().to_string(),
            cast_time,
            range,
            components,
            tags: serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string()),
            source: source.to_string(),
            full_spell_json: serde_json::to_string(&spell).unwrap_or_else(|_| "{}".to_string()),
        }
    }
}
