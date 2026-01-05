//! Monster appendix section

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Monster appendix section - renders monster stat blocks
pub struct MonsterAppendix {
    /// Monster data (JSON array of monsters)
    monsters: Value,
}

impl MonsterAppendix {
    /// Create from monster JSON data
    ///
    /// Expected format - flat array of monsters:
    /// ```json
    /// [
    ///   { "name": "Goblin", "cr": "1/4", ... },
    ///   { "name": "Orc", "cr": "1/2", ... }
    /// ]
    /// ```
    pub fn new(monsters: Value) -> Self {
        Self { monsters }
    }

    /// Check if there are any monsters to render
    pub fn is_empty(&self) -> bool {
        self.monsters
            .as_array()
            .map(|arr| arr.is_empty())
            .unwrap_or(true)
    }
}

impl Renderable for MonsterAppendix {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let monsters = match self.monsters.as_array() {
            Some(arr) if !arr.is_empty() => arr,
            _ => return Ok("// No monster data\n".to_string()),
        };

        let mut typst = String::new();

        // Section header
        typst.push_str("#pagebreak()\n");
        typst.push_str("#text(size: 16pt, weight: \"bold\")[Monster Reference]\n");
        typst.push_str("#v(1em)\n\n");

        for monster in monsters {
            typst.push_str(&render_monster_stat_block(monster));
            typst.push_str("\n#v(1em)\n");
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some("Monster Reference".to_string())
        }
    }
}

/// Render a single monster stat block
fn render_monster_stat_block(monster: &Value) -> String {
    let name = monster.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let cr = monster.get("cr").and_then(|v| v.as_str()).unwrap_or("?");

    // Size and type
    let size = monster
        .get("size")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .unwrap_or("M");

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

    // AC
    let ac = monster
        .get("ac")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .map(|ac| {
            if let Some(n) = ac.as_i64() {
                n.to_string()
            } else if let Some(obj) = ac.as_object() {
                obj.get("ac")
                    .and_then(|v| v.as_i64())
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "?".to_string())
            } else {
                "?".to_string()
            }
        })
        .unwrap_or_else(|| "?".to_string());

    // HP
    let hp = monster
        .get("hp")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("average"))
        .and_then(|v| v.as_i64())
        .map(|n| n.to_string())
        .unwrap_or_else(|| "?".to_string());

    // Ability scores
    let str_score = monster.get("str").and_then(|v| v.as_i64()).unwrap_or(10);
    let dex_score = monster.get("dex").and_then(|v| v.as_i64()).unwrap_or(10);
    let con_score = monster.get("con").and_then(|v| v.as_i64()).unwrap_or(10);
    let int_score = monster.get("int").and_then(|v| v.as_i64()).unwrap_or(10);
    let wis_score = monster.get("wis").and_then(|v| v.as_i64()).unwrap_or(10);
    let cha_score = monster.get("cha").and_then(|v| v.as_i64()).unwrap_or(10);

    format!(
        r#"#box(stroke: 1pt, inset: 8pt, width: 100%)[
  #text(size: 12pt, weight: "bold")[{}] #h(1fr) #text(size: 10pt)[CR {}]
  #v(2pt)
  #text(size: 9pt, style: "italic")[{} {} ]
  #v(4pt)
  #line(length: 100%, stroke: 0.5pt)
  #v(4pt)
  #grid(columns: 2, gutter: 8pt,
    [*AC* {}], [*HP* {}]
  )
  #v(4pt)
  #line(length: 100%, stroke: 0.5pt)
  #v(4pt)
  #grid(columns: 6, gutter: 4pt,
    align(center)[*STR*], align(center)[*DEX*], align(center)[*CON*],
    align(center)[*INT*], align(center)[*WIS*], align(center)[*CHA*],
    align(center)[{} ({})], align(center)[{} ({})], align(center)[{} ({})],
    align(center)[{} ({})], align(center)[{} ({})], align(center)[{} ({})]
  )
]
"#,
        escape_typst(name), cr,
        size_name(size), escape_typst(&creature_type),
        ac, hp,
        str_score, modifier(str_score),
        dex_score, modifier(dex_score),
        con_score, modifier(con_score),
        int_score, modifier(int_score),
        wis_score, modifier(wis_score),
        cha_score, modifier(cha_score)
    )
}

/// Convert size code to full name
fn size_name(size: &str) -> &'static str {
    match size.to_uppercase().as_str() {
        "T" => "Tiny",
        "S" => "Small",
        "M" => "Medium",
        "L" => "Large",
        "H" => "Huge",
        "G" => "Gargantuan",
        _ => "Medium",
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_monsters() {
        let appendix = MonsterAppendix::new(json!([]));
        assert!(appendix.is_empty());
    }

    #[test]
    fn test_with_monsters() {
        let appendix = MonsterAppendix::new(json!([
            {"name": "Goblin", "cr": "1/4"},
            {"name": "Orc", "cr": "1/2"}
        ]));
        assert!(!appendix.is_empty());
    }

    #[test]
    fn test_modifier() {
        assert_eq!(modifier(10), "+0");
        assert_eq!(modifier(18), "+4");
        assert_eq!(modifier(8), "-1");
    }
}
