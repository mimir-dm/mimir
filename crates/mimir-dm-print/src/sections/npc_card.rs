//! NPC index card section
//!
//! Generates 3" x 5" index cards for NPC roleplay reference.
//! Shows appearance, personality, goals, and secrets.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// NPC index card section - 3x5 roleplay reference
pub struct NpcIndexCardSection {
    /// NPC data (JSON array)
    npcs: Vec<Value>,
    /// Show cut lines between cards
    show_cut_lines: bool,
}

impl NpcIndexCardSection {
    /// Create a new NPC card section
    pub fn new(npcs: Vec<Value>) -> Self {
        Self {
            npcs,
            show_cut_lines: true,
        }
    }

    /// Create from a JSON value (expects array)
    pub fn from_json(npcs: Value) -> Self {
        let npc_vec = npcs
            .as_array()
            .map(|arr| arr.clone())
            .unwrap_or_default();
        Self::new(npc_vec)
    }

    /// Set whether to show cut lines
    pub fn with_cut_lines(mut self, show: bool) -> Self {
        self.show_cut_lines = show;
        self
    }

    /// Render a single NPC card
    fn render_card(npc: &Value) -> String {
        let name = npc
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown NPC");
        let race = npc.get("race").and_then(|v| v.as_str());
        let role = npc.get("role").and_then(|v| v.as_str());
        let occupation = npc.get("occupation").and_then(|v| v.as_str());
        let alignment = npc.get("alignment").and_then(|v| v.as_str());
        let location = npc.get("location").and_then(|v| v.as_str());

        let appearance = npc.get("appearance").and_then(|v| v.as_str());
        let personality = npc.get("personality").and_then(|v| v.as_str());
        let mannerisms = npc.get("mannerisms").and_then(|v| v.as_str());
        let voice = npc.get("voice").and_then(|v| v.as_str());

        let goal = npc.get("goal").and_then(|v| v.as_str());
        let motivation = npc.get("motivation").and_then(|v| v.as_str());
        let bond = npc.get("bond").and_then(|v| v.as_str());
        let flaw = npc.get("flaw").and_then(|v| v.as_str());

        let secret = npc.get("secret").and_then(|v| v.as_str());
        let key_info = npc.get("key_info").and_then(|v| v.as_str());

        // Build subtitle
        let mut subtitle_parts = Vec::new();
        if let Some(r) = race {
            subtitle_parts.push(r);
        }
        if let Some(o) = occupation {
            subtitle_parts.push(o);
        }
        if let Some(r) = role {
            subtitle_parts.push(r);
        }

        let mut typst = String::new();

        typst.push_str(&format!(
            r##"box(
  width: 5in,
  height: 3in,
  stroke: 1pt + colors.border,
  radius: 4pt,
  clip: true,
)[
  // Header
  #block(
    width: 100%,
    fill: colors.background-alt,
    inset: (x: spacing.sm, y: spacing.xs),
  )[
    #grid(
      columns: (1fr, auto),
      [
        #text(size: sizes.md, weight: "bold")[{}]
        {}
      ],
      align(right)[
        {}
      ]
    )
  ]

  // Main content - two columns
  #block(
    width: 100%,
    inset: spacing.xs,
  )[
    #grid(
      columns: (1fr, 1fr),
      column-gutter: spacing.sm,

      // Left column: Appearance & Personality
      [
"##,
            escape_typst(name),
            if !subtitle_parts.is_empty() {
                format!(
                    "\n        #linebreak()\n        #text(size: sizes.xs, fill: colors.text-secondary)[{}]",
                    escape_typst(&subtitle_parts.join(", "))
                )
            } else {
                String::new()
            },
            if let Some(a) = alignment {
                format!(
                    "#text(size: sizes.xs, fill: colors.text-secondary)[{}]",
                    escape_typst(a)
                )
            } else {
                String::new()
            },
        ));

