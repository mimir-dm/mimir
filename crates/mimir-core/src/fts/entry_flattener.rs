//! Entry Flattener
//!
//! Flattens 5etools entry arrays into searchable plain text.
//! Handles nested entry structures and strips 5etools tag markers.

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

/// Regex to match 5etools tag markers like {@spell fireball|PHB}
/// Captures the display text (first word after the tag type, before any pipe)
static TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{@\w+\s+([^|}]+)(?:\|[^}]*)?\}").expect("Failed to compile tag regex")
});

/// Flattens 5etools entry arrays into searchable plain text.
///
/// # Arguments
///
/// * `entries` - Array of 5etools entry values
///
/// # Returns
///
/// A single string with all text content joined by spaces.
///
/// # Example
///
/// ```
/// use mimir_core::fts::flatten_entries;
/// use serde_json::json;
///
/// let entries = vec![
///     json!("This is plain text."),
///     json!({"type": "entries", "entries": ["Nested text."]}),
/// ];
/// let result = flatten_entries(&entries);
/// assert!(result.contains("plain text"));
/// assert!(result.contains("Nested text"));
/// ```
pub fn flatten_entries(entries: &[Value]) -> String {
    let mut result = Vec::new();
    for entry in entries {
        flatten_entry_recursive(entry, &mut result);
    }
    result.join(" ")
}

/// Flattens a single 5etools entry value into searchable plain text.
///
/// Use this when you have a single entry rather than an array.
pub fn flatten_entry(entry: &Value) -> String {
    let mut result = Vec::new();
    flatten_entry_recursive(entry, &mut result);
    result.join(" ")
}

/// Recursively flattens an entry value into text fragments.
/// Tags like {@spell fireball|PHB} are preserved for render-time transformation.
fn flatten_entry_recursive(entry: &Value, output: &mut Vec<String>) {
    match entry {
        Value::String(s) => {
            if !s.is_empty() {
                output.push(s.clone());
            }
        }
        Value::Object(obj) => {
            // Handle name/title fields first
            if let Some(Value::String(name)) = obj.get("name") {
                output.push(name.clone());
            }
            if let Some(Value::String(title)) = obj.get("title") {
                output.push(title.clone());
            }

            // Handle entry objects with nested entries
            if let Some(entries) = obj.get("entries") {
                match entries {
                    Value::Array(arr) => {
                        for e in arr {
                            flatten_entry_recursive(e, output);
                        }
                    }
                    Value::String(s) => {
                        output.push(s.clone());
                    }
                    _ => {}
                }
            }

            // Handle items in lists
            if let Some(Value::Array(items)) = obj.get("items") {
                for item in items {
                    flatten_entry_recursive(item, output);
                }
            }

            // Handle table captions
            if let Some(Value::String(caption)) = obj.get("caption") {
                output.push(caption.clone());
            }

            // Handle quote text
            if let Some(Value::Array(quotes)) = obj.get("entries") {
                for quote in quotes {
                    flatten_entry_recursive(quote, output);
                }
            }

            // Handle "by" attribution in quotes
            if let Some(Value::String(by)) = obj.get("by") {
                output.push(by.clone());
            }
        }
        Value::Array(arr) => {
            for e in arr {
                flatten_entry_recursive(e, output);
            }
        }
        Value::Number(n) => {
            output.push(n.to_string());
        }
        _ => {}
    }
}

