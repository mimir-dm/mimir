//! Spell cards section
//!
//! Generates printable spell cards using shared Typst components.
//! Long descriptions are split at natural boundaries (paragraph, sentence,
//! comma, word) and produce foldable continuation cards.

use serde_json::Value;

use super::card_utils::{escape_typst, flatten_entries, split_text_natural, SMALL_CARD_DESC_BUDGET};
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

    /// Get school icon function name
    fn school_icon(school: &str) -> &'static str {
        match school.to_uppercase().as_str() {
            "A" | "ABJURATION" => "abjuration-icon",
            "C" | "CONJURATION" => "conjuration-icon",
            "D" | "DIVINATION" => "divination-icon",
            "E" | "ENCHANTMENT" => "enchantment-icon",
            "V" | "EVOCATION" => "evocation-icon",
            "I" | "ILLUSION" => "illusion-icon",
            "N" | "NECROMANCY" => "necromancy-icon",
            "T" | "TRANSMUTATION" => "transmutation-icon",
            _ => "magic-icon",
        }
    }

    /// Get school full name
    fn school_name(school: &str) -> &'static str {
        match school.to_uppercase().as_str() {
            "A" | "ABJURATION" => "Abjuration",
            "C" | "CONJURATION" => "Conjuration",
            "D" | "DIVINATION" => "Divination",
            "E" | "ENCHANTMENT" => "Enchantment",
            "V" | "EVOCATION" => "Evocation",
            "I" | "ILLUSION" => "Illusion",
            "N" | "NECROMANCY" => "Necromancy",
            "T" | "TRANSMUTATION" => "Transmutation",
            _ => "Unknown",
        }
    }

    /// Extract common spell fields for card rendering
    fn extract_fields(spell: &Value) -> SpellFields {
        let name = spell
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Spell");
        let level = spell.get("level").and_then(|v| v.as_i64()).unwrap_or(0);
        let school = spell
            .get("school")
            .and_then(|v| v.as_str())
            .unwrap_or("V");
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
                    let dist_type = dist.get("type").and_then(|v| v.as_str()).unwrap_or("feet");
                    format!("{} {}", amount, dist_type)
                } else {
                    dist.get("type")
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
        };

        // Components
        let components = if let Some(c) = spell.get("components").and_then(|v| v.as_str()) {
            c.to_string()
        } else if let Some(comp_obj) = spell.get("components").and_then(|v| v.as_object()) {
            let mut parts = Vec::new();
            if comp_obj
                .get("v")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                parts.push("V");
            }
            if comp_obj
                .get("s")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
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
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("instant");
                if dur_type == "instant" {
                    "Instantaneous".to_string()
                } else if dur_type == "timed" {
                    let amount = first
                        .get("duration")
                        .and_then(|d| d.get("amount"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1);
                    let unit = first
                        .get("duration")
                        .and_then(|d| d.get("type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("minute");
                    format!(
                        "{} {}{}",
                        amount,
                        unit,
                        if amount > 1 { "s" } else { "" }
                    )
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
        let is_ritual = spell
            .get("ritual")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Description - get from description or entries
        let desc_text = if let Some(desc) = spell.get("description").and_then(|v| v.as_str()) {
            desc.to_string()
        } else if let Some(entries) = spell.get("entries").and_then(|v| v.as_array()) {
            flatten_entries(entries)
        } else {
            String::new()
        };

        // Classes
        let classes = spell
            .get("classes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .take(3)
                    .filter_map(|c| {
                        if c.is_string() {
                            c.as_str().map(String::from)
                        } else {
                            c.get("name").and_then(|n| n.as_str()).map(String::from)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        SpellFields {
            name: name.to_string(),
            source: source.to_string(),
            icon: Self::school_icon(school),
            school_full: Self::school_name(school),
            level_text: Self::level_str(level),
            ritual_marker: if is_ritual {
                "#text(size: 5pt)[(R)]"
            } else {
                ""
            },
            conc_marker: if is_concentration { " (C)" } else { "" },
            casting_time,
            range,
            components,
            duration,
            classes,
            desc_text,
        }
    }

    /// Render a spell as front card + optional continuation card
    fn render_cards(spell: &Value) -> (String, Option<String>) {
        let f = Self::extract_fields(spell);
        let split = split_text_natural(&f.desc_text, SMALL_CARD_DESC_BUDGET);

        let fold_indicator = if split.is_foldable {
            " ▶ continued"
        } else {
            ""
        };

        let front = format!(
            r#"box(
  width: 2.5in,
  height: 3.25in,
  stroke: 0.5pt + luma(180),
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // Header
  #block(
    width: 100%,
    fill: luma(245),
    inset: (x: 4pt, y: 3pt),
  )[
    #grid(
      columns: (auto, 1fr),
      column-gutter: 3pt,
      {icon}(size: sizes.xs),
      [
        #text(size: 7pt, weight: "bold")[{name}]
        {ritual_marker}
      ]
    )
    #text(size: 5pt, fill: luma(100))[
      {level_text} #lower("{school_full}"){conc_marker}
    ]
  ]

  // Stats
  #block(
    width: 100%,
    inset: 4pt,
    stroke: (bottom: 0.5pt + luma(220)),
  )[
    #set text(size: 6pt)
    #grid(
      columns: (auto, 1fr),
      row-gutter: 1pt,
      [*Cast:*], [{casting_time}],
      [*Range:*], [{range}],
      [*Comp:*], [{components}],
      [*Dur:*], [{duration}],
    )
  ]

  // Description
  #block(
    width: 100%,
    inset: 4pt,
  )[
    #text(size: 5.5pt)[{desc_display}]
  ]

  // Footer
  #place(
    bottom + left,
    block(
      width: 100%,
      fill: luma(245),
      inset: (x: 4pt, y: 2pt),
    )[
      #text(size: 4pt, fill: luma(100))[
        {classes}
        #h(1fr)
        {source}{fold_indicator}
      ]
    ]
  )
]"#,
            icon = f.icon,
            name = escape_typst(&f.name),
            ritual_marker = f.ritual_marker,
            level_text = f.level_text,
            school_full = f.school_full,
            conc_marker = f.conc_marker,
            casting_time = escape_typst(&f.casting_time),
            range = escape_typst(&f.range),
            components = escape_typst(&f.components),
            duration = escape_typst(&f.duration),
            desc_display = escape_typst(&split.front),
            classes = escape_typst(&f.classes),
            source = escape_typst(&f.source),
            fold_indicator = fold_indicator,
        );

        let back = if split.is_foldable {
            Some(format!(
                r#"box(
  width: 2.5in,
  height: 3.25in,
  stroke: 0.5pt + luma(180),
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // Header - continuation
  #block(
    width: 100%,
    fill: luma(245),
    inset: (x: 4pt, y: 3pt),
  )[
    #grid(
      columns: (auto, 1fr),
      column-gutter: 3pt,
      {icon}(size: sizes.xs),
      [
        #text(size: 7pt, weight: "bold")[{name}]
        #h(1fr)
        #text(size: 5pt, style: "italic", fill: luma(100))[(continued)]
      ]
    )
    #text(size: 5pt, fill: luma(100))[
      {level_text} #lower("{school_full}")
    ]
  ]

  // Description continued
  #block(
    width: 100%,
    inset: 4pt,
  )[
    #text(size: 5.5pt)[{desc_continued}]
  ]

  // Footer
  #place(
    bottom + left,
    block(
      width: 100%,
      fill: luma(245),
      inset: (x: 4pt, y: 2pt),
    )[
      #text(size: 4pt, fill: luma(100))[
        ◀ fold
        #h(1fr)
        {source}
      ]
    ]
  )
]"#,
                icon = f.icon,
                name = escape_typst(&f.name),
                level_text = f.level_text,
                school_full = f.school_full,
                desc_continued = escape_typst(&split.back),
                source = escape_typst(&f.source),
            ))
        } else {
            None
        };

        (front, back)
    }

}

