//! Monster stat block section
//!
//! Generates full monster stat blocks using shared Typst components.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Monster stat block section - generates full stat blocks for monsters
pub struct MonsterStatBlockSection {
    /// Monster data (JSON)
    monsters: Vec<Value>,
}

impl MonsterStatBlockSection {
    /// Create a new monster stat block section with a single monster
    pub fn new(monster: Value) -> Self {
        Self {
            monsters: vec![monster],
        }
    }

    /// Create with multiple monsters
    pub fn with_monsters(monsters: Vec<Value>) -> Self {
        Self { monsters }
    }

    /// Create from a JSON value (expects object or array)
    pub fn from_json(data: Value) -> Self {
        if let Some(arr) = data.as_array() {
            Self::with_monsters(arr.clone())
        } else {
            Self::new(data)
        }
    }

    /// Render a single monster stat block
    fn render_monster(monster: &Value) -> String {
        let name = monster
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Monster");

        let size = monster
            .get("size")
            .and_then(|v| v.as_str())
            .or_else(|| monster.get("size").and_then(|v| v.as_array()).and_then(|a| a.first()).and_then(|v| v.as_str()))
            .unwrap_or("Medium");

        let creature_type = monster
            .get("type")
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(obj) = v.as_object() {
                    obj.get("type").and_then(|t| t.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "humanoid".to_string());

        let alignment = monster
            .get("alignment")
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(arr) = v.as_array() {
                    // Handle alignment arrays like ["C", "E"]
                    Some(arr.iter().filter_map(|a| a.as_str()).collect::<Vec<_>>().join(" "))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "neutral".to_string());

        // AC - handle different formats
        let ac = if let Some(ac_val) = monster.get("ac") {
            if let Some(ac_num) = ac_val.as_i64() {
                ac_num.to_string()
            } else if let Some(ac_arr) = ac_val.as_array() {
                if let Some(first) = ac_arr.first() {
                    if let Some(ac_num) = first.as_i64() {
                        ac_num.to_string()
                    } else if let Some(ac_obj) = first.as_object() {
                        let base = ac_obj.get("ac").and_then(|v| v.as_i64()).unwrap_or(10);
                        let from = ac_obj.get("from").and_then(|v| v.as_array()).map(|arr| {
                            arr.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                        });
                        if let Some(source) = from {
                            format!("{} ({})", base, source)
                        } else {
                            base.to_string()
                        }
                    } else {
                        "10".to_string()
                    }
                } else {
                    "10".to_string()
                }
            } else {
                "10".to_string()
            }
        } else {
            "10".to_string()
        };

        // HP
        let hp = if let Some(hp_val) = monster.get("hp") {
            if let Some(hp_obj) = hp_val.as_object() {
                let average = hp_obj.get("average").and_then(|v| v.as_i64()).unwrap_or(10);
                let formula = hp_obj.get("formula").and_then(|v| v.as_str());
                if let Some(f) = formula {
                    format!("{} ({})", average, f)
                } else {
                    average.to_string()
                }
            } else if let Some(hp_num) = hp_val.as_i64() {
                hp_num.to_string()
            } else {
                "10".to_string()
            }
        } else {
            "10".to_string()
        };

        // Speed
        let speed = if let Some(speed_val) = monster.get("speed") {
            if let Some(speed_obj) = speed_val.as_object() {
                let mut parts = Vec::new();
                if let Some(walk) = speed_obj.get("walk").and_then(|v| v.as_i64()) {
                    parts.push(format!("{} ft.", walk));
                }
                if let Some(fly) = speed_obj.get("fly").and_then(|v| v.as_i64()) {
                    parts.push(format!("fly {} ft.", fly));
                }
                if let Some(swim) = speed_obj.get("swim").and_then(|v| v.as_i64()) {
                    parts.push(format!("swim {} ft.", swim));
                }
                if let Some(climb) = speed_obj.get("climb").and_then(|v| v.as_i64()) {
                    parts.push(format!("climb {} ft.", climb));
                }
                if let Some(burrow) = speed_obj.get("burrow").and_then(|v| v.as_i64()) {
                    parts.push(format!("burrow {} ft.", burrow));
                }
                if parts.is_empty() {
                    "30 ft.".to_string()
                } else {
                    parts.join(", ")
                }
            } else {
                "30 ft.".to_string()
            }
        } else {
            "30 ft.".to_string()
        };

        // Ability scores
        let str_score = monster.get("str").and_then(|v| v.as_i64()).unwrap_or(10);
        let dex_score = monster.get("dex").and_then(|v| v.as_i64()).unwrap_or(10);
        let con_score = monster.get("con").and_then(|v| v.as_i64()).unwrap_or(10);
        let int_score = monster.get("int").and_then(|v| v.as_i64()).unwrap_or(10);
        let wis_score = monster.get("wis").and_then(|v| v.as_i64()).unwrap_or(10);
        let cha_score = monster.get("cha").and_then(|v| v.as_i64()).unwrap_or(10);

        // Challenge rating
        let cr = if let Some(cr_val) = monster.get("cr") {
            if let Some(cr_str) = cr_val.as_str() {
                cr_str.to_string()
            } else if let Some(cr_num) = cr_val.as_i64() {
                cr_num.to_string()
            } else if let Some(cr_obj) = cr_val.as_object() {
                cr_obj.get("cr").and_then(|v| v.as_str()).unwrap_or("0").to_string()
            } else {
                "0".to_string()
            }
        } else {
            "0".to_string()
        };

        // Senses
        let senses = monster
            .get("senses")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "passive Perception 10".to_string());

        // Languages
        let languages = monster
            .get("languages")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "—".to_string());

