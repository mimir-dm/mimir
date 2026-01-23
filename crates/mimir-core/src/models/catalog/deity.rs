//! Deity Model
//!
//! Represents a deity (god) from various pantheons.

use crate::schema::deities;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A deity from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = deities)]
#[diesel(primary_key(id))]
pub struct Deity {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub pantheon: Option<String>,
    pub data: String,
}

impl Deity {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = deities)]
pub struct NewDeity<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub pantheon: Option<&'a str>,
    pub data: &'a str,
}

impl<'a> NewDeity<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, pantheon: None, data }
    }

    pub fn with_pantheon(mut self, pantheon: &'a str) -> Self {
        self.pantheon = Some(pantheon);
        self
    }
}

/// Filters for searching deities.
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct DeityFilter {
    pub name_contains: Option<String>,
    pub source: Option<String>,
    pub sources: Option<Vec<String>>,
    pub pantheon: Option<String>,
}

impl DeityFilter {
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
    fn test_new_deity() {
        let deity = NewDeity::new("Tyr", "PHB", r#"{"name":"Tyr"}"#)
            .with_pantheon("Forgotten Realms");
        assert_eq!(deity.name, "Tyr");
        assert_eq!(deity.pantheon, Some("Forgotten Realms"));
    }
}
