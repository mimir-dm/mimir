//! Character sheet section
//!
//! Generates character sheets from the simplified mimir Character model.
//! Shows basic info, ability scores, classes, and roleplay elements.

use serde::{Deserialize, Serialize};

use crate::builder::{escape_typst_string, RenderContext, Renderable};
use crate::error::Result;

/// Character data for PDF rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterData {
    pub name: String,
    pub player_name: Option<String>,
    pub is_npc: bool,

    // Race and background
    pub race_name: Option<String>,
    pub background_name: Option<String>,

    // Ability scores
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,

    // Currency
    pub cp: i32,
    pub sp: i32,
    pub ep: i32,
    pub gp: i32,
    pub pp: i32,

    // Roleplay elements
    pub traits: Option<String>,
    pub ideals: Option<String>,
    pub bonds: Option<String>,
    pub flaws: Option<String>,

    // NPC-specific
    pub role: Option<String>,
    pub location: Option<String>,
    pub faction: Option<String>,

    // Classes (from CharacterClass table)
    pub classes: Vec<ClassInfo>,

    // Inventory summary
    pub inventory: Vec<InventoryItem>,

    // Proficiencies (grouped by type)
    pub proficiencies: Proficiencies,

    // Speed (from race, defaults to 30)
    pub speed: i32,

    // Armor Class (computed from equipped armor + DEX)
    pub ac: i32,

    // Hit points and hit dice
    pub hit_points_max: i32,
    pub hit_die: String, // e.g. "5d10 + 3d8"

    // Spellcasting (None if non-caster)
    pub spellcasting_ability: Option<String>, // "WIS" / "INT" / "CHA"
    pub spell_save_dc: Option<i32>,
    pub spell_attack_bonus: Option<i32>,
    pub spell_slots: Vec<i32>, // 9 entries for spell levels 1-9
}

/// Class level information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub class_name: String,
    pub level: i32,
    pub subclass_name: Option<String>,
    pub is_starting: bool,
}

/// Inventory item for display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InventoryItem {
    pub name: String,
    pub quantity: i32,
    pub equipped: bool,
    pub attuned: bool,
    /// Item type code from catalog (e.g., "M" = melee weapon, "HA" = heavy armor, "S" = shield)
    #[serde(default)]
    pub item_type: Option<String>,
    /// Damage dice (e.g., "1d8") for weapons
    #[serde(default)]
    pub damage: Option<String>,
    /// Damage type code (e.g., "S" = slashing) for weapons
    #[serde(default)]
    pub damage_type: Option<String>,
    /// Base AC for armor/shields
    #[serde(default)]
    pub armor_ac: Option<i32>,
    /// Whether this is a finesse weapon
    #[serde(default)]
    pub finesse: bool,
}

/// Proficiencies grouped by type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Proficiencies {
    pub skills: Vec<ProficiencyEntry>,
    pub saves: Vec<String>,
    pub languages: Vec<String>,
    pub armor: Vec<String>,
    pub weapons: Vec<String>,
    pub tools: Vec<String>,
}

/// A proficiency entry that may have expertise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProficiencyEntry {
    pub name: String,
    pub expertise: bool,
}

/// Character sheet section
pub struct CharacterSection {
    character: CharacterData,
}

impl CharacterSection {
    /// Create a new character section
    pub fn new(character: CharacterData) -> Self {
        Self { character }
    }

    /// Calculate total character level
    fn total_level(&self) -> i32 {
        self.character.classes.iter().map(|c| c.level).sum()
    }

    /// Calculate proficiency bonus from total level
    fn prof_bonus(&self) -> i32 {
        let level = self.total_level();
        if level <= 4 { 2 }
        else if level <= 8 { 3 }
        else if level <= 12 { 4 }
        else if level <= 16 { 5 }
        else { 6 }
    }

    /// Format class string (e.g., "Fighter 5 / Rogue 3")
    fn class_string(&self) -> String {
        if self.character.classes.is_empty() {
            return "No Class".to_string();
        }

        self.character.classes
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

    /// Calculate ability modifier
    fn modifier(score: i32) -> i32 {
        (score - 10).div_euclid(2)
    }

    /// Format modifier as string with sign
    fn modifier_str(score: i32) -> String {
        let m = Self::modifier(score);
        if m >= 0 {
            format!("+{}", m)
        } else {
            format!("{}", m)
        }
    }

    /// Calculate total gold value
    #[allow(dead_code)]
    fn total_gold(&self) -> f64 {
        (self.character.cp as f64 / 100.0)
            + (self.character.sp as f64 / 10.0)
            + (self.character.ep as f64 / 2.0)
            + (self.character.gp as f64)
            + (self.character.pp as f64 * 10.0)
    }

}

impl Renderable for CharacterSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let mut typst = String::new();

        let char = &self.character;
        let level = self.total_level();
        let prof = self.prof_bonus();
        let classes = self.class_string();

