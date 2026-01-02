//! Spell cards section
//!
//! Generates printable spell cards using shared Typst components.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Spell cards section - generates multi-up spell cards for printing
pub struct SpellCardsSection {
    /// Spell data (JSON array)
    spells: Vec<Value>,
    /// Show cut lines between cards
    show_cut_lines: bool,
}

impl SpellCardsSection {
    /// Create a new spell cards section
    pub fn new(spells: Vec<Value>) -> Self {
        Self {
            spells,
            show_cut_lines: true,
        }
    }

    /// Create from a JSON value (expects array)
    pub fn from_json(spells: Value) -> Self {
        let spell_vec = spells
            .as_array()
            .map(|arr| arr.clone())
            .unwrap_or_default();
        Self::new(spell_vec)
    }

    /// Set whether to show cut lines
    pub fn with_cut_lines(mut self, show: bool) -> Self {
        self.show_cut_lines = show;
        self
    }

    /// Get level string
    fn level_str(level: i64) -> String {
        match level {
            0 => "Cantrip".to_string(),
            1 => "1st-level".to_string(),
            2 => "2nd-level".to_string(),
            3 => "3rd-level".to_string(),
            n => format!("{}th-level", n),
        }
    }

    /// Render a single spell card
    fn render_card(spell: &Value) -> String {
        let name = spell
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Spell");
        let level = spell.get("level").and_then(|v| v.as_i64()).unwrap_or(0);
        let school = spell
            .get("school")
            .and_then(|v| v.as_str())
            .unwrap_or("Evocation");
        let source = spell
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Casting time - handle different formats
        let casting_time = if let Some(ct) = spell.get("casting_time").and_then(|v| v.as_str()) {
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
        };

        // Range
        let range = if let Some(r) = spell.get("range").and_then(|v| v.as_str()) {
            r.to_string()
        } else if let Some(range_obj) = spell.get("range").and_then(|v| v.as_object()) {
            if let Some(dist) = range_obj.get("distance").and_then(|v| v.as_object()) {
                if let Some(amount) = dist.get("amount").and_then(|v| v.as_i64()) {
                    let dist_type = dist.get("distance_type").and_then(|v| v.as_str()).unwrap_or("ft");
                    format!("{} {}", amount, dist_type)
                } else {
                    dist.get("distance_type").and_then(|v| v.as_str()).unwrap_or("Self").to_string()
                }
            } else {
                range_obj.get("type").and_then(|v| v.as_str()).unwrap_or("Self").to_string()
            }
        } else {
            "Self".to_string()
        };

        // Components
        let components = if let Some(c) = spell.get("components").and_then(|v| v.as_str()) {
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
        };

        // Duration
        let duration = if let Some(d) = spell.get("duration").and_then(|v| v.as_str()) {
            d.to_string()
        } else if let Some(dur_arr) = spell.get("duration").and_then(|v| v.as_array()) {
            if let Some(first) = dur_arr.first() {
                let dur_type = first
                    .get("duration_type")
                    .or_else(|| first.get("type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("instant");
                if dur_type == "instant" {
                    "Instantaneous".to_string()
                } else {
                    dur_type.to_string()
                }
            } else {
                "Instantaneous".to_string()
            }
        } else {
            "Instantaneous".to_string()
        };

        let is_concentration = spell
            .get("concentration")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_ritual = spell.get("ritual").and_then(|v| v.as_bool()).unwrap_or(false);

        // Description - get from description or entries
        let description = if let Some(desc) = spell.get("description").and_then(|v| v.as_str()) {
            desc.to_string()
        } else if let Some(entries) = spell.get("entries").and_then(|v| v.as_array()) {
            entries
                .iter()
                .filter_map(|e| e.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        };

        // Truncate description for card
        let desc_truncated: String = description.chars().take(400).collect();
        let desc_display = if description.len() > 400 {
            format!("{}...", desc_truncated)
        } else {
            desc_truncated
        };

        // Classes
        let classes = spell
            .get("classes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .take(3)
                    .filter_map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        format!(
            r##"box(
  width: 2.5in,
  height: 3.5in,
  stroke: 0.5pt + colors.border,
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // Header
  #block(
    width: 100%,
    fill: colors.background-alt,
    inset: (x: 4pt, y: 3pt),
  )[
    #grid(
      columns: (auto, 1fr),
      column-gutter: 3pt,
      spell-school-icon("{}", size: sizes.xs),
      [
        #text(size: 7pt, weight: "bold")[{}]
        {}
      ]
    )
    #text(size: 5pt, fill: colors.text-secondary)[
      {} #lower("{}"){}
    ]
  ]

  // Stats
  #block(
    width: 100%,
    inset: 4pt,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    #grid(
      columns: (auto, 1fr),
      row-gutter: 1pt,
      [*Cast:*], [{}],
      [*Range:*], [{}],
      [*Comp:*], [{}],
      [*Dur:*], [{}],
    )
  ]

  // Description
  #block(
    width: 100%,
    inset: 4pt,
  )[
    #text(size: 5.5pt)[{}]
  ]

