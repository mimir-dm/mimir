//! Monster Model
//!
//! Represents a monster in the catalog with indexed fields for filtering
//! and a full JSON data blob.

use crate::schema::monsters;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A monster from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = monsters)]
#[diesel(primary_key(id))]
pub struct Monster {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Monster name.
    pub name: String,
    /// Source book code (e.g., "MM", "PHB").
    pub source: String,
    /// Challenge rating as string (e.g., "1/4", "1", "10").
    pub cr: Option<String>,
    /// Creature type (e.g., "humanoid", "dragon", "undead").
    pub creature_type: Option<String>,
    /// Size (e.g., "M", "L", "H").
    pub size: Option<String>,
    /// Path to token image file.
    pub token_image_path: Option<String>,
    /// Full 5etools JSON data.
    pub data: String,
    /// Lore/flavor text and image paths from fluff files.
    pub fluff: Option<String>,
}

impl Monster {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Get the numeric CR value if it can be parsed.
    pub fn cr_numeric(&self) -> Option<f32> {
        self.cr.as_ref().and_then(|cr| parse_cr(cr))
    }

    /// Get a display-friendly size name.
    pub fn size_name(&self) -> &str {
        self.size.as_ref().map_or("Unknown", |s| match s.as_str() {
            "T" => "Tiny",
            "S" => "Small",
            "M" => "Medium",
            "L" => "Large",
            "H" => "Huge",
            "G" => "Gargantuan",
            _ => "Unknown",
        })
    }
}

/// Parse a CR string to a numeric value.
fn parse_cr(cr: &str) -> Option<f32> {
    match cr {
        "0" => Some(0.0),
        "1/8" => Some(0.125),
        "1/4" => Some(0.25),
        "1/2" => Some(0.5),
        _ => cr.parse::<f32>().ok(),
    }
}