        // =====================================================================
        // HELPERS — Three tiers of section treatment
        // =====================================================================

        // Primary: heavier border, prominent header (Combat, Abilities, Personality)
        fn primary_box(title: &str, content: &str, markup: bool) -> String {
            let outer = if markup { "#" } else { "" };
            format!(
                r#"{outer}block(width: 100%, stroke: 1pt + colors.border, radius: 2pt, clip: true)[
#block(width: 100%, fill: luma(60), inset: (x: spacing.sm, y: spacing.xs))[
  #text(fill: white, weight: "bold", size: sizes.base, tracking: 1pt)[{title}]
]
  #block(width: 100%, inset: spacing.sm)[
{content}  ]
]
"#,
                outer = outer,
                title = title,
                content = content,
            )
        }

        // Secondary: standard border, mid-gray header (Weapons, Proficiencies, Spellcasting)
        fn secondary_box(title: &str, content: &str, markup: bool) -> String {
            let outer = if markup { "#" } else { "" };
            format!(
                r#"{outer}block(width: 100%, stroke: 0.5pt + colors.border-light, radius: 2pt, clip: true)[
#block(width: 100%, fill: luma(60), inset: (x: spacing.sm, y: spacing.xs))[
  #text(fill: white, weight: "bold", size: sizes.sm, tracking: 0.5pt)[{title}]
]
  #block(width: 100%, inset: spacing.sm)[
{content}  ]
]
"#,
                outer = outer,
                title = title,
                content = content,
            )
        }

        // Tertiary: minimal — just a label and a line, no box (Notes, Currency)
        fn tertiary_label(title: &str, content: &str, markup: bool) -> String {
            let outer = if markup { "#" } else { "" };
            format!(
                r#"{outer}block(width: 100%)[
  #text(weight: "bold", size: sizes.sm, tracking: 0.5pt, fill: colors.text-secondary)[{title}]
  #v(spacing.xs)
  #line(length: 100%, stroke: 0.5pt + colors.border-light)
  #v(spacing.xs)
{content}]
"#,
                outer = outer,
                title = title,
                content = content,
            )
        }

        // Ability block: compact inline — no box border, just header line + skills
        let ability_block = |abbrev: &str,
                             full_name: &str,
                             score: i32,
                             skills: &[(&str, &str)]|
         -> String {
            let modifier = Self::modifier(score);
            let mod_str = Self::modifier_str(score);
            let is_save_prof = char.proficiencies.saves.iter().any(|s| s == full_name);
            let save_total = modifier + if is_save_prof { prof } else { 0 };
            let save_sign = if save_total >= 0 { "+" } else { "" };
            let save_bullet = if is_save_prof { "●" } else { "○" };
            let save_weight = if is_save_prof {
                ", weight: \"bold\""
            } else {
                ""
            };

            let mut s = String::new();

            // Compact header: ABR score (mod) — Save ±N  all on one line
            s.push_str(&format!(
                r#"    block(width: 100%, inset: (x: 0pt, y: 0pt))[
      #grid(columns: (auto, 1fr, auto), column-gutter: spacing.xs,
        text(weight: "bold", size: sizes.sm, tracking: 0.5pt)[{abbrev}],
        text(size: sizes.sm)[{score} #text(fill: colors.text-secondary, size: sizes.xs)[({mod_str})]],
        text(size: sizes.xs{save_weight})[{save_bullet} Save {save_sign}{save_total}],
      )
      #line(length: 100%, stroke: 0.5pt + colors.border-light)
"#,
                abbrev = abbrev,
                score = score,
                mod_str = mod_str,
                save_weight = save_weight,
                save_bullet = save_bullet,
                save_sign = save_sign,
                save_total = save_total,
            ));

            // Skills — compact list, no rules between
            for (skill_name, _ability) in skills.iter() {
                let ability_mod = Self::modifier(score);
                let prof_entry = char
                    .proficiencies
                    .skills
                    .iter()
                    .find(|p| &p.name == skill_name);
                let is_prof = prof_entry.is_some();
                let is_expert = prof_entry.map_or(false, |p| p.expertise);
                let bonus = ability_mod
                    + if is_expert {
                        prof * 2
                    } else if is_prof {
                        prof
                    } else {
                        0
                    };
                let sign = if bonus >= 0 { "+" } else { "" };
                let bullet = if is_expert {
                    "◆"
                } else if is_prof {
                    "●"
                } else {
                    "○"
                };
                let weight = if is_prof || is_expert {
                    ", weight: \"bold\""
                } else {
                    ""
                };
                s.push_str(&format!(
                    "      #text(size: sizes.xs{})[{} {} #h(1fr) {}{}]\n",
                    weight, bullet, skill_name, sign, bonus
                ));
            }

            s.push_str("    ]\n    v(spacing.xs)\n");
            s
        };

        // =================================================================
        // PAGE 1: Stats & Combat
        // =================================================================

        // --- HEADER: Large uppercase name ---
        let race = char.race_name.as_deref().unwrap_or("");
        let bg = char.background_name.as_deref().unwrap_or("");
        let player_line = char.player_name.as_deref().unwrap_or("");

        typst.push_str(&format!(
            r#"// Character: {name}
#block(width: 100%, inset: (x: spacing.sm, y: spacing.sm), stroke: (bottom: 2pt + colors.accent))[
  #grid(columns: (1fr, auto), column-gutter: spacing.sm,
    text(size: sizes.title, weight: "bold", font: font-heading, tracking: -0.5pt)[{name_upper}],
    {npc_badge}
  )
  #v(spacing.xs)
  #text(size: sizes.base)[{race} · {classes} · Level {level}]
  #h(spacing.md)
  #text(size: sizes.sm, fill: colors.text-secondary)[{bg}{player_sep}{player}]
]

