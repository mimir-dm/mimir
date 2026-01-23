//! Language Model
//!
//! Represents a language (Common, Elvish, Dwarvish, etc.).

use crate::schema::languages;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A language from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = languages)]
#[diesel(primary_key(id))]
pub struct Language {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub language_type: Option<String>,
    pub data: String,
    pub fluff: Option<String>,
}

impl Language {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Get the human-readable type name.
    pub fn type_name(&self) -> &str {
        self.language_type.as_ref().map_or("Unknown", |t| match t.as_str() {
            "standard" => "Standard",
            "exotic" => "Exotic",
            "secret" => "Secret",
            _ => "Unknown",
        })
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = languages)]
pub struct NewLanguage<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub language_type: Option<&'a str>,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewLanguage<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, language_type: None, data, fluff: None }
    }

    pub fn with_type(mut self, language_type: &'a str) -> Self {
        self.language_type = Some(language_type);
        self
    }
}

/// Filters for searching languages.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct LanguageFilter {
    pub name_contains: Option<String>,
    /// Single source filter (legacy).
    pub source: Option<String>,
    /// Multiple sources filter (preferred).
    pub sources: Option<Vec<String>>,
    pub language_type: Option<String>,
}

impl LanguageFilter {
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

    pub fn with_type(mut self, language_type: impl Into<String>) -> Self {
        self.language_type = Some(language_type.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_language() {
        let language = NewLanguage::new("Elvish", "PHB", r#"{"name":"Elvish"}"#)
            .with_type("standard");
        assert_eq!(language.name, "Elvish");
        assert_eq!(language.language_type, Some("standard"));
    }
}
