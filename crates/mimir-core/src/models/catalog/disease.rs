//! Disease Model
//!
//! Represents a disease in the catalog.

use crate::schema::diseases;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A disease from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = diseases)]
#[diesel(primary_key(id))]
pub struct Disease {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub data: String,
    pub fluff: Option<String>,
}

impl Disease {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = diseases)]
pub struct NewDisease<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewDisease<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data, fluff: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_disease() {
        let disease = NewDisease::new("Cackle Fever", "DMG", r#"{"name":"Cackle Fever"}"#);
        assert_eq!(disease.name, "Cackle Fever");
    }
}
