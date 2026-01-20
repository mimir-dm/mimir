//! Character battle cards section
//!
//! Generates half-page (4" x 5.5") character cards for combat reference.
//! Works for both PCs and NPCs - shows AC, HP, speed, attacks, saves, skills, etc.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Character battle cards - half-page combat reference cards (2x2 layout)
/// Works for both player characters and NPCs
pub struct CharacterBattleCardSection {
    /// Character data (JSON array)
    characters: Vec<Value>,
    /// Show cut lines between cards
    show_cut_lines: bool,
}

impl CharacterBattleCardSection {
    /// Create a new character battle cards section
    pub fn new(characters: Vec<Value>) -> Self {
        Self {
            characters,
            show_cut_lines: true,
        }
    }

    /// Create from a JSON value (expects array)
    pub fn from_json(characters: Value) -> Self {
        let char_vec = characters
            .as_array()
            .map(|arr| arr.clone())
            .unwrap_or_default();
        Self::new(char_vec)
    }

    /// Set whether to show cut lines
    pub fn with_cut_lines(mut self, show: bool) -> Self {
        self.show_cut_lines = show;
        self
    }

    /// Render a single character battle card (half-page format)
    fn render_card(character: &Value) -> String {
        // Try different name fields (character_name for PCs/NPCs, name for generic)
        let name = character
            .get("character_name")
            .or_else(|| character.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        // Role/subtitle (NPC role, or class for PCs)
        let subtitle = get_subtitle(character);

        // Determine card color based on type
        let is_npc = character.get("is_npc").and_then(|v| v.as_bool()).unwrap_or(false);
        let (header_color, accent_color) = if is_npc {
            ("rgb(\"#dbeafe\")", "rgb(\"#2563eb\")") // Blue for NPCs
        } else {
            ("rgb(\"#dcfce7\")", "rgb(\"#16a34a\")") // Green for PCs
        };

        // Core stats
        let ac = get_ac(character);
        let ac_source = get_ac_source(character);
        let hp = get_hp(character);
        let hp_max = get_hp_max(character);
        let speed = get_speed_full(character);

        // HP tracker
        let hp_tracker = render_hp_tracker(hp_max);

        // Ability scores
        let abilities = character.get("abilities");
        let str_score = get_ability(abilities, "strength", character.get("str"));
        let dex_score = get_ability(abilities, "dexterity", character.get("dex"));
        let con_score = get_ability(abilities, "constitution", character.get("con"));
        let int_score = get_ability(abilities, "intelligence", character.get("int"));
        let wis_score = get_ability(abilities, "wisdom", character.get("wis"));
        let cha_score = get_ability(abilities, "charisma", character.get("cha"));

        // Saving throw proficiencies
        let save_profs = get_save_proficiencies(character);

        // Skill proficiencies
        let skills = get_skill_proficiencies(character);

        // Key attacks/actions with full details
        let attacks = get_attacks_full(character);

        // Special abilities/features
        let features = get_key_features(character);

        // Proficiency bonus (estimate from level)
        let level = character.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
        let prof_bonus = ((level - 1) / 4 + 2) as i64;

        // Conditions/status
        let conditions = get_conditions(character);

        // Senses
        let senses = get_senses(character);

        format!(
            r##"box(
  width: 3.875in,
  height: 5.125in,
  stroke: (
    top: 3pt + {accent_color},
    bottom: 3pt + {accent_color},
    left: 0.5pt + colors.border,
    right: 0.5pt + colors.border,
  ),
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // HP Tracker
  #block(
    width: 100%,
    inset: (x: 6pt, y: 4pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    {hp_tracker}
  ]

  // Header
  #block(
    width: 100%,
    fill: {header_color},
    inset: (x: 6pt, y: 4pt),
  )[
    #grid(
      columns: (1fr, auto),
      [
        #text(size: 10pt, weight: "bold")[{name}]
        #linebreak()
        #text(size: 7pt, style: "italic", fill: {accent_color})[{subtitle}]
      ],
      align(right + horizon)[
        #text(size: 12pt, weight: "bold", fill: {accent_color})[Lvl {level}]
      ]
    )
  ]

  // Core stats row
  #block(
    width: 100%,
    inset: 6pt,
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 7pt)
    #grid(
      columns: (1fr, 1fr, 1fr, 1fr),
      [*AC* {ac}{ac_source_text}],
      [*HP* {hp}/{hp_max}],
      [*Prof* +{prof_bonus}],
      [*Init* {init}],
    )
    #v(2pt)
    #text(size: 6pt)[*Speed* {speed}]
  ]

  // Ability scores
  #block(
    width: 100%,
    inset: (x: 6pt, y: 4pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    #grid(
      columns: (1fr,) * 6,
      align(center)[*STR*\ {str_score} ({str_mod})],
      align(center)[*DEX*\ {dex_score} ({dex_mod})],
      align(center)[*CON*\ {con_score} ({con_mod})],
      align(center)[*INT*\ {int_score} ({int_mod})],
      align(center)[*WIS*\ {wis_score} ({wis_mod})],
      align(center)[*CHA*\ {cha_score} ({cha_mod})],
    )
  ]

  // Saves and Senses
  #block(
    width: 100%,
    inset: (x: 6pt, y: 3pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    {saves_line}
    {senses_line}
  ]

  // Skills
  {skills_block}

  // Attacks
  {attacks_block}

  // Features
  {features_block}

  // Conditions
  {conditions_block}

  // Footer
  #place(
    bottom + left,
    block(
      width: 100%,
      fill: {header_color},
      inset: (x: 6pt, y: 2pt),
    )[
      #text(size: 5pt, fill: colors.text-secondary)[
        {footer_text}
      ]
    ]
  )
]"##,
            accent_color = accent_color,
            header_color = header_color,
            hp_tracker = hp_tracker,
            name = escape_typst(name),
            subtitle = escape_typst(&subtitle),
            level = level,
            ac = ac,
            ac_source_text = if ac_source.is_empty() {
                String::new()
            } else {
                format!(" ({})", escape_typst(&ac_source))
            },
            hp = hp,
            hp_max = hp_max,
            prof_bonus = prof_bonus,
            init = modifier(dex_score),
            speed = escape_typst(&speed),
            str_score = str_score,
            str_mod = modifier(str_score),
            dex_score = dex_score,
            dex_mod = modifier(dex_score),
            con_score = con_score,
            con_mod = modifier(con_score),
            int_score = int_score,
            int_mod = modifier(int_score),
            wis_score = wis_score,
            wis_mod = modifier(wis_score),
            cha_score = cha_score,
            cha_mod = modifier(cha_score),
            saves_line = if save_profs.is_empty() {
                String::new()
            } else {
                format!("*Saves* {}\n", escape_typst(&save_profs))
            },
            senses_line = if senses.is_empty() {
                String::new()
            } else {
                format!("*Senses* {}", escape_typst(&senses))
            },
            skills_block = if skills.is_empty() {
                String::new()
            } else {
                format!(
                    r#"#block(
    width: 100%,
    inset: (x: 6pt, y: 3pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    *Skills* {}
  ]"#,
                    escape_typst(&skills)
                )
            },
            attacks_block = if attacks.is_empty() {
                String::new()
            } else {
                format!(
                    r#"#block(
    width: 100%,
    inset: (x: 6pt, y: 3pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    #text(size: 7pt, weight: "bold", fill: {accent_color})[Attacks]
    #v(2pt)
    {}
  ]"#,
                    attacks,
                    accent_color = accent_color,
                )
            },
            features_block = if features.is_empty() {
                String::new()
            } else {
                format!(
                    r#"#block(
    width: 100%,
    inset: (x: 6pt, y: 3pt),
  )[
    #set text(size: 6pt)
    #text(size: 7pt, weight: "bold", fill: {accent_color})[Features]
    #v(2pt)
    {}
  ]"#,
                    features,
                    accent_color = accent_color,
                )
            },
            conditions_block = if conditions.is_empty() {
                String::new()
            } else {
                format!(
                    r#"#block(
    width: 100%,
    inset: (x: 6pt, y: 3pt),
  )[
    #set text(size: 6pt)
    *Conditions:* {}
  ]"#,
                    escape_typst(&conditions)
                )
            },
            footer_text = if is_npc { "NPC" } else { "Player Character" },
        )
    }
}