#v(spacing.sm)

"#,
            name = escape_typst_string(&char.name),
            name_upper = escape_typst_string(&char.name.to_uppercase()),
            npc_badge = if char.is_npc {
                r#"box(fill: colors.accent, radius: 2pt, inset: (x: spacing.sm, y: spacing.xs))[
      #text(fill: white, weight: "bold", size: sizes.sm, tracking: 1pt)[NPC]
    ]"#
            } else {
                ""
            },
            race = escape_typst_string(race),
            classes = escape_typst_string(&classes),
            level = level,
            bg = escape_typst_string(bg),
            player_sep = if !bg.is_empty() && !player_line.is_empty() {
                " · "
            } else {
                ""
            },
            player = if !player_line.is_empty() {
                format!("Player: {}", escape_typst_string(player_line))
            } else {
                String::new()
            },
        ));

        // --- COMBAT STATS: 2-row × 4-column grid, AC+HP dominant ---
        let passive_perception = {
            let wis_mod = Self::modifier(char.wisdom);
            let perc_prof = char
                .proficiencies
                .skills
                .iter()
                .find(|s| s.name == "Perception");
            let perc_bonus = wis_mod
                + if perc_prof.map_or(false, |p| p.expertise) {
                    prof * 2
                } else if perc_prof.is_some() {
                    prof
                } else {
                    0
                };
            10 + perc_bonus
        };

        let combat_content = format!(
            r#"    #grid(columns: (1fr, 1fr, 1fr, 1fr), column-gutter: spacing.sm, row-gutter: spacing.sm,
      [#align(center)[#text(size: sizes.xxl, weight: "bold")[{ac}] #linebreak() #label-text("ARMOR CLASS")]],
      [#align(center)[#text(size: sizes.xxl, weight: "bold")[{hp}] #linebreak() #label-text("HIT POINTS")]],
      [#align(center)[#text(size: sizes.xl, weight: "bold")[{init}] #linebreak() #label-text("INITIATIVE")]],
      [#align(center)[#text(size: sizes.xl, weight: "bold")[{speed} ft] #linebreak() #label-text("SPEED")]],
      [#align(center)[#text(size: sizes.lg, weight: "bold")[+{prof}] #linebreak() #label-text("PROF BONUS")]],
      [#align(center)[#text(size: sizes.lg, weight: "bold")[{hit_die}] #linebreak() #label-text("HIT DICE")]],
      [#align(center)[#text(size: sizes.lg, weight: "bold")[{pp}] #linebreak() #label-text("PASSIVE")]],
      [#align(center)[
        #grid(columns: (1fr, 1fr, 1fr), column-gutter: 2pt,
          box(width: 100%, stroke: 0.5pt + colors.border-light, radius: 2pt, inset: spacing.xs)[#align(center)[#text(size: sizes.xs, fill: luma(200))[S]]],
          box(width: 100%, stroke: 0.5pt + colors.border-light, radius: 2pt, inset: spacing.xs)[#align(center)[#text(size: sizes.xs, fill: luma(200))[S]]],
          box(width: 100%, stroke: 0.5pt + colors.border-light, radius: 2pt, inset: spacing.xs)[#align(center)[#text(size: sizes.xs, fill: luma(200))[F]]],
        )
        #label-text("DEATH SAVES")
      ]],
    )
"#,
            ac = char.ac,
            hp = char.hit_points_max,
            init = Self::modifier_str(char.dexterity),
            speed = char.speed,
            prof = prof,
            hit_die = escape_typst_string(&char.hit_die),
            pp = passive_perception,
        );
        typst.push_str(&primary_box("COMBAT", &combat_content, true));
        typst.push_str("#v(spacing.sm)\n\n");

        // --- MAIN TWO-COLUMN LAYOUT (35% | 65%) ---
        typst.push_str("#grid(\n  columns: (35%, 1fr),\n  column-gutter: spacing.md,\n\n");

        // ===== LEFT COLUMN: Ability Score Blocks =====
        typst.push_str("  // Ability Scores with grouped skills\n  {\n");

        typst.push_str(&ability_block("STR", "Strength", char.strength, &[("Athletics", "STR")]));
        typst.push_str(&ability_block("DEX", "Dexterity", char.dexterity, &[
            ("Acrobatics", "DEX"), ("Sleight of Hand", "DEX"), ("Stealth", "DEX"),
        ]));
        typst.push_str(&ability_block("CON", "Constitution", char.constitution, &[]));
        typst.push_str(&ability_block("INT", "Intelligence", char.intelligence, &[
            ("Arcana", "INT"), ("History", "INT"), ("Investigation", "INT"),
            ("Nature", "INT"), ("Religion", "INT"),
        ]));
        typst.push_str(&ability_block("WIS", "Wisdom", char.wisdom, &[
            ("Animal Handling", "WIS"), ("Insight", "WIS"), ("Medicine", "WIS"),
            ("Perception", "WIS"), ("Survival", "WIS"),
        ]));
        typst.push_str(&ability_block("CHA", "Charisma", char.charisma, &[
            ("Deception", "CHA"), ("Intimidation", "CHA"),
            ("Performance", "CHA"), ("Persuasion", "CHA"),
        ]));

        typst.push_str("  },\n\n");

        // ===== RIGHT COLUMN: Weapons + Proficiencies + Your Turn + Conditions =====
        typst.push_str("  // Weapons, Proficiencies, Actions\n  {\n");

        // Weapons & Damage table
        let weapons: Vec<_> = char
            .inventory
            .iter()
            .filter(|i| i.equipped && matches!(i.item_type.as_deref(), Some("M") | Some("R")))
            .collect();

        if !weapons.is_empty() {
            let mut wpn_content = String::new();
            wpn_content.push_str(
                "    #grid(columns: (1fr, auto, auto), column-gutter: spacing.md, row-gutter: spacing.xs,\n",
            );
            wpn_content.push_str("      text(size: sizes.xs, weight: \"bold\", tracking: 0.5pt)[NAME], text(size: sizes.xs, weight: \"bold\", tracking: 0.5pt)[ATK], text(size: sizes.xs, weight: \"bold\", tracking: 0.5pt)[DAMAGE],\n");

            for item in &weapons {
                let str_mod = Self::modifier(char.strength);
                let dex_mod_val = Self::modifier(char.dexterity);
                let is_ranged = item.item_type.as_deref() == Some("R");
                let is_finesse = item.finesse;

                let atk_mod = if is_ranged {
                    dex_mod_val + prof
                } else if is_finesse {
                    std::cmp::max(str_mod, dex_mod_val) + prof
                } else {
                    str_mod + prof
                };
                let dmg_mod = if is_ranged {
                    dex_mod_val
                } else if is_finesse {
                    std::cmp::max(str_mod, dex_mod_val)
                } else {
                    str_mod
                };

                let atk_sign = if atk_mod >= 0 { "+" } else { "" };
                let dmg_suffix = if dmg_mod != 0 {
                    let dmg_sign = if dmg_mod > 0 { "+" } else { "" };
                    format!("{}{}", dmg_sign, dmg_mod)
                } else {
                    String::new()
                };
                let dmg_str = if let Some(ref dice) = item.damage {
                    format!("{}{}", dice, dmg_suffix)
                } else {
                    format!("1{}", dmg_suffix)
                };

                let dmg_type = item
                    .damage_type
                    .as_deref()
                    .map(|t| match t {
                        "S" => " slashing",
                        "P" => " piercing",
                        "B" => " bludgeoning",
                        _ => "",
                    })
                    .unwrap_or("");

                wpn_content.push_str(&format!(
                    "      text(size: sizes.sm, weight: \"bold\")[{}], text(size: sizes.sm)[{}{}], text(size: sizes.sm)[{}{}],\n",
                    escape_typst_string(&item.name), atk_sign, atk_mod, dmg_str, dmg_type
                ));
            }
            wpn_content.push_str("    )\n");
            typst.push_str(&secondary_box("WEAPONS & DAMAGE", &wpn_content, false));
            typst.push_str("    v(spacing.xs)\n\n");
        }

        // Proficiencies & Languages
        let mut prof_content = String::new();
        if !char.proficiencies.armor.is_empty() {
            prof_content.push_str(&format!(
                "    #text(size: sizes.xs, weight: \"bold\")[Armor:] #text(size: sizes.xs)[ {}] #linebreak()\n",
                escape_typst_string(&char.proficiencies.armor.join(", "))
            ));
        }
        if !char.proficiencies.weapons.is_empty() {
            prof_content.push_str(&format!(
                "    #text(size: sizes.xs, weight: \"bold\")[Weapons:] #text(size: sizes.xs)[ {}] #linebreak()\n",
                escape_typst_string(&char.proficiencies.weapons.join(", "))
            ));
        }
        if !char.proficiencies.tools.is_empty() {
            prof_content.push_str(&format!(
                "    #text(size: sizes.xs, weight: \"bold\")[Tools:] #text(size: sizes.xs)[ {}] #linebreak()\n",
                escape_typst_string(&char.proficiencies.tools.join(", "))
            ));
        }
        if !char.proficiencies.languages.is_empty() {
            prof_content.push_str(&format!(
                "    #text(size: sizes.xs, weight: \"bold\")[Languages:] #text(size: sizes.xs)[ {}] #linebreak()\n",
                escape_typst_string(&char.proficiencies.languages.join(", "))
            ));
        }
        if !prof_content.is_empty() {
            typst.push_str(&secondary_box("PROFICIENCIES & LANGUAGES", &prof_content, false));
            typst.push_str("    v(spacing.xs)\n\n");
        }

        // Your Turn reference
        let actions_content = r#"    #text(size: sizes.xs, weight: "bold")[MOVE] #text(size: sizes.xs)[ — Up to your Speed (can split around actions)]
    #linebreak()
    #text(size: sizes.xs, weight: "bold")[ACTION] #text(size: sizes.xs)[ — Attack · Cast a Spell · Dash · Disengage · Dodge · Help · Hide · Ready · Use Object]
    #linebreak()
    #text(size: sizes.xs, weight: "bold")[BONUS ACTION] #text(size: sizes.xs)[ — Class/spell features (e.g. offhand attack, Cunning Action)]
    #linebreak()
    #text(size: sizes.xs, weight: "bold")[REACTION] #text(size: sizes.xs)[ — Opportunity Attack · Readied action · Shield, Counterspell, etc.]
    #linebreak()
    #text(size: sizes.xs, weight: "bold")[FREE] #text(size: sizes.xs)[ — Interact with one object · Drop an item · Speak briefly]
"#;
        typst.push_str(&secondary_box("YOUR TURN", actions_content, false));
        typst.push_str("    v(spacing.xs)\n\n");

        // Spellcasting (if caster) — on page 1 right column
        let has_slots = char.spell_slots.iter().any(|&s| s > 0);
        if has_slots {
            let mut spell_content = String::new();
            spell_content.push_str("    #grid(columns: (1fr, 1fr, 1fr, 1fr), column-gutter: spacing.sm,\n");
            if let Some(ref ability) = char.spellcasting_ability {
                spell_content.push_str(&format!("      labeled-value(\"Ability\", [{}]),\n", ability));
            }
            if let Some(dc) = char.spell_save_dc {
                spell_content.push_str(&format!("      labeled-value(\"Save DC\", str({})),\n", dc));
            }
            if let Some(atk) = char.spell_attack_bonus {
                let sign = if atk >= 0 { "+" } else { "" };
                spell_content.push_str(&format!("      labeled-value(\"Attack\", [{}{}]),\n", sign, atk));
            }
            spell_content.push_str(&format!("      labeled-value(\"Hit Die\", [{}]),\n", escape_typst_string(&char.hit_die)));
            spell_content.push_str("    )\n    #v(spacing.sm)\n");

            // Slot grid: header row, total row, then checkbox rows for tracking
            spell_content.push_str("    #grid(columns: (");
            spell_content.push_str(&vec!["1fr"; 9].join(", "));
            spell_content.push_str("), column-gutter: 2pt, row-gutter: 2pt,\n");

            // Row 1: level headers
            for lvl in 1..=9 {
                let suffix = match lvl { 1 => "st", 2 => "nd", 3 => "rd", _ => "th" };
                spell_content.push_str(&format!(
                    "      align(center, block(width: 100%, fill: luma(60), inset: spacing.xs)[#text(size: sizes.xs, weight: \"bold\", fill: white)[{}{}]]),\n",
                    lvl, suffix
                ));
            }

            // Row 2: total slots
            for i in 0..9 {
                let count = char.spell_slots.get(i).copied().unwrap_or(0);
                let display = if count > 0 { format!("{}", count) } else { "—".to_string() };
                spell_content.push_str(&format!(
                    "      align(center, block(width: 100%, stroke: 0.5pt + colors.border-light, inset: spacing.xs)[#text(size: sizes.md, weight: \"bold\")[{}]]),\n",
                    display
                ));
            }

            // Row 3+: empty circle checkboxes for tracking cast spells
            // Number of rows = max slots across all levels
            let max_slots = char.spell_slots.iter().copied().max().unwrap_or(0);
            for row in 0..max_slots {
                for i in 0..9 {
                    let count = char.spell_slots.get(i).copied().unwrap_or(0);
                    if row < count {
                        spell_content.push_str(
                            "      align(center, block(width: 100%, inset: spacing.xs)[#text(size: sizes.sm)[○]]),\n",
                        );
                    } else {
                        spell_content.push_str(
                            "      align(center, block(width: 100%, inset: spacing.xs)[]),\n",
                        );
                    }
                }
            }
            spell_content.push_str("    )\n");

            typst.push_str(&secondary_box("SPELLCASTING", &spell_content, false));
        }

        typst.push_str("  }\n)\n\n");

        // --- Page 1 footer ---
        typst.push_str("#v(1fr)\n#align(center)[#text(size: sizes.xs, fill: luma(180))[Generated by Mimir]]\n\n");

        // =================================================================
        // PAGE 2: Backstory & Reference
        // =================================================================
        typst.push_str("#pagebreak()\n\n");

        // Page 2 header
        typst.push_str(&format!(
            r#"#block(width: 100%, inset: (x: spacing.sm, y: spacing.sm), stroke: (bottom: 2pt + colors.accent))[
  #text(size: sizes.xl, weight: "bold", font: font-heading, tracking: -0.5pt)[{name_upper}]
  #h(spacing.md)
  #text(size: sizes.sm, fill: colors.text-secondary)[Backstory & Reference]
]

#v(spacing.sm)

"#,
            name_upper = escape_typst_string(&char.name.to_uppercase()),
        ));

        // --- PERSONALITY: 2×2 grid (full width) ---
        let mut personality_cells = String::new();
        for (label, value) in [
            ("TRAITS", &char.traits),
            ("IDEALS", &char.ideals),
            ("BONDS", &char.bonds),
            ("FLAWS", &char.flaws),
        ] {
            let text = value
                .as_ref()
                .map(|t| escape_typst_string(t))
                .unwrap_or_default();
            personality_cells.push_str(&format!(
                "    box(width: 100%, stroke: 0.5pt + colors.border-light, radius: 2pt, inset: spacing.sm)[\n      #text(weight: \"bold\", size: sizes.xs, tracking: 0.5pt, fill: colors.text-secondary)[{}]\n      #v(spacing.xs)\n      #text(size: sizes.xs)[{}]\n    ],\n",
                label, text
            ));
        }
        let personality_content = format!(
            "    #grid(columns: (1fr, 1fr), column-gutter: spacing.sm, row-gutter: spacing.sm,\n{}\n    )\n",
            personality_cells
        );
        typst.push_str(&primary_box("PERSONALITY", &personality_content, true));
        typst.push_str("#v(spacing.sm)\n\n");

        // --- Two-column: Spellcasting+NPC (left) | Currency+Inventory (right) ---
        typst.push_str("#grid(\n  columns: (1fr, 1fr),\n  column-gutter: spacing.md,\n\n");

        // ===== LEFT COLUMN =====
        typst.push_str("  // Left column\n  [\n");

        // NPC info
        if char.is_npc && (char.role.is_some() || char.location.is_some() || char.faction.is_some()) {
            let mut npc_content = String::new();
            if let Some(ref role) = char.role {
                npc_content.push_str(&format!(
                    "    #text(size: sizes.xs, weight: \"bold\", tracking: 0.5pt, fill: colors.text-secondary)[ROLE] #linebreak() #text(size: sizes.sm)[{}] #v(spacing.sm)\n",
                    escape_typst_string(role)
                ));
            }
            if let Some(ref loc) = char.location {
                npc_content.push_str(&format!(
                    "    #text(size: sizes.xs, weight: \"bold\", tracking: 0.5pt, fill: colors.text-secondary)[LOCATION] #linebreak() #text(size: sizes.sm)[{}] #v(spacing.sm)\n",
                    escape_typst_string(loc)
                ));
            }
            if let Some(ref faction) = char.faction {
                npc_content.push_str(&format!(
                    "    #text(size: sizes.xs, weight: \"bold\", tracking: 0.5pt, fill: colors.text-secondary)[FACTION] #linebreak() #text(size: sizes.sm)[{}] #v(spacing.sm)\n",
                    escape_typst_string(faction)
                ));
            }
            typst.push_str(&secondary_box("NPC INFORMATION", &npc_content, true));
            typst.push_str("#v(spacing.sm)\n\n");
        }

        // Notes (fills remaining left column)
        let notes_content = "    #v(1fr)\n";
        typst.push_str(&secondary_box("NOTES", notes_content, true));

        typst.push_str("  ],\n\n");

        // ===== RIGHT COLUMN =====
        typst.push_str("  // Right column\n  [\n");

        // Currency — tertiary (minimal)
        let mut currency_content = String::new();
        currency_content.push_str("    #grid(columns: (1fr, 1fr, 1fr, 1fr, 1fr), column-gutter: spacing.sm,\n");
        for (label, val) in [("PP", char.pp), ("GP", char.gp), ("EP", char.ep), ("SP", char.sp), ("CP", char.cp)] {
            currency_content.push_str(&format!(
                "      align(center, box(stroke: 0.5pt + colors.border-light, width: 100%, radius: 2pt, inset: spacing.xs)[#align(center)[#text(size: sizes.md, weight: \"bold\")[{}] #linebreak() #text(size: sizes.xs, fill: colors.text-secondary, tracking: 0.5pt)[{}]]]),",
                val, label
            ));
        }
        currency_content.push_str("\n    )\n");
        typst.push_str(&tertiary_label("CURRENCY", &currency_content, true));
        typst.push_str("#v(spacing.sm)\n\n");

        // Inventory — zebra-striped table
        let mut inv_content = String::new();
        if char.inventory.is_empty() {
            inv_content.push_str("    #text(size: sizes.sm, fill: colors.text-secondary)[No items]\n");
        } else {
            inv_content.push_str("    #grid(columns: (auto, 1fr, auto), column-gutter: spacing.md, row-gutter: spacing.xs,\n");
            inv_content.push_str("      text(size: sizes.xs, weight: \"bold\", tracking: 0.5pt)[QTY], text(size: sizes.xs, weight: \"bold\", tracking: 0.5pt)[ITEM], text(size: sizes.xs, weight: \"bold\", tracking: 0.5pt)[STATUS],\n");

            for (i, item) in char.inventory.iter().enumerate() {
                let status = match (item.equipped, item.attuned) {
                    (true, true) => "E ★",
                    (true, false) => "E",
                    _ => "—",
                };
                let fill = if i % 2 == 0 { "fill: colors.background-alt, " } else { "" };
                inv_content.push_str(&format!(
                    "      block({}inset: spacing.xs)[#text(size: sizes.xs)[{}]], block({}inset: spacing.xs)[#text(size: sizes.xs)[{}]], block({}inset: spacing.xs)[#text(size: sizes.xs)[{}]],\n",
                    fill, item.quantity, fill, escape_typst_string(&item.name), fill, status
                ));
            }
            inv_content.push_str("    )\n");
        }
        typst.push_str(&tertiary_label("INVENTORY", &inv_content, true));

        typst.push_str("  ]\n)\n\n");

        // --- Page 2 footer ---
        typst.push_str("#v(1fr)\n#align(center)[#text(size: sizes.xs, fill: luma(180))[Generated by Mimir]]\n");

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        Some(self.character.name.clone())
    }

    fn page_break_before(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_character() -> CharacterData {
        CharacterData {
            name: "Aragorn".to_string(),
            player_name: Some("John".to_string()),
            is_npc: false,
            race_name: Some("Human".to_string()),
            background_name: Some("Noble".to_string()),
            strength: 16,
            dexterity: 14,
            constitution: 15,
            intelligence: 12,
            wisdom: 14,
            charisma: 16,
            cp: 50,
            sp: 20,
            ep: 0,
            gp: 150,
            pp: 5,
            traits: Some("I always speak in a serious tone".to_string()),
            ideals: Some("Duty above all".to_string()),
            bonds: Some("I must protect the realm".to_string()),
            flaws: Some("I am haunted by my destiny".to_string()),
            role: None,
            location: None,
            faction: None,
            classes: vec![
                ClassInfo {
                    class_name: "Fighter".to_string(),
                    level: 5,
                    subclass_name: Some("Champion".to_string()),
                    is_starting: true,
                },
                ClassInfo {
                    class_name: "Ranger".to_string(),
                    level: 3,
                    subclass_name: None,
                    is_starting: false,
                },
            ],
            inventory: vec![
                InventoryItem {
                    name: "Longsword".to_string(),
                    quantity: 1,
                    equipped: true,
                    attuned: false,
                    item_type: Some("M".to_string()),
                    damage: Some("1d8".to_string()),
                    damage_type: Some("S".to_string()),
                    armor_ac: None,
                    finesse: false,
                },
                InventoryItem {
                    name: "Chain Mail".to_string(),
                    quantity: 1,
                    equipped: true,
                    attuned: false,
                    item_type: Some("HA".to_string()),
                    damage: None,
                    damage_type: None,
                    armor_ac: Some(16),
                    finesse: false,
                },
            ],
            proficiencies: Proficiencies {
                skills: vec![
                    ProficiencyEntry { name: "Athletics".to_string(), expertise: false },
                    ProficiencyEntry { name: "Perception".to_string(), expertise: false },
                    ProficiencyEntry { name: "Survival".to_string(), expertise: false },
                ],
                saves: vec!["Strength".to_string(), "Constitution".to_string()],
                languages: vec!["Common".to_string(), "Elvish".to_string()],
                armor: vec!["All armor".to_string(), "Shields".to_string()],
                weapons: vec!["Simple weapons".to_string(), "Martial weapons".to_string()],
                tools: vec![],
            },
            speed: 30,
            ac: 16, // Chain Mail
            hit_points_max: 67,
            hit_die: "5d10 + 3d8".to_string(),
            spellcasting_ability: None,
            spell_save_dc: None,
            spell_attack_bonus: None,
            spell_slots: vec![0; 9],
        }
    }

    fn test_spellcaster() -> CharacterData {
        CharacterData {
            name: "Gandalf".to_string(),
            player_name: Some("Jane".to_string()),
            is_npc: false,
            race_name: Some("Human".to_string()),
            background_name: Some("Sage".to_string()),
            strength: 8,
            dexterity: 14,
            constitution: 12,
            intelligence: 20,
            wisdom: 14,
            charisma: 10,
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 500,
            pp: 10,
            traits: Some("Always curious".to_string()),
            ideals: Some("Knowledge is power".to_string()),
            bonds: Some("My spellbook is my life".to_string()),
            flaws: Some("I overlook obvious solutions in favor of complicated ones".to_string()),
            role: None,
            location: None,
            faction: None,
            classes: vec![
                ClassInfo {
                    class_name: "Wizard".to_string(),
                    level: 9,
                    subclass_name: Some("Evocation".to_string()),
                    is_starting: true,
                },
            ],
            inventory: vec![
                InventoryItem {
                    name: "Staff of Power".to_string(),
                    quantity: 1,
                    equipped: true,
                    attuned: true,
                    item_type: Some("M".to_string()),
                    damage: Some("1d6".to_string()),
                    damage_type: Some("B".to_string()),
                    armor_ac: None,
                    finesse: false,
                },
                InventoryItem {
                    name: "Spellbook".to_string(),
                    quantity: 1,
                    equipped: false,
                    attuned: false,
                    item_type: None,
                    damage: None,
                    damage_type: None,
                    armor_ac: None,
                    finesse: false,
                },
            ],
            proficiencies: Proficiencies {
                skills: vec![
                    ProficiencyEntry { name: "Arcana".to_string(), expertise: false },
                    ProficiencyEntry { name: "History".to_string(), expertise: false },
                    ProficiencyEntry { name: "Investigation".to_string(), expertise: false },
                ],
                saves: vec!["Intelligence".to_string(), "Wisdom".to_string()],
                languages: vec!["Common".to_string(), "Draconic".to_string(), "Elvish".to_string()],
                armor: vec![],
                weapons: vec!["Daggers".to_string(), "Quarterstaffs".to_string()],
                tools: vec![],
            },
            speed: 30,
            ac: 10,
            hit_points_max: 45,
            hit_die: "9d6".to_string(),
            spellcasting_ability: Some("INT".to_string()),
            spell_save_dc: Some(17),
            spell_attack_bonus: Some(9),
            spell_slots: vec![4, 3, 3, 3, 1, 0, 0, 0, 0],
        }
    }

    #[test]
    fn test_total_level() {
        let char = test_character();
        let section = CharacterSection::new(char);
        assert_eq!(section.total_level(), 8);
    }

    #[test]
    fn test_prof_bonus() {
        let char = test_character();
        let section = CharacterSection::new(char);
        assert_eq!(section.prof_bonus(), 3);
    }

    #[test]
    fn test_class_string() {
        let char = test_character();
        let section = CharacterSection::new(char);
        assert_eq!(section.class_string(), "Fighter (Champion) 5 / Ranger 3");
    }

    #[test]
    fn test_modifier() {
        assert_eq!(CharacterSection::modifier(10), 0);
        assert_eq!(CharacterSection::modifier(14), 2);
        assert_eq!(CharacterSection::modifier(8), -1);
        assert_eq!(CharacterSection::modifier(1), -5);
    }

    #[test]
    fn test_total_gold() {
        let char = test_character();
        let section = CharacterSection::new(char);
        assert!((section.total_gold() - 202.5).abs() < 0.01);
    }

    #[test]
    fn test_toc_title() {
        let char = test_character();
        let section = CharacterSection::new(char);
        assert_eq!(section.toc_title(), Some("Aragorn".to_string()));
    }

    #[test]
    fn test_spellcaster_renders_spell_slots() {
        let char = test_spellcaster();
        let section = CharacterSection::new(char);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();
        assert!(typst.contains("SPELLCASTING"));
        assert!(typst.contains("Save DC"));
        assert!(typst.contains("1st"));
        assert!(typst.contains("9th"));
        assert!(typst.contains("pagebreak()"));
    }

    #[test]
    fn test_non_caster_no_spell_section() {
        let char = test_character();
        let section = CharacterSection::new(char);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();
        assert!(!typst.contains("SPELLCASTING"));
        assert!(typst.contains("pagebreak()"));
    }

    #[test]
    fn test_all_18_skills_rendered() {
        let char = test_character();
        let section = CharacterSection::new(char);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();
        assert!(typst.contains("Athletics"));
        assert!(typst.contains("Acrobatics"));
        assert!(typst.contains("Arcana"));
        assert!(typst.contains("Perception"));
        assert!(typst.contains("Stealth"));
        assert!(typst.contains("Persuasion"));
    }

    #[test]
    fn test_actions_reference() {
        let char = test_character();
        let section = CharacterSection::new(char);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();
        assert!(typst.contains("Attack"));
        assert!(typst.contains("Dash"));
        assert!(typst.contains("Dodge"));
        assert!(typst.contains("YOUR TURN"));
    }
}
