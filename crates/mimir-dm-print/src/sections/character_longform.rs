//! Long form character section
//!
//! Extended character details for roleplay: appearance, personality,
//! background, and roleplaying notes.

use mimir_dm_core::models::character::data::CharacterData;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

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
                if let Some(ref sub) = c.subclass {
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

        // Page setup
        typst.push_str(r#"#set page(paper: "us-letter", margin: (x: 0.75in, y: 0.75in))
"#);

        // ==== HEADER ====
        typst.push_str(&format!(
            r#"#text(size: 10pt, fill: luma(100))[CHARACTER LONG FORM]

#grid(
  columns: (1fr, auto),
  column-gutter: 16pt,
  [#text(size: 20pt, weight: "bold")[{}]],
  [#text(size: 12pt)[{}]],
)

#line(length: 100%, stroke: 2pt + colors.accent)
#v(16pt)

"#,
            escape_typst(&c.character_name),
            escape_typst(&class_str)
        ));

        // ==== APPEARANCE SECTION ====
        typst.push_str(r#"#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 11pt, weight: "bold", fill: colors.accent)[APPEARANCE]
  #v(8pt)

  // Physical stats row
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr, 1fr),
    column-gutter: 8pt,
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 6pt)[
      #text(size: 8pt, fill: luma(100))[HEIGHT]
      #v(2pt)
      #text(size: 10pt)[\_\_\_\_\_\_\_]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 6pt)[
      #text(size: 8pt, fill: luma(100))[WEIGHT]
      #v(2pt)
      #text(size: 10pt)[\_\_\_\_\_\_\_]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 6pt)[
      #text(size: 8pt, fill: luma(100))[AGE]
      #v(2pt)
      #text(size: 10pt)[\_\_\_\_\_\_\_]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 6pt)[
      #text(size: 8pt, fill: luma(100))[EYES]
      #v(2pt)
      #text(size: 10pt)[\_\_\_\_\_\_\_]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 6pt)[
      #text(size: 8pt, fill: luma(100))[HAIR]
      #v(2pt)
      #text(size: 10pt)[\_\_\_\_\_\_\_]
    ],
    box(stroke: 0.5pt + luma(200), radius: 2pt, inset: 6pt)[
      #text(size: 8pt, fill: luma(100))[SKIN]
      #v(2pt)
      #text(size: 10pt)[\_\_\_\_\_\_\_]
    ],
  )

  #v(12pt)

  // Physical Description
  #text(size: 9pt, weight: "bold")[PHYSICAL DESCRIPTION]
  #v(4pt)
  #box(
    width: 100%,
    stroke: 0.5pt + luma(200),
    radius: 2pt,
    inset: 8pt,
  )[
    #v(40pt)
  ]

  #v(12pt)

  // Distinctive Features
  #text(size: 9pt, weight: "bold")[DISTINCTIVE FEATURES]
  #v(4pt)
  #box(
    width: 100%,
    stroke: 0.5pt + luma(200),
    radius: 2pt,
    inset: 8pt,
  )[
    #v(24pt)
  ]
]

#v(16pt)

