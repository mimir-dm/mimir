use super::types::Entry;
use crate::schema::catalog_deities;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deity {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,

    pub title: Option<String>,
    pub pantheon: Option<String>,
    pub alignment: Option<Vec<String>>,
    pub domains: Option<Vec<String>>,
    pub symbol: Option<String>,

    #[serde(rename = "additionalSources")]
    pub additional_sources: Option<Vec<SourceReference>>,

    #[serde(default)]
    pub entries: Vec<Entry>,

    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,

    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReference {
    pub source: String,
    pub page: Option<i32>,
}

// Summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeitySummary {
    pub name: String,
    pub source: String,
    pub title: String,
    pub pantheon: String,
    pub alignment: String,
    pub domains: Vec<String>,
    pub symbol: String,
}

impl From<&Deity> for DeitySummary {
    fn from(deity: &Deity) -> Self {
        Self {
            name: deity.name.clone(),
            source: deity.source.clone(),
            title: deity.title.clone().unwrap_or_default(),
            pantheon: deity.pantheon.clone().unwrap_or_default(),
            alignment: format_alignment(&deity.alignment),
            domains: deity.domains.clone().unwrap_or_default(),
            symbol: deity.symbol.clone().unwrap_or_default(),
        }
    }
}

fn format_alignment(alignment: &Option<Vec<String>>) -> String {
    if let Some(align) = alignment {
        align
            .iter()
            .map(|a| match a.as_str() {
                "L" => "Lawful",
                "N" => "Neutral",
                "C" => "Chaotic",
                "G" => "Good",
                "E" => "Evil",
                "U" => "Unaligned",
                "A" => "Any",
                _ => a,
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        String::new()
    }
}

// Container for JSON parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct DeityData {
    pub deity: Option<Vec<Deity>>,
}

// Database models
#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = catalog_deities)]
pub struct CatalogDeity {
    pub id: i32,
    pub name: String,
    pub title: Option<String>,
    pub pantheon: Option<String>,
    pub alignment: Option<String>,
    pub domains: Option<String>, // JSON array as comma-separated string
    pub symbol: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_deity_json: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = catalog_deities)]
pub struct NewCatalogDeity {
    pub name: String,
    pub title: Option<String>,
    pub pantheon: Option<String>,
    pub alignment: Option<String>,
    pub domains: Option<String>, // JSON array as comma-separated string
    pub symbol: Option<String>,
    pub source: String,
    pub page: Option<i32>,
    pub full_deity_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeityFilters {
    pub name: Option<String>,
    pub sources: Option<Vec<String>>,
    pub pantheons: Option<Vec<String>>,
    pub domains: Option<Vec<String>>,
    pub alignments: Option<Vec<String>>,
}

impl From<&CatalogDeity> for DeitySummary {
    fn from(catalog: &CatalogDeity) -> Self {
        let domains = catalog
            .domains
            .as_ref()
            .map(|d| d.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        Self {
            name: catalog.name.clone(),
            source: catalog.source.clone(),
            title: catalog.title.clone().unwrap_or_default(),
            pantheon: catalog.pantheon.clone().unwrap_or_default(),
            alignment: catalog.alignment.clone().unwrap_or_default(),
            domains,
            symbol: catalog.symbol.clone().unwrap_or_default(),
        }
    }
}

impl From<&Deity> for NewCatalogDeity {
    fn from(deity: &Deity) -> Self {
        // Convert domains Vec<String> to comma-separated string for database storage
        let domains_str = deity
            .domains
            .as_ref()
            .map(|d| d.join(", "))
            .filter(|s| !s.is_empty());

        Self {
            name: deity.name.clone(),
            title: deity.title.clone(),
            pantheon: deity.pantheon.clone(),
            alignment: Some(format_alignment(&deity.alignment)).filter(|s| !s.is_empty()),
            domains: domains_str,
            symbol: deity.symbol.clone(),
            source: deity.source.clone(),
            page: deity.page,
            full_deity_json: serde_json::to_string(deity).unwrap_or_default(),
        }
    }
}
