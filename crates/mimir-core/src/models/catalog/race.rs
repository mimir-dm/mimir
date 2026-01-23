//! Race Model
//!
//! Represents a character race/lineage in the catalog (Elf, Dwarf, etc.).

use crate::schema::races;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A character race from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = races)]
#[diesel(primary_key(id))]
pub struct Race {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Race name (e.g., "Elf", "Dwarf").
    pub name: String,
    /// Source book code (e.g., "PHB", "MPMM").
    pub source: String,
    /// Full 5etools JSON data.
    pub data: String,
    /// Lore/flavor text and image paths from fluff files.
    pub fluff: Option<String>,
}

impl Race {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

/// Data for inserting a new race.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = races)]
pub struct NewRace<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewRace<'a> {
    /// Create a new race entry.
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data, fluff: None }
    }
}

/// Filters for searching races.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct RaceFilter {
    pub name_contains: Option<String>,
    /// Single source filter (legacy).
    pub source: Option<String>,
    /// Multiple sources filter (preferred).
    pub sources: Option<Vec<String>>,
}

impl RaceFilter {
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
    fn test_new_race() {
        let data = r#"{"name":"Elf","source":"PHB"}"#;
        let race = NewRace::new("Elf", "PHB", data);
        assert_eq!(race.name, "Elf");
        assert_eq!(race.source, "PHB");
    }
}
