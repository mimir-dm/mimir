//! Trap/Hazard cards section
//!
//! Generates half-page (3.875" x 5.125") trap cards for combat reference.
//! Shows trap name, type, trigger, effect, countermeasures, and full details.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Trap cards section - generates half-page trap reference cards (2x2 layout)
pub struct TrapCardSection {
    /// Trap data (JSON array)
    traps: Vec<Value>,
    /// Show cut lines between cards
    show_cut_lines: bool,
}

impl TrapCardSection {
    /// Create a new trap cards section
    pub fn new(traps: Vec<Value>) -> Self {
        Self {
            traps,
            show_cut_lines: true,
        }
    }

    /// Create from a JSON value (expects array)
    pub fn from_json(traps: Value) -> Self {
        let trap_vec = traps
            .as_array()
            .map(|arr| arr.clone())
            .unwrap_or_default();
        Self::new(trap_vec)
    }

    /// Set whether to show cut lines
    pub fn with_cut_lines(mut self, show: bool) -> Self {
        self.show_cut_lines = show;
        self
    }

    /// Get trap type info (label and accent color)
    fn get_trap_type_info(trap: &Value) -> (&'static str, &'static str) {
        let trap_type = trap
            .get("trapHazType")
            .or_else(|| trap.get("trap_haz_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("TRAP");

        match trap_type.to_uppercase().as_str() {
            "MECH" => ("Mechanical Trap", "rgb(\"#b45309\")"),
            "MAG" => ("Magical Trap", "rgb(\"#7c3aed\")"),
            "WLD" => ("Wilderness Hazard", "rgb(\"#15803d\")"),
            "WTH" => ("Weather Hazard", "rgb(\"#0369a1\")"),
            "ENV" => ("Environmental Hazard", "rgb(\"#64748b\")"),
            "HAZ" => ("Hazard", "rgb(\"#dc2626\")"),
            _ => ("Trap/Hazard", "rgb(\"#b45309\")"),
        }
    }

    /// Render a single trap card (half-page format)
    fn render_card(trap: &Value) -> String {
        let name = trap
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Trap");
        let source = trap
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let (type_label, accent_color) = Self::get_trap_type_info(trap);

        // Threat level (simple, moderate, dangerous, deadly)
        let threat = trap
            .get("threat")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Plan layout and get front entries
        let layout = plan_trap_layout(trap, accent_color);
        let front_entries_content: String = layout
            .front_entries
            .iter()
            .map(|e| e.typst.clone())
            .collect::<Vec<_>>()
            .join("\n\n  ");

        let fold_indicator = if layout.is_foldable {
            " | fold ▶"
        } else {
            ""
        };

        format!(
            r##"box(
  width: 3.875in,
  height: 5.125in,
  stroke: (
    top: 3pt + {accent_color},
    bottom: 3pt + {accent_color},
    left: 0.5pt + colors.border,
    right: 0.5pt + colors.border,
  ),
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // Header
  #block(
    width: 100%,
    fill: rgb("#fef3c7"),
    inset: (x: 6pt, y: 4pt),
  )[
    #grid(
      columns: (1fr, auto),
      [
        #text(size: 10pt, weight: "bold", fill: rgb("#92400e"))[{name}]
        #linebreak()
        #text(size: 7pt, style: "italic", fill: {accent_color})[{type_label}]
      ],
      align(right + horizon)[
        {threat_badge}
      ]
    )
  ]

  // Detection & Disable DCs
  {dc_block}

  // Structured entries
  {front_entries_content}

  // Footer
  #place(
    bottom + left,
    block(
      width: 100%,
      fill: rgb("#fef3c7"),
      inset: (x: 6pt, y: 2pt),
    )[
      #text(size: 5pt, fill: rgb("#92400e"))[{source}{fold_indicator}]
    ]
  )
]"##,
            accent_color = accent_color,
            name = escape_typst(name),
            type_label = type_label,
            threat_badge = if threat.is_empty() {
                String::new()
            } else {
                let (badge_color, badge_text_color) = match threat.to_lowercase().as_str() {
                    "simple" => ("rgb(\"#dcfce7\")", "rgb(\"#166534\")"),
                    "moderate" => ("rgb(\"#fef3c7\")", "rgb(\"#92400e\")"),
                    "dangerous" => ("rgb(\"#fed7aa\")", "rgb(\"#c2410c\")"),
                    "deadly" => ("rgb(\"#fecaca\")", "rgb(\"#991b1b\")"),
                    _ => ("rgb(\"#e5e7eb\")", "rgb(\"#374151\")"),
                };
                format!(
                    r#"#box(
        fill: {},
        inset: (x: 4pt, y: 2pt),
        radius: 2pt,
      )[#text(size: 6pt, weight: "bold", fill: {})[{}]]"#,
                    badge_color,
                    badge_text_color,
                    escape_typst(&threat.to_uppercase())
                )
            },
            dc_block = render_dc_block(trap),
            front_entries_content = front_entries_content,
            source = escape_typst(source),
            fold_indicator = fold_indicator,
        )
    }

    /// Render back card for overflow content
    fn render_back_card(trap: &Value) -> Option<String> {
        let (_, accent_color) = Self::get_trap_type_info(trap);
        let layout = plan_trap_layout(trap, accent_color);

        if !layout.is_foldable {
            return None;
        }

        let name = trap
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Trap");
        let source = trap
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let back_entries_content: String = layout
            .back_entries
            .iter()
            .map(|e| e.typst.clone())
            .collect::<Vec<_>>()
            .join("\n\n  ");

        Some(format!(
            r##"box(
  width: 3.875in,
  height: 5.125in,
  stroke: (
    top: 3pt + {accent_color},
    bottom: 3pt + {accent_color},
    left: 0.5pt + colors.border,
    right: 0.5pt + colors.border,
  ),
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // Header (continued)
  #block(
    width: 100%,
    fill: rgb("#fef3c7"),
    inset: (x: 6pt, y: 4pt),
  )[
    #text(size: 10pt, weight: "bold", fill: rgb("#92400e"))[{name}]
    #h(1fr)
    #text(size: 7pt, fill: rgb("#92400e"))[(continued)]
  ]

  // Back entries
  {back_entries_content}

  // Footer
  #place(
    bottom + left,
    block(
      width: 100%,
      fill: rgb("#fef3c7"),
      inset: (x: 6pt, y: 2pt),
    )[
      #text(size: 5pt, fill: rgb("#92400e"))[◀ fold | {source}]
    ]
  )
]"##,
            accent_color = accent_color,
            name = escape_typst(name),
            back_entries_content = back_entries_content,
            source = escape_typst(source),
        ))
    }
}

