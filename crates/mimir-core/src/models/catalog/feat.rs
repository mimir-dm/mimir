//! Feat Model
//!
//! Represents a character feat in the catalog (Alert, Sharpshooter, etc.).

use crate::schema::feats;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A character feat from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = feats)]
#[diesel(primary_key(id))]
pub struct Feat {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Feat name (e.g., "Alert", "Sharpshooter").
    pub name: String,
    /// Source book code (e.g., "PHB", "XGE").
    pub source: String,
    /// Full 5etools JSON data.
    pub data: String,
    /// Lore/flavor text and image paths from fluff files.
    pub fluff: Option<String>,
}

impl Feat {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }
}

/// Data for inserting a new feat.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = feats)]
pub struct NewFeat<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewFeat<'a> {
    /// Create a new feat entry.
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self { name, source, data, fluff: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feat() {
        let data = r#"{"name":"Alert","source":"PHB"}"#;
        let feat = NewFeat::new("Alert", "PHB", data);
        assert_eq!(feat.name, "Alert");
        assert_eq!(feat.source, "PHB");
    }
}
