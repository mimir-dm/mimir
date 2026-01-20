//! Source Filtering
//!
//! Filters 5etools JSON entities by their source code.
//! Handles multiple source reference patterns found in 5etools data.
//!
//! Adapted from mimir-5etools-splitter/src/filter.rs.

use serde_json::Value;
use std::collections::HashSet;

/// Trait for filtering content by source.
///
/// Implemented for `serde_json::Value` to allow filtering any 5etools JSON data.
pub trait SourceFilter {
    /// Filter content items by their source identifier.
    ///
    /// Returns all items from common entity arrays that match the given source.
    /// Handles multiple source reference patterns:
    /// - Direct `source` field
    /// - `inheritsFrom[].source` (for classes/subclasses)
    /// - `sources[].source` (for multi-source items)
    fn filter_by_source(&self, source: &str) -> Vec<Value>;

    /// Filter a specific key's array by source.
    ///
    /// More targeted than `filter_by_source` - only checks a single JSON key.
    fn filter_key_by_source(&self, key: &str, source: &str) -> Vec<Value>;
}

impl SourceFilter for Value {
    fn filter_by_source(&self, source: &str) -> Vec<Value> {
        let mut results = Vec::new();

        // Common keys where arrays of items might be found
        for key in ENTITY_KEYS {
            if let Some(array) = self.get(key).and_then(|v| v.as_array()) {
                for item in array {
                    if matches_source(item, source) {
                        results.push(item.clone());
                    }
                }
            }
        }

        results
    }

    fn filter_key_by_source(&self, key: &str, source: &str) -> Vec<Value> {
        let mut results = Vec::new();

        if let Some(array) = self.get(key).and_then(|v| v.as_array()) {
            for item in array {
                if matches_source(item, source) {
                    results.push(item.clone());
                }
            }
        }

        results
    }
}

/// Common JSON keys containing entity arrays in 5etools data.
pub const ENTITY_KEYS: &[&str] = &[
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
    "psionic",
    "classFeature",
    "subclassFeature",
];

/// Check if an item matches the given source.
///
/// Handles multiple source reference patterns found in 5etools data:
/// 1. Direct `source` field match
/// 2. `inheritsFrom[].source` match (for classes/subclasses)
/// 3. `sources[].source` match (for multi-source items)
pub fn matches_source(item: &Value, source: &str) -> bool {
    // Direct source match
    if let Some(item_source) = item.get("source").and_then(|v| v.as_str()) {
        if item_source == source {
            return true;
        }
    }

    // Check inheritsFrom for classes/subclasses
    if let Some(inherits) = item.get("inheritsFrom").and_then(|v| v.as_array()) {
        for inherit in inherits {
            if let Some(inherit_source) = inherit.get("source").and_then(|v| v.as_str()) {
                if inherit_source == source {
                    return true;
                }
            }
        }
    }

    // Check sources array (some items have multiple sources)
    if let Some(sources) = item.get("sources").and_then(|v| v.as_array()) {
        for src in sources {
            if let Some(src_obj) = src.as_object() {
                if let Some(src_source) = src_obj.get("source").and_then(|v| v.as_str()) {
                    if src_source == source {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Get all unique sources from a JSON value.
///
/// Recursively searches through the entire JSON structure to find
/// all unique source codes referenced anywhere.
pub fn get_all_sources(data: &Value) -> HashSet<String> {
    let mut sources = HashSet::new();
    collect_sources_recursive(data, &mut sources);
    sources
}

fn collect_sources_recursive(value: &Value, sources: &mut HashSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(source) = map.get("source").and_then(|v| v.as_str()) {
                sources.insert(source.to_string());
            }
            for v in map.values() {
                collect_sources_recursive(v, sources);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_sources_recursive(v, sources);
            }
        }
        _ => {}
    }
}

/// Filter entities by a list of allowed sources.
///
/// Returns items that match ANY of the provided sources.
pub fn filter_by_sources(items: &[Value], sources: &[&str]) -> Vec<Value> {
    items
        .iter()
        .filter(|item| sources.iter().any(|source| matches_source(item, source)))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_direct_source_match() {
        let item = json!({
            "name": "Fireball",
            "source": "PHB"
        });
        assert!(matches_source(&item, "PHB"));
        assert!(!matches_source(&item, "DMG"));
    }

    #[test]
    fn test_inherits_from_match() {
        let item = json!({
            "name": "Champion",
            "source": "PHB",
            "inheritsFrom": [
                {"source": "PHB", "name": "Fighter"}
            ]
        });
        assert!(matches_source(&item, "PHB"));
    }

    #[test]
    fn test_sources_array_match() {
        let item = json!({
            "name": "Multi-Source Item",
            "sources": [
                {"source": "PHB"},
                {"source": "XGE"}
            ]
        });
        assert!(matches_source(&item, "PHB"));
        assert!(matches_source(&item, "XGE"));
        assert!(!matches_source(&item, "DMG"));
    }

    #[test]
    fn test_filter_by_source() {
        let data = json!({
            "spell": [
                {"name": "Fireball", "source": "PHB"},
                {"name": "Eldritch Blast", "source": "PHB"},
                {"name": "Custom Spell", "source": "HB"}
            ]
        });

        let phb_spells = data.filter_by_source("PHB");
        assert_eq!(phb_spells.len(), 2);
        assert!(phb_spells.iter().all(|s| s["source"] == "PHB"));
    }

    #[test]
    fn test_filter_key_by_source() {
        let data = json!({
            "spell": [
                {"name": "Fireball", "source": "PHB"},
                {"name": "Custom Spell", "source": "HB"}
            ],
            "item": [
                {"name": "Longsword", "source": "PHB"}
            ]
        });

        let phb_spells = data.filter_key_by_source("spell", "PHB");
        assert_eq!(phb_spells.len(), 1);
        assert_eq!(phb_spells[0]["name"], "Fireball");
    }

    #[test]
    fn test_get_all_sources() {
        let data = json!({
            "spell": [
                {"name": "Fireball", "source": "PHB"},
                {"name": "Custom Spell", "source": "HB"}
            ],
            "item": [
                {"name": "Longsword", "source": "PHB"},
                {"name": "Holy Avenger", "source": "DMG"}
            ]
        });

        let sources = get_all_sources(&data);
        assert!(sources.contains("PHB"));
        assert!(sources.contains("DMG"));
        assert!(sources.contains("HB"));
        assert_eq!(sources.len(), 3);
    }

    #[test]
    fn test_filter_by_sources() {
        let items = vec![
            json!({"name": "A", "source": "PHB"}),
            json!({"name": "B", "source": "DMG"}),
            json!({"name": "C", "source": "XGE"}),
        ];

        let filtered = filter_by_sources(&items, &["PHB", "DMG"]);
        assert_eq!(filtered.len(), 2);
    }
}
