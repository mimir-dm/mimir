//! Monster markdown renderer.
//!
//! Converts monster data to markdown format for file-based persistence.

use serde_json::Value;

/// Render a complete monsters markdown file from grouped encounter data.
pub fn render_monsters_file(
    encounters: &[(Option<String>, Vec<MonsterData>)],
    module_name: &str,
) -> String {
    let mut md = String::new();

    // YAML frontmatter
    md.push_str("---\n");
    md.push_str(&format!("title: \"{} - Monsters\"\n", module_name));
    md.push_str("type: module_monsters\n");
    md.push_str("---\n\n");

    md.push_str("# Module Monsters\n\n");

    for (tag, monsters) in encounters {
        let section_name = tag
            .as_ref()
            .map(|t| t.as_str())
            .unwrap_or("Untagged");

        md.push_str(&format!("## {}\n\n", section_name));

        for monster in monsters {
            md.push_str(&render_monster_card(monster));
            md.push_str("\n---\n\n");
        }
    }

    md
}

/// Monster data needed for rendering.
pub struct MonsterData {
    /// Monster name.
    pub name: String,
    /// Source book abbreviation (e.g., "MM").
    pub source: String,
    /// Number of this monster type.
    pub quantity: i32,
    /// Full monster JSON data from catalog.
    pub full_data: Option<Value>,
}

/// Render a single monster stat block to markdown.
fn render_monster_card(monster: &MonsterData) -> String {
    let mut md = String::new();

    // Header with quantity
    if monster.quantity > 1 {
        md.push_str(&format!("### {} (x{})\n\n", monster.name, monster.quantity));
    } else {
        md.push_str(&format!("### {}\n\n", monster.name));
    }

    md.push_str(&format!("*Source: {}*\n\n", monster.source));

    // If we have full data, render the stat block
    if let Some(data) = &monster.full_data {
        md.push_str(&render_stat_block(data));
    }

    md
}

