//! Encounter section
//!
//! Generates encounter reference sheets with multiple compact stat blocks,
//! an encounter title, and optional DM notes.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Encounter section - compact stat blocks for combat reference
pub struct EncounterSection {
    /// Encounter title
    title: String,
    /// Monster data (JSON array)
    monsters: Vec<Value>,
    /// Optional DM notes
    notes: Option<String>,
}

impl EncounterSection {
    /// Create a new encounter section
    pub fn new(title: impl Into<String>, monsters: Vec<Value>) -> Self {
        Self {
            title: title.into(),
            monsters,
            notes: None,
        }
    }

    /// Create from JSON data
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "title": "Encounter Name",
    ///   "monsters": [{ monster data... }],
    ///   "notes": "Optional DM notes"
    /// }
    /// ```
    pub fn from_json(data: Value) -> Self {
        let title = data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Encounter")
            .to_string();

        let monsters = data
            .get("monsters")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let notes = data
            .get("notes")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            title,
            monsters,
            notes,
        }
    }

    /// Set DM notes
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Render a compact stat block for encounter reference
    fn render_compact_stat_block(monster: &Value) -> String {
        let name = monster
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let source = monster
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Size
        let size = monster
            .get("size")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("M");
        let size_name = match size.to_uppercase().as_str() {
            "T" => "Tiny",
            "S" => "Small",
            "M" => "Medium",
            "L" => "Large",
            "H" => "Huge",
            "G" => "Gargantuan",
            _ => "Medium",
        };

        // Creature type
        let creature_type = monster
            .get("creature_type")
            .map(|v| {
                if let Some(s) = v.as_str() {
                    s.to_string()
                } else if let Some(obj) = v.as_object() {
                    obj.get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("creature")
                        .to_string()
                } else {
                    "creature".to_string()
                }
            })
            .unwrap_or_else(|| "creature".to_string());

        // Alignment
        let alignment = extract_alignment(monster);

        // AC
        let ac = extract_ac(monster);

        // HP
        let hp = extract_hp(monster);

        // Speed
        let speed = extract_speed(monster);

        // Passive perception
        let passive = monster
            .get("passive")
            .and_then(|v| v.as_i64())
            .unwrap_or(10);

        // Saves
        let saves = extract_saves(monster);

        // Ability scores
        let str_score = monster.get("str").and_then(|v| v.as_i64()).unwrap_or(10);
        let dex_score = monster.get("dex").and_then(|v| v.as_i64()).unwrap_or(10);
        let con_score = monster.get("con").and_then(|v| v.as_i64()).unwrap_or(10);
        let int_score = monster.get("int").and_then(|v| v.as_i64()).unwrap_or(10);
        let wis_score = monster.get("wis").and_then(|v| v.as_i64()).unwrap_or(10);
        let cha_score = monster.get("cha").and_then(|v| v.as_i64()).unwrap_or(10);

        // CR
        let cr = extract_cr(monster);

        // Key actions (first 3)
        let key_actions: Vec<String> = monster
            .get("action")
            .and_then(|v| v.as_array())
            .map(|actions| {
                actions
                    .iter()
                    .take(3)
                    .filter_map(|a| a.get("name").and_then(|n| n.as_str()))
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        format!(
            r##"#box(
  width: 100%,
  stroke: (
    top: 2pt + colors.accent,
    bottom: 2pt + colors.accent,
  ),
  inset: 0pt,
)[
  // Header
  #block(
    width: 100%,
    fill: colors.background-alt,
    inset: spacing.sm,
  )[
    #grid(
      columns: (1fr, auto),
      [
        #text(size: sizes.md, weight: "bold")[{}]
        #linebreak()
        #text(size: sizes.xs, style: "italic")[{} {}, {}]
      ],
      align(right + horizon)[
        #cr-indicator("{}")
      ]
    )
  ]

  // Stats row
  #block(
    width: 100%,
    inset: spacing.sm,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: sizes.xs)
    #grid(
      columns: (auto, auto, auto, auto, 1fr),
      column-gutter: spacing.md,
      [*AC* {}],
      [*HP* {}],
      [*Speed* {}],
      [*PP* {}],
      {},
    )
  ]

  // Abilities
  #block(
    width: 100%,
    inset: spacing.sm,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: sizes.xs)
    #grid(
      columns: (1fr,) * 6,
      align(center)[*STR* {} ({})],
      align(center)[*DEX* {} ({})],
      align(center)[*CON* {} ({})],
      align(center)[*INT* {} ({})],
      align(center)[*WIS* {} ({})],
      align(center)[*CHA* {} ({})],
    )
  ]

  // Key actions
  {}
]"##,
            escape_typst(name),
            size_name,
            escape_typst(&creature_type),
            escape_typst(&alignment),
            escape_typst(&cr),
            escape_typst(&ac),
            escape_typst(&hp),
            escape_typst(&speed),
            passive,
            if let Some(ref s) = saves {
                format!("[*Saves* {}]", escape_typst(s))
            } else {
                "[]".to_string()
            },
            str_score,
            modifier(str_score),
            dex_score,
            modifier(dex_score),
            con_score,
            modifier(con_score),
            int_score,
            modifier(int_score),
            wis_score,
            modifier(wis_score),
            cha_score,
            modifier(cha_score),
            if key_actions.is_empty() {
                String::new()
            } else {
                format!(
                    r#"#block(
    width: 100%,
    inset: spacing.sm,
  )[
    #set text(size: sizes.xs)
    *Key Actions:* {}
  ]"#,
                    escape_typst(&key_actions.join(", "))
                )
            },
        )
    }
}