        // Left column fields
        if let Some(a) = appearance {
            typst.push_str(&format!(
                r#"        #text(size: 6pt)[
          #text(weight: "bold")[Appearance: ]
          {}
        ]
        #v(2pt)
"#,
                escape_typst(a)
            ));
        }
        if let Some(p) = personality {
            typst.push_str(&format!(
                r#"        #text(size: 6pt)[
          #text(weight: "bold")[Personality: ]
          {}
        ]
        #v(2pt)
"#,
                escape_typst(p)
            ));
        }
        if let Some(m) = mannerisms {
            typst.push_str(&format!(
                r#"        #text(size: 6pt)[
          #text(weight: "bold")[Mannerisms: ]
          {}
        ]
        #v(2pt)
"#,
                escape_typst(m)
            ));
        }
        if let Some(v) = voice {
            typst.push_str(&format!(
                r#"        #text(size: 6pt)[
          #text(weight: "bold")[Voice: ]
          {}
        ]
"#,
                escape_typst(v)
            ));
        }

        typst.push_str(
            r#"      ],

      // Right column: Goals & Info
      [
"#,
        );

        // Right column fields
        if let Some(g) = goal {
            typst.push_str(&format!(
                r#"        #text(size: 6pt)[
          #text(weight: "bold")[Goal: ]
          {}
        ]
        #v(2pt)
"#,
                escape_typst(g)
            ));
        }
        if let Some(m) = motivation {
            typst.push_str(&format!(
                r#"        #text(size: 6pt)[
          #text(weight: "bold")[Motivation: ]
          {}
        ]
        #v(2pt)
"#,
                escape_typst(m)
            ));
        }
        if let Some(b) = bond {
            typst.push_str(&format!(
                r#"        #text(size: 6pt)[
          #text(weight: "bold")[Bond: ]
          {}
        ]
        #v(2pt)
"#,
                escape_typst(b)
            ));
        }
        if let Some(f) = flaw {
            typst.push_str(&format!(
                r#"        #text(size: 6pt)[
          #text(weight: "bold")[Flaw: ]
          {}
        ]
"#,
                escape_typst(f)
            ));
        }

        typst.push_str("      ],\n    )\n  ]\n\n");

        // Key info section
        if let Some(k) = key_info {
            typst.push_str(&format!(
                r#"  #block(
    width: 100%,
    inset: (x: spacing.xs, y: 2pt),
    stroke: (top: 0.5pt + colors.border-light),
  )[
    #text(size: 6pt)[
      #text(weight: "bold")[Key Info: ]
      {}
    ]
  ]

"#,
                escape_typst(k)
            ));
        }

        // Secret or location footer
        if let Some(s) = secret {
            typst.push_str(&format!(
                r##"  #place(
    bottom + left,
    block(
      width: 100%,
      fill: rgb("#fef2f2"),
      stroke: (top: 1pt + rgb("#dc2626")),
      inset: (x: spacing.xs, y: 2pt),
    )[
      #text(size: 5pt, fill: rgb("#dc2626"))[
        #text(weight: "bold")[SECRET: ]
        {}
      ]
    ]
  )
"##,
                escape_typst(s)
            ));
        } else if let Some(l) = location {
            typst.push_str(&format!(
                r#"  #place(
    bottom + left,
    block(
      width: 100%,
      fill: colors.background-alt,
      inset: (x: spacing.xs, y: 2pt),
    )[
      #text(size: 5pt, fill: colors.text-secondary)[
        #text(weight: "bold")[Location: ]
        {}
      ]
    ]
  )
"#,
                escape_typst(l)
            ));
        }

        typst.push_str("]");
        typst
    }
}

impl Renderable for NpcIndexCardSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.npcs.is_empty() {
            return Ok("// No NPCs to display\n".to_string());
        }

        let mut typst = String::new();

        // 3x5 cards - 2 per page (landscape orientation fits 2 side by side)
        let cards_per_page = 2;
        let total_pages = (self.npcs.len() + cards_per_page - 1) / cards_per_page;

        for page_num in 0..total_pages {
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, self.npcs.len());
            let page_npcs = &self.npcs[start_idx..end_idx];

            if page_num > 0 {
                typst.push_str("\n#pagebreak()\n");
            }

            // Center the card grid
            typst.push_str("#align(center)[\n  #grid(\n");
            typst.push_str("    columns: (5in,) * 2,\n");
            typst.push_str("    rows: (3in,),\n");
            typst.push_str(&format!(
                "    column-gutter: {},\n\n",
                if self.show_cut_lines { "0pt" } else { "8pt" }
            ));

            // Render each card
            for (i, npc) in page_npcs.iter().enumerate() {
                typst.push_str("    ");
                typst.push_str(&Self::render_card(npc));
                if i < page_npcs.len() - 1 || page_npcs.len() < 2 {
                    typst.push(',');
                }
                typst.push('\n');
            }

            // Fill remaining slot
            if page_npcs.len() < 2 {
                typst.push_str("    box(width: 5in, height: 3in),\n");
            }

            typst.push_str("  )\n]\n");

            // Cut lines indicator
            if self.show_cut_lines && !page_npcs.is_empty() {
                typst.push_str("#place(\n  bottom + center,\n  dy: 0.1in,\n");
                typst.push_str(
                    "  text(size: 6pt, fill: colors.text-secondary)[Cut along card borders]\n)\n",
                );
            }
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.npcs.is_empty() {
            None
        } else {
            Some("NPC Cards".to_string())
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
    fn test_npc_cards_empty() {
        let section = NpcIndexCardSection::new(vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_npc_cards_with_npcs() {
        let npcs = vec![json!({
            "name": "Bartender Bob",
            "race": "Human",
            "occupation": "Innkeeper",
            "personality": "Friendly but suspicious"
        })];
        let section = NpcIndexCardSection::new(npcs);
        assert_eq!(section.toc_title(), Some("NPC Cards".to_string()));
    }

    #[test]
    fn test_from_json() {
        let data = json!([
            {"name": "Alice", "role": "Merchant"},
            {"name": "Bob", "role": "Guard"}
        ]);
        let section = NpcIndexCardSection::from_json(data);
        assert_eq!(section.npcs.len(), 2);
    }

    #[test]
    fn test_with_cut_lines() {
        let section = NpcIndexCardSection::new(vec![]).with_cut_lines(false);
        assert!(!section.show_cut_lines);
    }
}
