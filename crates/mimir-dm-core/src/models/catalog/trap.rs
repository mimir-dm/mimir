use super::types::Entry;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trap {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,

    #[serde(rename = "trapHazType")]
    pub trap_haz_type: Option<String>, // MECH, MAG, WLD, WTH, ENV

    #[serde(default)]
    pub entries: Vec<Entry>,

    pub srd: Option<bool>,

    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,

    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hazard {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,

    #[serde(rename = "trapHazType")]
    pub trap_haz_type: Option<String>, // MECH, MAG, WLD, WTH, ENV

    #[serde(default)]
    pub entries: Vec<Entry>,

    pub srd: Option<bool>,

    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,

    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
}

// Combined type for unified handling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TrapOrHazard {
    Trap(Trap),
    Hazard(Hazard),
}

impl TrapOrHazard {
    pub fn name(&self) -> &str {
        match self {
            TrapOrHazard::Trap(t) => &t.name,
            TrapOrHazard::Hazard(h) => &h.name,
        }
    }

    pub fn source(&self) -> &str {
        match self {
            TrapOrHazard::Trap(t) => &t.source,
            TrapOrHazard::Hazard(h) => &h.source,
        }
    }

    pub fn trap_haz_type(&self) -> Option<&String> {
        match self {
            TrapOrHazard::Trap(t) => t.trap_haz_type.as_ref(),
            TrapOrHazard::Hazard(h) => h.trap_haz_type.as_ref(),
        }
    }

    pub fn is_trap(&self) -> bool {
        matches!(self, TrapOrHazard::Trap(_))
    }

    pub fn is_srd(&self) -> bool {
        match self {
            TrapOrHazard::Trap(t) => t.srd.unwrap_or(false),
            TrapOrHazard::Hazard(h) => h.srd.unwrap_or(false),
        }
    }
}

// Summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapSummary {
    pub name: String,
    pub source: String,
    pub trap_type: String,
    pub category: String,
}

impl From<&TrapOrHazard> for TrapSummary {
    fn from(item: &TrapOrHazard) -> Self {
        Self {
            name: item.name().to_string(),
            source: item.source().to_string(),
            trap_type: format_trap_type(item.trap_haz_type()),
            category: if item.is_trap() {
                "Trap".to_string()
            } else {
                "Hazard".to_string()
            },
        }
    }
}

fn format_trap_type(trap_type: Option<&String>) -> String {
    match trap_type.map(|s| s.as_str()) {
        Some("MECH") => "Mechanical".to_string(),
        Some("MAG") => "Magical".to_string(),
        Some("WLD") => "Wilderness".to_string(),
        Some("WTH") => "Weather".to_string(),
        Some("ENV") => "Environmental".to_string(),
        Some(other) => other.to_string(),
        None => "Unknown".to_string(),
    }
}

// Container for JSON parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct TrapData {
    pub trap: Option<Vec<Trap>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HazardData {
    pub hazard: Option<Vec<Hazard>>,
}

// Database models for Diesel
#[derive(Queryable, Selectable, Debug, Clone, Serialize)]
#[diesel(table_name = crate::schema::catalog_traps)]
pub struct CatalogTrap {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub trap_type: Option<String>,
    pub source: String,
    pub full_trap_json: String,
    #[serde(skip)]
    pub created_at: Option<String>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::catalog_traps)]
pub struct NewCatalogTrap {
    pub name: String,
    pub category: String,
    pub trap_type: Option<String>,
    pub source: String,
    pub full_trap_json: String,
}

// Filter struct for search parameters
#[derive(Debug, Default, Deserialize)]
pub struct TrapFilters {
    pub search: Option<String>,
    pub sources: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub trap_types: Option<Vec<String>>,
}

impl From<&TrapOrHazard> for NewCatalogTrap {
    fn from(item: &TrapOrHazard) -> Self {
        Self {
            name: item.name().to_string(),
            category: if item.is_trap() {
                "Trap".to_string()
            } else {
                "Hazard".to_string()
            },
            trap_type: item.trap_haz_type().map(|t| format_trap_type(Some(t))),
            source: item.source().to_string(),
            full_trap_json: serde_json::to_string(item).unwrap_or_default(),
        }
    }
}
