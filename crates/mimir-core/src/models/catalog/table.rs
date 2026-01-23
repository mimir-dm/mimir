//! CatalogTable Model
//!
//! Represents random tables, encounter tables, and other game tables.

use crate::schema::catalog_tables;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A table (random table, encounter table, etc.) from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = catalog_tables)]
#[diesel(primary_key(id))]
pub struct CatalogTable {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub data: String,
}

impl CatalogTable {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = catalog_tables)]
pub struct NewCatalogTable<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
}

impl<'a> NewCatalogTable<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data }
    }
}

/// Filters for searching tables.
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct CatalogTableFilter {
    pub name_contains: Option<String>,
    pub source: Option<String>,
    pub sources: Option<Vec<String>>,
}

impl CatalogTableFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_empty_sources_filter(&self) -> bool {
        matches!(&self.sources, Some(sources) if sources.is_empty())
    }

    pub fn effective_sources(&self) -> Option<Vec<String>> {
        match (&self.sources, &self.source) {
            (Some(sources), _) if !sources.is_empty() => Some(sources.clone()),
            (_, Some(source)) => Some(vec![source.clone()]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_catalog_table() {
        let table = NewCatalogTable::new("Wild Magic Surge", "PHB", r#"{"name":"Wild Magic Surge"}"#);
        assert_eq!(table.name, "Wild Magic Surge");
        assert_eq!(table.source, "PHB");
    }
}
