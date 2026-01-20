//! SRD (System Reference Document) Detection and Transformation
//!
//! Identifies content that is part of the SRD or Basic Rules, which can be
//! freely distributed as Open Game Content. Handles both boolean SRD flags
//! and string values that indicate renamed items.
//!
//! Adapted from mimir-5etools-splitter/src/srd_filter.rs.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Trait for filtering content by SRD status.
pub trait SrdFilter {
    /// Filter and transform items that are SRD content.
    ///
    /// Returns items with their SRD transformations applied.
    fn filter_srd_content(&self) -> Vec<SrdItem>;
}

/// An item that has been identified as SRD content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrdItem {
    /// The (possibly transformed) JSON data.
    pub data: Value,
    /// The original name before SRD transformation (if renamed).
    pub original_name: Option<String>,
    /// The SRD name (if different from original).
    pub srd_name: Option<String>,
    /// Whether this item was renamed for SRD compliance.
    pub was_renamed: bool,
}

/// SRD status for an entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SrdStatus {
    /// Include in SRD as-is.
    Include,
    /// Include in SRD with a renamed title.
    IncludeRenamed(String),
    /// Not part of SRD.
    Exclude,
}

impl SrdFilter for Value {
    fn filter_srd_content(&self) -> Vec<SrdItem> {
        let mut results = Vec::new();

        for key in SRD_ENTITY_KEYS {
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

/// Entity keys that may contain SRD content.
const SRD_ENTITY_KEYS: &[&str] = &[
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
    "deity",
    "language",
    "table",
    "variantrule",
    "cult",
    "boon",
];

/// Check the SRD status of an item.
///
/// Checks in order:
/// 1. `srd` field: `true` → Include, `false` → Exclude, `"string"` → Include renamed
/// 2. `basicRules` field: `true` → Include
/// 3. Default: Exclude
pub fn check_srd_status(item: &Value) -> SrdStatus {
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

/// Check if an item is SRD content (quick check without transformation).
pub fn is_srd(item: &Value) -> bool {
    !matches!(check_srd_status(item), SrdStatus::Exclude)
}

/// Process a single item for SRD inclusion.
///
/// Returns `Some(SrdItem)` if the item is SRD content, `None` otherwise.
pub fn process_srd_item(item: &Value) -> Option<SrdItem> {
    let srd_status = check_srd_status(item);

    match srd_status {
        SrdStatus::Include => Some(SrdItem {
            data: transform_item_for_srd(item, None),
            original_name: item
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            srd_name: None,
            was_renamed: false,
        }),
        SrdStatus::IncludeRenamed(new_name) => Some(SrdItem {
            data: transform_item_for_srd(item, Some(&new_name)),
            original_name: item
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            srd_name: Some(new_name.clone()),
            was_renamed: true,
        }),
        SrdStatus::Exclude => None,
    }
}

/// Transform an item for SRD inclusion.
///
/// - If renaming, updates the `name` field and stores original in `originalName`
/// - Sets `source` to "SRD"
/// - Sets `srd` to `true`
/// - Removes `basicRules` field (non-OGC metadata)
pub fn transform_item_for_srd(item: &Value, new_name: Option<&str>) -> Value {
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

/// Get all SRD content from a complete 5etools dataset.
///
/// Returns a map of content type to SRD items.
pub fn extract_all_srd_content(data: &Value) -> HashMap<String, Vec<SrdItem>> {
    let mut content_by_type = HashMap::new();

    // Content type mappings (json_key -> output_key)
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
                content_by_type
                    .entry(output_key.to_string())
                    .or_insert_with(Vec::new)
                    .extend(srd_items);
            }
        }
    }

    content_by_type
}

/// Count SRD items in a dataset without transforming them.
pub fn count_srd_items(data: &Value) -> usize {
    let mut count = 0;

    for key in SRD_ENTITY_KEYS {
        if let Some(array) = data.get(key).and_then(|v| v.as_array()) {
            count += array.iter().filter(|item| is_srd(item)).count();
        }
    }

    count
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
        // basicRules should be removed
        assert!(srd_item.data.get("basicRules").is_none());
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

    #[test]
    fn test_no_srd_field() {
        let item = json!({
            "name": "Custom Spell",
            "source": "HB"
        });

        let result = process_srd_item(&item);
        assert!(result.is_none());
    }

    #[test]
    fn test_is_srd() {
        assert!(is_srd(&json!({"srd": true})));
        assert!(is_srd(&json!({"srd": "Renamed"})));
        assert!(is_srd(&json!({"basicRules": true})));
        assert!(!is_srd(&json!({"srd": false})));
        assert!(!is_srd(&json!({"name": "Test"})));
    }

    #[test]
    fn test_check_srd_status() {
        assert_eq!(
            check_srd_status(&json!({"srd": true})),
            SrdStatus::Include
        );
        assert_eq!(
            check_srd_status(&json!({"srd": "Orb"})),
            SrdStatus::IncludeRenamed("Orb".to_string())
        );
        assert_eq!(
            check_srd_status(&json!({"srd": false})),
            SrdStatus::Exclude
        );
        assert_eq!(
            check_srd_status(&json!({"basicRules": true})),
            SrdStatus::Include
        );
    }

    #[test]
    fn test_extract_all_srd_content() {
        let data = json!({
            "spell": [
                {"name": "Fireball", "srd": true},
                {"name": "Custom", "srd": false}
            ],
            "item": [
                {"name": "Longsword", "srd": true}
            ]
        });

        let srd_content = extract_all_srd_content(&data);
        assert_eq!(srd_content.get("spells").map(|v| v.len()), Some(1));
        assert_eq!(srd_content.get("items").map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_count_srd_items() {
        let data = json!({
            "spell": [
                {"name": "Fireball", "srd": true},
                {"name": "Custom", "srd": false},
                {"name": "Aid", "basicRules": true}
            ]
        });

        assert_eq!(count_srd_items(&data), 2);
    }
}
