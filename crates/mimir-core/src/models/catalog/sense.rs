//! Sense Model
//!
//! Represents a sense (darkvision, tremorsense, blindsight, etc.).

use crate::schema::senses;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A sense from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = senses)]
#[diesel(primary_key(id))]
pub struct Sense {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub data: String,
}

impl Sense {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = senses)]
pub struct NewSense<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
}

impl<'a> NewSense<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sense() {
        let sense = NewSense::new("Darkvision", "PHB", r#"{"name":"Darkvision"}"#);
        assert_eq!(sense.name, "Darkvision");
    }
}