impl Renderable for EncounterSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let mut typst = String::new();

        // Title
        typst.push_str(&format!(
            "#align(center)[\n  #title-text(\"{}\")\n]\n\n#v(spacing.sm)\n\n",
            escape_typst(&self.title)
        ));

        // Encounter summary
        if !self.monsters.is_empty() {
            typst.push_str(&format!(
                r#"#block(
  width: 100%,
  fill: colors.background-alt,
  inset: spacing.md,
  radius: 2pt,
)[
  #text(weight: "bold")[Encounter Summary]
  #h(1fr)
  #text(size: sizes.sm)[{} creature{}]
]

#v(spacing.md)

"#,
                self.monsters.len(),
                if self.monsters.len() != 1 { "s" } else { "" }
            ));
        }

        // Render each monster
        for (i, monster) in self.monsters.iter().enumerate() {
            typst.push_str(&Self::render_compact_stat_block(monster));
            if i < self.monsters.len() - 1 {
                typst.push_str("\n#v(spacing.md)\n");
            }
        }

        // DM Notes
        if let Some(ref notes) = self.notes {
            typst.push_str(&format!(
                r#"

#v(spacing.lg)
#block(
  width: 100%,
  stroke: 1pt + colors.border,
  inset: spacing.md,
  radius: 2pt,
)[
  #text(weight: "bold")[DM Notes]
  #v(spacing.sm)
  {}
]"#,
                escape_typst(notes)
            ));
        }

        // Empty state
        if self.monsters.is_empty() {
            typst.push_str(
                r#"#align(center + horizon)[
  #text(fill: colors.text-secondary)[No monsters in this encounter]
]"#,
            );
        }

        // Footer
        typst.push_str(&format!(
            r#"

#v(1fr)
#align(center)[
  #small-text[{} - Generated by Mimir]
]"#,
            escape_typst(&self.title)
        ));

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        Some(self.title.clone())
    }
}

/// Extract AC from monster data
fn extract_ac(monster: &Value) -> String {
    monster
        .get("ac")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .map(|ac| {
            if let Some(n) = ac.as_i64() {
                n.to_string()
            } else if let Some(obj) = ac.as_object() {
                let ac_val = obj.get("ac").and_then(|v| v.as_i64()).unwrap_or(10);
                if let Some(from) = obj.get("from").and_then(|v| v.as_array()) {
                    let sources: Vec<_> = from.iter().filter_map(|f| f.as_str()).collect();
                    if !sources.is_empty() {
                        return format!("{} ({})", ac_val, sources.join(", "));
                    }
                }
                ac_val.to_string()
            } else {
                "10".to_string()
            }
        })
        .unwrap_or_else(|| "10".to_string())
}

