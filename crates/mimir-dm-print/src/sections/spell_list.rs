//! Spell list section
//!
//! Generates table-format spell lists grouped by level.
//! Good for quick reference sheets and character spell lists.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Spell list section - table format grouped by spell level
pub struct SpellListSection {
    /// List title
    title: String,
    /// Spell data (JSON array)
    spells: Vec<Value>,
    /// Whether to show the description column
    show_description: bool,
}

impl SpellListSection {
    /// Create a new spell list section
    pub fn new(title: impl Into<String>, spells: Vec<Value>) -> Self {
        Self {
            title: title.into(),
            spells,
            show_description: false,
        }
    }

    /// Create from JSON data
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "title": "Wizard Spell List",
    ///   "spells": [{ spell data... }],
    ///   "show_description": false
    /// }
    /// ```
    pub fn from_json(data: Value) -> Self {
        let title = data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Spell List")
            .to_string();

        let spells = data
            .get("spells")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let show_description = data
            .get("show_description")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Self {
            title,
            spells,
            show_description,
        }
    }

    /// Set whether to show description column
    pub fn with_description(mut self, show: bool) -> Self {
        self.show_description = show;
        self
    }

    /// Get school abbreviation
    fn school_abbrev(school: &str) -> &'static str {
        match school.to_lowercase().as_str() {
            "abjuration" => "Abj",
            "conjuration" => "Con",
            "divination" => "Div",
            "enchantment" => "Enc",
            "evocation" => "Evo",
            "illusion" => "Ill",
            "necromancy" => "Nec",
            "transmutation" => "Tra",
            _ => "???",
        }
    }

    /// Get level display string
    fn level_header(level: i64) -> String {
        match level {
            0 => "Cantrips".to_string(),
            1 => "1st Level".to_string(),
            2 => "2nd Level".to_string(),
            3 => "3rd Level".to_string(),
            n => format!("{}th Level", n),
        }
    }

    /// Extract casting time from spell data
    fn extract_casting_time(spell: &Value) -> String {
        if let Some(ct) = spell.get("casting_time").and_then(|v| v.as_str()) {
            ct.to_string()
        } else if let Some(ct) = spell.get("cast_time").and_then(|v| v.as_str()) {
            ct.to_string()
        } else if let Some(time_arr) = spell.get("time").and_then(|v| v.as_array()) {
            if let Some(first) = time_arr.first() {
                let num = first.get("number").and_then(|v| v.as_i64()).unwrap_or(1);
                let unit = first.get("unit").and_then(|v| v.as_str()).unwrap_or("action");
                format!("{} {}", num, unit)
            } else {
                "1 action".to_string()
            }
        } else {
            "1 action".to_string()
        }
    }

    /// Extract range from spell data
    fn extract_range(spell: &Value) -> String {
        if let Some(r) = spell.get("range").and_then(|v| v.as_str()) {
            r.to_string()
        } else if let Some(range_obj) = spell.get("range").and_then(|v| v.as_object()) {
            if let Some(dist) = range_obj.get("distance").and_then(|v| v.as_object()) {
                if let Some(amount) = dist.get("amount").and_then(|v| v.as_i64()) {
                    let dist_type = dist
                        .get("distance_type")
                        .or_else(|| dist.get("type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("ft");
                    format!("{} {}", amount, dist_type)
                } else {
                    dist.get("distance_type")
                        .or_else(|| dist.get("type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Self")
                        .to_string()
                }
            } else {
                range_obj
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Self")
                    .to_string()
            }
        } else {
            "Self".to_string()
        }
    }

    /// Extract components from spell data
    fn extract_components(spell: &Value) -> String {
        if let Some(c) = spell.get("components").and_then(|v| v.as_str()) {
            c.to_string()
        } else if let Some(comp_obj) = spell.get("components").and_then(|v| v.as_object()) {
            let mut parts = Vec::new();
            if comp_obj.get("v").and_then(|v| v.as_bool()).unwrap_or(false) {
                parts.push("V");
            }
            if comp_obj.get("s").and_then(|v| v.as_bool()).unwrap_or(false) {
                parts.push("S");
            }
            if comp_obj.get("m").is_some() {
                parts.push("M");
            }
            parts.join(", ")
        } else {
            "V, S".to_string()
        }
    }

    /// Extract description (truncated)
    fn extract_description(spell: &Value, max_len: usize) -> String {
        let desc = if let Some(d) = spell.get("description").and_then(|v| v.as_str()) {
            d.to_string()
        } else if let Some(entries) = spell.get("entries").and_then(|v| v.as_array()) {
            entries
                .iter()
                .filter_map(|e| e.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        };

        if desc.len() > max_len {
            format!("{}...", &desc[..max_len])
        } else {
            desc
        }
    }
}

impl Renderable for SpellListSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.spells.is_empty() {
            return Ok("// No spells to display\n".to_string());
        }

        let mut typst = String::new();

        // Title
        typst.push_str(&format!(
            "#align(center)[\n  #title-text(\"{}\")\n]\n\n#v(spacing.md)\n\n",
            escape_typst(&self.title)
        ));

        // Group spells by level
        let mut grouped: BTreeMap<i64, Vec<&Value>> = BTreeMap::new();
        for spell in &self.spells {
            let level = spell.get("level").and_then(|v| v.as_i64()).unwrap_or(0);
            grouped.entry(level).or_default().push(spell);
        }

        // Render each level group
        for (level, level_spells) in grouped {
            // Level header
            typst.push_str(&format!(
                r#"#block(
  width: 100%,
  fill: colors.background-alt,
  inset: spacing.sm,
  above: spacing.md,
  below: spacing.sm,
)[
  #text(weight: "bold")[{}]
  #h(1fr)
  #text(size: sizes.sm, fill: colors.text-secondary)[{} spell{}]
]

"#,
                Self::level_header(level),
                level_spells.len(),
                if level_spells.len() != 1 { "s" } else { "" }
            ));

            // Sort by name
            let mut sorted: Vec<_> = level_spells.clone();
            sorted.sort_by(|a, b| {
                let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                a_name.cmp(b_name)
            });

            // Table columns
            let columns = if self.show_description {
                "(auto, auto, auto, auto, auto, 1fr)"
            } else {
                "(1fr, auto, auto, auto, auto)"
            };

            typst.push_str(&format!(
                r#"#table(
  columns: {},
  stroke: none,
  inset: (x: spacing.sm, y: spacing.xs),
  fill: (_, y) => if calc.rem(y, 2) == 0 {{ colors.background-alt }} else {{ white }},

  // Header row
  [*Name*],
  [*School*],
  [*Time*],
  [*Range*],
  [*Comp*],
{}
"#,
                columns,
                if self.show_description {
                    "  [*Description*],\n"
                } else {
                    ""
                }
            ));

            // Data rows
            for spell in sorted {
                let name = spell.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let school = spell.get("school").and_then(|v| v.as_str()).unwrap_or("?");
                let cast_time = Self::extract_casting_time(spell);
                let range = Self::extract_range(spell);
                let components = Self::extract_components(spell);
                let concentration = spell
                    .get("concentration")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let ritual = spell.get("ritual").and_then(|v| v.as_bool()).unwrap_or(false);

                let indicators = format!(
                    "{}{}",
                    if concentration { " (C)" } else { "" },
                    if ritual { " (R)" } else { "" }
                );

                typst.push_str(&format!(
                    r#"  [#text(weight: "medium")[{}]#text(size: sizes.xs, fill: colors.text-secondary)[{}]],
  text(size: sizes.sm)[{}],
  text(size: sizes.sm)[{}],
  text(size: sizes.sm)[{}],
  text(size: sizes.sm)[{}],
"#,
                    escape_typst(name),
                    indicators,
                    Self::school_abbrev(school),
                    escape_typst(&cast_time),
                    escape_typst(&range),
                    escape_typst(&components),
                ));

                if self.show_description {
                    let desc = Self::extract_description(spell, 100);
                    typst.push_str(&format!(
                        "  text(size: sizes.xs)[{}],\n",
                        escape_typst(&desc)
                    ));
                }
            }

            typst.push_str(")\n\n");
        }

        // Legend
        typst.push_str(
            r#"#v(1fr)

#block(
  width: 100%,
  inset: spacing.sm,
  stroke: (top: 0.5pt + colors.border-light),
)[
  #text(size: sizes.xs, fill: colors.text-secondary)[
    *Legend:* (C) = Concentration, (R) = Ritual |
    *Schools:* Abj = Abjuration, Con = Conjuration, Div = Divination, Enc = Enchantment, Evo = Evocation, Ill = Illusion, Nec = Necromancy, Tra = Transmutation
  ]
]

#align(center)[
  #small-text[Generated by Mimir]
]
"#,
        );

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.spells.is_empty() {
            None
        } else {
            Some(self.title.clone())
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_spell_list_empty() {
        let section = SpellListSection::new("Test List", vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_spell_list_with_spells() {
        let spells = vec![
            json!({"name": "Fireball", "level": 3, "school": "Evocation"}),
            json!({"name": "Magic Missile", "level": 1, "school": "Evocation"}),
        ];
        let section = SpellListSection::new("Wizard Spells", spells);
        assert_eq!(section.toc_title(), Some("Wizard Spells".to_string()));
    }

    #[test]
    fn test_from_json() {
        let data = json!({
            "title": "Cleric Spells",
            "spells": [{"name": "Cure Wounds", "level": 1}],
            "show_description": true
        });
        let section = SpellListSection::from_json(data);
        assert_eq!(section.title, "Cleric Spells");
        assert!(section.show_description);
    }

    #[test]
    fn test_school_abbrev() {
        assert_eq!(SpellListSection::school_abbrev("Evocation"), "Evo");
        assert_eq!(SpellListSection::school_abbrev("Necromancy"), "Nec");
        assert_eq!(SpellListSection::school_abbrev("Unknown"), "???");
    }

    #[test]
    fn test_level_header() {
        assert_eq!(SpellListSection::level_header(0), "Cantrips");
        assert_eq!(SpellListSection::level_header(1), "1st Level");
        assert_eq!(SpellListSection::level_header(2), "2nd Level");
        assert_eq!(SpellListSection::level_header(3), "3rd Level");
        assert_eq!(SpellListSection::level_header(4), "4th Level");
    }
}