/// Render the full stat block from monster JSON.
fn render_stat_block(data: &Value) -> String {
    let mut md = String::new();

    // Size, type, alignment line
    let size = extract_size(data);
    let creature_type = extract_creature_type(data);
    let alignment = extract_alignment(data);
    md.push_str(&format!("*{} {}, {}*\n\n", size, creature_type, alignment));

    // AC, HP, Speed
    let ac = extract_ac(data);
    let hp = extract_hp(data);
    let speed = extract_speed(data);

    md.push_str(&format!("**Armor Class** {}\n\n", ac));
    md.push_str(&format!("**Hit Points** {}\n\n", hp));
    md.push_str(&format!("**Speed** {}\n\n", speed));

    // Ability scores table
    md.push_str("| STR | DEX | CON | INT | WIS | CHA |\n");
    md.push_str("|:---:|:---:|:---:|:---:|:---:|:---:|\n");
    md.push_str(&format!(
        "| {} ({}) | {} ({}) | {} ({}) | {} ({}) | {} ({}) | {} ({}) |\n\n",
        get_ability(data, "str"),
        get_modifier(data, "str"),
        get_ability(data, "dex"),
        get_modifier(data, "dex"),
        get_ability(data, "con"),
        get_modifier(data, "con"),
        get_ability(data, "int"),
        get_modifier(data, "int"),
        get_ability(data, "wis"),
        get_modifier(data, "wis"),
        get_ability(data, "cha"),
        get_modifier(data, "cha"),
    ));

    // Saving throws
    if let Some(saves) = extract_saves(data) {
        md.push_str(&format!("**Saving Throws** {}\n\n", saves));
    }

    // Skills
    if let Some(skills) = extract_skills(data) {
        md.push_str(&format!("**Skills** {}\n\n", skills));
    }

    // Damage vulnerabilities
    if let Some(vulns) = extract_string_array(data, "damageVulnerabilities") {
        if !vulns.is_empty() {
            md.push_str(&format!("**Damage Vulnerabilities** {}\n\n", vulns));
        }
    }

    // Damage resistances
    if let Some(resist) = extract_string_array(data, "damageResistances") {
        if !resist.is_empty() {
            md.push_str(&format!("**Damage Resistances** {}\n\n", resist));
        }
    }

    // Damage immunities
    if let Some(immune) = extract_string_array(data, "damageImmunities") {
        if !immune.is_empty() {
            md.push_str(&format!("**Damage Immunities** {}\n\n", immune));
        }
    }

    // Condition immunities
    if let Some(cond_immune) = extract_string_array(data, "conditionImmunities") {
        if !cond_immune.is_empty() {
            md.push_str(&format!("**Condition Immunities** {}\n\n", cond_immune));
        }
    }

    // Senses
    let senses = extract_senses(data);
    md.push_str(&format!("**Senses** {}\n\n", senses));

    // Languages
    let languages = extract_languages(data);
    md.push_str(&format!("**Languages** {}\n\n", languages));

    // Challenge rating
    let cr = extract_cr(data);
    let xp = cr_to_xp(&cr);
    md.push_str(&format!("**Challenge** {} ({} XP)\n\n", cr, xp));

    // Traits
    if let Some(traits) = data.get("trait") {
        if let Some(arr) = traits.as_array() {
            for trait_entry in arr {
                md.push_str(&render_entry(trait_entry));
            }
        }
    }

    // Actions
    if let Some(actions) = data.get("action") {
        if let Some(arr) = actions.as_array() {
            if !arr.is_empty() {
                md.push_str("#### Actions\n\n");
                for action in arr {
                    md.push_str(&render_entry(action));
                }
            }
        }
    }

    // Bonus Actions
    if let Some(bonus) = data.get("bonus") {
        if let Some(arr) = bonus.as_array() {
            if !arr.is_empty() {
                md.push_str("#### Bonus Actions\n\n");
                for b in arr {
                    md.push_str(&render_entry(b));
                }
            }
        }
    }

    // Reactions
    if let Some(reactions) = data.get("reaction") {
        if let Some(arr) = reactions.as_array() {
            if !arr.is_empty() {
                md.push_str("#### Reactions\n\n");
                for reaction in arr {
                    md.push_str(&render_entry(reaction));
                }
            }
        }
    }

    // Legendary Actions
    if let Some(legendary) = data.get("legendary") {
        if let Some(arr) = legendary.as_array() {
            if !arr.is_empty() {
                md.push_str("#### Legendary Actions\n\n");
                // Legendary action description
                if let Some(legendary_header) = data.get("legendaryHeader") {
                    if let Some(headers) = legendary_header.as_array() {
                        for h in headers {
                            if let Some(s) = h.as_str() {
                                md.push_str(&format!("{}\n\n", s));
                            }
                        }
                    }
                } else {
                    let name = data.get("name").and_then(|n| n.as_str()).unwrap_or("The creature");
                    md.push_str(&format!(
                        "{} can take 3 legendary actions, choosing from the options below. \
                         Only one legendary action option can be used at a time and only at the \
                         end of another creature's turn. {} regains spent legendary actions at \
                         the start of its turn.\n\n",
                        name, name
                    ));
                }
                for leg in arr {
                    md.push_str(&render_entry(leg));
                }
            }
        }
    }

    // Mythic Actions
    if let Some(mythic) = data.get("mythic") {
        if let Some(arr) = mythic.as_array() {
            if !arr.is_empty() {
                md.push_str("#### Mythic Actions\n\n");
                for m in arr {
                    md.push_str(&render_entry(m));
                }
            }
        }
    }

    md
}