impl Renderable for TrapCardSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.traps.is_empty() {
            return Ok(String::new());
        }

        let mut typst = String::new();

        // Set page margins for half-page cards
        typst.push_str("#set page(paper: \"us-letter\", margin: 0.25in)\n");

        // Build list of card units (single cards or front+back pairs that must stay together)
        let mut card_units: Vec<Vec<String>> = Vec::new();
        for trap in &self.traps {
            let front = Self::render_card(trap);
            if let Some(back) = Self::render_back_card(trap) {
                // Foldable: front and back must stay together
                card_units.push(vec![front, back]);
            } else {
                // Single card
                card_units.push(vec![front]);
            }
        }

        // Pack units into pages (4 cards per page), keeping pairs together
        let mut pages: Vec<Vec<String>> = Vec::new();
        let mut current_page: Vec<String> = Vec::new();

        for unit in card_units {
            let unit_size = unit.len();
            let space_left = 4 - current_page.len();

            // If unit won't fit on current page, start a new page
            if unit_size > space_left && !current_page.is_empty() {
                pages.push(current_page);
                current_page = Vec::new();
            }

            // Add all cards from this unit to current page
            for card in unit {
                current_page.push(card);
            }

            // If page is full, start a new one
            if current_page.len() >= 4 {
                pages.push(current_page);
                current_page = Vec::new();
            }
        }

        // Don't forget the last page
        if !current_page.is_empty() {
            pages.push(current_page);
        }

        for (page_num, page_cards) in pages.iter().enumerate() {
            if page_num > 0 {
                typst.push_str("\n#pagebreak()\n");
            }

            // Card grid (2x2) - cards sized to fit with gutters for cutting
            typst.push_str("#grid(\n");
            typst.push_str("    columns: (3.875in,) * 2,\n");
            typst.push_str("    rows: (5.125in,) * 2,\n");
            typst.push_str("    column-gutter: 0.25in,\n");
            typst.push_str("    row-gutter: 0.25in,\n\n");

            // Render each card
            for (i, card) in page_cards.iter().enumerate() {
                typst.push_str("    ");
                typst.push_str(card);
                if i < page_cards.len() - 1 || page_cards.len() < 4 {
                    typst.push(',');
                }
                typst.push('\n');
            }

            // Fill remaining slots with empty boxes
            for _ in page_cards.len()..4 {
                typst.push_str("    box(width: 3.875in, height: 5.125in),\n");
            }

            typst.push_str(")\n");

            // Cut lines indicator
            if self.show_cut_lines && !page_cards.is_empty() {
                typst.push_str("#place(\n  bottom + center,\n  dy: 0.1in,\n");
                typst.push_str(
                    "  text(size: 6pt, fill: colors.text-secondary)[Cut along card borders - fold adjacent cards for extended content]\n)\n",
                );
            }
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.traps.is_empty() {
            None
        } else {
            Some("Trap Cards".to_string())
        }
    }
}

