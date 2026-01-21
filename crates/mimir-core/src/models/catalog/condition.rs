//! Condition Model
//!
//! Represents a game condition (blinded, charmed, frightened, etc.).

use crate::schema::conditions;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A game condition from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = conditions)]
#[diesel(primary_key(id))]
pub struct Condition {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub data: String,
    pub fluff: Option<String>,
}

impl Condition {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = conditions)]
pub struct NewCondition<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewCondition<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data, fluff: None }
    }
}

/// Filters for searching conditions.
#[derive(Debug, Default, Clone)]
pub struct ConditionFilter {
    pub name_contains: Option<String>,
    pub source: Option<String>,
}

impl ConditionFilter {
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
    fn test_new_condition() {
        let condition = NewCondition::new("Blinded", "PHB", r#"{"name":"Blinded"}"#);
        assert_eq!(condition.name, "Blinded");
    }
}
