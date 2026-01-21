//! Hazard Model
//!
//! Represents an environmental hazard.

use crate::schema::hazards;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A hazard from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = hazards)]
#[diesel(primary_key(id))]
pub struct Hazard {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub data: String,
    pub fluff: Option<String>,
}

impl Hazard {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = hazards)]
pub struct NewHazard<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewHazard<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data, fluff: None }
    }
}

/// Filters for searching hazards.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct HazardFilter {
    pub name_contains: Option<String>,
    pub source: Option<String>,
}

impl HazardFilter {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hazard() {
        let hazard = NewHazard::new("Brown Mold", "DMG", r#"{"name":"Brown Mold"}"#);
        assert_eq!(hazard.name, "Brown Mold");
    }
}