impl Renderable for CharacterBattleCardSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.characters.is_empty() {
            return Ok(String::new());
        }

        let mut typst = String::new();
        let cards_per_page = 4; // 2x2 grid of half-page cards
        let total_pages = (self.characters.len() + cards_per_page - 1) / cards_per_page;

        // Set page margins for half-page cards (centered with gutters for cutting)
        typst.push_str("#set page(paper: \"us-letter\", margin: 0.25in)\n");

        for page_num in 0..total_pages {
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, self.characters.len());
            let page_chars = &self.characters[start_idx..end_idx];

            if page_num > 0 {
                typst.push_str("\n#pagebreak()\n");
            }

            // Card grid (2x2) - cards sized to fit with gutters for cutting
            typst.push_str("#grid(\n");
            typst.push_str("    columns: (3.875in,) * 2,\n");
            typst.push_str("    rows: (5.125in,) * 2,\n");
            typst.push_str("    column-gutter: 0.25in,\n");
            typst.push_str("    row-gutter: 0.25in,\n\n");

            // Render each card
            for (i, character) in page_chars.iter().enumerate() {
                typst.push_str("    ");
                typst.push_str(&Self::render_card(character));
                if i < page_chars.len() - 1 || page_chars.len() < 4 {
                    typst.push(',');
                }
                typst.push('\n');
            }

            // Fill remaining slots with empty boxes
            for _ in page_chars.len()..4 {
                typst.push_str("    box(width: 3.875in, height: 5.125in),\n");
            }

            typst.push_str(")\n");

            // Cut lines indicator
            if self.show_cut_lines && !page_chars.is_empty() {
                typst.push_str("#place(\n  bottom + center,\n  dy: 0.1in,\n");
                typst.push_str(
                    "  text(size: 6pt, fill: colors.text-secondary)[Cut along card borders]\n)\n",
                );
            }
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.characters.is_empty() {
            None
        } else {
            Some("Character Battle Cards".to_string())
        }
    }
}

