//! Cult Model
//!
//! Represents a cult or supernatural gift/boon.

use crate::schema::cults;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A cult or supernatural gift from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = cults)]
#[diesel(primary_key(id))]
pub struct Cult {
    pub id: Option<i32>,
    pub name: String,
    pub source: String,
    pub data: String,
}

impl Cult {
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = cults)]
pub struct NewCult<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
}

impl<'a> NewCult<'a> {
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cult() {
        let cult = NewCult::new("Cult of the Dragon", "MM", r#"{"name":"Cult of the Dragon"}"#);
        assert_eq!(cult.name, "Cult of the Dragon");
    }
}