        // Saving throws
        let saves = monster
            .get("save")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| format!("{} {}", k.to_uppercase(), v.as_str().unwrap_or("+0")))
                    .collect::<Vec<_>>()
                    .join(", ")
            });

        // Skills
        let skills = monster
            .get("skill")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| format!("{} {}", titlecase(k), v.as_str().unwrap_or("+0")))
                    .collect::<Vec<_>>()
                    .join(", ")
            });

        // Damage immunities/resistances/vulnerabilities
        let damage_immunities = monster
            .get("immune")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.as_str().or_else(|| d.get("immune").and_then(|i| i.as_array()).map(|_| "special")))
                    .collect::<Vec<_>>()
                    .join(", ")
            });

        let damage_resistances = monster
            .get("resist")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join(", "));

        let condition_immunities = monster
            .get("conditionImmune")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join(", "));

        // Build the stat block using shared component
        let mut typst = format!(
            r##"#stat-block(
  name: "{}",
  size: "{}",
  type: "{}",
  alignment: "{}",
  ac: "{}",
  hp: "{}",
  speed: "{}",
  str: {},
  dex: {},
  con: {},
  int: {},
  wis: {},
  cha: {},
  senses: "{}",
  languages: "{}",
  cr: "{}",
"##,
            escape_typst(name),
            escape_typst(&titlecase(size)),
            escape_typst(&creature_type),
            escape_typst(&alignment),
            escape_typst(&ac),
            escape_typst(&hp),
            escape_typst(&speed),
            str_score, dex_score, con_score, int_score, wis_score, cha_score,
            escape_typst(&senses),
            escape_typst(&languages),
            escape_typst(&cr),
        );

        // Add optional fields
        if let Some(ref s) = saves {
            typst.push_str(&format!("  saves: \"{}\",\n", escape_typst(s)));
        }
        if let Some(ref s) = skills {
            typst.push_str(&format!("  skills: \"{}\",\n", escape_typst(s)));
        }
        if let Some(ref d) = damage_immunities {
            if !d.is_empty() {
                typst.push_str(&format!("  damage-immunities: \"{}\",\n", escape_typst(d)));
            }
        }
        if let Some(ref d) = damage_resistances {
            if !d.is_empty() {
                typst.push_str(&format!("  damage-resistances: \"{}\",\n", escape_typst(d)));
            }
        }
        if let Some(ref c) = condition_immunities {
            if !c.is_empty() {
                typst.push_str(&format!("  condition-immunities: \"{}\",\n", escape_typst(c)));
            }
        }

        // Traits
        if let Some(traits) = monster.get("trait").and_then(|v| v.as_array()) {
            typst.push_str("  traits: (\n");
            for trait_entry in traits {
                let trait_name = trait_entry.get("name").and_then(|v| v.as_str()).unwrap_or("Trait");
                let trait_desc = get_entries_text(trait_entry);
                typst.push_str(&format!(
                    "    (name: \"{}\", description: [{}]),\n",
                    escape_typst(trait_name),
                    escape_typst(&trait_desc)
                ));
            }
            typst.push_str("  ),\n");
        }

        // Actions
        if let Some(actions) = monster.get("action").and_then(|v| v.as_array()) {
            typst.push_str("  actions: (\n");
            for action in actions {
                let action_name = action.get("name").and_then(|v| v.as_str()).unwrap_or("Action");
                let action_desc = get_entries_text(action);
                typst.push_str(&format!(
                    "    (name: \"{}\", description: [{}]),\n",
                    escape_typst(action_name),
                    escape_typst(&action_desc)
                ));
            }
            typst.push_str("  ),\n");
        }

        // Reactions
        if let Some(reactions) = monster.get("reaction").and_then(|v| v.as_array()) {
            typst.push_str("  reactions: (\n");
            for reaction in reactions {
                let reaction_name = reaction.get("name").and_then(|v| v.as_str()).unwrap_or("Reaction");
                let reaction_desc = get_entries_text(reaction);
                typst.push_str(&format!(
                    "    (name: \"{}\", description: [{}]),\n",
                    escape_typst(reaction_name),
                    escape_typst(&reaction_desc)
                ));
            }
            typst.push_str("  ),\n");
        }

        // Legendary actions
        if let Some(legendary) = monster.get("legendary").and_then(|v| v.as_array()) {
            typst.push_str("  legendary-actions: (\n");
            for action in legendary {
                let action_name = action.get("name").and_then(|v| v.as_str()).unwrap_or("Legendary Action");
                let action_desc = get_entries_text(action);
                typst.push_str(&format!(
                    "    (name: \"{}\", description: [{}]),\n",
                    escape_typst(action_name),
                    escape_typst(&action_desc)
                ));
            }
            typst.push_str("  ),\n");
        }

        typst.push_str(")\n");
        typst
    }
}