/// Get subtitle (class info or NPC role)
fn get_subtitle(character: &Value) -> String {
    // Try NPC role first
    if let Some(role) = character.get("npc_role").and_then(|v| v.as_str()) {
        if !role.is_empty() {
            return role.to_string();
        }
    }

    // Try class info
    if let Some(classes) = character.get("classes").and_then(|v| v.as_array()) {
        let class_str: Vec<String> = classes
            .iter()
            .filter_map(|c| {
                let class_name = c.get("class_name").and_then(|v| v.as_str())?;
                let level = c.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
                let subclass = c.get("subclass_name").and_then(|v| v.as_str()).unwrap_or("");
                if subclass.is_empty() {
                    Some(format!("{} {}", class_name, level))
                } else {
                    Some(format!("{} ({}) {}", class_name, subclass, level))
                }
            })
            .collect();
        if !class_str.is_empty() {
            return class_str.join(" / ");
        }
    }

    // Try race
    if let Some(race) = character.get("race").and_then(|v| v.as_str()) {
        return race.to_string();
    }

    "Adventurer".to_string()
}

/// Get AC value
fn get_ac(character: &Value) -> i64 {
    character
        .get("armor_class")
        .or_else(|| character.get("ac"))
        .and_then(|v| v.as_i64())
        .unwrap_or(10)
}

/// Get AC source (armor type)
fn get_ac_source(character: &Value) -> String {
    // Check for equipped armor
    if let Some(equipment) = character.get("equipment").and_then(|v| v.as_array()) {
        for item in equipment {
            if item.get("equipped").and_then(|v| v.as_bool()).unwrap_or(false) {
                if let Some(item_type) = item.get("item_type").and_then(|v| v.as_str()) {
                    if item_type == "armor" || item_type == "shield" {
                        if let Some(name) = item.get("item_name").and_then(|v| v.as_str()) {
                            return name.to_string();
                        }
                    }
                }
            }
        }
    }
    String::new()
}

