//! Subclass Feature Model
//!
//! Represents a subclass feature in the catalog (Channel Divinity, etc.).

use crate::schema::subclass_features;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A subclass feature from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = subclass_features)]
#[diesel(primary_key(id))]
pub struct SubclassFeature {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Feature name (e.g., "Channel Divinity: Turn Undead").
    pub name: String,
    /// Source book code (e.g., "PHB", "XGE").
    pub source: String,
    /// Parent class name (e.g., "Cleric").
    pub class_name: String,
    /// Parent class source (e.g., "PHB").
    pub class_source: String,
    /// Subclass name (e.g., "Life Domain").
    pub subclass_name: String,
    /// Subclass source (e.g., "PHB").
    pub subclass_source: String,
    /// Level at which this feature is gained.
    pub level: i32,
    /// Full 5etools JSON data with entries.
    pub data: String,
}

impl SubclassFeature {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

/// Data for inserting a new subclass feature.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = subclass_features)]
pub struct NewSubclassFeature<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub class_name: &'a str,
    pub class_source: &'a str,
    pub subclass_name: &'a str,
    pub subclass_source: &'a str,
    pub level: i32,
    pub data: &'a str,
}

impl<'a> NewSubclassFeature<'a> {
    /// Create a new subclass feature entry.
    pub fn new(
        name: &'a str,
        source: &'a str,
        class_name: &'a str,
        class_source: &'a str,
        subclass_name: &'a str,
        subclass_source: &'a str,
        level: i32,
        data: &'a str,
    ) -> Self {
        Self {
            name, source, class_name, class_source,
            subclass_name, subclass_source, level, data
        }
    }
}

/// Filters for searching subclass features.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct SubclassFeatureFilter {
    pub name_contains: Option<String>,
    pub class_name: Option<String>,
    pub class_source: Option<String>,
    pub subclass_name: Option<String>,
    pub subclass_source: Option<String>,
    pub level: Option<i32>,
    /// Single source filter (legacy).
    pub source: Option<String>,
    /// Multiple sources filter (preferred).
    pub sources: Option<Vec<String>>,
}

impl SubclassFeatureFilter {
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

    pub fn with_subclass(mut self, subclass_name: impl Into<String>, subclass_source: impl Into<String>) -> Self {
        self.subclass_name = Some(subclass_name.into());
        self.subclass_source = Some(subclass_source.into());
        self
    }

    pub fn with_level(mut self, level: i32) -> Self {
        self.level = Some(level);
        self
    }

    /// Returns true if sources filter is explicitly set to an empty array.
    pub fn has_empty_sources_filter(&self) -> bool {
        matches!(&self.sources, Some(sources) if sources.is_empty())
    }

    /// Get effective sources list.
    pub fn effective_sources(&self) -> Option<Vec<String>> {
        match (&self.sources, &self.source) {
            (Some(sources), _) if !sources.is_empty() => Some(sources.clone()),
            (_, Some(source)) => Some(vec![source.clone()]),
            _ => None,
        }
    }
}
