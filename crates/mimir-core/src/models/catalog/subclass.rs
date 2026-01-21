//! Subclass Model
//!
//! Represents a character subclass in the catalog (School of Evocation, Champion, etc.).

use crate::schema::subclasses;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A character subclass from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = subclasses)]
#[diesel(primary_key(id))]
pub struct Subclass {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Subclass name (e.g., "School of Evocation", "Champion").
    pub name: String,
    /// Parent class name (e.g., "Wizard", "Fighter").
    pub class_name: String,
    /// Source book code (e.g., "PHB", "XGE").
    pub source: String,
    /// Full 5etools JSON data.
    pub data: String,
    /// Lore/flavor text and image paths from fluff files.
    pub fluff: Option<String>,
}

impl Subclass {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

/// Data for inserting a new subclass.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = subclasses)]
pub struct NewSubclass<'a> {
    pub name: &'a str,
    pub class_name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewSubclass<'a> {
    /// Create a new subclass entry.
    pub fn new(name: &'a str, class_name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self {
            name,
            class_name,
            source,
            data,
            fluff: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_subclass() {
        let data = r#"{"name":"School of Evocation","className":"Wizard"}"#;
        let subclass = NewSubclass::new("School of Evocation", "Wizard", "PHB", data);
        assert_eq!(subclass.name, "School of Evocation");
        assert_eq!(subclass.class_name, "Wizard");
        assert_eq!(subclass.source, "PHB");
    }
}
