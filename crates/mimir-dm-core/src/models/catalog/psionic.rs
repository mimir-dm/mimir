use super::types::Entry;
use crate::schema::catalog_psionics;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsionicSummary {
    pub name: String,
    pub source: String,
    #[serde(rename = "type")]
    pub psionic_type: String, // "D" for Discipline, "T" for Talent
    pub order: Option<String>, // Avatar, Awakened, Immortal, Nomad, Wu Jen, etc.
    pub page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Psionic {
    pub name: String,
    pub source: String,
    #[serde(rename = "type")]
    pub psionic_type: String,
    pub order: Option<String>,
    pub page: Option<i32>,
    #[serde(default)]
    pub entries: Vec<Entry>,
    pub focus: Option<String>,           // Focus benefit for disciplines
    pub modes: Option<Vec<PsionicMode>>, // Modes for disciplines
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsionicMode {
    pub name: String,
    pub cost: PsionicCost,
    #[serde(default)]
    pub entries: Vec<Entry>,
    pub concentration: Option<ConcentrationDuration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsionicCost {
    pub min: i32,
    pub max: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationDuration {
    pub duration: i32,
    pub unit: String, // "min", "hr", etc.
}

// Database models for catalog_psionics table
#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = catalog_psionics)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogPsionic {
    pub id: i32,
    pub name: String,
    pub psionic_type: String,
    pub psionic_order: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_psionic_json: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = catalog_psionics)]
pub struct NewCatalogPsionic {
    pub name: String,
    pub psionic_type: String,
    pub psionic_order: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_psionic_json: String,
}

#[derive(Debug, Default)]
pub struct PsionicFilters {
    pub name: Option<String>,
    pub psionic_types: Option<Vec<String>>, // "D", "T"
    pub orders: Option<Vec<String>>,        // Avatar, Awakened, etc.
    pub sources: Option<Vec<String>>,
}

impl From<&Psionic> for NewCatalogPsionic {
    fn from(psionic: &Psionic) -> Self {
        NewCatalogPsionic {
            name: psionic.name.clone(),
            psionic_type: psionic.psionic_type.clone(),
            psionic_order: psionic.order.clone(),
            source: psionic.source.clone(),
            page: psionic.page,
            full_psionic_json: serde_json::to_string(psionic).unwrap_or_default(),
        }
    }
}

impl From<&CatalogPsionic> for PsionicSummary {
    fn from(catalog_psionic: &CatalogPsionic) -> Self {
        PsionicSummary {
            name: catalog_psionic.name.clone(),
            source: catalog_psionic.source.clone(),
            psionic_type: catalog_psionic.psionic_type.clone(),
            order: catalog_psionic.psionic_order.clone(),
            page: catalog_psionic.page,
        }
    }
}