/// Strips 5etools tag markers like {@spell fireball} -> "fireball".
///
/// Common tags include:
/// - {@spell fireball} -> fireball
/// - {@creature goblin|MM} -> goblin
/// - {@item longsword|PHB} -> longsword
/// - {@condition blinded} -> blinded
/// - {@dice 1d6} -> 1d6
/// - {@damage 2d6} -> 2d6
/// - {@hit +5} -> +5
/// - {@dc 15} -> 15
///
/// # Arguments
///
/// * `s` - String potentially containing 5etools tags
///
/// # Returns
///
/// String with all tags replaced by their display text.
pub fn strip_5etools_tags(s: &str) -> String {
    TAG_REGEX.replace_all(s, "$1").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_strip_simple_tag() {
        assert_eq!(strip_5etools_tags("{@spell fireball}"), "fireball");
    }

    #[test]
    fn test_strip_tag_with_source() {
        assert_eq!(strip_5etools_tags("{@creature goblin|MM}"), "goblin");
    }

    #[test]
    fn test_strip_multiple_tags() {
        let input = "Cast {@spell fireball} at the {@creature goblin|MM}.";
        let expected = "Cast fireball at the goblin.";
        assert_eq!(strip_5etools_tags(input), expected);
    }

    #[test]
    fn test_strip_dice_tag() {
        assert_eq!(strip_5etools_tags("{@dice 2d6+3}"), "2d6+3");
    }

    #[test]
    fn test_strip_damage_tag() {
        assert_eq!(strip_5etools_tags("{@damage 3d8}"), "3d8");
    }

    #[test]
    fn test_strip_hit_tag() {
        assert_eq!(strip_5etools_tags("{@hit +7}"), "+7");
    }

    #[test]
    fn test_strip_dc_tag() {
        assert_eq!(strip_5etools_tags("{@dc 15}"), "15");
    }

    #[test]
    fn test_strip_condition_tag() {
        assert_eq!(strip_5etools_tags("{@condition frightened}"), "frightened");
    }

    #[test]
    fn test_no_tags() {
        let input = "Plain text without any tags.";
        assert_eq!(strip_5etools_tags(input), input);
    }

    #[test]
    fn test_flatten_plain_string() {
        let entries = vec![json!("Hello world")];
        assert_eq!(flatten_entries(&entries), "Hello world");
    }

    #[test]
    fn test_flatten_multiple_strings() {
        let entries = vec![json!("First."), json!("Second.")];
        assert_eq!(flatten_entries(&entries), "First. Second.");
    }

    #[test]
    fn test_flatten_nested_entries() {
        let entries = vec![json!({
            "type": "entries",
            "name": "Feature Name",
            "entries": ["Description of the feature."]
        })];
        let result = flatten_entries(&entries);
        assert!(result.contains("Feature Name"));
        assert!(result.contains("Description of the feature"));
    }

    #[test]
    fn test_flatten_deeply_nested() {
        let entries = vec![json!({
            "type": "entries",
            "entries": [{
                "type": "entries",
                "name": "Inner",
                "entries": ["Deep text"]
            }]
        })];
        let result = flatten_entries(&entries);
        assert!(result.contains("Inner"));
        assert!(result.contains("Deep text"));
    }

    #[test]
    fn test_flatten_preserves_tags() {
        // Tags are preserved for render-time transformation to cross-links
        let entries = vec![json!("The {@creature goblin|MM} attacks with {@dice 1d6} damage.")];
        let result = flatten_entries(&entries);
        assert!(result.contains("{@creature goblin|MM}"));
        assert!(result.contains("{@dice 1d6}"));
    }

    #[test]
    fn test_flatten_list() {
        let entries = vec![json!({
            "type": "list",
            "items": ["Item one", "Item two", "Item three"]
        })];
        let result = flatten_entries(&entries);
        assert!(result.contains("Item one"));
        assert!(result.contains("Item two"));
        assert!(result.contains("Item three"));
    }

    #[test]
    fn test_flatten_empty() {
        let entries: Vec<Value> = vec![];
        assert_eq!(flatten_entries(&entries), "");
    }

    #[test]
    fn test_flatten_quote() {
        let entries = vec![json!({
            "type": "quote",
            "entries": ["To be or not to be."],
            "by": "Shakespeare"
        })];
        let result = flatten_entries(&entries);
        assert!(result.contains("To be or not to be"));
        assert!(result.contains("Shakespeare"));
    }

    #[test]
    fn test_flatten_table_caption() {
        let entries = vec![json!({
            "type": "table",
            "caption": "Random Encounters",
            "rows": []
        })];
        let result = flatten_entries(&entries);
        assert!(result.contains("Random Encounters"));
    }

    #[test]
    fn test_flatten_mixed_content() {
        let entries = vec![
            json!("Introduction text."),
            json!({
                "type": "entries",
                "name": "Section One",
                "entries": [
                    "First paragraph.",
                    {
                        "type": "list",
                        "items": ["Bullet A", "Bullet B"]
                    }
                ]
            }),
            json!("Conclusion."),
        ];
        let result = flatten_entries(&entries);
        assert!(result.contains("Introduction text"));
        assert!(result.contains("Section One"));
        assert!(result.contains("First paragraph"));
        assert!(result.contains("Bullet A"));
        assert!(result.contains("Conclusion"));
    }
}