/// Render an entry (trait, action, etc.) to markdown.
fn render_entry(entry: &Value) -> String {
    let mut md = String::new();

    let name = entry.get("name").and_then(|n| n.as_str()).unwrap_or("");
    if !name.is_empty() {
        md.push_str(&format!("**{}** ", name));
    }

    if let Some(entries) = entry.get("entries") {
        if let Some(arr) = entries.as_array() {
            for e in arr.iter() {
                if let Some(s) = e.as_str() {
                    let cleaned = clean_formatting_tags(s);
                    md.push_str(&format!("{}\n\n", cleaned));
                } else if let Some(obj) = e.as_object() {
                    // Handle nested entries like lists
                    if obj.get("type").and_then(|t| t.as_str()) == Some("list") {
                        if let Some(items) = obj.get("items") {
                            if let Some(item_arr) = items.as_array() {
                                for item in item_arr {
                                    if let Some(s) = item.as_str() {
                                        md.push_str(&format!("- {}\n", clean_formatting_tags(s)));
                                    }
                                }
                                md.push('\n');
                            }
                        }
                    }
                }
            }
        }
    }

    md
}

/// Clean 5etools formatting tags from text.
fn clean_formatting_tags(text: &str) -> String {
    let mut result = text.to_string();

    // Replace common 5etools tags with markdown equivalents
    // {@damage XdY+Z} -> XdY+Z
    let damage_re = regex::Regex::new(r"\{@damage ([^}]+)\}").unwrap();
    result = damage_re.replace_all(&result, "$1").to_string();

    // {@dice XdY} -> XdY
    let dice_re = regex::Regex::new(r"\{@dice ([^}]+)\}").unwrap();
    result = dice_re.replace_all(&result, "$1").to_string();

    // {@hit +X} -> +X
    let hit_re = regex::Regex::new(r"\{@hit ([^}]+)\}").unwrap();
    result = hit_re.replace_all(&result, "+$1").to_string();

    // {@dc X} -> DC X
    let dc_re = regex::Regex::new(r"\{@dc ([^}]+)\}").unwrap();
    result = dc_re.replace_all(&result, "DC $1").to_string();

    // {@condition X} -> X
    let condition_re = regex::Regex::new(r"\{@condition ([^}|]+)(?:\|[^}]*)?\}").unwrap();
    result = condition_re.replace_all(&result, "$1").to_string();

    // {@spell X} -> *X*
    let spell_re = regex::Regex::new(r"\{@spell ([^}|]+)(?:\|[^}]*)?\}").unwrap();
    result = spell_re.replace_all(&result, "*$1*").to_string();

    // {@creature X} -> X
    let creature_re = regex::Regex::new(r"\{@creature ([^}|]+)(?:\|[^}]*)?\}").unwrap();
    result = creature_re.replace_all(&result, "$1").to_string();

    // {@item X} -> X
    let item_re = regex::Regex::new(r"\{@item ([^}|]+)(?:\|[^}]*)?\}").unwrap();
    result = item_re.replace_all(&result, "$1").to_string();

    // {@skill X} -> X
    let skill_re = regex::Regex::new(r"\{@skill ([^}]+)\}").unwrap();
    result = skill_re.replace_all(&result, "$1").to_string();

    // {@atk mw} -> Melee Weapon Attack:
    let atk_re = regex::Regex::new(r"\{@atk mw\}").unwrap();
    result = atk_re.replace_all(&result, "*Melee Weapon Attack:*").to_string();

    // {@atk rw} -> Ranged Weapon Attack:
    let atk_rw_re = regex::Regex::new(r"\{@atk rw\}").unwrap();
    result = atk_rw_re.replace_all(&result, "*Ranged Weapon Attack:*").to_string();

    // {@atk ms} -> Melee Spell Attack:
    let atk_ms_re = regex::Regex::new(r"\{@atk ms\}").unwrap();
    result = atk_ms_re.replace_all(&result, "*Melee Spell Attack:*").to_string();

    // {@atk rs} -> Ranged Spell Attack:
    let atk_rs_re = regex::Regex::new(r"\{@atk rs\}").unwrap();
    result = atk_rs_re.replace_all(&result, "*Ranged Spell Attack:*").to_string();

    // {@atk mw,rw} -> Melee or Ranged Weapon Attack:
    let atk_mwrw_re = regex::Regex::new(r"\{@atk mw,rw\}").unwrap();
    result = atk_mwrw_re.replace_all(&result, "*Melee or Ranged Weapon Attack:*").to_string();

    // {@recharge X} -> (Recharge X-6)
    let recharge_re = regex::Regex::new(r"\{@recharge (\d+)\}").unwrap();
    result = recharge_re.replace_all(&result, "(Recharge $1-6)").to_string();

    // {@recharge} -> (Recharge 6)
    let recharge_empty_re = regex::Regex::new(r"\{@recharge\}").unwrap();
    result = recharge_empty_re.replace_all(&result, "(Recharge 6)").to_string();

    // Generic fallback for any remaining tags
    let generic_re = regex::Regex::new(r"\{@\w+ ([^}|]+)(?:\|[^}]*)?\}").unwrap();
    result = generic_re.replace_all(&result, "$1").to_string();

    result
}

