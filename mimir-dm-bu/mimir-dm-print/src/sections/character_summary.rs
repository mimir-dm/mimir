//! Character summary section
//!
//! Generates half-page (8.5" x 5.5") character quick reference cards.
//! Shows key stats, abilities, saves, skills, equipment, and spells/features.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Character summary section - half-page quick reference
pub struct CharacterSummarySection {
    /// Character data (JSON)
    character: Value,
}

impl CharacterSummarySection {
    /// Create a new character summary section
    pub fn new(character: Value) -> Self {
        Self { character }
    }

    /// Get value from character data
    fn get_str(&self, key: &str) -> String {
        self.character
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    fn get_i64(&self, key: &str, default: i64) -> i64 {
        self.character
            .get(key)
            .and_then(|v| v.as_i64())
            .unwrap_or(default)
    }

    /// Calculate proficiency bonus from level
    fn prof_bonus(level: i64) -> i64 {
        if level <= 4 {
            2
        } else if level <= 8 {
            3
        } else if level <= 12 {
            4
        } else if level <= 16 {
            5
        } else {
            6
        }
    }
}

impl Renderable for CharacterSummarySection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let name = self
            .character
            .get("character_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let level = self.get_i64("level", 1);
        let race = self
            .character
            .get("race")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let max_hp = self.get_i64("max_hp", 10);
        let current_hp = self.get_i64("current_hp", 10);
        let speed = self.get_i64("speed", 30);
        let prof = Self::prof_bonus(level);

        // Abilities
        let abilities = self.character.get("abilities");
        let str_score = abilities
            .and_then(|a| a.get("strength"))
            .and_then(|v| v.as_i64())
            .unwrap_or(10);
        let dex_score = abilities
            .and_then(|a| a.get("dexterity"))
            .and_then(|v| v.as_i64())
            .unwrap_or(10);
        let con_score = abilities
            .and_then(|a| a.get("constitution"))
            .and_then(|v| v.as_i64())
            .unwrap_or(10);
        let int_score = abilities
            .and_then(|a| a.get("intelligence"))
            .and_then(|v| v.as_i64())
            .unwrap_or(10);
        let wis_score = abilities
            .and_then(|a| a.get("wisdom"))
            .and_then(|v| v.as_i64())
            .unwrap_or(10);
        let cha_score = abilities
            .and_then(|a| a.get("charisma"))
            .and_then(|v| v.as_i64())
            .unwrap_or(10);

        // AC calculation (simplified)
        let has_shield = self
            .character
            .get("equipped")
            .and_then(|e| e.get("shield"))
            .is_some();
        let dex_mod = (dex_score - 10) / 2;
        let ac = 10 + dex_mod + if has_shield { 2 } else { 0 };

        // Classes
        let classes = self
            .character
            .get("classes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        let class_name = c.get("class_name").and_then(|v| v.as_str())?;
                        let lvl = c.get("level").and_then(|v| v.as_i64()).unwrap_or(0);
                        Some(format!("{} {}", class_name, lvl))
                    })
                    .collect::<Vec<_>>()
                    .join(" / ")
            })
            .unwrap_or_else(|| "No Class".to_string());

        // Proficient saves
        let prof_saves: Vec<String> = self
            .character
            .get("proficiencies")
            .and_then(|p| p.get("saves"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect())
            .unwrap_or_default();

        // Skills
        let prof_skills: Vec<String> = self
            .character
            .get("proficiencies")
            .and_then(|p| p.get("skills"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect())
            .unwrap_or_default();

        // Equipment
        let equipped = self.character.get("equipped");
        let armor = equipped
            .and_then(|e| e.get("armor"))
            .and_then(|v| v.as_str());
        let shield = equipped
            .and_then(|e| e.get("shield"))
            .and_then(|v| v.as_str());
        let main_hand = equipped
            .and_then(|e| e.get("main_hand"))
            .and_then(|v| v.as_str());

        // Spells
        let spells = self.character.get("spells");
        let cantrips: Vec<String> = spells
            .and_then(|s| s.get("cantrips"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect())
            .unwrap_or_default();
        let prepared: Vec<String> = spells
            .and_then(|s| s.get("prepared_spells"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect())
            .unwrap_or_default();
        let known: Vec<String> = spells
            .and_then(|s| s.get("known_spells"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect())
            .unwrap_or_default();
        let has_spells = !cantrips.is_empty() || !prepared.is_empty() || !known.is_empty();

        // Features
        let features: Vec<String> = self
            .character
            .get("class_features")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect())
            .unwrap_or_default();

        let mut typst = String::new();

        // Page setup for half-page
        typst.push_str(
            r#"#set page(
  width: 8.5in,
  height: 5.5in,
  margin: 0.3in,
)
#set text(font: font-body, size: sizes.sm)

"#,
        );

        // Header
        typst.push_str(&format!(
            r#"#grid(
  columns: (1fr, auto),
  [
    #text(size: sizes.xl, weight: "bold")[{}]
    #linebreak()
    #text(size: sizes.md)[Level {} {} {}]
  ],
  grid(
    columns: (auto, auto, auto, auto),
    column-gutter: spacing.md,
    box(
      stroke: 1pt + colors.border,
      inset: spacing.sm,
      radius: 2pt,
      align(center)[
        #label-text("HP")
        #linebreak()
        #text(size: sizes.lg, weight: "bold")[{}/{}]
      ]
    ),
    box(
      stroke: 1pt + colors.border,
      inset: spacing.sm,
      radius: 2pt,
      align(center)[
        #label-text("AC")
        #linebreak()
        #text(size: sizes.lg, weight: "bold")[{}]
      ]
    ),
    box(
      stroke: 1pt + colors.border,
      inset: spacing.sm,
      radius: 2pt,
      align(center)[
        #label-text("Speed")
        #linebreak()
        #text(size: sizes.lg, weight: "bold")[{}]
      ]
    ),
    box(
      stroke: 1pt + colors.border,
      inset: spacing.sm,
      radius: 2pt,
      align(center)[
        #label-text("Prof")
        #linebreak()
        #text(size: sizes.lg, weight: "bold")[+{}]
      ]
    ),
  )
)

#divider-heavy()

"#,
            escape_typst(name),
            level,
            escape_typst(race),
            escape_typst(&classes),
            current_hp,
            max_hp,
            ac,
            speed,
            prof,
        ));

        // Three column layout
        typst.push_str(
            r#"#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: spacing.md,

  // Column 1: Abilities
  [
    #text(weight: "bold")[Abilities]
    #v(spacing.xs)
"#,
        );

        // Abilities
        for (name, score) in [
            ("STR", str_score),
            ("DEX", dex_score),
            ("CON", con_score),
            ("INT", int_score),
            ("WIS", wis_score),
            ("CHA", cha_score),
        ] {
            typst.push_str(&format!(
                "    {} #h(1fr) {} ({})\n    #linebreak()\n",
                name,
                score,
                modifier(score)
            ));
        }

        typst.push_str(
            r#"
    #v(spacing.md)

    #text(weight: "bold")[Saves]
    #v(spacing.xs)
"#,
        );

        // Saves
        for (name, score, save_name) in [
            ("STR", str_score, "Strength"),
            ("DEX", dex_score, "Dexterity"),
            ("CON", con_score, "Constitution"),
            ("INT", int_score, "Intelligence"),
            ("WIS", wis_score, "Wisdom"),
            ("CHA", cha_score, "Charisma"),
        ] {
            let is_prof = prof_saves.iter().any(|s| s == save_name);
            let mod_val = (score - 10) / 2;
            let total = mod_val + if is_prof { prof } else { 0 };
            let sign = if total >= 0 { "+" } else { "" };

            if is_prof {
                typst.push_str(&format!(
                    "    #text(weight: \"bold\")[{}] #h(1fr) {}{}\n    #linebreak()\n",
                    name, sign, total
                ));
            } else {
                typst.push_str(&format!(
                    "    {} #h(1fr) {}{}\n    #linebreak()\n",
                    name, sign, total
                ));
            }
        }

        typst.push_str(
            r#"  ],

  // Column 2: Skills & Equipment
  [
    #text(weight: "bold")[Skills]
    #v(spacing.xs)
"#,
        );

        // Skills
        if prof_skills.is_empty() {
            typst.push_str("    #text(fill: colors.text-secondary)[None]\n");
        } else {
            for skill in &prof_skills {
                typst.push_str(&format!("    - {}\n    #linebreak()\n", escape_typst(skill)));
            }
        }

        typst.push_str(
            r#"
    #v(spacing.md)

    #text(weight: "bold")[Equipment]
    #v(spacing.xs)
"#,
        );

        // Equipment
        if let Some(a) = armor {
            typst.push_str(&format!("    Armor: {} #linebreak()\n", escape_typst(a)));
        }
        if let Some(s) = shield {
            typst.push_str(&format!("    Shield: {} #linebreak()\n", escape_typst(s)));
        }
        if let Some(w) = main_hand {
            typst.push_str(&format!("    Weapon: {} #linebreak()\n", escape_typst(w)));
        }

        typst.push_str("  ],\n\n  // Column 3: Spells or Features\n  [\n");

        // Column 3: Spells or Features
        if has_spells {
            typst.push_str(
                r#"    #text(weight: "bold")[Spellcasting]
    #v(spacing.xs)
"#,
            );

            if !cantrips.is_empty() {
                let display: Vec<_> = cantrips.iter().take(4).map(|s| s.as_str()).collect();
                let more = if cantrips.len() > 4 { "..." } else { "" };
                typst.push_str(&format!(
                    "    Cantrips: {}{}\n    #linebreak()\n",
                    escape_typst(&display.join(", ")),
                    more
                ));
            }

            let spell_list = if !prepared.is_empty() {
                &prepared
            } else {
                &known
            };
            if !spell_list.is_empty() {
                let display: Vec<_> = spell_list.iter().take(6).map(|s| s.as_str()).collect();
                let more = if spell_list.len() > 6 { "..." } else { "" };
                typst.push_str(&format!(
                    "    Spells: {}{}\n",
                    escape_typst(&display.join(", ")),
                    more
                ));
            }
        } else {
            typst.push_str(
                r#"    #text(weight: "bold")[Features]
    #v(spacing.xs)
"#,
            );
            if features.is_empty() {
                typst.push_str("    #text(fill: colors.text-secondary)[None]\n");
            } else {
                for feature in features.iter().take(8) {
                    typst.push_str(&format!("    - {}\n    #linebreak()\n", escape_typst(feature)));
                }
                if features.len() > 8 {
                    typst.push_str(&format!(
                        "    #text(fill: colors.text-secondary)[+ {} more]\n",
                        features.len() - 8
                    ));
                }
            }
        }

        typst.push_str("  ]\n)\n\n");

        // Footer
        typst.push_str(&format!(
            r#"#v(1fr)
#align(center)[
  #small-text[{} - Level {} {} - Generated by Mimir]
]
"#,
            escape_typst(name),
            level,
            escape_typst(&classes)
        ));

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        self.character
            .get("character_name")
            .and_then(|v| v.as_str())
            .map(|s| format!("{} Summary", s))
    }
}

/// Calculate ability modifier
fn modifier(score: i64) -> String {
    let m = (score - 10) / 2;
    if m >= 0 {
        format!("+{}", m)
    } else {
        format!("{}", m)
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
    fn test_character_summary_basic() {
        let char = json!({
            "character_name": "Gandalf",
            "level": 20,
            "race": "Human",
            "max_hp": 100,
            "current_hp": 75
        });
        let section = CharacterSummarySection::new(char);
        assert_eq!(
            section.toc_title(),
            Some("Gandalf Summary".to_string())
        );
    }

    #[test]
    fn test_prof_bonus() {
        assert_eq!(CharacterSummarySection::prof_bonus(1), 2);
        assert_eq!(CharacterSummarySection::prof_bonus(5), 3);
        assert_eq!(CharacterSummarySection::prof_bonus(9), 4);
        assert_eq!(CharacterSummarySection::prof_bonus(13), 5);
        assert_eq!(CharacterSummarySection::prof_bonus(17), 6);
    }

    #[test]
    fn test_modifier() {
        assert_eq!(modifier(10), "+0");
        assert_eq!(modifier(18), "+4");
        assert_eq!(modifier(8), "-1");
    }
}
