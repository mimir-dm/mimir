//! MCP Tool Implementations
//!
//! Each module provides tool definitions and handlers for a category of operations.

use serde_json::Value;
use std::collections::HashMap;

pub mod campaign;
pub mod catalog;
pub mod character;
pub mod document;
pub mod homebrew;
pub mod map;
pub mod module;

/// Create a properties map for tool input schema.
///
/// Takes a list of (name, type, description) tuples and returns a HashMap
/// suitable for `ToolInputSchema::new()`.
pub fn create_properties(
    props: Vec<(&str, &str, &str)>,
) -> Option<HashMap<String, serde_json::Map<String, Value>>> {
    if props.is_empty() {
        return None;
    }

    let mut map = HashMap::new();
    for (name, prop_type, description) in props {
        let mut inner = serde_json::Map::new();
        inner.insert("type".to_string(), Value::String(prop_type.to_string()));
        inner.insert(
            "description".to_string(),
            Value::String(description.to_string()),
        );
        map.insert(name.to_string(), inner);
    }
    Some(map)
}
