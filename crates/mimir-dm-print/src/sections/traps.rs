//! Trap/Hazard appendix section for PDF export

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Trap appendix section - renders trap and hazard cards
pub struct TrapAppendix {
    /// Trap data (JSON array)
    traps: Value,
}

impl TrapAppendix {
    /// Create from trap JSON data
    ///
    /// Expected format:
    /// ```json
    /// [
    ///   {
    ///     "name": "Trap Name",
    ///     "source": "DMG",
    ///     "trap_haz_type": "MECH",
    ///     "entries": [...]
    ///   }
    /// ]
    /// ```
    pub fn new(traps: Value) -> Self {
        Self { traps }
    }

    /// Check if there are any traps to render
    pub fn is_empty(&self) -> bool {
        self.traps
            .as_array()
            .map(|arr| arr.is_empty())
            .unwrap_or(true)
    }
}

impl Renderable for TrapAppendix {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let traps = match self.traps.as_array() {
            Some(arr) if !arr.is_empty() => arr,
            _ => return Ok("// No trap data\n".to_string()),
        };

        let mut typst = String::new();

        // Section header
        typst.push_str("#pagebreak()\n");
        typst.push_str("#text(size: 16pt, weight: \"bold\")[Traps & Hazards]\n");
        typst.push_str("#v(1em)\n\n");

        for trap in traps {
            typst.push_str(&render_trap_card(trap));
            typst.push_str("\n#v(1em)\n");
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some("Traps & Hazards".to_string())
        }
    }
}

/// Render a single trap card
fn render_trap_card(trap: &Value) -> String {
    let name = trap.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Trap");
    let source = trap.get("source").and_then(|v| v.as_str()).unwrap_or("?");

    // Get trap type (MECH, MAG, WLD, etc.)
    let trap_type = trap
        .get("trap_haz_type")
        .or_else(|| trap.get("trapHazType"))
        .and_then(|v| v.as_str())
        .map(|t| match t {
            "MECH" => "Mechanical Trap",
            "MAG" => "Magical Trap",
            "WLD" => "Wilderness Hazard",
            "WTH" => "Weather Hazard",
            "ENV" => "Environmental Hazard",
            _ => "Trap/Hazard",
        })
        .unwrap_or("Trap/Hazard");

    // Render entries content
    let entries_content = render_entries(trap.get("entries"));

    format!(
        r##"#box(stroke: (paint: rgb("#f59e0b"), thickness: 2pt), inset: 10pt, width: 100%, radius: 4pt)[
  #text(size: 12pt, weight: "bold", fill: rgb("#92400e"))[{}] #h(1fr) #text(size: 9pt, fill: rgb("#78350f"))[{}]
  #v(2pt)
  #text(size: 9pt, style: "italic", fill: rgb("#a16207"))[{}]
  #v(6pt)
  #line(length: 100%, stroke: (paint: rgb("#fbbf24"), thickness: 0.5pt))
  #v(6pt)
  {}
]
"##,
        escape_typst(name),
        escape_typst(source),
        trap_type,
        entries_content
    )
}

/// Render entries array to Typst markup
fn render_entries(entries: Option<&Value>) -> String {
    let entries = match entries {
        Some(Value::Array(arr)) => arr,
        _ => return String::new(),
    };

    let mut result = String::new();

    for entry in entries {
        match entry {
            Value::String(s) => {
                result.push_str(&format!("#text(size: 9pt)[{}]\n#v(4pt)\n", escape_typst(s)));
            }
            Value::Object(obj) => {
                // Handle entry objects
                let entry_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");

                match entry_type {
                    "entries" => {
                        // Named subsection
                        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                            result.push_str(&format!(
                                "#text(size: 10pt, weight: \"bold\", fill: rgb(\"#92400e\"))[{}]\n#v(2pt)\n",
                                escape_typst(name)
                            ));
                        }
                        if let Some(sub_entries) = obj.get("entries") {
                            result.push_str(&render_entries(Some(sub_entries)));
                        }
                    }
                    "list" => {
                        // Bulleted list
                        if let Some(items) = obj.get("items").and_then(|v| v.as_array()) {
                            for item in items {
                                match item {
                                    Value::String(s) => {
                                        result.push_str(&format!(
                                            "#text(size: 9pt)[• {}]\n",
                                            escape_typst(s)
                                        ));
                                    }
                                    Value::Object(item_obj) => {
                                        if let Some(item_name) = item_obj.get("name").and_then(|v| v.as_str()) {
                                            let item_entry = item_obj
                                                .get("entry")
                                                .or_else(|| item_obj.get("entries").and_then(|e| e.as_array().and_then(|a| a.first())))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            result.push_str(&format!(
                                                "#text(size: 9pt)[• *{}.* {}]\n",
                                                escape_typst(item_name),
                                                escape_typst(item_entry)
                                            ));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        result.push_str("#v(4pt)\n");
                    }
                    "table" => {
                        // Table
                        if let Some(caption) = obj.get("caption").and_then(|v| v.as_str()) {
                            result.push_str(&format!(
                                "#text(size: 9pt, weight: \"bold\")[{}]\n#v(2pt)\n",
                                escape_typst(caption)
                            ));
                        }

                        if let (Some(col_labels), Some(rows)) = (
                            obj.get("colLabels").and_then(|v| v.as_array()),
                            obj.get("rows").and_then(|v| v.as_array()),
                        ) {
                            let col_count = col_labels.len();
                            result.push_str(&format!(
                                "#table(columns: {}, stroke: 0.5pt,\n",
                                col_count
                            ));

                            // Header row
                            for label in col_labels {
                                if let Some(l) = label.as_str() {
                                    result.push_str(&format!(
                                        "  table.header([*{}*]),\n",
                                        escape_typst(l)
                                    ));
                                }
                            }

                            // Data rows
                            for row in rows {
                                if let Some(cells) = row.as_array() {
                                    for cell in cells {
                                        let cell_text = format_table_cell(cell);
                                        result.push_str(&format!("  [{}],\n", escape_typst(&cell_text)));
                                    }
                                }
                            }

                            result.push_str(")\n#v(4pt)\n");
                        }
                    }
                    _ => {
                        // Generic object - try to extract entries
                        if let Some(sub_entries) = obj.get("entries") {
                            result.push_str(&render_entries(Some(sub_entries)));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    result
}

/// Format a table cell value
fn format_table_cell(cell: &Value) -> String {
    match cell {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Object(obj) => {
            // Handle roll objects like { "roll": { "exact": 1 } }
            if let Some(roll) = obj.get("roll").and_then(|v| v.as_object()) {
                if let Some(exact) = roll.get("exact") {
                    return format_table_cell(exact);
                }
                if let (Some(min), Some(max)) = (roll.get("min"), roll.get("max")) {
                    return format!("{}-{}", format_table_cell(min), format_table_cell(max));
                }
            }
            String::new()
        }
        _ => String::new(),
    }
}

/// Escape special Typst characters
fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('*', "\\*")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_traps() {
        let appendix = TrapAppendix::new(json!([]));
        assert!(appendix.is_empty());
    }

    #[test]
    fn test_with_traps() {
        let appendix = TrapAppendix::new(json!([
            {
                "name": "Pit Trap",
                "source": "DMG",
                "trap_haz_type": "MECH",
                "entries": ["A simple pit trap."]
            }
        ]));
        assert!(!appendix.is_empty());
    }

    #[test]
    fn test_trap_type_mapping() {
        let trap = json!({
            "name": "Fire Trap",
            "trap_haz_type": "MAG"
        });
        let typst = render_trap_card(&trap);
        assert!(typst.contains("Magical Trap"));
    }
}
