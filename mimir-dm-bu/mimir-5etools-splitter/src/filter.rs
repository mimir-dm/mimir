use serde_json::Value;
use std::collections::HashSet;

/// Trait for filtering content by source
pub trait SourceFilter {
    /// Filters content items by their source identifier.
    fn filter_by_source(&self, source: &str) -> Vec<Value>;
}

impl SourceFilter for Value {
    fn filter_by_source(&self, source: &str) -> Vec<Value> {
        let mut results = Vec::new();

        // Try common keys where arrays of items might be
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
        ];

        for key in &keys {
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
}

/// Check if an item matches the given source
fn matches_source(item: &Value, source: &str) -> bool {
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

/// Get all unique sources from a JSON value
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
