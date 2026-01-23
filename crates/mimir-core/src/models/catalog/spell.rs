//! Spell Model
//!
//! Represents a spell in the catalog.

use crate::schema::spells;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// A spell from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = spells)]
#[diesel(primary_key(id))]
pub struct Spell {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Spell name.
    pub name: String,
    /// Source book code (e.g., "PHB", "XGE").
    pub source: String,
    /// Spell level (0 = cantrip, 1-9 = spell level).
    pub level: i32,
    /// School code (A, C, D, E, V, I, N, T).
    pub school: Option<String>,
    /// Whether this is a ritual spell.
    pub ritual: i32,
    /// Whether this spell requires concentration.
    pub concentration: i32,
    /// Full 5etools JSON data.
    pub data: String,
    /// Lore/flavor text and image paths from fluff files.
    pub fluff: Option<String>,
}

impl Spell {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Check if this is a cantrip.
    pub fn is_cantrip(&self) -> bool {
        self.level == 0
    }

    /// Check if this is a ritual spell.
    pub fn is_ritual(&self) -> bool {
        self.ritual != 0
    }

    /// Check if this spell requires concentration.
    pub fn requires_concentration(&self) -> bool {
        self.concentration != 0
    }

    /// Get the human-readable school name.
    pub fn school_name(&self) -> &str {
        self.school.as_ref().map_or("Unknown", |s| match s.as_str() {
            "A" => "Abjuration",
            "C" => "Conjuration",
            "D" => "Divination",
            "E" => "Enchantment",
            "V" => "Evocation",
            "I" => "Illusion",
            "N" => "Necromancy",
            "T" => "Transmutation",
            _ => "Unknown",
        })
    }

    /// Get the level as a display string (e.g., "Cantrip", "1st", "2nd").
    pub fn level_display(&self) -> String {
        match self.level {
            0 => "Cantrip".to_string(),
            1 => "1st".to_string(),
            2 => "2nd".to_string(),
            3 => "3rd".to_string(),
            n => format!("{}th", n),
        }
    }
}

/// Data for inserting a new spell.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = spells)]
pub struct NewSpell<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub level: i32,
    pub school: Option<&'a str>,
    pub ritual: i32,
    pub concentration: i32,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewSpell<'a> {
    /// Create a new spell entry.
    pub fn new(name: &'a str, source: &'a str, level: i32, data: &'a str) -> Self {
        Self {
            name,
            source,
            level,
            school: None,
            ritual: 0,
            concentration: 0,
            data,
            fluff: None,
        }
    }

    /// Set the school.
    pub fn with_school(mut self, school: &'a str) -> Self {
        self.school = Some(school);
        self
    }

    /// Mark as a ritual spell.
    pub fn with_ritual(mut self, ritual: bool) -> Self {
        self.ritual = if ritual { 1 } else { 0 };
        self
    }

    /// Mark as requiring concentration.
    pub fn with_concentration(mut self, concentration: bool) -> Self {
        self.concentration = if concentration { 1 } else { 0 };
        self
    }
}

/// Filters for searching spells.
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct SpellFilter {
    pub name_contains: Option<String>,
    /// Single source filter (legacy).
    pub source: Option<String>,
    /// Multiple sources filter (preferred).
    pub sources: Option<Vec<String>>,
    pub level: Option<i32>,
    pub school: Option<String>,
    pub ritual: Option<bool>,
    pub concentration: Option<bool>,
}

impl SpellFilter {
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

    pub fn with_level(mut self, level: i32) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_school(mut self, school: impl Into<String>) -> Self {
        self.school = Some(school.into());
        self
    }

    pub fn with_ritual(mut self, ritual: bool) -> Self {
        self.ritual = Some(ritual);
        self
    }

