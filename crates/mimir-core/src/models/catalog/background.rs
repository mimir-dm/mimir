//! Background Model
//!
//! Represents a character background in the catalog (Acolyte, Criminal, etc.).

use crate::schema::backgrounds;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A character background from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = backgrounds)]
#[diesel(primary_key(id))]
pub struct Background {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Background name (e.g., "Acolyte", "Criminal").
    pub name: String,
    /// Source book code (e.g., "PHB", "BGG").
    pub source: String,
    /// Full 5etools JSON data.
    pub data: String,
    /// Lore/flavor text and image paths from fluff files.
    pub fluff: Option<String>,
}

impl Background {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

/// Data for inserting a new background.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = backgrounds)]
pub struct NewBackground<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewBackground<'a> {
    /// Create a new background entry.
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data, fluff: None }
    }
}

/// Filters for searching backgrounds.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct BackgroundFilter {
    pub name_contains: Option<String>,
    /// Single source filter (legacy).
    pub source: Option<String>,
    /// Multiple sources filter (preferred).
    pub sources: Option<Vec<String>>,
}

impl BackgroundFilter {
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
    fn test_new_background() {
        let data = r#"{"name":"Acolyte","source":"PHB"}"#;
        let background = NewBackground::new("Acolyte", "PHB", data);
        assert_eq!(background.name, "Acolyte");
        assert_eq!(background.source, "PHB");
    }
}
