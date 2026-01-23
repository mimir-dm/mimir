//! Object Model
//!
//! Represents an interactive object (doors, chests, siege weapons, etc.).

use crate::schema::objects;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// An object from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = objects)]
#[diesel(primary_key(id))]
pub struct Object {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub object_type: Option<String>,
    pub data: String,
    pub fluff: Option<String>,
}

impl Object {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = objects)]
pub struct NewObject<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub object_type: Option<&'a str>,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewObject<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, object_type: None, data, fluff: None }
    }

    pub fn with_type(mut self, object_type: &'a str) -> Self {
        self.object_type = Some(object_type);
        self
    }
}

/// Filters for searching objects.
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct ObjectFilter {
    pub name_contains: Option<String>,
    pub source: Option<String>,
    pub sources: Option<Vec<String>>,
    pub object_type: Option<String>,
}

impl ObjectFilter {
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
    fn test_new_object() {
        let object = NewObject::new("Ballista", "DMG", r#"{"name":"Ballista"}"#)
            .with_type("SW");
        assert_eq!(object.name, "Ballista");
        assert_eq!(object.object_type, Some("SW"));
    }
}