    pub fn with_concentration(mut self, concentration: bool) -> Self {
        self.concentration = Some(concentration);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_spell() {
        let data = r#"{"name":"Fireball"}"#;
        let spell = NewSpell::new("Fireball", "PHB", 3, data);
        assert_eq!(spell.name, "Fireball");
        assert_eq!(spell.source, "PHB");
        assert_eq!(spell.level, 3);
        assert!(spell.school.is_none());
        assert_eq!(spell.ritual, 0);
        assert_eq!(spell.concentration, 0);
    }

    #[test]
    fn test_new_spell_with_fields() {
        let data = r#"{"name":"Fireball"}"#;
        let spell = NewSpell::new("Fireball", "PHB", 3, data)
            .with_school("V")
            .with_ritual(false)
            .with_concentration(false);

        assert_eq!(spell.school, Some("V"));
        assert_eq!(spell.ritual, 0);
        assert_eq!(spell.concentration, 0);
    }

    #[test]
    fn test_new_spell_ritual_concentration() {
        let data = r#"{"name":"Detect Magic"}"#;
        let spell = NewSpell::new("Detect Magic", "PHB", 1, data)
            .with_school("D")
            .with_ritual(true)
            .with_concentration(true);

        assert_eq!(spell.ritual, 1);
        assert_eq!(spell.concentration, 1);
    }

    #[test]
    fn test_spell_filter() {
        let filter = SpellFilter::new()
            .with_name_contains("fire")
            .with_level(3)
            .with_school("V")
            .with_concentration(false);

        assert_eq!(filter.name_contains, Some("fire".to_string()));
        assert_eq!(filter.level, Some(3));
        assert_eq!(filter.school, Some("V".to_string()));
        assert_eq!(filter.concentration, Some(false));
    }

    #[test]
    fn test_school_name() {
        // Test that school_name works with Option<String>
        let spell = Spell {
            id: Some(1),
            name: "Fireball".to_string(),
            source: "PHB".to_string(),
            level: 3,
            school: Some("V".to_string()),
            ritual: 0,
            concentration: 0,
            data: "{}".to_string(),
            fluff: None,
        };
        assert_eq!(spell.school_name(), "Evocation");
    }

    #[test]
    fn test_level_display() {
        let make_spell = |level: i32| Spell {
            id: Some(1),
            name: "Test".to_string(),
            source: "PHB".to_string(),
            level,
            school: None,
            ritual: 0,
            concentration: 0,
            data: "{}".to_string(),
            fluff: None,
        };

        assert_eq!(make_spell(0).level_display(), "Cantrip");
        assert_eq!(make_spell(1).level_display(), "1st");
        assert_eq!(make_spell(2).level_display(), "2nd");
        assert_eq!(make_spell(3).level_display(), "3rd");
        assert_eq!(make_spell(4).level_display(), "4th");
        assert_eq!(make_spell(9).level_display(), "9th");
    }

    #[test]
    fn test_is_cantrip() {
        let cantrip = Spell {
            id: Some(1),
            name: "Fire Bolt".to_string(),
            source: "PHB".to_string(),
            level: 0,
            school: Some("V".to_string()),
            ritual: 0,
            concentration: 0,
            data: "{}".to_string(),
            fluff: None,
        };
        assert!(cantrip.is_cantrip());

        let spell = Spell {
            id: Some(2),
            name: "Fireball".to_string(),
            source: "PHB".to_string(),
            level: 3,
            school: Some("V".to_string()),
            ritual: 0,
            concentration: 0,
            data: "{}".to_string(),
            fluff: None,
        };
        assert!(!spell.is_cantrip());
    }

    #[test]
    fn test_is_ritual() {
        let ritual = Spell {
            id: Some(1),
            name: "Detect Magic".to_string(),
            source: "PHB".to_string(),
            level: 1,
            school: Some("D".to_string()),
            ritual: 1,
            concentration: 1,
            data: "{}".to_string(),
            fluff: None,
        };
        assert!(ritual.is_ritual());
        assert!(ritual.requires_concentration());

        let non_ritual = Spell {
            id: Some(2),
            name: "Magic Missile".to_string(),
            source: "PHB".to_string(),
            level: 1,
            school: Some("V".to_string()),
            ritual: 0,
            concentration: 0,
            data: "{}".to_string(),
            fluff: None,
        };
        assert!(!non_ritual.is_ritual());
        assert!(!non_ritual.requires_concentration());
    }
}
