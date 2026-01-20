//! NPC appendix section

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// NPC appendix section - renders NPC reference cards
pub struct NpcAppendix {
    /// NPC data (JSON from frontend)
    npcs: Value,
}

impl NpcAppendix {
    /// Create from NPC JSON data
    ///
    /// Expected format:
    /// ```json
    /// [
    ///   {
    ///     "character_name": "Name",
    ///     "npc_role": "Role",
    ///     "npc_location": "Location",
    ///     "race": "Human",
    ///     "level": 5,
    ///     ...
    ///   }
    /// ]
    /// ```
    pub fn new(npcs: Value) -> Self {
        Self { npcs }
    }

    /// Check if there are any NPCs to render
    pub fn is_empty(&self) -> bool {
        self.npcs
            .as_array()
            .map(|arr| arr.is_empty())
            .unwrap_or(true)
    }
}

impl Renderable for NpcAppendix {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let npcs = match self.npcs.as_array() {
            Some(arr) if !arr.is_empty() => arr,
            _ => return Ok("// No NPC data\n".to_string()),
        };

        let mut typst = String::new();

        for npc in npcs {
            typst.push_str(&render_npc_card(npc));
            typst.push_str("\n#v(1em)\n");
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some("NPC Reference".to_string())
        }
    }
}

/// Render a single NPC card
fn render_npc_card(npc: &Value) -> String {
    let name = npc
        .get("character_name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown NPC");

    let role = npc
        .get("npc_role")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let location = npc
        .get("npc_location")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let race = npc
        .get("race")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let level = npc.get("level").and_then(|v| v.as_i64()).unwrap_or(1);

    // Get class info
    let class_info = npc
        .get("classes")
        .and_then(|v| v.as_array())
        .map(|classes| {
            classes
                .iter()
                .filter_map(|c| {
                    let class_name = c.get("class_name").and_then(|v| v.as_str())?;
                    let class_level = c.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
                    Some(format!("{} {}", class_name, class_level))
                })
                .collect::<Vec<_>>()
                .join(" / ")
        })
        .unwrap_or_else(|| "Commoner 1".to_string());

    // Get ability scores
    let abilities = npc.get("abilities");
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

    let hp = npc.get("max_hp").and_then(|v| v.as_i64()).unwrap_or(10);
    let speed = npc.get("speed").and_then(|v| v.as_i64()).unwrap_or(30);

    let mut card = format!(
        r#"#box(stroke: 1pt, inset: 8pt, width: 100%)[
  #text(size: 12pt, weight: "bold")[{}]"#,
        escape_typst(name)
    );

    // Add role and location if present
    if !role.is_empty() || !location.is_empty() {
        card.push_str("\n  #h(1fr) #text(size: 9pt, style: \"italic\")[");
        if !role.is_empty() {
            card.push_str(&escape_typst(role));
        }
        if !role.is_empty() && !location.is_empty() {
            card.push_str(" — ");
        }
        if !location.is_empty() {
            card.push_str(&escape_typst(location));
        }
        card.push(']');
    }

    card.push_str(&format!(
        r#"
  #v(2pt)
  #text(size: 9pt)[{} {} | Level {}]
  #v(4pt)
  #line(length: 100%, stroke: 0.5pt)
  #v(4pt)
  #grid(columns: 3, gutter: 8pt,
    [*HP* {}], [*Speed* {} ft], [*Class* {}]
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
        escape_typst(race), escape_typst(&class_info), level,
        hp, speed, escape_typst(&class_info),
        str_score, modifier(str_score),
        dex_score, modifier(dex_score),
        con_score, modifier(con_score),
        int_score, modifier(int_score),
        wis_score, modifier(wis_score),
        cha_score, modifier(cha_score)
    ));

    card
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
    fn test_empty_npcs() {
        let appendix = NpcAppendix::new(json!([]));
        assert!(appendix.is_empty());
    }

    #[test]
    fn test_with_npcs() {
        let appendix = NpcAppendix::new(json!([
            {"character_name": "Test NPC", "race": "Human"}
        ]));
        assert!(!appendix.is_empty());
    }
}