impl Renderable for MonsterStatBlockSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.monsters.is_empty() {
            return Ok("// No monsters to display\n".to_string());
        }

        let mut typst = String::new();

        for (i, monster) in self.monsters.iter().enumerate() {
            if i > 0 {
                typst.push_str("\n#v(spacing.lg)\n\n");
            }
            typst.push_str(&Self::render_monster(monster));
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.monsters.is_empty() {
            None
        } else if self.monsters.len() == 1 {
            let name = self.monsters[0]
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Monster");
            Some(name.to_string())
        } else {
            Some("Monster Stat Blocks".to_string())
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

/// Convert string to title case
fn titlecase(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Extract text from entries array
fn get_entries_text(entry: &Value) -> String {
    if let Some(entries) = entry.get("entries").and_then(|v| v.as_array()) {
        entries
            .iter()
            .filter_map(|e| {
                if let Some(s) = e.as_str() {
                    Some(s.to_string())
                } else if let Some(obj) = e.as_object() {
                    // Handle nested entries like lists
                    obj.get("entries")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join(" "))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_single_monster() {
        let monster = json!({
            "name": "Goblin",
            "size": "Small",
            "type": "humanoid",
            "cr": "1/4"
        });
        let section = MonsterStatBlockSection::new(monster);
        assert_eq!(section.toc_title(), Some("Goblin".to_string()));
    }

    #[test]
    fn test_multiple_monsters() {
        let monsters = vec![
            json!({"name": "Goblin"}),
            json!({"name": "Hobgoblin"}),
        ];
        let section = MonsterStatBlockSection::with_monsters(monsters);
        assert_eq!(section.toc_title(), Some("Monster Stat Blocks".to_string()));
    }

    #[test]
    fn test_titlecase() {
        assert_eq!(titlecase("small"), "Small");
        assert_eq!(titlecase("LARGE"), "LARGE");
        assert_eq!(titlecase(""), "");
    }
}