/// A rendered entry block with its character count for layout planning
struct RenderedEntry {
    typst: String,
    char_count: usize,
}

/// Card layout - single or foldable double
struct TrapCardLayout {
    front_entries: Vec<RenderedEntry>,
    back_entries: Vec<RenderedEntry>,
    is_foldable: bool,
}

/// Render trap entries as individual blocks with char counts for layout planning
fn render_trap_entries(trap: &Value, accent_color: &str) -> Vec<RenderedEntry> {
    let entries = match trap.get("entries").and_then(|v| v.as_array()) {
        Some(e) => e,
        None => return Vec::new(),
    };

    let mut rendered = Vec::new();

    for entry in entries {
        if let Some(text) = entry.as_str() {
            // Plain text paragraph
            let cleaned = strip_5etools_tags(text);
            if !cleaned.is_empty() {
                let typst = format!(
                    r#"#block(width: 100%, inset: (x: 6pt, y: 3pt), stroke: (bottom: 0.5pt + colors.border-light))[
    #set text(size: 6.5pt)
    {}
  ]"#,
                    escape_typst(&cleaned)
                );
                rendered.push(RenderedEntry {
                    char_count: cleaned.len(),
                    typst,
                });
            }
        } else if let Some(obj) = entry.as_object() {
            let entry_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let entry_name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");

            match entry_type {
                "entries" | "" if obj.contains_key("entries") => {
                    // Named section with sub-entries
                    let sub_entries = obj.get("entries").and_then(|v| v.as_array());
                    let mut content_parts = Vec::new();

                    if let Some(subs) = sub_entries {
                        for sub in subs {
                            if let Some(text) = sub.as_str() {
                                content_parts.push(strip_5etools_tags(text));
                            }
                        }
                    }

                    if !entry_name.is_empty() || !content_parts.is_empty() {
                        let header = if !entry_name.is_empty() {
                            format!(
                                r#"#text(size: 7pt, weight: "bold", fill: {})[{}]
    #v(2pt)"#,
                                accent_color,
                                escape_typst(entry_name)
                            )
                        } else {
                            String::new()
                        };

                        let content = content_parts.join("\n\n");
                        let typst = format!(
                            r#"#block(
    width: 100%,
    inset: 6pt,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    {}
    #set text(size: 6.5pt)
    {}
  ]"#,
                            header,
                            escape_typst(&content)
                        );
                        rendered.push(RenderedEntry {
                            char_count: entry_name.len() + content.len(),
                            typst,
                        });
                    }
                }
                "list" => {
                    // Bullet list
                    let items = obj.get("items").and_then(|v| v.as_array());
                    if let Some(items) = items {
                        let mut total_chars = entry_name.len();
                        let list_items: Vec<String> = items
                            .iter()
                            .filter_map(|item| {
                                if let Some(text) = item.as_str() {
                                    let cleaned = strip_5etools_tags(text);
                                    total_chars += cleaned.len();
                                    Some(format!("[{}]", escape_typst(&cleaned)))
                                } else if let Some(obj) = item.as_object() {
                                    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                    let item_entries = obj.get("entries").and_then(|v| v.as_array());
                                    let text = item_entries
                                        .map(|e| {
                                            e.iter()
                                                .filter_map(|x| x.as_str())
                                                .map(strip_5etools_tags)
                                                .collect::<Vec<_>>()
                                                .join(" ")
                                        })
                                        .unwrap_or_default();
                                    total_chars += name.len() + text.len();
                                    if !name.is_empty() {
                                        Some(format!("[*{}.* {}]", escape_typst(name), escape_typst(&text)))
                                    } else {
                                        Some(format!("[{}]", escape_typst(&text)))
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if !list_items.is_empty() {
                            let header = if !entry_name.is_empty() {
                                format!(
                                    r#"#text(size: 7pt, weight: "bold", fill: {})[{}]
    #v(2pt)"#,
                                    accent_color,
                                    escape_typst(entry_name)
                                )
                            } else {
                                String::new()
                            };

                            let typst = format!(
                                r#"#block(
    width: 100%,
    inset: 6pt,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    {}
    #set text(size: 6.5pt)
    #list(
      marker: [•],
      indent: 4pt,
      body-indent: 4pt,
      {}
    )
  ]"#,
                                header,
                                list_items.join(",\n      ")
                            );
                            rendered.push(RenderedEntry {
                                char_count: total_chars,
                                typst,
                            });
                        }
                    }
                }
                "table" => {
                    // Table rendering
                    let col_labels = obj.get("colLabels").and_then(|v| v.as_array());
                    let rows = obj.get("rows").and_then(|v| v.as_array());
                    let caption = obj.get("caption").and_then(|v| v.as_str()).unwrap_or("");

                    if let Some(rows) = rows {
                        let num_cols = col_labels.map(|c| c.len()).unwrap_or_else(|| {
                            rows.first()
                                .and_then(|r| r.as_array())
                                .map(|a| a.len())
                                .unwrap_or(2)
                        });

                        let mut total_chars = caption.len();

                        let header_row = col_labels.map(|cols| {
                            let cells: Vec<String> = cols
                                .iter()
                                .filter_map(|c| c.as_str())
                                .map(|s| {
                                    total_chars += s.len();
                                    format!("[*{}*]", escape_typst(s))
                                })
                                .collect();
                            cells.join(", ")
                        });

                        let data_rows: Vec<String> = rows
                            .iter()
                            .filter_map(|row| {
                                row.as_array().map(|cells| {
                                    let formatted: Vec<String> = cells
                                        .iter()
                                        .map(|c| {
                                            let text = if let Some(s) = c.as_str() {
                                                s.to_string()
                                            } else {
                                                c.to_string()
                                            };
                                            let cleaned = strip_5etools_tags(&text);
                                            total_chars += cleaned.len();
                                            format!("[{}]", escape_typst(&cleaned))
                                        })
                                        .collect();
                                    formatted.join(", ")
                                })
                            })
                            .collect();

                        let caption_block = if !caption.is_empty() {
                            format!(
                                r#"#text(size: 7pt, weight: "bold")[{}]
    #v(2pt)"#,
                                escape_typst(caption)
                            )
                        } else {
                            String::new()
                        };

                        let all_rows = if let Some(header) = header_row {
                            let mut r = vec![header];
                            r.extend(data_rows);
                            r
                        } else {
                            data_rows
                        };

                        let typst = format!(
                            r#"#block(
    width: 100%,
    inset: 6pt,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    {}
    #set text(size: 6pt)
    #table(
      columns: {},
      stroke: 0.5pt + colors.border-light,
      inset: 3pt,
      {}
    )
  ]"#,
                            caption_block,
                            num_cols,
                            all_rows.join(",\n      ")
                        );
                        rendered.push(RenderedEntry {
                            char_count: total_chars,
                            typst,
                        });
                    }
                }
                _ => {
                    // Unknown type, try to extract text
                    let text = extract_entry_text_full(obj);
                    let cleaned = strip_5etools_tags(&text);
                    if !cleaned.is_empty() {
                        let typst = format!(
                            r#"#block(width: 100%, inset: (x: 6pt, y: 3pt), stroke: (bottom: 0.5pt + colors.border-light))[
    #set text(size: 6.5pt)
    {}
  ]"#,
                            escape_typst(&cleaned)
                        );
                        rendered.push(RenderedEntry {
                            char_count: cleaned.len(),
                            typst,
                        });
                    }
                }
            }
        }
    }

    rendered
}