"#);

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

        if let Some(ref traits) = c.personality.traits {
            typst.push_str(&format!(
                "      #text(size: 9pt)[{}]\n",
                escape_typst(traits)
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

        if let Some(ref ideals) = c.personality.ideals {
            typst.push_str(&format!(
                "      #text(size: 9pt)[{}]\n",
                escape_typst(ideals)
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

        if let Some(ref bonds) = c.personality.bonds {
            typst.push_str(&format!(
                "      #text(size: 9pt)[{}]\n",
                escape_typst(bonds)
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

        if let Some(ref flaws) = c.personality.flaws {
            typst.push_str(&format!(
                "      #text(size: 9pt)[{}]\n",
                escape_typst(flaws)
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
      #text(size: 8pt, fill: luma(100))[FEATURE]
      #v(2pt)
      #text(size: 10pt)[\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_]
    ],
  )

  #v(12pt)

  // Backstory
  #text(size: 9pt, weight: "bold")[BACKSTORY]
  #v(4pt)
  #box(
    width: 100%,
    stroke: 0.5pt + luma(200),
    radius: 2pt,
    inset: 8pt,
  )[
"#,
            escape_typst(&c.background)
        ));

        // Backstory is a placeholder - character data doesn't include backstory
        typst.push_str("    #v(80pt)\n");

        typst.push_str("  ]\n]\n\n#v(16pt)\n\n");

        // ==== ROLEPLAYING NOTES SECTION ====
        typst.push_str(r#"#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 11pt, weight: "bold", fill: colors.accent)[ROLEPLAYING NOTES]
  #v(8pt)

  // Voice & Mannerisms
  #text(size: 9pt, weight: "bold")[VOICE & MANNERISMS]
  #v(4pt)
  #box(
    width: 100%,
    stroke: 0.5pt + luma(200),
    radius: 2pt,
    inset: 8pt,
  )[
    #v(24pt)
  ]

  #v(12pt)

  // Key Relationships
  #text(size: 9pt, weight: "bold")[KEY RELATIONSHIPS]
  #v(4pt)
  #box(
    width: 100%,
    stroke: 0.5pt + luma(200),
    radius: 2pt,
    inset: 8pt,
  )[
    #v(24pt)
  ]

  #v(12pt)

  // Character Goals
  #text(size: 9pt, weight: "bold")[CHARACTER GOALS]
  #v(4pt)
  #box(
    width: 100%,
    stroke: 0.5pt + luma(200),
    radius: 2pt,
    inset: 8pt,
  )[
    #v(24pt)
  ]

  #v(12pt)

  // Play Reminders
  #text(size: 9pt, weight: "bold")[PLAY REMINDERS]
  #v(4pt)
  #box(
    width: 100%,
    stroke: 0.5pt + luma(200),
    radius: 2pt,
    inset: 8pt,
  )[
    #text(size: 9pt)[
      - \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
      #linebreak()
      - \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
      #linebreak()
      - \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
      #linebreak()
      - \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
      #linebreak()
      - \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
    ]
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
        Some(format!("{} - Long Form", self.character.character_name))
    }

    fn page_break_before(&self) -> bool {
        true
    }
}

/// Escape special Typst characters
fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mimir_dm_core::models::character::data::{
        AbilityScores, ClassLevel, Currency, EquippedItems, Personality, Proficiencies,
        SpellData as CharacterSpellData,
    };

    fn sample_character() -> CharacterData {
        CharacterData {
            character_name: "Matrim Cauthon".to_string(),
            player_id: Some(1),
            level: 7,
            experience_points: 23000,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-01".to_string(),
            race: "Human".to_string(),
            subrace: None,
            classes: vec![ClassLevel {
                class_name: "Rogue".to_string(),
                level: 7,
                subclass: Some("Swashbuckler".to_string()),
                hit_dice_type: "d8".to_string(),
                hit_dice_remaining: 7,
            }],
            background: "Gambler".to_string(),
            alignment: Some("Chaotic Neutral".to_string()),
            abilities: AbilityScores {
                strength: 10,
                dexterity: 18,
                constitution: 14,
                intelligence: 12,
                wisdom: 8,
                charisma: 16,
            },
            max_hp: 52,
            current_hp: 52,
            speed: 30,
            proficiencies: Proficiencies::default(),
            class_features: vec![],
            feats: vec![],
            spells: CharacterSpellData::default(),
            inventory: vec![],
            currency: Currency::default(),
            equipped: EquippedItems::default(),
            personality: Personality {
                traits: Some("I have a joke for every occasion, especially occasions where humor is inappropriate.".to_string()),
                ideals: Some("People. I'm loyal to my friends, not to any ideals.".to_string()),
                bonds: Some("I owe everything to my mentor - a horrible person who's probably rotting in jail somewhere.".to_string()),
                flaws: Some("I can't resist a pretty face or a game of chance.".to_string()),
            },
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: vec![],
            legendary_action_count: None,
        }
    }

    #[test]
    fn test_long_form_basic() {
        let character = sample_character();
        let section = CharacterLongFormSection::new(character);
        assert_eq!(
            section.toc_title(),
            Some("Matrim Cauthon - Long Form".to_string())
        );
    }

    #[test]
    fn test_long_form_generates_typst() {
        let character = sample_character();
        let section = CharacterLongFormSection::new(character);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Check for character name
        assert!(typst.contains("Matrim Cauthon"));
        // Check for section headers
        assert!(typst.contains("APPEARANCE"));
        assert!(typst.contains("PERSONALITY"));
        assert!(typst.contains("BACKGROUND"));
        assert!(typst.contains("ROLEPLAYING NOTES"));
        // Check for personality content
        assert!(typst.contains("I have a joke"));
    }
}
