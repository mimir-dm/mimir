//! 5etools entries in a detail view (a catalog entry, a cross-reference):
//! a port of the desktop's `formatEntries`. Named entries get `<h5>`, a
//! section `<h4>`; insets are read-aloud boxes.

use serde_json::Value;

use crate::tags::tags;
use crate::{esc, link};

/// A list of entries (anything else: nothing).
pub fn entries(list: &Value) -> String {
    let Some(items) = list.as_array() else {
        return String::new();
    };
    items.iter().map(entry).collect()
}

/// The text of a JSON string, else empty.
pub fn s(v: &Value) -> &str {
    v.as_str().unwrap_or("")
}

/// One entry.
pub fn entry(e: &Value) -> String {
    if let Some(text) = e.as_str() {
        return format!("<p>{}</p>", tags(text));
    }
    let Some(obj) = e.as_object() else {
        return String::new();
    };
    let name = obj.get("name").and_then(Value::as_str);
    let inner = || obj.get("entries").map(entries).unwrap_or_default();
    match obj.get("type").and_then(Value::as_str).unwrap_or("") {
        "entries" => {
            let head = name
                .map(|n| format!("<h5>{}</h5>", tags(n)))
                .unwrap_or_default();
            head + &inner()
        }
        "section" => {
            let head = name
                .map(|n| format!("<h4>{}</h4>", tags(n)))
                .unwrap_or_default();
            head + &inner()
        }
        "list" => list(obj.get("items")),
        "table" => table(e),
        "inset" | "insetReadaloud" => {
            let head = name
                .map(|n| format!("<h5>{}</h5>", tags(n)))
                .unwrap_or_default();
            format!(r#"<div class="inset-readaloud">{head}{}</div>"#, inner())
        }
        "quote" => {
            let by = obj
                .get("by")
                .and_then(Value::as_str)
                .map(|b| format!("<footer>— {}</footer>", tags(b)))
                .unwrap_or_default();
            format!("<blockquote>{}{by}</blockquote>", inner())
        }
        "statblock" => statblock(e),
        "image" => String::new(),
        "item" | "itemSub" | "itemSpell" => {
            let head = name
                .map(|n| {
                    format!(
                        "<strong>{}</strong> ",
                        tags(n.trim_end_matches(['.', ':']).trim())
                    )
                })
                .unwrap_or_default();
            let body = obj
                .get("entry")
                .and_then(Value::as_str)
                .map(tags)
                .unwrap_or_else(|| {
                    obj.get("entries")
                        .and_then(Value::as_array)
                        .map(|l| {
                            l.iter()
                                .map(|x| x.as_str().map(tags).unwrap_or_else(|| entry(x)))
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                        .unwrap_or_default()
                });
            format!("<p>{head}{body}</p>")
        }
        _ => {
            if obj.contains_key("entries") {
                inner()
            } else if let Some(t) = obj.get("text").and_then(Value::as_str) {
                format!("<p>{}</p>", tags(t))
            } else if let Some(t) = obj.get("entry").and_then(Value::as_str) {
                format!("<p>{}</p>", tags(t))
            } else {
                String::new()
            }
        }
    }
}

fn list(items: Option<&Value>) -> String {
    let lis: String = items
        .and_then(Value::as_array)
        .map(|l| {
            l.iter()
                .map(|i| match i.as_str() {
                    Some(t) => format!("<li>{}</li>", tags(t)),
                    None => format!("<li>{}</li>", entry(i)),
                })
                .collect()
        })
        .unwrap_or_default();
    format!("<ul>{lis}</ul>")
}

/// A `statblock` entry: a link to the creature, else its name.
pub fn statblock(e: &Value) -> String {
    let (name, source) = (s(&e["name"]), s(&e["source"]));
    if name.is_empty() {
        return String::new();
    }
    let tag = e["tag"].as_str().unwrap_or("creature");
    if tag == "creature" {
        format!(
            r#"<p class="statblock-ref"><em>See: </em>{}</p>"#,
            link("creature", name, source, &esc(name))
        )
    } else {
        format!(
            r#"<p class="statblock-ref"><em>See: {}</em></p>"#,
            esc(name)
        )
    }
}

/// A roll cell: "3", or "2-5".
pub fn roll_cell(roll: &Value) -> Option<String> {
    if let Some(x) = roll.get("exact").and_then(Value::as_i64) {
        return Some(x.to_string());
    }
    match (
        roll.get("min").and_then(Value::as_i64),
        roll.get("max").and_then(Value::as_i64),
    ) {
        (Some(a), Some(b)) if a == b => Some(a.to_string()),
        (Some(a), Some(b)) => Some(format!("{a}-{b}")),
        _ => None,
    }
}

/// A table entry.
pub fn table(t: &Value) -> String {
    let mut html = String::from(r#"<table class="entry-table">"#);
    if let Some(c) = t["caption"].as_str() {
        html += &format!("<caption>{}</caption>", tags(c));
    }
    let labels = t["colLabels"].as_array();
    if let Some(labels) = labels {
        html += "<thead><tr>";
        for l in labels {
            html += &format!("<th>{}</th>", tags(s(l)));
        }
        html += "</tr></thead>";
    }
    let dice_column = labels
        .and_then(|l| l.first())
        .and_then(Value::as_str)
        .is_some_and(|l| l.contains("{@dice"));
    if let Some(rows) = t["rows"].as_array() {
        html += "<tbody>";
        for (i, row) in rows.iter().enumerate() {
            html += "<tr>";
            let cells = row
                .as_array()
                .cloned()
                .or_else(|| row.get("row").and_then(Value::as_array).cloned());
            match cells {
                Some(cells) => {
                    for (j, cell) in cells.iter().enumerate() {
                        let empty = cell.is_null() || cell.as_str() == Some("");
                        let content = if j == 0 && dice_column && empty {
                            (i + 1).to_string()
                        } else if let Some(text) = cell.as_str() {
                            tags(text)
                        } else if let Some(n) = cell.as_i64() {
                            n.to_string()
                        } else if cell["type"] == "cell" && cell.get("roll").is_some() {
                            roll_cell(&cell["roll"]).unwrap_or_default()
                        } else {
                            entry(cell)
                        };
                        html += &format!("<td>{content}</td>");
                    }
                }
                None => html += &format!("<td>{}</td>", tags(s(row))),
            }
            html += "</tr>";
        }
        html += "</tbody>";
    }
    html + "</table>"
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // The desktop's formatEntries tests, ported.

    #[test]
    fn strings_are_paragraphs() {
        assert!(entries(&json!(["Hello world"])).contains("<p>Hello world</p>"));
    }

    #[test]
    fn named_entries() {
        let r = entries(
            &json!([{"type": "entries", "name": "My Section", "entries": ["Content here"]}]),
        );
        assert!(r.contains("<h5>My Section</h5>") && r.contains("<p>Content here</p>"));
    }

    #[test]
    fn lists() {
        let r = entries(&json!([{"type": "list", "items": ["Item 1", "Item 2"]}]));
        assert!(
            r.contains("<ul>") && r.contains("<li>Item 1</li>") && r.contains("<li>Item 2</li>")
        );
    }

    #[test]
    fn tables() {
        let r = entries(
            &json!([{"type": "table", "colLabels": ["Name", "Value"], "rows": [["Sword", "10 gp"]]}]),
        );
        assert!(
            r.contains("<table") && r.contains("<th>Name</th>") && r.contains("<td>Sword</td>")
        );
        // Roll cells, and the row number in an empty dice column.
        let r = table(&json!({
            "colLabels": ["{@dice d6}", "Result"],
            "rows": [["", "A"], [{"type": "cell", "roll": {"min": 2, "max": 5}}, "B"], [{"type": "cell", "roll": {"exact": 6}}, "C"]]
        }));
        assert!(r.contains("<td>1</td>") && r.contains("<td>2-5</td>") && r.contains("<td>6</td>"));
    }

    #[test]
    fn insets_quotes_sections() {
        let r = entries(&json!([{"type": "inset", "name": "Sidebar", "entries": ["Extra info"]}]));
        assert!(r.contains(r#"class="inset-readaloud""#) && r.contains("<h5>Sidebar</h5>"));
        let r = entries(&json!([{"type": "quote", "entries": ["Wise words"], "by": "Gandalf"}]));
        assert!(r.contains("<blockquote>") && r.contains("Wise words") && r.contains("— Gandalf"));
        let r = entries(&json!([{"type": "section", "name": "Chapter 1", "entries": ["Text"]}]));
        assert!(r.contains("<h4>Chapter 1</h4>"));
    }

    #[test]
    fn tags_in_entries() {
        let r = entries(&json!(["Cast {@spell fireball} for {@damage 8d6} damage"]));
        assert!(
            r.contains(r#"class="cross-ref-link spell-ref""#)
                && r.contains(r#"class="damage-roll""#)
        );
    }

    #[test]
    fn not_a_list() {
        assert_eq!(entries(&json!(null)), "");
        assert_eq!(entries(&json!("not an array")), "");
    }

    #[test]
    fn items_and_statblocks() {
        let r = entries(
            &json!([{"type": "item", "name": "Keen Smell.", "entry": "Advantage on {@skill Perception}."}]),
        );
        assert!(r.contains("<strong>Keen Smell</strong>") && r.contains(r#"class="skill""#));
        let r = entries(&json!([{"type": "statblock", "name": "Goblin", "source": "MM"}]));
        assert!(r.contains("creature-ref") && r.contains("Goblin</a>"));
    }
}
