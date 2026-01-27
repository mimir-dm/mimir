//! Long form character section
//!
//! Extended character details for roleplay: appearance, personality,
//! background, and roleplaying notes.

use crate::builder::{escape_typst_string, RenderContext, Renderable};
use crate::error::Result;
use crate::sections::character::CharacterData;

/// Long form character section - narrative content for roleplay
pub struct CharacterLongFormSection {
    character: CharacterData,
}

impl CharacterLongFormSection {
    pub fn new(character: CharacterData) -> Self {
        Self { character }
    }

    fn build_class_string(&self) -> String {
        if self.character.classes.is_empty() {
            return "No Class".to_string();
        }

        self.character
            .classes
            .iter()
            .map(|c| {
                if let Some(ref sub) = c.subclass_name {
                    format!("{} ({}) {}", c.class_name, sub, c.level)
                } else {
                    format!("{} {}", c.class_name, c.level)
                }
            })
            .collect::<Vec<_>>()
            .join(" / ")
    }
}

impl Renderable for CharacterLongFormSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let c = &self.character;
        let class_str = self.build_class_string();
        let mut typst = String::new();

        // ==== HEADER ====
        typst.push_str(&format!(
            r#"#pagebreak(weak: true)
#text(size: 10pt, fill: luma(100))[CHARACTER LONG FORM]

#grid(
  columns: (1fr, auto),
  column-gutter: 16pt,
  [#text(size: 20pt, weight: "bold")[{}]],
  [#text(size: 12pt)[{}]],
)

#line(length: 100%, stroke: 2pt + colors.accent)
#v(16pt)

"#,
            escape_typst_string(&c.name),
            escape_typst_string(&class_str)
        ));

        // ==== NPC DETAILS SECTION (if NPC) ====
        if c.is_npc {
            typst.push_str(r#"#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 11pt, weight: "bold", fill: colors.accent)[NPC DETAILS]
  #v(8pt)

  #grid(
    columns: (1fr, 1fr, 1fr),
    column-gutter: 12pt,
"#);

            // Role
            typst.push_str(r#"    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 8pt)[
      #text(size: 8pt, fill: luma(100))[ROLE]
      #v(2pt)
"#);
            if let Some(ref role) = c.role {
                typst.push_str(&format!(
                    "      #text(size: 10pt, weight: \"bold\")[{}]\n",
                    escape_typst_string(role)
                ));
            } else {
                typst.push_str("      #text(size: 10pt)[\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_]\n");
            }
            typst.push_str("    ],\n\n");

            // Location
            typst.push_str(r#"    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 8pt)[
      #text(size: 8pt, fill: luma(100))[LOCATION]
      #v(2pt)
"#);
            if let Some(ref location) = c.location {
                typst.push_str(&format!(
                    "      #text(size: 10pt)[{}]\n",
                    escape_typst_string(location)
                ));
            } else {
                typst.push_str("      #text(size: 10pt)[\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_]\n");
            }
            typst.push_str("    ],\n\n");

            // Faction
            typst.push_str(r#"    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 8pt)[
      #text(size: 8pt, fill: luma(100))[FACTION]
      #v(2pt)
"#);
            if let Some(ref faction) = c.faction {
                typst.push_str(&format!(
                    "      #text(size: 10pt)[{}]\n",
                    escape_typst_string(faction)
                ));
            } else {
                typst.push_str("      #text(size: 10pt)[\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_\\_]\n");
            }
            typst.push_str("    ],\n");

            typst.push_str("  )\n]\n\n#v(16pt)\n\n");
        }

        // ==== PERSONALITY SECTION (2x2 grid) ====
        typst.push_str(r#"#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 11pt, weight: "bold", fill: colors.accent)[PERSONALITY]
  #v(8pt)

  #grid(
    columns: (1fr, 1fr),
    column-gutter: 12pt,
    row-gutter: 12pt,

    // Traits
    box(
      width: 100%,
      stroke: 0.5pt + luma(200),
      radius: 2pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[PERSONALITY TRAITS]
      #v(4pt)
"#);

        if let Some(ref traits) = c.traits {
            typst.push_str(&format!(
                "      #text(size: 9pt)[{}]\n",
                escape_typst_string(traits)
            ));
        } else {
            typst.push_str("      #v(40pt)\n");
        }

        typst.push_str(r#"    ],

    // Ideals
    box(
      width: 100%,
      stroke: 0.5pt + luma(200),
      radius: 2pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[IDEALS]
      #v(4pt)
"#);

        if let Some(ref ideals) = c.ideals {
            typst.push_str(&format!(
                "      #text(size: 9pt)[{}]\n",
                escape_typst_string(ideals)
            ));
        } else {
            typst.push_str("      #v(40pt)\n");
        }

        typst.push_str(r#"    ],

    // Bonds
    box(
      width: 100%,
      stroke: 0.5pt + luma(200),
      radius: 2pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[BONDS]
      #v(4pt)
"#);

        if let Some(ref bonds) = c.bonds {
            typst.push_str(&format!(
                "      #text(size: 9pt)[{}]\n",
                escape_typst_string(bonds)
            ));
        } else {
            typst.push_str("      #v(40pt)\n");
        }

        typst.push_str(r#"    ],

    // Flaws
    box(
      width: 100%,
      stroke: 0.5pt + luma(200),
      radius: 2pt,
      inset: 8pt,
    )[
      #text(size: 9pt, weight: "bold")[FLAWS]
      #v(4pt)
"#);

        if let Some(ref flaws) = c.flaws {
            typst.push_str(&format!(
                "      #text(size: 9pt)[{}]\n",
                escape_typst_string(flaws)
            ));
        } else {
            typst.push_str("      #v(40pt)\n");
        }

        typst.push_str("    ],\n  )\n]\n\n#v(16pt)\n\n");

        // ==== BACKGROUND SECTION ====
        typst.push_str(&format!(
            r#"#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 11pt, weight: "bold", fill: colors.accent)[BACKGROUND]
  #v(8pt)

  #grid(
    columns: (1fr, 1fr),
    column-gutter: 16pt,
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 8pt)[
      #text(size: 8pt, fill: luma(100))[BACKGROUND TYPE]
      #v(2pt)
      #text(size: 11pt, weight: "bold")[{}]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 8pt)[
      #text(size: 8pt, fill: luma(100))[RACE]
      #v(2pt)
      #text(size: 10pt)[{}]
    ],
  )
]

#v(16pt)

"#,
            c.background_name
                .as_deref()
                .map(escape_typst_string)
                .unwrap_or_else(|| "Unknown".to_string()),
            c.race_name
                .as_deref()
                .map(escape_typst_string)
                .unwrap_or_else(|| "Unknown".to_string())
        ));

        // ==== EQUIPMENT SECTION ====
        if !c.inventory.is_empty() {
            typst.push_str(r#"#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 11pt, weight: "bold", fill: colors.accent)[EQUIPMENT]
  #v(8pt)

  #grid(
    columns: (1fr, 1fr),
    column-gutter: 16pt,

    // Equipped items
    [
      #text(size: 9pt, weight: "bold")[EQUIPPED]
      #v(4pt)
"#);

            let equipped: Vec<_> = c.inventory.iter().filter(|i| i.equipped).collect();
            if equipped.is_empty() {
                typst.push_str("      #text(size: 9pt, fill: luma(150))[No equipped items]\n");
            } else {
                for item in equipped {
                    let qty = if item.quantity > 1 {
                        format!(" (x{})", item.quantity)
                    } else {
                        String::new()
                    };
                    let attuned = if item.attuned { " \\*" } else { "" };
                    typst.push_str(&format!(
                        "      - {}{}{}\n",
                        escape_typst_string(&item.name),
                        qty,
                        attuned
                    ));
                }
            }

            typst.push_str(r#"    ],

    // Other items
    [
      #text(size: 9pt, weight: "bold")[OTHER ITEMS]
      #v(4pt)
"#);

            let other: Vec<_> = c.inventory.iter().filter(|i| !i.equipped).collect();
            if other.is_empty() {
                typst.push_str("      #text(size: 9pt, fill: luma(150))[No other items]\n");
            } else {
                for item in other.iter().take(10) {
                    let qty = if item.quantity > 1 {
                        format!(" (x{})", item.quantity)
                    } else {
                        String::new()
                    };
                    typst.push_str(&format!(
                        "      - {}{}\n",
                        escape_typst_string(&item.name),
                        qty
                    ));
                }
                if other.len() > 10 {
                    typst.push_str(&format!(
                        "      #text(size: 8pt, fill: luma(150))[...and {} more items]\n",
                        other.len() - 10
                    ));
                }
            }

            typst.push_str("    ],\n  )\n]\n\n#v(16pt)\n\n");
        }

        // ==== NOTES SECTION ====
        typst.push_str(r#"#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 11pt, weight: "bold", fill: colors.accent)[NOTES]
  #v(8pt)

  #box(
    width: 100%,
    stroke: 0.5pt + luma(200),
    radius: 2pt,
    inset: 8pt,
  )[
    #v(80pt)
  ]
]

#v(1fr)
#align(center)[
  #text(size: 8pt, fill: luma(150))[Generated by Mimir]
]
"#);

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        Some(format!("{} - Long Form", self.character.name))
    }

    fn page_break_before(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections::character::{ClassInfo, InventoryItem};

    fn sample_character() -> CharacterData {
        CharacterData {
            name: "Sildar Hallwinter".to_string(),
            player_name: None,
            is_npc: true,
            race_name: Some("Human".to_string()),
            background_name: Some("Soldier".to_string()),
            strength: 14,
            dexterity: 10,
            constitution: 12,
            intelligence: 10,
            wisdom: 12,
            charisma: 14,
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 50,
            pp: 0,
            traits: Some("I face problems head-on with direct action.".to_string()),
            ideals: Some("Honor guides all my decisions.".to_string()),
            bonds: Some("I serve the Lords' Alliance loyally.".to_string()),
            flaws: Some("I trust too easily in official authority.".to_string()),
            role: Some("Retired soldier and Lords' Alliance agent".to_string()),
            location: Some("Phandalin".to_string()),
            faction: Some("Lords' Alliance".to_string()),
            classes: vec![ClassInfo {
                class_name: "Fighter".to_string(),
                level: 5,
                subclass_name: Some("Champion".to_string()),
                is_starting: true,
            }],
            inventory: vec![
                InventoryItem {
                    name: "Longsword".to_string(),
                    quantity: 1,
                    equipped: true,
                    attuned: false,
                },
                InventoryItem {
                    name: "Chain Mail".to_string(),
                    quantity: 1,
                    equipped: true,
                    attuned: false,
                },
            ],
        }
    }

    #[test]
    fn test_long_form_basic() {
        let character = sample_character();
        let section = CharacterLongFormSection::new(character);
        assert_eq!(
            section.toc_title(),
            Some("Sildar Hallwinter - Long Form".to_string())
        );
    }

    #[test]
    fn test_long_form_generates_typst() {
        let character = sample_character();
        let section = CharacterLongFormSection::new(character);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Check for character name
        assert!(typst.contains("Sildar Hallwinter"));
        // Check for section headers
        assert!(typst.contains("PERSONALITY"));
        assert!(typst.contains("BACKGROUND"));
        assert!(typst.contains("NPC DETAILS"));
        // Check for personality content
        assert!(typst.contains("Honor guides"));
    }
}
