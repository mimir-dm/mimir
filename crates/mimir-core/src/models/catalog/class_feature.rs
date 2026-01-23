//! Class Feature Model
//!
//! Represents a class feature in the catalog (Spellcasting, Extra Attack, etc.).

use crate::schema::class_features;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A class feature from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = class_features)]
#[diesel(primary_key(id))]
pub struct ClassFeature {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Feature name (e.g., "Spellcasting", "Extra Attack").
    pub name: String,
    /// Source book code (e.g., "PHB", "TCE").
    pub source: String,
    /// Parent class name (e.g., "Fighter", "Wizard").
    pub class_name: String,
    /// Parent class source (e.g., "PHB").
    pub class_source: String,
    /// Level at which this feature is gained.
    pub level: i32,
    /// Full 5etools JSON data with entries.
    pub data: String,
}

impl ClassFeature {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

/// Data for inserting a new class feature.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = class_features)]
pub struct NewClassFeature<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub class_name: &'a str,
    pub class_source: &'a str,
    pub level: i32,
    pub data: &'a str,
}

impl<'a> NewClassFeature<'a> {
    /// Create a new class feature entry.
    pub fn new(
        name: &'a str,
        source: &'a str,
        class_name: &'a str,
        class_source: &'a str,
        level: i32,
        data: &'a str,
    ) -> Self {
        Self { name, source, class_name, class_source, level, data }
    }
}

/// Filters for searching class features.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ClassFeatureFilter {
    pub name_contains: Option<String>,
    pub class_name: Option<String>,
    pub class_source: Option<String>,
    pub level: Option<i32>,
    /// Single source filter (legacy).
    pub source: Option<String>,
    /// Multiple sources filter (preferred).
    pub sources: Option<Vec<String>>,
}

impl ClassFeatureFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name_contains(mut self, name: impl Into<String>) -> Self {
        self.name_contains = Some(name.into());
        self
    }

    pub fn with_class(mut self, class_name: impl Into<String>, class_source: impl Into<String>) -> Self {
        self.class_name = Some(class_name.into());
        self.class_source = Some(class_source.into());
        self
    }

    pub fn with_level(mut self, level: i32) -> Self {
        self.level = Some(level);
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
    fn test_new_class_feature() {
        let data = r#"{"name":"Spellcasting","entries":["You can cast spells..."]}"#;
        let feature = NewClassFeature::new("Spellcasting", "PHB", "Wizard", "PHB", 1, data);
        assert_eq!(feature.name, "Spellcasting");
        assert_eq!(feature.class_name, "Wizard");
        assert_eq!(feature.level, 1);
    }
}
