//! SRD (System Reference Document) filtering functionality
//!
//! This module provides functionality to filter 5etools content for items
//! that are part of the SRD or Basic Rules, which can be freely distributed
//! as open game content. It handles both boolean SRD flags and string values
//! that indicate renamed items.

use serde_json::{json, Value};
use std::collections::HashMap;

/// Trait for filtering content by SRD status
pub trait SrdFilter {
    /// Filter and transform items that are SRD content
    /// Returns a map of items with their SRD transformations applied
    fn filter_srd_content(&self) -> Vec<SrdItem>;
}

/// An item that has been processed for SRD inclusion
#[derive(Debug, Clone)]
pub struct SrdItem {
    /// The transformed JSON data
    pub data: Value,
    /// The original name before SRD transformation
    pub original_name: Option<String>,
    /// The SRD name (if renamed)
    pub srd_name: Option<String>,
    /// Whether this item was renamed for SRD
    pub was_renamed: bool,
}

impl SrdFilter for Value {
    fn filter_srd_content(&self) -> Vec<SrdItem> {
        let mut results = Vec::new();

        // Common keys where arrays of items might be found
        let keys = [
            "spell",
            "item",
            "race",
            "class",
            "subclass",
            "background",
            "feat",
            "optionalfeature",
            "reward",
            "object",
            "trap",
            "hazard",
            "action",
            "condition",
            "disease",
            "status",
            "creature",
            "monster",
            "npc",
            "vehicle",
            "object",
            "deity",
            "language",
            "table",
            "variantrule",
            "cult",
            "boon",
        ];

        for key in &keys {
            if let Some(array) = self.get(key).and_then(|v| v.as_array()) {
                for item in array {
                    if let Some(srd_item) = process_srd_item(item) {
                        results.push(srd_item);
                    }
                }
            }
        }

        results
    }
}

/// Process a single item for SRD inclusion
fn process_srd_item(item: &Value) -> Option<SrdItem> {
    // Check for SRD field
    let srd_status = check_srd_status(item);

    match srd_status {
        SrdStatus::Include => {
            // Include as-is
            Some(SrdItem {
                data: transform_item_for_srd(item, None),
                original_name: item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                srd_name: None,
                was_renamed: false,
            })
        }
        SrdStatus::IncludeRenamed(new_name) => {
            // Include with renamed title
            Some(SrdItem {
                data: transform_item_for_srd(item, Some(&new_name)),
                original_name: item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                srd_name: Some(new_name.clone()),
                was_renamed: true,
            })
        }
        SrdStatus::Exclude => None,
    }
}

/// Check the SRD status of an item
fn check_srd_status(item: &Value) -> SrdStatus {
    // Check explicit SRD field first
    if let Some(srd_value) = item.get("srd") {
        match srd_value {
            Value::Bool(true) => return SrdStatus::Include,
            Value::Bool(false) => return SrdStatus::Exclude,
            Value::String(name) if !name.is_empty() => {
                return SrdStatus::IncludeRenamed(name.clone());
            }
            _ => {} // Continue to check other fields
        }
    }

    // Check basicRules field
    if let Some(Value::Bool(true)) = item.get("basicRules") {
        return SrdStatus::Include;
    }

    // Default to exclude
    SrdStatus::Exclude
}

/// Transform an item for SRD inclusion
fn transform_item_for_srd(item: &Value, new_name: Option<&str>) -> Value {
    let mut transformed = item.clone();

    // If renaming, update the name field
    if let Some(srd_name) = new_name {
        if let Some(original_name) = item.get("name") {
            transformed["originalName"] = original_name.clone();
            transformed["srdName"] = json!(srd_name);
            transformed["name"] = json!(srd_name);
        }
    }

    // Always set source to SRD for the compiled book
    transformed["source"] = json!("SRD");

    // Mark as SRD content
    transformed["srd"] = json!(true);

    // Remove any non-SRD specific fields that shouldn't be in the open content
    if let Some(obj) = transformed.as_object_mut() {
        obj.remove("basicRules");
    }

    transformed
}

/// Internal enum for SRD status determination
#[derive(Debug)]
enum SrdStatus {
    Include,
    IncludeRenamed(String),
    Exclude,
}

/// Get all SRD content from a complete 5etools dataset
pub fn extract_all_srd_content(data: &Value) -> HashMap<String, Vec<SrdItem>> {
    let mut content_by_type = HashMap::new();

    // Content type mappings
    let content_types = [
        ("spell", "spells"),
        ("item", "items"),
        ("race", "races"),
        ("class", "classes"),
        ("subclass", "subclasses"),
        ("background", "backgrounds"),
        ("feat", "feats"),
        ("optionalfeature", "optionalfeatures"),
        ("reward", "rewards"),
        ("object", "objects"),
        ("trap", "traps"),
        ("hazard", "hazards"),
        ("action", "actions"),
        ("condition", "conditions"),
        ("disease", "diseases"),
        ("status", "status"),
        ("creature", "monsters"),
        ("monster", "monsters"),
        ("npc", "npcs"),
        ("vehicle", "vehicles"),
        ("deity", "deities"),
        ("language", "languages"),
        ("table", "tables"),
        ("variantrule", "variantrules"),
        ("cult", "cults"),
        ("boon", "boons"),
    ];

    for (json_key, output_key) in content_types.iter() {
        if let Some(array) = data.get(json_key).and_then(|v| v.as_array()) {
            let srd_items: Vec<SrdItem> = array.iter().filter_map(process_srd_item).collect();

            if !srd_items.is_empty() {
                content_by_type.insert(output_key.to_string(), srd_items);
            }
        }
    }

    content_by_type
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_srd_true() {
        let item = json!({
            "name": "Fireball",
            "srd": true,
            "source": "PHB"
        });

        let result = process_srd_item(&item);
        assert!(result.is_some());

        let srd_item = result.unwrap();
        assert!(!srd_item.was_renamed);
        assert_eq!(srd_item.data["source"], "SRD");
        assert_eq!(srd_item.data["name"], "Fireball");
    }

    #[test]
    fn test_srd_renamed() {
        let item = json!({
            "name": "Crystal Ball",
            "srd": "Orb",
            "source": "DMG"
        });

        let result = process_srd_item(&item);
        assert!(result.is_some());

        let srd_item = result.unwrap();
        assert!(srd_item.was_renamed);
        assert_eq!(srd_item.original_name, Some("Crystal Ball".to_string()));
        assert_eq!(srd_item.srd_name, Some("Orb".to_string()));
        assert_eq!(srd_item.data["name"], "Orb");
        assert_eq!(srd_item.data["originalName"], "Crystal Ball");
        assert_eq!(srd_item.data["srdName"], "Orb");
        assert_eq!(srd_item.data["source"], "SRD");
    }

    #[test]
    fn test_basic_rules() {
        let item = json!({
            "name": "Aid",
            "basicRules": true,
            "source": "PHB"
        });

        let result = process_srd_item(&item);
        assert!(result.is_some());

        let srd_item = result.unwrap();
        assert!(!srd_item.was_renamed);
        assert_eq!(srd_item.data["source"], "SRD");
    }

    #[test]
    fn test_exclude() {
        let item = json!({
            "name": "Proprietary Spell",
            "srd": false,
            "source": "PHB"
        });

        let result = process_srd_item(&item);
        assert!(result.is_none());
    }
}