/// Extract HP from monster data
fn extract_hp(monster: &Value) -> String {
    monster
        .get("hp")
        .map(|hp_val| {
            if let Some(n) = hp_val.as_i64() {
                n.to_string()
            } else if let Some(obj) = hp_val.as_object() {
                let avg = obj.get("average").and_then(|v| v.as_i64()).unwrap_or(1);
                if let Some(formula) = obj.get("formula").and_then(|v| v.as_str()) {
                    format!("{} ({})", avg, formula)
                } else {
                    avg.to_string()
                }
            } else {
                "1".to_string()
            }
        })
        .unwrap_or_else(|| "1".to_string())
}

/// Extract speed from monster data
fn extract_speed(monster: &Value) -> String {
    monster
        .get("speed")
        .map(|s| {
            if let Some(obj) = s.as_object() {
                let walk = obj
                    .get("walk")
                    .and_then(|v| v.as_i64())
                    .map(|n| format!("{} ft.", n));
                walk.unwrap_or_else(|| "30 ft.".to_string())
            } else {
                "30 ft.".to_string()
            }
        })
        .unwrap_or_else(|| "30 ft.".to_string())
}

/// Extract CR from monster data
fn extract_cr(monster: &Value) -> String {
    monster
        .get("cr")
        .map(|cr_val| {
            if let Some(s) = cr_val.as_str() {
                s.to_string()
            } else if let Some(obj) = cr_val.as_object() {
                obj.get("cr")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .to_string()
            } else if let Some(n) = cr_val.as_i64() {
                n.to_string()
            } else {
                "0".to_string()
            }
        })
        .unwrap_or_else(|| "0".to_string())
}

/// Extract saves from monster data
fn extract_saves(monster: &Value) -> Option<String> {
    monster.get("save").and_then(|save_obj| {
        if let Some(obj) = save_obj.as_object() {
            let parts: Vec<String> = [
                ("str", "Str"),
                ("dex", "Dex"),
                ("con", "Con"),
                ("int", "Int"),
                ("wis", "Wis"),
                ("cha", "Cha"),
            ]
            .iter()
            .filter_map(|(key, label)| {
                obj.get(*key)
                    .and_then(|v| v.as_str())
                    .map(|val| format!("{} {}", label, val))
            })
            .collect();

            if parts.is_empty() {
                None
            } else {
                Some(parts.join(", "))
            }
        } else {
            None
        }
    })
}

/// Extract alignment from monster data
fn extract_alignment(monster: &Value) -> String {
    monster
        .get("alignment")
        .map(|align| {
            if let Some(s) = align.as_str() {
                s.to_string()
            } else if let Some(arr) = align.as_array() {
                arr.iter()
                    .filter_map(|a| {
                        if let Some(s) = a.as_str() {
                            Some(match s {
                                "L" => "lawful",
                                "N" | "NX" | "NY" => "neutral",
                                "C" => "chaotic",
                                "G" => "good",
                                "E" => "evil",
                                "U" => "unaligned",
                                _ => s,
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                "unaligned".to_string()
            }
        })
        .unwrap_or_else(|| "unaligned".to_string())
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
    fn test_encounter_empty() {
        let section = EncounterSection::new("Test Encounter", vec![]);
        assert_eq!(section.toc_title(), Some("Test Encounter".to_string()));
    }

    #[test]
    fn test_encounter_with_monsters() {
        let monsters = vec![
            json!({"name": "Goblin", "cr": "1/4"}),
            json!({"name": "Bugbear", "cr": "1"}),
        ];
        let section = EncounterSection::new("Goblin Ambush", monsters);
        assert_eq!(section.monsters.len(), 2);
    }

    #[test]
    fn test_from_json() {
        let data = json!({
            "title": "Dragon's Lair",
            "monsters": [{"name": "Adult Red Dragon", "cr": "17"}],
            "notes": "Watch for the breath weapon"
        });
        let section = EncounterSection::from_json(data);
        assert_eq!(section.title, "Dragon's Lair");
        assert_eq!(section.monsters.len(), 1);
        assert!(section.notes.is_some());
    }

    #[test]
    fn test_with_notes() {
        let section = EncounterSection::new("Test", vec![])
            .with_notes("Important note");
        assert_eq!(section.notes, Some("Important note".to_string()));
    }
}
