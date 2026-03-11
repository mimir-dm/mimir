//! Shared utilities for card rendering
//!
//! Natural text splitting, 5etools entry flattening, and Typst escaping
//! used by equipment cards, spell cards, and potentially other card types.

use serde_json::Value;

/// Character budget for the description area on a 2.5" x 3.25" card.
///
/// At 5.5pt font on a ~2.3in text width, roughly 55 chars/line x 18 lines
/// ≈ 990 chars. We use 800 as a conservative budget accounting for variable
/// header/stats height and the footer overlay.
pub const SMALL_CARD_DESC_BUDGET: usize = 800;

/// Result of splitting card text for foldable layout
#[derive(Debug)]
pub struct CardTextSplit {
    /// Text for the front card
    pub front: String,
    /// Text for the continuation card (empty if it all fits)
    pub back: String,
    /// Whether a continuation card is needed
    pub is_foldable: bool,
}

/// Split text at a natural boundary near `budget` characters.
///
/// Boundary priority (searches backward from budget):
/// 1. Paragraph break (double newline or ` - ` list item boundary)
/// 2. Sentence end (`. ` or `.\n`)
/// 3. Clause boundary (`, `)
/// 4. Word boundary (last space)
/// 5. Hard cut at budget (fallback)
pub fn split_text_natural(text: &str, budget: usize) -> CardTextSplit {
    if text.len() <= budget {
        return CardTextSplit {
            front: text.to_string(),
            back: String::new(),
            is_foldable: false,
        };
    }

    // Search backward from budget for a natural break point
    let search_region = &text[..budget];

    // 1. Paragraph break — look for double newline or list item boundary
    let break_pos = search_region
        .rfind("\n\n")
        .map(|p| p + 2) // split after the double newline
        .or_else(|| {
            // List item boundary: find last ` - ` that starts a list item
            search_region.rfind("\n- ").map(|p| p + 1)
        })
        // 2. Sentence boundary: `. ` or `.\n`
        .or_else(|| {
            search_region.rfind(". ").map(|p| p + 2)
        })
        .or_else(|| {
            search_region.rfind(".\n").map(|p| p + 2)
        })
        // 3. Clause boundary: `, `
        .or_else(|| {
            // Only use comma splits in the back half to avoid tiny front cards
            let half = budget / 2;
            search_region[half..].rfind(", ").map(|p| half + p + 2)
        })
        // 4. Word boundary: last space
        .or_else(|| {
            search_region.rfind(' ').map(|p| p + 1)
        })
        // 5. Hard cut
        .unwrap_or(budget);

    let front = text[..break_pos].trim_end().to_string();
    let back = text[break_pos..].trim_start().to_string();

    CardTextSplit {
        is_foldable: !back.is_empty(),
        front,
        back,
    }
}

/// Recursively flatten 5etools entry arrays into plain text.
///
/// Handles string entries, objects with "entries" sub-arrays,
/// list items, and other structured content.
pub fn flatten_entries(entries: &[Value]) -> String {
    let mut parts = Vec::new();
    for entry in entries {
        if let Some(s) = entry.as_str() {
            parts.push(s.to_string());
        } else if let Some(obj) = entry.as_object() {
            // Handle {"type": "list", "items": [...]}
            if let Some(items) = obj.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(s) = item.as_str() {
                        parts.push(format!("- {}", s));
                    } else if let Some(sub_entries) =
                        item.get("entries").and_then(|v| v.as_array())
                    {
                        parts.push(flatten_entries(sub_entries));
                    } else if let Some(entry_str) = item.get("entry").and_then(|v| v.as_str()) {
                        parts.push(format!("- {}", entry_str));
                    }
                }
            }
            // Handle {"type": "entries", "name": "...", "entries": [...]}
            if let Some(sub_entries) = obj.get("entries").and_then(|v| v.as_array()) {
                if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                    parts.push(format!("{}.", name));
                }
                parts.push(flatten_entries(sub_entries));
            }
            // Handle {"type": "table", ...} - just note it exists
            if obj.get("type").and_then(|v| v.as_str()) == Some("table") {
                if let Some(caption) = obj.get("caption").and_then(|v| v.as_str()) {
                    parts.push(format!("[Table: {}]", caption));
                }
            }
        }
    }
    parts.join(" ")
}