/// Get current HP value
fn get_hp(character: &Value) -> i64 {
    character
        .get("current_hp")
        .or_else(|| character.get("hp"))
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| get_hp_max(character))
}

/// Get max HP value
fn get_hp_max(character: &Value) -> i64 {
    character
        .get("max_hp")
        .or_else(|| character.get("hp_max"))
        .and_then(|v| v.as_i64())
        .unwrap_or(10)
}

/// Get full speed value including all movement types
fn get_speed_full(character: &Value) -> String {
    character
        .get("speed")
        .map(|s| {
            if let Some(n) = s.as_i64() {
                format!("{} ft.", n)
            } else if let Some(obj) = s.as_object() {
                let mut parts = Vec::new();

                if let Some(walk) = obj.get("walk").and_then(|v| v.as_i64()) {
                    parts.push(format!("{} ft.", walk));
                }
                if let Some(fly) = obj.get("fly").and_then(|v| v.as_i64()) {
                    parts.push(format!("fly {} ft.", fly));
                }
                if let Some(swim) = obj.get("swim").and_then(|v| v.as_i64()) {
                    parts.push(format!("swim {} ft.", swim));
                }
                if let Some(climb) = obj.get("climb").and_then(|v| v.as_i64()) {
                    parts.push(format!("climb {} ft.", climb));
                }
                if let Some(burrow) = obj.get("burrow").and_then(|v| v.as_i64()) {
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
        })
        .unwrap_or_else(|| "30 ft.".to_string())
}

/// Get ability score from either nested abilities or flat field
fn get_ability(abilities: Option<&Value>, ability_name: &str, flat_field: Option<&Value>) -> i64 {
    abilities
        .and_then(|a| a.get(ability_name))
        .and_then(|v| v.as_i64())
        .or_else(|| flat_field.and_then(|v| v.as_i64()))
        .unwrap_or(10)
}

/// Get saving throw proficiencies with bonuses
fn get_save_proficiencies(character: &Value) -> String {
    let mut saves = Vec::new();
    let level = character.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
    let prof_bonus = ((level - 1) / 4 + 2) as i64;

    let abilities = character.get("abilities");

    // Check saves object
    if let Some(saves_obj) = character.get("saves").and_then(|v| v.as_object()) {
        for (key, _) in saves_obj {
            let (abbrev, ability_key) = match key.to_lowercase().as_str() {
                "str" | "strength" => ("Str", "strength"),
                "dex" | "dexterity" => ("Dex", "dexterity"),
                "con" | "constitution" => ("Con", "constitution"),
                "int" | "intelligence" => ("Int", "intelligence"),
                "wis" | "wisdom" => ("Wis", "wisdom"),
                "cha" | "charisma" => ("Cha", "charisma"),
                _ => continue,
            };
            let ability_mod = (get_ability(abilities, ability_key, None) - 10) / 2;
            let total = ability_mod + prof_bonus;
            saves.push(format!("{} {:+}", abbrev, total));
        }
    }

    // Check saving_throw_proficiencies array
    if saves.is_empty() {
        if let Some(profs) = character
            .get("saving_throw_proficiencies")
            .and_then(|v| v.as_array())
        {
            for prof in profs {
                if let Some(s) = prof.as_str() {
                    let (abbrev, ability_key) = match s.to_lowercase().as_str() {
                        "str" | "strength" => ("Str", "strength"),
                        "dex" | "dexterity" => ("Dex", "dexterity"),
                        "con" | "constitution" => ("Con", "constitution"),
                        "int" | "intelligence" => ("Int", "intelligence"),
                        "wis" | "wisdom" => ("Wis", "wisdom"),
                        "cha" | "charisma" => ("Cha", "charisma"),
                        _ => continue,
                    };
                    let ability_mod = (get_ability(abilities, ability_key, None) - 10) / 2;
                    let total = ability_mod + prof_bonus;
                    saves.push(format!("{} {:+}", abbrev, total));
                }
            }
        }
    }

    saves.join(", ")
}

/// Get skill proficiencies with bonuses
fn get_skill_proficiencies(character: &Value) -> String {
    let mut skills = Vec::new();
    let level = character.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
    let prof_bonus = ((level - 1) / 4 + 2) as i64;
    let abilities = character.get("abilities");

    // Skill to ability mapping
    let skill_map = [
        ("acrobatics", "dexterity"),
        ("animal_handling", "wisdom"),
        ("arcana", "intelligence"),
        ("athletics", "strength"),
        ("deception", "charisma"),
        ("history", "intelligence"),
        ("insight", "wisdom"),
        ("intimidation", "charisma"),
        ("investigation", "intelligence"),
        ("medicine", "wisdom"),
        ("nature", "intelligence"),
        ("perception", "wisdom"),
        ("performance", "charisma"),
        ("persuasion", "charisma"),
        ("religion", "intelligence"),
        ("sleight_of_hand", "dexterity"),
        ("stealth", "dexterity"),
        ("survival", "wisdom"),
    ];

    // Check skill_proficiencies array
    if let Some(profs) = character
        .get("skill_proficiencies")
        .and_then(|v| v.as_array())
    {
        for prof in profs.iter().take(6) {
            // Limit to 6 skills for card space
            if let Some(skill_name) = prof.as_str() {
                let skill_lower = skill_name.to_lowercase().replace(' ', "_");
                if let Some((_, ability)) = skill_map
                    .iter()
                    .find(|(s, _)| *s == skill_lower.as_str())
                {
                    let ability_mod = (get_ability(abilities, ability, None) - 10) / 2;
                    let total = ability_mod + prof_bonus;
                    let display_name = skill_name.split('_').next().unwrap_or(skill_name);
                    let capitalized =
                        display_name[..1].to_uppercase() + &display_name[1..].to_lowercase();
                    skills.push(format!("{} {:+}", capitalized, total));
                }
            }
        }
    }

    skills.join(", ")
}

/// Get senses (darkvision, etc.)
fn get_senses(character: &Value) -> String {
    let mut senses = Vec::new();

    // Check for darkvision from race
    if let Some(dv) = character.get("darkvision").and_then(|v| v.as_i64()) {
        senses.push(format!("darkvision {} ft.", dv));
    }

    // Check senses array
    if let Some(sense_arr) = character.get("senses").and_then(|v| v.as_array()) {
        for sense in sense_arr {
            if let Some(s) = sense.as_str() {
                senses.push(s.to_string());
            }
        }
    }

    // Passive perception
    let abilities = character.get("abilities");
    let wis_mod = (get_ability(abilities, "wisdom", character.get("wis")) - 10) / 2;
    let passive = 10 + wis_mod;
    senses.push(format!("PP {}", passive));

    senses.join(", ")
}

/// Get attacks with full details
fn get_attacks_full(character: &Value) -> String {
    let mut attacks = Vec::new();
    let level = character.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
    let prof_bonus = ((level - 1) / 4 + 2) as i64;
    let abilities = character.get("abilities");
    let str_mod = (get_ability(abilities, "strength", character.get("str")) - 10) / 2;
    let dex_mod = (get_ability(abilities, "dexterity", character.get("dex")) - 10) / 2;

    // Check attacks array
    if let Some(attack_arr) = character.get("attacks").and_then(|v| v.as_array()) {
        for attack in attack_arr.iter().take(4) {
            if let Some(name) = attack.get("name").and_then(|v| v.as_str()) {
                let to_hit = attack
                    .get("to_hit")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(prof_bonus + str_mod);
                let damage = attack
                    .get("damage")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1d6");
                let damage_type = attack
                    .get("damage_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let attack_line = if damage_type.is_empty() {
                    format!("*{}.* {:+} to hit, {} damage", escape_typst(name), to_hit, damage)
                } else {
                    format!(
                        "*{}.* {:+} to hit, {} {} damage",
                        escape_typst(name),
                        to_hit,
                        damage,
                        damage_type
                    )
                };
                attacks.push(attack_line);
            }
        }
    }

    // Check weapons in equipment if no attacks defined
    if attacks.is_empty() {
        if let Some(equipment) = character.get("equipment").and_then(|v| v.as_array()) {
            for item in equipment.iter().take(4) {
                if let Some(item_name) = item.get("item_name").and_then(|v| v.as_str()) {
                    let lower = item_name.to_lowercase();
                    if lower.contains("sword")
                        || lower.contains("bow")
                        || lower.contains("axe")
                        || lower.contains("mace")
                        || lower.contains("dagger")
                        || lower.contains("staff")
                        || lower.contains("crossbow")
                        || lower.contains("hammer")
                        || lower.contains("spear")
                    {
                        // Estimate attack stats
                        let is_finesse = lower.contains("rapier")
                            || lower.contains("dagger")
                            || lower.contains("shortsword");
                        let is_ranged =
                            lower.contains("bow") || lower.contains("crossbow");
                        let ability_mod = if is_finesse || is_ranged {
                            dex_mod
                        } else {
                            str_mod
                        };
                        let to_hit = prof_bonus + ability_mod;

                        attacks.push(format!("*{}.* {:+} to hit", escape_typst(item_name), to_hit));
                    }
                }
            }
        }
    }

    if attacks.is_empty() {
        String::new()
    } else {
        attacks.join("\n#v(2pt)\n")
    }
}

/// Get key features/abilities
fn get_key_features(character: &Value) -> String {
    let mut features = Vec::new();

    // Check features array
    if let Some(feature_arr) = character.get("features").and_then(|v| v.as_array()) {
        for feature in feature_arr.iter().take(3) {
            if let Some(name) = feature.get("name").and_then(|v| v.as_str()) {
                features.push(format!("*{}*", escape_typst(name)));
            } else if let Some(s) = feature.as_str() {
                features.push(format!("*{}*", escape_typst(s)));
            }
        }
    }

    // Check class features
    if features.is_empty() {
        if let Some(classes) = character.get("classes").and_then(|v| v.as_array()) {
            for class in classes {
                if let Some(class_features) = class.get("features").and_then(|v| v.as_array()) {
                    for feature in class_features.iter().take(3 - features.len()) {
                        if let Some(name) = feature.get("name").and_then(|v| v.as_str()) {
                            features.push(format!("*{}*", escape_typst(name)));
                        }
                    }
                }
            }
        }
    }

    if features.is_empty() {
        String::new()
    } else {
        features.join(", ")
    }
}

/// Get current conditions
fn get_conditions(character: &Value) -> String {
    if let Some(conditions) = character.get("conditions").and_then(|v| v.as_array()) {
        conditions
            .iter()
            .filter_map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        String::new()
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

/// Render HP tracker based on HP value
/// - 1-20 HP: Individual boxes with styled 10th box
/// - 21-100 HP: 5s and 1s grouping
/// - 101+ HP: 10s and 1s grouping
fn render_hp_tracker(hp: i64) -> String {
    match hp {
        1..=20 => render_individual_boxes(hp),
        21..=100 => render_fives_and_ones(hp),
        _ => render_tens_and_ones(hp),
    }
}

/// Render individual HP boxes (1-20 HP)
/// Every 10th box is styled differently for easier counting
fn render_individual_boxes(hp: i64) -> String {
    let mut boxes = Vec::new();
    for i in 1..=hp {
        if i % 10 == 0 {
            // Styled 10th box - slightly larger with fill
            boxes.push(r##"#box(width: 7pt, height: 7pt, stroke: 1pt + black, fill: rgb("#e5e5e5"))"##.to_string());
        } else {
            // Regular box
            boxes.push(r##"#box(width: 6pt, height: 6pt, stroke: 0.5pt + black)"##.to_string());
        }
    }
    boxes.join("#h(1pt)")
}

/// Render 5-HP boxes plus 1-HP boxes (for 21-100 HP)
fn render_fives_and_ones(hp: i64) -> String {
    let fives = hp / 5;
    let ones = hp % 5;

    let mut parts = Vec::new();

    // 5-HP boxes with "5" label
    for i in 0..fives {
        // Every other 5-box (at 10, 20, 30...) gets extra styling
        if (i + 1) % 2 == 0 {
            parts.push(r##"#box(width: 10pt, height: 10pt, stroke: 1pt + black, fill: rgb("#d4d4d4"))[#align(center + horizon)[#text(size: 5pt, weight: "bold")[5]]]"##.to_string());
        } else {
            parts.push(r##"#box(width: 10pt, height: 10pt, stroke: 0.75pt + black, fill: rgb("#f5f5f5"))[#align(center + horizon)[#text(size: 5pt)[5]]]"##.to_string());
        }
    }

    // Add spacing before 1-HP boxes if we have both
    if fives > 0 && ones > 0 {
        parts.push("#h(3pt)".to_string());
    }

    // 1-HP boxes
    for _ in 0..ones {
        parts.push(r##"#box(width: 6pt, height: 6pt, stroke: 0.5pt + black)"##.to_string());
    }

    parts.join("#h(1pt)")
}

/// Render 10-HP boxes plus 1-HP boxes (for 101+ HP)
fn render_tens_and_ones(hp: i64) -> String {
    let tens = hp / 10;
    let ones = hp % 10;

    let mut parts = Vec::new();

    // 10-HP boxes with "10" label
    for i in 0..tens {
        // Every 5th ten-box (at 50, 100, 150...) gets extra styling
        if (i + 1) % 5 == 0 {
            parts.push(r##"#box(width: 12pt, height: 10pt, stroke: 1.5pt + black, fill: rgb("#c4c4c4"))[#align(center + horizon)[#text(size: 5pt, weight: "bold")[10]]]"##.to_string());
        } else {
            parts.push(r##"#box(width: 12pt, height: 10pt, stroke: 0.75pt + black, fill: rgb("#e5e5e5"))[#align(center + horizon)[#text(size: 5pt)[10]]]"##.to_string());
        }
    }

    // Add spacing before 1-HP boxes if we have both
    if tens > 0 && ones > 0 {
        parts.push("#h(3pt)".to_string());
    }

    // 1-HP boxes
    for _ in 0..ones {
        parts.push(r##"#box(width: 6pt, height: 6pt, stroke: 0.5pt + black)"##.to_string());
    }

    parts.join("#h(1pt)")
}

/// Escape special Typst characters
fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('"', "\\\"")
        .replace('_', "\\_")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('@', "\\@")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_character_cards_empty() {
        let section = CharacterBattleCardSection::new(vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_character_cards_with_characters() {
        let chars = vec![json!({
            "character_name": "Gandalf",
            "level": 20,
            "is_npc": false
        })];
        let section = CharacterBattleCardSection::new(chars);
        assert_eq!(
            section.toc_title(),
            Some("Character Battle Cards".to_string())
        );
    }

    #[test]
    fn test_npc_card() {
        let chars = vec![json!({
            "character_name": "Bartender Bob",
            "npc_role": "Tavern Owner",
            "level": 3,
            "is_npc": true
        })];
        let section = CharacterBattleCardSection::new(chars);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // NPC should have blue header
        assert!(typst.contains("dbeafe"));
    }

    #[test]
    fn test_modifier() {
        assert_eq!(modifier(10), "+0");
        assert_eq!(modifier(18), "+4");
        assert_eq!(modifier(8), "-1");
    }

    #[test]
    fn test_get_subtitle() {
        // NPC with role
        let npc = json!({"npc_role": "Blacksmith"});
        assert_eq!(get_subtitle(&npc), "Blacksmith");

        // PC with class
        let pc = json!({
            "classes": [{"class_name": "Fighter", "level": 5}]
        });
        assert_eq!(get_subtitle(&pc), "Fighter 5");
    }

    #[test]
    fn test_half_page_format() {
        let chars = vec![json!({"character_name": "Test"})];
        let section = CharacterBattleCardSection::new(chars);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Verify half-page dimensions and grid layout
        assert!(typst.contains("width: 3.875in"));
        assert!(typst.contains("height: 5.125in"));
        assert!(typst.contains("columns: (3.875in,) * 2"));
        assert!(typst.contains("#grid("));
    }
}
