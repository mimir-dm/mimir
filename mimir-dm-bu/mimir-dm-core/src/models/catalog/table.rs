use super::types::{Entry, Image, TableCell};
use crate::schema::catalog_tables;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub source: String,
    pub page: Option<i32>,

    #[serde(rename = "caption")]
    pub caption: Option<String>,

    #[serde(rename = "colLabels")]
    pub col_labels: Option<Vec<String>>,

    #[serde(rename = "colStyles")]
    pub col_styles: Option<Vec<String>>,

    #[serde(default)]
    pub rows: Vec<Vec<TableCell>>,

    // Optional fields
    #[serde(default)]
    pub intro: Vec<Entry>,
    #[serde(default)]
    pub outro: Vec<Entry>,

    #[serde(rename = "tableInclude")]
    pub table_include: Option<serde_json::Value>,

    #[serde(rename = "footnotes", default)]
    pub footnotes: Vec<Entry>,

    pub srd: Option<bool>,

    #[serde(rename = "basicRules")]
    pub basic_rules: Option<bool>,

    #[serde(rename = "hasFluff")]
    pub has_fluff: Option<bool>,

    #[serde(rename = "hasFluffImages")]
    pub has_fluff_images: Option<bool>,
}

// Summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSummary {
    pub name: String,
    pub source: String,
    pub caption: String,
    pub columns: usize,
    pub rows: usize,
    pub category: String,
}

impl From<&Table> for TableSummary {
    fn from(table: &Table) -> Self {
        Self {
            name: table.name.clone(),
            source: table.source.clone(),
            caption: table.caption.clone().unwrap_or_else(|| table.name.clone()),
            columns: table
                .col_labels
                .as_ref()
                .map(|c| c.len())
                .unwrap_or(table.rows.first().map(|r| r.len()).unwrap_or(0)),
            rows: table.rows.len(),
            category: categorize_table(&table.name),
        }
    }
}

fn categorize_table(name: &str) -> String {
    let name_lower = name.to_lowercase();

    if name_lower.contains("madness") || name_lower.contains("insanity") {
        "Madness".to_string()
    } else if name_lower.contains("treasure")
        || name_lower.contains("loot")
        || name_lower.contains("hoard")
    {
        "Treasure".to_string()
    } else if name_lower.contains("encounter") || name_lower.contains("random") {
        "Encounters".to_string()
    } else if name_lower.contains("trinket") {
        "Trinkets".to_string()
    } else if name_lower.contains("wild magic") || name_lower.contains("surge") {
        "Wild Magic".to_string()
    } else if name_lower.contains("damage") || name_lower.contains("critical") {
        "Combat".to_string()
    } else if name_lower.contains("npc")
        || name_lower.contains("name")
        || name_lower.contains("personality")
    {
        "NPCs".to_string()
    } else if name_lower.contains("quest")
        || name_lower.contains("adventure")
        || name_lower.contains("plot")
    {
        "Adventures".to_string()
    } else if name_lower.contains("magic item") || name_lower.contains("artifact") {
        "Magic Items".to_string()
    } else {
        "Miscellaneous".to_string()
    }
}

// Container for JSON parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct TableData {
    pub table: Option<Vec<Table>>,
}

// Fluff data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableFluff {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub images: Vec<Image>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableFluffData {
    #[serde(rename = "tableFluff")]
    pub table_fluff: Option<Vec<TableFluff>>,
}

// Database models for catalog_tables table
#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = catalog_tables)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CatalogTable {
    pub id: i32,
    pub name: String,
    pub caption: Option<String>,
    pub category: String,
    pub source: String,
    pub page: Option<i32>,
    pub columns_count: i32,
    pub rows_count: i32,
    pub full_table_json: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = catalog_tables)]
pub struct NewCatalogTable {
    pub name: String,
    pub caption: Option<String>,
    pub category: String,
    pub source: String,
    pub page: Option<i32>,
    pub columns_count: i32,
    pub rows_count: i32,
    pub full_table_json: String,
}

#[derive(Debug, Default)]
pub struct TableFilters {
    pub name: Option<String>,
    pub categories: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
}

impl From<&Table> for NewCatalogTable {
    fn from(table: &Table) -> Self {
        let columns = table
            .col_labels
            .as_ref()
            .map(|c| c.len())
            .unwrap_or(table.rows.first().map(|r| r.len()).unwrap_or(0));
        NewCatalogTable {
            name: table.name.clone(),
            caption: table.caption.clone(),
            category: categorize_table(&table.name),
            source: table.source.clone(),
            page: table.page,
            columns_count: columns as i32,
            rows_count: table.rows.len() as i32,
            full_table_json: serde_json::to_string(table).unwrap_or_default(),
        }
    }
}

impl From<&CatalogTable> for TableSummary {
    fn from(catalog_table: &CatalogTable) -> Self {
        TableSummary {
            name: catalog_table.name.clone(),
            source: catalog_table.source.clone(),
            caption: catalog_table
                .caption
                .clone()
                .unwrap_or_else(|| catalog_table.name.clone()),
            columns: catalog_table.columns_count as usize,
            rows: catalog_table.rows_count as usize,
            category: catalog_table.category.clone(),
        }
    }
}
