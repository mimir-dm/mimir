//! Class Model
//!
//! Represents a character class in the catalog (Wizard, Fighter, etc.).

use crate::schema::classes;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A character class from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = classes)]
#[diesel(primary_key(id))]
pub struct Class {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Class name (e.g., "Wizard", "Fighter").
    pub name: String,
    /// Source book code (e.g., "PHB", "XGE").
    pub source: String,
    /// Full 5etools JSON data.
    pub data: String,
    /// Lore/flavor text and image paths from fluff files.
    pub fluff: Option<String>,
}

impl Class {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

/// Data for inserting a new class.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = classes)]
pub struct NewClass<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewClass<'a> {
    /// Create a new class entry.
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data, fluff: None }
    }
}

/// Filters for searching classes.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ClassFilter {
    pub name_contains: Option<String>,
    /// Single source filter (legacy).
    pub source: Option<String>,
    /// Multiple sources filter (preferred).
    pub sources: Option<Vec<String>>,
}

impl ClassFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name_contains(mut self, name: impl Into<String>) -> Self {
        self.name_contains = Some(name.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_sources(mut self, sources: Vec<String>) -> Self {
        self.sources = Some(sources);
        self
    }

    /// Returns true if sources filter is explicitly set to an empty array.
    pub fn has_empty_sources_filter(&self) -> bool {
        matches!(&self.sources, Some(sources) if sources.is_empty())
    }

    /// Get effective sources list (combines single source and sources array).
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
    fn test_new_class() {
        let data = r#"{"name":"Wizard","source":"PHB"}"#;
        let class = NewClass::new("Wizard", "PHB", data);
        assert_eq!(class.name, "Wizard");
        assert_eq!(class.source, "PHB");
    }
}