/// Escape special Typst characters in content text
pub fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('"', "\\\"")
        .replace('_', "\\_")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('@', "\\@")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── split_text_natural ─────────────────────────────────────────────

    #[test]
    fn test_short_text_no_split() {
        let result = split_text_natural("Hello world.", 800);
        assert!(!result.is_foldable);
        assert_eq!(result.front, "Hello world.");
        assert!(result.back.is_empty());
    }

    #[test]
    fn test_split_on_paragraph() {
        let text = format!("{}.\n\nSecond paragraph here.", "A".repeat(790));
        let result = split_text_natural(&text, 800);
        assert!(result.is_foldable);
        assert!(result.front.ends_with('.'));
        assert_eq!(result.back, "Second paragraph here.");
    }

    #[test]
    fn test_split_on_sentence() {
        let text = format!("{}. More text follows here.", "B".repeat(790));
        let result = split_text_natural(&text, 800);
        assert!(result.is_foldable);
        assert!(result.front.ends_with('.'));
        assert!(result.back.starts_with("More"));
    }

    #[test]
    fn test_split_on_comma() {
        // No periods or paragraphs — should fall back to comma
        let text = format!("{}word, rest of the content here", "no-period ".repeat(80));
        let budget = text.len() - 20;
        let result = split_text_natural(&text, budget);
        assert!(result.is_foldable);
        // Should split at a comma
        assert!(result.front.ends_with(',') || result.front.ends_with("word"));
    }

    #[test]
    fn test_split_on_word_boundary() {
        // No punctuation at all — should split at last space
        let text = "word ".repeat(200);
        let result = split_text_natural(&text, 100);
        assert!(result.is_foldable);
        assert!(!result.front.ends_with(' ')); // trimmed
    }

    #[test]
    fn test_split_on_list_item() {
        let text = format!(
            "{}.\n- First item.\n- Second item that pushes over.",
            "A".repeat(770)
        );
        let result = split_text_natural(&text, 800);
        assert!(result.is_foldable);
        assert!(result.back.starts_with("- ") || result.back.starts_with("Second"));
    }

    // ── flatten_entries ────────────────────────────────────────────────

    #[test]
    fn test_flatten_plain_strings() {
        let entries = vec![json!("First."), json!("Second.")];
        assert_eq!(flatten_entries(&entries), "First. Second.");
    }

    #[test]
    fn test_flatten_list_items() {
        let entries = vec![json!({
            "type": "list",
            "items": ["apple", "banana"]
        })];
        assert_eq!(flatten_entries(&entries), "- apple - banana");
    }

    #[test]
    fn test_flatten_named_sub_entries() {
        let entries = vec![json!({
            "type": "entries",
            "name": "Special",
            "entries": ["Does a thing."]
        })];
        assert_eq!(flatten_entries(&entries), "Special. Does a thing.");
    }

    #[test]
    fn test_flatten_table_caption() {
        let entries = vec![json!({
            "type": "table",
            "caption": "Random Effects"
        })];
        assert_eq!(flatten_entries(&entries), "[Table: Random Effects]");
    }

    #[test]
    fn test_flatten_nested() {
        let entries = vec![
            json!("Intro text."),
            json!({
                "type": "entries",
                "name": "At Higher Levels",
                "entries": ["The damage increases by 1d6."]
            }),
        ];
        let result = flatten_entries(&entries);
        assert!(result.contains("Intro text."));
        assert!(result.contains("At Higher Levels."));
        assert!(result.contains("damage increases"));
    }

    // ── escape_typst ──────────────────────────────────────────────────

    #[test]
    fn test_escape_5etools_tags() {
        assert_eq!(
            escape_typst("{@damage 1d6}"),
            "\\{\\@damage 1d6\\}"
        );
    }

    #[test]
    fn test_escape_angle_brackets() {
        assert_eq!(escape_typst("<condition>"), "\\<condition\\>");
    }
}