impl Renderable for SpellCardsSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.spells.is_empty() {
            return Ok("// No spells to display\n".to_string());
        }

        let mut typst = String::new();

        // Spell school icons
        typst.push_str(
            r#"// Spell school icons
#let abjuration-icon(size: 12pt) = text(size: size)[🛡]
#let conjuration-icon(size: 12pt) = text(size: size)[✨]
#let divination-icon(size: 12pt) = text(size: size)[👁]
#let enchantment-icon(size: 12pt) = text(size: size)[💫]
#let evocation-icon(size: 12pt) = text(size: size)[🔥]
#let illusion-icon(size: 12pt) = text(size: size)[🌀]
#let necromancy-icon(size: 12pt) = text(size: size)[💀]
#let transmutation-icon(size: 12pt) = text(size: size)[⚗]
#let magic-icon(size: 12pt) = text(size: size)[✦]

"#,
        );

        // Set page margins for spell cards (centered with gutters for cutting)
        typst.push_str("#set page(paper: \"us-letter\", margin: 0.25in)\n");

        // Pre-render all cards, collecting front + continuation cards into a flat list.
        // Continuation cards are placed immediately after their front card so they
        // end up adjacent in the grid (easy to fold together after cutting).
        let mut all_cards: Vec<String> = Vec::new();
        for spell in &self.spells {
            let (front, back) = Self::render_cards(spell);
            all_cards.push(front);
            if let Some(back_card) = back {
                all_cards.push(back_card);
            }
        }

        let cards_per_page = 9;
        let total_pages = (all_cards.len() + cards_per_page - 1) / cards_per_page;
        let has_foldable = all_cards.len() > self.spells.len();

        for page_num in 0..total_pages {
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, all_cards.len());
            let page_cards = &all_cards[start_idx..end_idx];

            if page_num > 0 {
                typst.push_str("\n#pagebreak()\n");
            }

            // Card grid (3x3) - cards sized to fit with gutters for cutting
            // align(center) centers the grid on the page; set align(left) inside
            // to prevent center alignment from cascading into card text
            typst.push_str("#align(center)[#align(left)[\n  #grid(\n");
            typst.push_str("    columns: (2.5in,) * 3,\n");
            typst.push_str("    rows: (3.25in,) * 3,\n");
            typst.push_str("    column-gutter: 0.25in,\n");
            typst.push_str("    row-gutter: 0.25in,\n\n");

            for (i, card) in page_cards.iter().enumerate() {
                typst.push_str("    ");
                typst.push_str(card);
                if i < page_cards.len() - 1 || page_cards.len() < 9 {
                    typst.push(',');
                }
                typst.push('\n');
            }

            // Fill remaining slots with empty boxes
            for _ in page_cards.len()..9 {
                typst.push_str("    box(width: 2.5in, height: 3.25in),\n");
            }

            typst.push_str("  )\n]]\n");

            // Cut lines indicator
            if self.show_cut_lines && !page_cards.is_empty() {
                let hint = if has_foldable {
                    "Cut along card borders — fold adjacent cards for extended descriptions"
                } else {
                    "Cut along card borders"
                };
                typst.push_str(&format!(
                    "#place(\n  bottom + center,\n  dy: 0.1in,\n  text(size: 6pt, fill: luma(150))[{}]\n)\n",
                    hint
                ));
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

    fn page_break_before(&self) -> bool {
        true
    }
}

