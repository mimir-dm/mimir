//! Action Model
//!
//! Represents a combat action (Dash, Dodge, Help, Hide, etc.).

use crate::schema::actions;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A combat action from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = actions)]
#[diesel(primary_key(id))]
pub struct Action {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub data: String,
}

impl Action {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = actions)]
pub struct NewAction<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
}

impl<'a> NewAction<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data }
    }
}

/// Filters for searching actions.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ActionFilter {
    pub name_contains: Option<String>,
    pub source: Option<String>,
}

impl ActionFilter {
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
    fn test_new_action() {
        let action = NewAction::new("Dash", "PHB", r#"{"name":"Dash"}"#);
        assert_eq!(action.name, "Dash");
    }
}