/// Plan trap card layout - determine if foldable needed
fn plan_trap_layout(trap: &Value, accent_color: &str) -> TrapCardLayout {
    let entries = render_trap_entries(trap, accent_color);

    // Budget for single card content area (half-page minus header/footer)
    const SINGLE_CARD_BUDGET: usize = 800;

    let total_chars: usize = entries.iter().map(|e| e.char_count).sum();

    // If everything fits, single card
    if total_chars <= SINGLE_CARD_BUDGET {
        return TrapCardLayout {
            front_entries: entries,
            back_entries: Vec::new(),
            is_foldable: false,
        };
    }

    // Split entries between front and back
    let mut front_entries = Vec::new();
    let mut back_entries = Vec::new();
    let mut front_chars = 0;

    for entry in entries {
        if front_chars + entry.char_count <= SINGLE_CARD_BUDGET {
            front_chars += entry.char_count;
            front_entries.push(entry);
        } else {
            back_entries.push(entry);
        }
    }

    let is_foldable = !back_entries.is_empty();

    TrapCardLayout {
        front_entries,
        back_entries,
        is_foldable,
    }
}

/// Render the DC block with detection and disable DCs
fn render_dc_block(trap: &Value) -> String {
    let mut dc_parts = Vec::new();

    // Detection DC
    if let Some(dc) = trap.get("perception_dc").or_else(|| trap.get("stealth")).and_then(|v| v.as_i64()) {
        dc_parts.push(format!("*Detect* DC {}", dc));
    }

    // Disable DC
    if let Some(dc) = trap.get("disable_dc").or_else(|| trap.get("disarm")).and_then(|v| v.as_i64()) {
        dc_parts.push(format!("*Disable* DC {}", dc));
    }

    // Save DC from entries
    if let Some(entries) = trap.get("entries").and_then(|v| v.as_array()) {
        for entry in entries {
            if let Some(text) = entry.as_str() {
                // Look for DC patterns
                if let Some(captures) = extract_dc_from_text(text) {
                    for cap in captures {
                        if !dc_parts.iter().any(|p| p.contains(&cap)) {
                            dc_parts.push(cap);
                        }
                    }
                }
            }
        }
    }

    if dc_parts.is_empty() {
        String::new()
    } else {
        format!(
            r#"#block(
    width: 100%,
    inset: (x: 6pt, y: 4pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 7pt)
    #grid(
      columns: (auto,) * {},
      column-gutter: 16pt,
      {}
    )
  ]"#,
            dc_parts.len().min(4),
            dc_parts.iter().map(|p| format!("[{}]", p)).collect::<Vec<_>>().join(",\n      ")
        )
    }
}