  // Footer
  #place(
    bottom + left,
    block(
      width: 100%,
      fill: colors.background-alt,
      inset: (x: 4pt, y: 2pt),
    )[
      #text(size: 4pt, fill: colors.text-secondary)[
        {}
        #h(1fr)
        {}
      ]
    ]
  )
]"##,
            escape_typst(school),
            escape_typst(name),
            if is_ritual { "#text(size: 5pt)[(R)]" } else { "" },
            Self::level_str(level),
            escape_typst(school),
            if is_concentration { " (C)" } else { "" },
            escape_typst(&casting_time),
            escape_typst(&range),
            escape_typst(&components),
            escape_typst(&duration),
            escape_typst(&desc_display),
            escape_typst(&classes),
            escape_typst(source),
        )
    }
}

impl Renderable for SpellCardsSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.spells.is_empty() {
            return Ok("// No spells to display\n".to_string());
        }

        let mut typst = String::new();
        let cards_per_page = 9;
        let total_pages = (self.spells.len() + cards_per_page - 1) / cards_per_page;

        // Set tight margins to fit 3x3 grid of 2.5in x 3.5in cards
        // 3 × 3.5in = 10.5in height, so need 0.25in top/bottom margins
        typst.push_str("#set page(paper: \"us-letter\", margin: 0.25in)\n");

        for page_num in 0..total_pages {
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, self.spells.len());
            let page_spells = &self.spells[start_idx..end_idx];

            if page_num > 0 {
                typst.push_str("\n#pagebreak()\n");
            }

            // Center the card grid
            typst.push_str("#align(center)[\n  #grid(\n");
            typst.push_str("    columns: (2.5in,) * 3,\n");
            typst.push_str("    rows: (3.5in,) * 3,\n");
            typst.push_str(&format!(
                "    column-gutter: {},\n",
                if self.show_cut_lines { "0pt" } else { "4pt" }
            ));
            typst.push_str(&format!(
                "    row-gutter: {},\n\n",
                if self.show_cut_lines { "0pt" } else { "4pt" }
            ));

            // Render each card in this page
            for (i, spell) in page_spells.iter().enumerate() {
                typst.push_str("    ");
                typst.push_str(&Self::render_card(spell));
                if i < page_spells.len() - 1 || page_spells.len() < 9 {
                    typst.push(',');
                }
                typst.push('\n');
            }

            // Fill remaining slots with empty boxes
            for _ in page_spells.len()..9 {
                typst.push_str("    box(width: 2.5in, height: 3.5in),\n");
            }

            typst.push_str("  )\n]\n");

            // Cut lines indicator
            if self.show_cut_lines && !page_spells.is_empty() {
                typst.push_str("#place(\n  bottom + center,\n  dy: 0.1in,\n");
                typst.push_str("  text(size: 6pt, fill: colors.text-secondary)[Cut along card borders]\n)\n");
            }
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.spells.is_empty() {
            None
        } else {
            Some("Spell Cards".to_string())
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
    fn test_spell_cards_empty() {
        let section = SpellCardsSection::new(vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_spell_cards_with_spells() {
        let spells = vec![json!({
            "name": "Fireball",
            "level": 3,
            "school": "Evocation"
        })];
        let section = SpellCardsSection::new(spells);
        assert_eq!(section.toc_title(), Some("Spell Cards".to_string()));
    }

    #[test]
    fn test_level_str() {
        assert_eq!(SpellCardsSection::level_str(0), "Cantrip");
        assert_eq!(SpellCardsSection::level_str(1), "1st-level");
        assert_eq!(SpellCardsSection::level_str(2), "2nd-level");
        assert_eq!(SpellCardsSection::level_str(3), "3rd-level");
        assert_eq!(SpellCardsSection::level_str(4), "4th-level");
    }

    #[test]
    fn test_escape_typst_special_chars() {
        // Angle brackets (some renderers convert 5etools tags to <damage>, etc.)
        assert_eq!(escape_typst("<damage>"), "\\<damage\\>");
        assert_eq!(escape_typst("<condition>blinded</condition>"), "\\<condition\\>blinded\\</condition\\>");

        // Underscores (subscript markers)
        assert_eq!(escape_typst("fire_damage"), "fire\\_damage");

        // Brackets
        assert_eq!(escape_typst("[test]"), "\\[test\\]");

        // Hash (Typst code mode)
        assert_eq!(escape_typst("#test"), "\\#test");

        // 5etools tags with curly braces and @ symbol
        assert_eq!(escape_typst("{@damage 1d6}"), "\\{\\@damage 1d6\\}");
        assert_eq!(escape_typst("{@condition charmed}"), "\\{\\@condition charmed\\}");

        // Combined with 5etools format
        assert_eq!(
            escape_typst("Deal {@damage 2d6} fire_damage"),
            "Deal \\{\\@damage 2d6\\} fire\\_damage"
        );
    }

    #[test]
    fn test_spell_card_with_5etools_tags() {
        let spells = vec![json!({
            "name": "Acid Splash",
            "level": 0,
            "school": "Conjuration",
            "description": "You hurl a bubble of acid. A target must succeed on a Dexterity saving throw or take {@damage 1d6} acid damage. The {@condition blinded} condition applies on a critical."
        })];

        let section = SpellCardsSection::new(spells);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Verify 5etools tags are escaped
        assert!(typst.contains("\\{\\@damage 1d6\\}"));
        assert!(typst.contains("\\{\\@condition blinded\\}"));
        // Should not contain unescaped curly braces/@ in description
        assert!(!typst.contains("{@damage"));
    }
}