/// Data for inserting a new monster.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = monsters)]
pub struct NewMonster<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub cr: Option<&'a str>,
    pub creature_type: Option<&'a str>,
    pub size: Option<&'a str>,
    pub token_image_path: Option<&'a str>,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewMonster<'a> {
    /// Create a new monster entry.
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self {
            name,
            source,
            cr: None,
            creature_type: None,
            size: None,
            token_image_path: None,
            data,
            fluff: None,
        }
    }

    /// Set the challenge rating.
    pub fn with_cr(mut self, cr: &'a str) -> Self {
        self.cr = Some(cr);
        self
    }

    /// Set the creature type.
    pub fn with_creature_type(mut self, creature_type: &'a str) -> Self {
        self.creature_type = Some(creature_type);
        self
    }

    /// Set the size.
    pub fn with_size(mut self, size: &'a str) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the token image path.
    pub fn with_token_image_path(mut self, path: &'a str) -> Self {
        self.token_image_path = Some(path);
        self
    }

    /// Extract indexed fields from a JSON value.
    ///
    /// This parses the 5etools monster JSON and extracts:
    /// - `cr` from `data.cr` (handles both string and object formats)
    /// - `creature_type` from `data.type.type` or `data.type` (string)
    /// - `size` from `data.size[0]`
    pub fn from_json(name: &'a str, source: &'a str, data: &'a str, json: &serde_json::Value) -> Self {
        let mut monster = Self::new(name, source, data);

        // Extract CR (can be string, number, or object with "cr" field)
        if let Some(cr_val) = json.get("cr") {
            if let Some(cr_str) = cr_val.as_str() {
                monster.cr = Some(unsafe { std::mem::transmute::<&str, &'a str>(cr_str) });
            } else if let Some(cr_num) = cr_val.as_f64() {
                // We can't return a reference to a temporary, so skip numeric CRs
                // They should be handled by the caller
                let _ = cr_num;
            } else if let Some(cr_obj) = cr_val.as_object() {
                if let Some(cr_str) = cr_obj.get("cr").and_then(|v| v.as_str()) {
                    monster.cr = Some(unsafe { std::mem::transmute::<&str, &'a str>(cr_str) });
                }
            }
        }

        // Extract creature type
        if let Some(type_val) = json.get("type") {
            if let Some(type_str) = type_val.as_str() {
                monster.creature_type = Some(unsafe { std::mem::transmute::<&str, &'a str>(type_str) });
            } else if let Some(type_obj) = type_val.as_object() {
                if let Some(type_str) = type_obj.get("type").and_then(|v| v.as_str()) {
                    monster.creature_type = Some(unsafe { std::mem::transmute::<&str, &'a str>(type_str) });
                }
            }
        }

        // Extract size (first element of array)
        if let Some(size_arr) = json.get("size").and_then(|v| v.as_array()) {
            if let Some(size_str) = size_arr.first().and_then(|v| v.as_str()) {
                monster.size = Some(unsafe { std::mem::transmute::<&str, &'a str>(size_str) });
            }
        }

        monster
    }
}

/// Filters for searching monsters.
#[derive(Debug, Default, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonsterFilter {
    pub name_contains: Option<String>,
    /// Single source filter (legacy).
    pub source: Option<String>,
    /// Multiple sources filter (preferred).
    pub sources: Option<Vec<String>>,
    pub cr: Option<String>,
    pub creature_type: Option<String>,
    pub size: Option<String>,
}

impl MonsterFilter {
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

    pub fn with_sources(mut self, sources: Vec<String>) -> Self {
        self.sources = Some(sources);
        self
    }

    /// Returns true if sources filter is explicitly set to an empty array.
    pub fn has_empty_sources_filter(&self) -> bool {
        matches!(&self.sources, Some(sources) if sources.is_empty())
    }

    /// Get effective sources list (combines single source and sources array).
    pub fn effective_sources(&self) -> Option<Vec<String>> {
        match (&self.sources, &self.source) {
            (Some(sources), _) if !sources.is_empty() => Some(sources.clone()),
            (_, Some(source)) => Some(vec![source.clone()]),
            _ => None,
        }
    }

    pub fn with_cr(mut self, cr: impl Into<String>) -> Self {
        self.cr = Some(cr.into());
        self
    }

    pub fn with_creature_type(mut self, creature_type: impl Into<String>) -> Self {
        self.creature_type = Some(creature_type.into());
        self
    }

    pub fn with_size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_monster() {
        let data = r#"{"name":"Goblin","source":"MM"}"#;
        let monster = NewMonster::new("Goblin", "MM", data);
        assert_eq!(monster.name, "Goblin");
        assert_eq!(monster.source, "MM");
        assert!(monster.cr.is_none());
    }

    #[test]
    fn test_new_monster_with_fields() {
        let data = r#"{"name":"Goblin"}"#;
        let monster = NewMonster::new("Goblin", "MM", data)
            .with_cr("1/4")
            .with_creature_type("humanoid")
            .with_size("S");

        assert_eq!(monster.cr, Some("1/4"));
        assert_eq!(monster.creature_type, Some("humanoid"));
        assert_eq!(monster.size, Some("S"));
    }

    #[test]
    fn test_parse_cr() {
        assert_eq!(parse_cr("0"), Some(0.0));
        assert_eq!(parse_cr("1/8"), Some(0.125));
        assert_eq!(parse_cr("1/4"), Some(0.25));
        assert_eq!(parse_cr("1/2"), Some(0.5));
        assert_eq!(parse_cr("1"), Some(1.0));
        assert_eq!(parse_cr("10"), Some(10.0));
        assert_eq!(parse_cr("30"), Some(30.0));
    }

    #[test]
    fn test_monster_filter() {
        let filter = MonsterFilter::new()
            .with_name_contains("dragon")
            .with_creature_type("dragon")
            .with_cr("10");

        assert_eq!(filter.name_contains, Some("dragon".to_string()));
        assert_eq!(filter.creature_type, Some("dragon".to_string()));
        assert_eq!(filter.cr, Some("10".to_string()));
    }

    #[test]
    fn test_from_json_simple_type() {
        let json = json!({
            "name": "Goblin",
            "source": "MM",
            "type": "humanoid",
            "size": ["S"],
            "cr": "1/4"
        });
        let data = json.to_string();
        let monster = NewMonster::from_json("Goblin", "MM", &data, &json);

        assert_eq!(monster.creature_type, Some("humanoid"));
        assert_eq!(monster.size, Some("S"));
        assert_eq!(monster.cr, Some("1/4"));
    }

    #[test]
    fn test_from_json_complex_type() {
        let json = json!({
            "name": "Orc",
            "source": "MM",
            "type": {
                "type": "humanoid",
                "tags": ["orc"]
            },
            "size": ["M"],
            "cr": "1/2"
        });
        let data = json.to_string();
        let monster = NewMonster::from_json("Orc", "MM", &data, &json);

        assert_eq!(monster.creature_type, Some("humanoid"));
        assert_eq!(monster.size, Some("M"));
    }
}