// Helper extraction functions

fn extract_size(data: &Value) -> String {
    if let Some(size) = data.get("size") {
        if let Some(arr) = size.as_array() {
            let sizes: Vec<&str> = arr.iter().filter_map(|s| s.as_str()).collect();
            return sizes.join("/");
        }
        if let Some(s) = size.as_str() {
            return s.to_string();
        }
    }
    "Medium".to_string()
}

fn extract_creature_type(data: &Value) -> String {
    if let Some(ct) = data.get("type") {
        match ct {
            Value::String(s) => return s.clone(),
            Value::Object(obj) => {
                let base = obj.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
                if let Some(tags) = obj.get("tags") {
                    if let Some(tag_arr) = tags.as_array() {
                        let tag_strs: Vec<String> = tag_arr
                            .iter()
                            .filter_map(|t| {
                                if let Some(s) = t.as_str() {
                                    Some(s.to_string())
                                } else if let Some(obj) = t.as_object() {
                                    obj.get("tag").and_then(|tg| tg.as_str()).map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if !tag_strs.is_empty() {
                            return format!("{} ({})", base, tag_strs.join(", "));
                        }
                    }
                }
                return base.to_string();
            }
            _ => {}
        }
    }
    "unknown".to_string()
}

fn extract_alignment(data: &Value) -> String {
    if let Some(align) = data.get("alignment") {
        if let Some(arr) = align.as_array() {
            let alignments: Vec<String> = arr
                .iter()
                .filter_map(|a| {
                    if let Some(s) = a.as_str() {
                        Some(expand_alignment(s))
                    } else if let Some(obj) = a.as_object() {
                        obj.get("alignment")
                            .and_then(|al| al.as_array())
                            .map(|al_arr| {
                                al_arr
                                    .iter()
                                    .filter_map(|x| x.as_str())
                                    .map(expand_alignment)
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            })
                    } else {
                        None
                    }
                })
                .collect();
            return alignments.join(" ");
        }
    }
    "unaligned".to_string()
}

fn expand_alignment(abbrev: &str) -> String {
    match abbrev {
        "L" => "lawful".to_string(),
        "N" => "neutral".to_string(),
        "C" => "chaotic".to_string(),
        "G" => "good".to_string(),
        "E" => "evil".to_string(),
        "U" => "unaligned".to_string(),
        "A" => "any alignment".to_string(),
        _ => abbrev.to_string(),
    }
}

fn extract_ac(data: &Value) -> String {
    if let Some(ac) = data.get("ac") {
        if let Some(n) = ac.as_u64() {
            return n.to_string();
        }
        if let Some(arr) = ac.as_array() {
            if let Some(first) = arr.first() {
                if let Some(n) = first.as_u64() {
                    return n.to_string();
                }
                if let Some(obj) = first.as_object() {
                    let ac_val = obj.get("ac").and_then(|a| a.as_u64()).unwrap_or(10);
                    if let Some(from) = obj.get("from") {
                        if let Some(from_arr) = from.as_array() {
                            let sources: Vec<&str> =
                                from_arr.iter().filter_map(|f| f.as_str()).collect();
                            return format!("{} ({})", ac_val, sources.join(", "));
                        }
                    }
                    return ac_val.to_string();
                }
            }
        }
    }
    "10".to_string()
}

fn extract_hp(data: &Value) -> String {
    if let Some(hp) = data.get("hp") {
        if let Some(n) = hp.as_u64() {
            return n.to_string();
        }
        if let Some(obj) = hp.as_object() {
            let avg = obj.get("average").and_then(|a| a.as_u64()).unwrap_or(1);
            if let Some(formula) = obj.get("formula").and_then(|f| f.as_str()) {
                return format!("{} ({})", avg, formula);
            }
            return avg.to_string();
        }
    }
    "1".to_string()
}

fn extract_speed(data: &Value) -> String {
    if let Some(speed) = data.get("speed") {
        if let Some(obj) = speed.as_object() {
            let mut speeds = Vec::new();

            if let Some(walk) = obj.get("walk") {
                if let Some(n) = walk.as_u64() {
                    speeds.push(format!("{} ft.", n));
                } else if let Some(w_obj) = walk.as_object() {
                    if let Some(n) = w_obj.get("number").and_then(|x| x.as_u64()) {
                        let cond = w_obj
                            .get("condition")
                            .and_then(|c| c.as_str())
                            .unwrap_or("");
                        speeds.push(format!("{} ft. {}", n, cond).trim().to_string());
                    }
                }
            }

            for (key, label) in [
                ("burrow", "burrow"),
                ("climb", "climb"),
                ("fly", "fly"),
                ("swim", "swim"),
            ] {
                if let Some(val) = obj.get(key) {
                    if let Some(n) = val.as_u64() {
                        speeds.push(format!("{} {} ft.", label, n));
                    } else if let Some(v_obj) = val.as_object() {
                        if let Some(n) = v_obj.get("number").and_then(|x| x.as_u64()) {
                            let cond = v_obj
                                .get("condition")
                                .and_then(|c| c.as_str())
                                .unwrap_or("");
                            speeds.push(format!("{} {} ft. {}", label, n, cond).trim().to_string());
                        }
                    }
                }
            }

            if obj.get("hover").and_then(|h| h.as_bool()).unwrap_or(false)
                || obj.get("canHover").and_then(|h| h.as_bool()).unwrap_or(false)
            {
                // Find fly speed and add hover
                for s in speeds.iter_mut() {
                    if s.contains("fly") && !s.contains("hover") {
                        *s = format!("{} (hover)", s);
                    }
                }
            }

            return speeds.join(", ");
        }
    }
    "30 ft.".to_string()
}

fn get_ability(data: &Value, ability: &str) -> u64 {
    data.get(ability).and_then(|a| a.as_u64()).unwrap_or(10)
}

fn get_modifier(data: &Value, ability: &str) -> String {
    let score = get_ability(data, ability) as i64;
    let modifier = (score - 10) / 2;
    if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("{}", modifier)
    }
}

fn extract_saves(data: &Value) -> Option<String> {
    if let Some(save) = data.get("save") {
        if let Some(obj) = save.as_object() {
            let saves: Vec<String> = ["str", "dex", "con", "int", "wis", "cha"]
                .iter()
                .filter_map(|&s| {
                    obj.get(s)
                        .and_then(|v| v.as_str())
                        .map(|val| format!("{} {}", s.to_uppercase(), val))
                })
                .collect();
            if !saves.is_empty() {
                return Some(saves.join(", "));
            }
        }
    }
    None
}

fn extract_skills(data: &Value) -> Option<String> {
    if let Some(skill) = data.get("skill") {
        if let Some(obj) = skill.as_object() {
            let skills: Vec<String> = obj
                .iter()
                .map(|(k, v)| {
                    let val = v.as_str().unwrap_or("+0");
                    format!("{} {}", capitalize(k), val)
                })
                .collect();
            if !skills.is_empty() {
                return Some(skills.join(", "));
            }
        }
    }
    None
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

fn extract_string_array(data: &Value, field: &str) -> Option<String> {
    if let Some(arr) = data.get(field) {
        if let Some(values) = arr.as_array() {
            let strs: Vec<String> = values
                .iter()
                .filter_map(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(obj) = v.as_object() {
                        // Handle complex damage immunity/resistance objects
                        obj.get("resist")
                            .or(obj.get("immune"))
                            .or(obj.get("vulnerable"))
                            .and_then(|r| r.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|x| x.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                    } else {
                        None
                    }
                })
                .collect();
            if !strs.is_empty() {
                return Some(strs.join(", "));
            }
        }
    }
    None
}

fn extract_senses(data: &Value) -> String {
    let mut senses = Vec::new();

    if let Some(sense_arr) = data.get("senses") {
        if let Some(arr) = sense_arr.as_array() {
            for s in arr {
                if let Some(sense_str) = s.as_str() {
                    senses.push(sense_str.to_string());
                }
            }
        }
    }

    if let Some(passive) = data.get("passive").and_then(|p| p.as_u64()) {
        senses.push(format!("passive Perception {}", passive));
    }

    if senses.is_empty() {
        "passive Perception 10".to_string()
    } else {
        senses.join(", ")
    }
}

fn extract_languages(data: &Value) -> String {
    if let Some(lang) = data.get("languages") {
        if let Some(arr) = lang.as_array() {
            let langs: Vec<&str> = arr.iter().filter_map(|l| l.as_str()).collect();
            if !langs.is_empty() {
                return langs.join(", ");
            }
        }
    }
    "--".to_string()
}

fn extract_cr(data: &Value) -> String {
    if let Some(cr) = data.get("cr") {
        match cr {
            Value::String(s) => return s.clone(),
            Value::Object(obj) => {
                if let Some(c) = obj.get("cr").and_then(|c| c.as_str()) {
                    return c.to_string();
                }
            }
            Value::Number(n) => {
                if let Some(i) = n.as_u64() {
                    return i.to_string();
                }
            }
            _ => {}
        }
    }
    "0".to_string()
}

fn cr_to_xp(cr: &str) -> String {
    match cr {
        "0" => "0 or 10",
        "1/8" => "25",
        "1/4" => "50",
        "1/2" => "100",
        "1" => "200",
        "2" => "450",
        "3" => "700",
        "4" => "1,100",
        "5" => "1,800",
        "6" => "2,300",
        "7" => "2,900",
        "8" => "3,900",
        "9" => "5,000",
        "10" => "5,900",
        "11" => "7,200",
        "12" => "8,400",
        "13" => "10,000",
        "14" => "11,500",
        "15" => "13,000",
        "16" => "15,000",
        "17" => "18,000",
        "18" => "20,000",
        "19" => "22,000",
        "20" => "25,000",
        "21" => "33,000",
        "22" => "41,000",
        "23" => "50,000",
        "24" => "62,000",
        "25" => "75,000",
        "26" => "90,000",
        "27" => "105,000",
        "28" => "120,000",
        "29" => "135,000",
        "30" => "155,000",
        _ => "0",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_formatting_tags() {
        assert_eq!(clean_formatting_tags("{@damage 2d6+3}"), "2d6+3");
        assert_eq!(clean_formatting_tags("{@hit 5}"), "+5");
        assert_eq!(clean_formatting_tags("{@dc 15}"), "DC 15");
        assert_eq!(clean_formatting_tags("{@condition poisoned}"), "poisoned");
        assert_eq!(clean_formatting_tags("{@spell fireball}"), "*fireball*");
    }

    #[test]
    fn test_cr_to_xp() {
        assert_eq!(cr_to_xp("1/4"), "50");
        assert_eq!(cr_to_xp("5"), "1,800");
        assert_eq!(cr_to_xp("20"), "25,000");
    }

    #[test]
    fn test_get_modifier() {
        let data = serde_json::json!({"str": 16, "dex": 8});
        assert_eq!(get_modifier(&data, "str"), "+3");
        assert_eq!(get_modifier(&data, "dex"), "-1");
    }
}
