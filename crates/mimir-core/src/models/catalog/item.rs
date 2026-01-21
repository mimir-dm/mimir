//! Item Model
//!
//! Represents an item in the catalog (weapons, armor, magic items, equipment).

use crate::schema::items;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// An item from the catalog.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = items)]
#[diesel(primary_key(id))]
pub struct Item {
    /// Unique database ID.
    pub id: Option<i32>,
    /// Item name.
    pub name: String,
    /// Source book code (e.g., "PHB", "DMG").
    pub source: String,
    /// Item type code (e.g., "R" for rod, "A" for armor, "M" for melee weapon).
    pub item_type: Option<String>,
    /// Rarity (common, uncommon, rare, very rare, legendary, artifact).
    pub rarity: Option<String>,
    /// Full 5etools JSON data.
    pub data: String,
    /// Lore/flavor text and image paths from fluff files.
    pub fluff: Option<String>,
}

impl Item {
    /// Parse the JSON data blob into a serde_json::Value.
    pub fn parse_data(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    /// Get the human-readable type name.
    pub fn type_name(&self) -> &str {
        self.item_type.as_ref().map_or("Unknown", |t| match t.as_str() {
            "A" => "Armor",
            "AF" => "Ammunition",
            "AIR" => "Vehicle (Air)",
            "AT" => "Artisan's Tools",
            "EM" => "Eldritch Machine",
            "EXP" => "Explosive",
            "FD" => "Food and Drink",
            "G" => "Adventuring Gear",
            "GS" => "Gaming Set",
            "GV" => "Generic Variant",
            "HA" => "Heavy Armor",
            "INS" => "Instrument",
            "LA" => "Light Armor",
            "M" => "Melee Weapon",
            "MA" => "Medium Armor",
            "MNT" => "Mount",
            "MR" => "Master Rune",
            "OTH" => "Other",
            "P" => "Potion",
            "R" => "Ranged Weapon",
            "RD" => "Rod",
            "RG" => "Ring",
            "S" => "Shield",
            "SC" => "Scroll",
            "SCF" => "Spellcasting Focus",
            "SHP" => "Vehicle (Water)",
            "T" => "Tools",
            "TAH" => "Tack and Harness",
            "TG" => "Trade Good",
            "VEH" => "Vehicle (Land)",
            "WD" => "Wand",
            "$" => "Treasure",
            "$A" => "Treasure (Art Object)",
            "$C" => "Treasure (Coinage)",
            "$G" => "Treasure (Gemstone)",
            _ => "Unknown",
        })
    }

    /// Check if this is a magic item.
    pub fn is_magic(&self) -> bool {
        self.rarity.is_some() && self.rarity.as_ref().map_or(false, |r| r != "none")
    }
}

/// Data for inserting a new item.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = items)]
pub struct NewItem<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub item_type: Option<&'a str>,
    pub rarity: Option<&'a str>,
    pub data: &'a str,
    pub fluff: Option<&'a str>,
}

impl<'a> NewItem<'a> {
    /// Create a new item entry.
    pub fn new(name: &'a str, source: &'a str, data: &'a str) -> Self {
        Self {
            name,
            source,
            item_type: None,
            rarity: None,
            data,
            fluff: None,
        }
    }

    /// Set the item type.
    pub fn with_type(mut self, item_type: &'a str) -> Self {
        self.item_type = Some(item_type);
        self
    }

    /// Set the rarity.
    pub fn with_rarity(mut self, rarity: &'a str) -> Self {
        self.rarity = Some(rarity);
        self
    }
}

/// Filters for searching items.
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct ItemFilter {
    pub name_contains: Option<String>,
    pub source: Option<String>,
    pub item_type: Option<String>,
    pub rarity: Option<String>,
}

impl ItemFilter {
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

    pub fn with_type(mut self, item_type: impl Into<String>) -> Self {
        self.item_type = Some(item_type.into());
        self
    }

    pub fn with_rarity(mut self, rarity: impl Into<String>) -> Self {
        self.rarity = Some(rarity.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_item() {
        let data = r#"{"name":"Longsword"}"#;
        let item = NewItem::new("Longsword", "PHB", data);
        assert_eq!(item.name, "Longsword");
        assert_eq!(item.source, "PHB");
        assert!(item.item_type.is_none());
    }

    #[test]
    fn test_new_item_with_fields() {
        let data = r#"{"name":"Longsword"}"#;
        let item = NewItem::new("Longsword", "PHB", data)
            .with_type("M")
            .with_rarity("common");

        assert_eq!(item.item_type, Some("M"));
        assert_eq!(item.rarity, Some("common"));
    }

    #[test]
    fn test_item_filter() {
        let filter = ItemFilter::new()
            .with_name_contains("sword")
            .with_type("M")
            .with_rarity("rare");

        assert_eq!(filter.name_contains, Some("sword".to_string()));
        assert_eq!(filter.item_type, Some("M".to_string()));
        assert_eq!(filter.rarity, Some("rare".to_string()));
    }
}
