//! Monster cards section
//!
//! Generates condensed 2.5" x 3.5" monster cards for quick reference.
//! These are more compact than full stat blocks, showing only key info.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Monster cards section - generates poker-sized monster reference cards
pub struct MonsterCardSection {
    /// Monster data (JSON array)
    monsters: Vec<Value>,
    /// Show cut lines between cards
    show_cut_lines: bool,
}

impl MonsterCardSection {
    /// Create a new monster cards section
    pub fn new(monsters: Vec<Value>) -> Self {
        Self {
            monsters,
            show_cut_lines: true,
        }
    }

    /// Create from a JSON value (expects array)
    pub fn from_json(monsters: Value) -> Self {
        let monster_vec = monsters
            .as_array()
            .map(|arr| arr.clone())
            .unwrap_or_default();
        Self::new(monster_vec)
    }

    /// Set whether to show cut lines
    pub fn with_cut_lines(mut self, show: bool) -> Self {
        self.show_cut_lines = show;
        self
    }

    /// Render a single monster card
    fn render_card(monster: &Value) -> String {
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
        let ac = monster
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
            .unwrap_or_else(|| "10".to_string());

        // HP
        let hp = monster
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
            .unwrap_or_else(|| "1".to_string());

        // Speed
        let speed = monster
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
            .unwrap_or_else(|| "30 ft.".to_string());

        // Passive perception
        let passive = monster
            .get("passive")
            .and_then(|v| v.as_i64())
            .unwrap_or(10);

        // Ability scores
        let str_score = monster.get("str").and_then(|v| v.as_i64()).unwrap_or(10);
        let dex_score = monster.get("dex").and_then(|v| v.as_i64()).unwrap_or(10);
        let con_score = monster.get("con").and_then(|v| v.as_i64()).unwrap_or(10);
        let int_score = monster.get("int").and_then(|v| v.as_i64()).unwrap_or(10);
        let wis_score = monster.get("wis").and_then(|v| v.as_i64()).unwrap_or(10);
        let cha_score = monster.get("cha").and_then(|v| v.as_i64()).unwrap_or(10);

        // CR
        let cr = monster
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
            .unwrap_or_else(|| "0".to_string());

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
            r##"box(
  width: 2.5in,
  height: 3.5in,
  stroke: (
    top: 2pt + colors.accent,
    bottom: 2pt + colors.accent,
    left: 0.5pt + colors.border,
    right: 0.5pt + colors.border,
  ),
  radius: 2pt,
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
      columns: (1fr, auto),
      [
        #text(size: 7pt, weight: "bold")[{}]
        #linebreak()
        #text(size: 5pt, style: "italic")[{} {}, {}]
      ],
      align(right + horizon)[
        #cr-indicator("{}")
      ]
    )
  ]

  // Stats row
  #block(
    width: 100%,
    inset: 4pt,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 5.5pt)
    #grid(
      columns: (auto, auto, auto, auto),
      column-gutter: 6pt,
      [*AC* {}],
      [*HP* {}],
      [*Spd* {}],
      [*PP* {}],
    )
  ]

  // Ability scores
  #block(
    width: 100%,
    inset: 4pt,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 5pt)
    #grid(
      columns: (1fr,) * 6,
      align(center)[*STR*\ {} ({})],
      align(center)[*DEX*\ {} ({})],
      align(center)[*CON*\ {} ({})],
      align(center)[*INT*\ {} ({})],
      align(center)[*WIS*\ {} ({})],
      align(center)[*CHA*\ {} ({})],
    )
  ]

  // Key actions
  {}

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
      ]
    ]
  )
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
    inset: 4pt,
  )[
    #set text(size: 5pt)
    *Key Actions:* {}
  ]"#,
                    escape_typst(&key_actions.join(", "))
                )
            },
            escape_typst(source),
        )
    }
}

impl Renderable for MonsterCardSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.monsters.is_empty() {
            return Ok("// No monsters to display\n".to_string());
        }

        let mut typst = String::new();
        let cards_per_page = 9;
        let total_pages = (self.monsters.len() + cards_per_page - 1) / cards_per_page;

        for page_num in 0..total_pages {
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, self.monsters.len());
            let page_monsters = &self.monsters[start_idx..end_idx];

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

            // Render each card
            for (i, monster) in page_monsters.iter().enumerate() {
                typst.push_str("    ");
                typst.push_str(&Self::render_card(monster));
                if i < page_monsters.len() - 1 || page_monsters.len() < 9 {
                    typst.push(',');
                }
                typst.push('\n');
            }

            // Fill remaining slots with empty boxes
            for _ in page_monsters.len()..9 {
                typst.push_str("    box(width: 2.5in, height: 3.5in),\n");
            }

            typst.push_str("  )\n]\n");

            // Cut lines indicator
            if self.show_cut_lines && !page_monsters.is_empty() {
                typst.push_str("#place(\n  bottom + center,\n  dy: 0.1in,\n");
                typst.push_str(
                    "  text(size: 6pt, fill: colors.text-secondary)[Cut along card borders]\n)\n",
                );
            }
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.monsters.is_empty() {
            None
        } else {
            Some("Monster Cards".to_string())
        }
    }
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
    fn test_monster_cards_empty() {
        let section = MonsterCardSection::new(vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_monster_cards_with_monsters() {
        let monsters = vec![json!({
            "name": "Goblin",
            "cr": "1/4",
            "size": ["S"],
            "creature_type": "humanoid"
        })];
        let section = MonsterCardSection::new(monsters);
        assert_eq!(section.toc_title(), Some("Monster Cards".to_string()));
    }

    #[test]
    fn test_from_json() {
        let data = json!([
            {"name": "Goblin", "cr": "1/4"},
            {"name": "Orc", "cr": "1/2"}
        ]);
        let section = MonsterCardSection::from_json(data);
        assert_eq!(section.monsters.len(), 2);
    }

    #[test]
    fn test_modifier() {
        assert_eq!(modifier(10), "+0");
        assert_eq!(modifier(18), "+4");
        assert_eq!(modifier(8), "-1");
    }

    #[test]
    fn test_extract_alignment() {
        let monster = json!({"alignment": ["L", "G"]});
        assert_eq!(extract_alignment(&monster), "lawful good");

        let monster = json!({"alignment": "chaotic evil"});
        assert_eq!(extract_alignment(&monster), "chaotic evil");
    }
}
