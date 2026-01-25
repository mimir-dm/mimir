//! Trap Model
//!
//! Represents a trap.

use crate::schema::traps;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A trap from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = traps)]
#[diesel(primary_key(id))]
pub struct Trap {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub trap_tier: Option<String>,
    pub data: String,
    pub fluff: Option<String>,
}

impl Trap {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Check if this is a simple trap.
    pub fn is_simple(&self) -> bool {
        self.trap_tier.as_ref().map_or(false, |t| t == "simple")
    }

    /// Check if this is a complex trap.
    pub fn is_complex(&self) -> bool {
        self.trap_tier.as_ref().map_or(false, |t| t == "complex")
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = traps)]
pub struct NewTrap<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub trap_tier: Option<&'a str>,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewTrap<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, trap_tier: None, data, fluff: None }
    }

    pub fn with_tier(mut self, tier: &'a str) -> Self {
        self.trap_tier = Some(tier);
        self
    }
}

/// Filters for searching traps.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrapFilter {
    pub name_contains: Option<String>,
    /// Single source filter (legacy).
    pub source: Option<String>,
    /// Multiple sources filter (preferred).
    pub sources: Option<Vec<String>>,
    pub tier: Option<String>,
}

impl TrapFilter {
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

    pub fn with_tier(mut self, tier: impl Into<String>) -> Self {
        self.tier = Some(tier.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_trap() {
        let trap = NewTrap::new("Pit Trap", "DMG", r#"{"name":"Pit Trap"}"#)
            .with_tier("simple");
        assert_eq!(trap.name, "Pit Trap");
        assert_eq!(trap.trap_tier, Some("simple"));
    }
}