/// Extracted spell fields for card rendering
struct SpellFields {
    name: String,
    source: String,
    icon: &'static str,
    school_full: &'static str,
    level_text: String,
    ritual_marker: &'static str,
    conc_marker: &'static str,
    casting_time: String,
    range: String,
    components: String,
    duration: String,
    classes: String,
    desc_text: String,
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
            "school": "V"
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
        assert_eq!(escape_typst("<damage>"), "\\<damage\\>");
        assert_eq!(escape_typst("fire_damage"), "fire\\_damage");
        assert_eq!(escape_typst("[test]"), "\\[test\\]");
        assert_eq!(escape_typst("#test"), "\\#test");
        assert_eq!(escape_typst("{@damage 1d6}"), "\\{\\@damage 1d6\\}");
    }

    #[test]
    fn test_school_icon() {
        assert_eq!(SpellCardsSection::school_icon("V"), "evocation-icon");
        assert_eq!(SpellCardsSection::school_icon("A"), "abjuration-icon");
        assert_eq!(
            SpellCardsSection::school_icon("EVOCATION"),
            "evocation-icon"
        );
    }

    #[test]
    fn test_school_name() {
        assert_eq!(SpellCardsSection::school_name("V"), "Evocation");
        assert_eq!(SpellCardsSection::school_name("A"), "Abjuration");
        assert_eq!(SpellCardsSection::school_name("N"), "Necromancy");
    }

    #[test]
    fn test_short_spell_no_fold() {
        let spell = json!({
            "name": "Fire Bolt",
            "level": 0,
            "school": "V",
            "entries": ["You hurl a mote of fire at a creature."]
        });
        let (front, back) = SpellCardsSection::render_cards(&spell);
        assert!(front.contains("Fire Bolt"));
        assert!(front.contains("hurl a mote"));
        assert!(back.is_none());
    }

    #[test]
    fn test_long_spell_foldable() {
        let long_desc = "This spell creates a massive effect. ".repeat(30);
        let spell = json!({
            "name": "Wish",
            "level": 9,
            "school": "C",
            "entries": [long_desc],
            "source": "PHB"
        });
        let (front, back) = SpellCardsSection::render_cards(&spell);
        assert!(front.contains("continued"));
        let back = back.expect("should have continuation card");
        assert!(back.contains("continued"));
        assert!(back.contains("fold"));
        assert!(back.contains("Wish"));
    }

    #[test]
    fn test_foldable_cards_in_grid() {
        let long_desc = "This spell does something. ".repeat(40);
        let spells = vec![json!({
            "name": "Long Spell",
            "level": 5,
            "school": "V",
            "entries": [long_desc]
        })];
        let section = SpellCardsSection::new(spells);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Should have fold hint in cut lines
        assert!(typst.contains("fold adjacent cards"));
        // Front + back = 2 rendered cards, plus 7 empty slots = 9 total
    }
}