/// Extract DC values from text (e.g., "DC 15 Dexterity saving throw")
fn extract_dc_from_text(text: &str) -> Option<Vec<String>> {
    let re = regex::Regex::new(r"DC (\d+)\s+(\w+)").ok()?;
    let results: Vec<String> = re
        .captures_iter(text)
        .take(2)
        .map(|cap| {
            let dc = cap.get(1).map_or("?", |m| m.as_str());
            let ability = cap.get(2).map_or("", |m| m.as_str());
            format!("*{}* DC {}", abbreviate_ability(ability), dc)
        })
        .collect();

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

fn abbreviate_ability(ability: &str) -> &str {
    match ability.to_lowercase().as_str() {
        "strength" => "Str",
        "dexterity" => "Dex",
        "constitution" => "Con",
        "intelligence" => "Int",
        "wisdom" => "Wis",
        "charisma" => "Cha",
        _ => ability,
    }
}

/// Extract text from an entry object, including nested entries
fn extract_entry_text_full(obj: &serde_json::Map<String, Value>) -> String {
    let mut parts = Vec::new();

    if let Some(entries) = obj.get("entries").and_then(|v| v.as_array()) {
        for entry in entries {
            if let Some(text) = entry.as_str() {
                parts.push(text.to_string());
            } else if let Some(nested_obj) = entry.as_object() {
                // Recursively extract from nested entries
                parts.push(extract_entry_text_full(nested_obj));
            }
        }
    }

    parts.join(" ")
}

/// Strip 5etools formatting tags and convert to plain text
fn strip_5etools_tags(text: &str) -> String {
    let mut result = text.to_string();

    // {@damage XdY+Z} -> XdY+Z
    result = regex_replace(&result, r"\{@damage ([^}]+)\}", "$1");

    // {@dice XdY} -> XdY
    result = regex_replace(&result, r"\{@dice ([^}]+)\}", "$1");

    // {@dc N} -> DC N
    result = regex_replace(&result, r"\{@dc (\d+)\}", "DC $1");

    // {@condition X} -> X
    result = regex_replace(&result, r"\{@condition ([^|}]+)[^}]*\}", "$1");

    // {@skill X} -> X
    result = regex_replace(&result, r"\{@skill ([^|}]+)[^}]*\}", "$1");

    // {@item X} -> X
    result = regex_replace(&result, r"\{@item ([^|}]+)[^}]*\}", "$1");

    // {@spell X} -> X
    result = regex_replace(&result, r"\{@spell ([^|}]+)[^}]*\}", "$1");

    // Generic tag removal
    result = regex_replace(&result, r"\{@\w+ ([^|}]+)[^}]*\}", "$1");

    result
}

fn regex_replace(text: &str, pattern: &str, replacement: &str) -> String {
    if let Ok(re) = regex::Regex::new(pattern) {
        re.replace_all(text, replacement).to_string()
    } else {
        text.to_string()
    }
}

/// Escape special Typst characters
fn escape_typst(s: &str) -> String {
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

    #[test]
    fn test_trap_cards_empty() {
        let section = TrapCardSection::new(vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_trap_cards_with_traps() {
        let traps = vec![json!({
            "name": "Pit Trap",
            "source": "DMG",
            "trapHazType": "MECH"
        })];
        let section = TrapCardSection::new(traps);
        assert_eq!(section.toc_title(), Some("Trap Cards".to_string()));
    }

    #[test]
    fn test_from_json() {
        let data = json!([
            {"name": "Pit Trap", "trapHazType": "MECH"},
            {"name": "Fire Glyph", "trapHazType": "MAG"}
        ]);
        let section = TrapCardSection::from_json(data);
        assert_eq!(section.traps.len(), 2);
    }

    #[test]
    fn test_strip_5etools_tags() {
        assert_eq!(strip_5etools_tags("{@dc 15}"), "DC 15");
        assert_eq!(strip_5etools_tags("{@damage 2d6}"), "2d6");
        assert_eq!(strip_5etools_tags("{@condition poisoned}"), "poisoned");
    }

    #[test]
    fn test_half_page_format() {
        let traps = vec![json!({"name": "Test Trap"})];
        let section = TrapCardSection::new(traps);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Verify half-page dimensions and grid layout
        assert!(typst.contains("width: 3.875in"));
        assert!(typst.contains("height: 5.125in"));
        assert!(typst.contains("columns: (3.875in,) * 2"));
        assert!(typst.contains("#grid("));
    }
}
