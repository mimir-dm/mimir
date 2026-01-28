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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub name: String,
    pub quantity: i32,
    pub equipped: bool,
    pub attuned: bool,
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

        // Calculate derived stats
        let dex_mod = Self::modifier(char.dexterity);
        // Basic AC calculation: 10 + DEX mod (armor would modify this)
        let ac = 10 + dex_mod;

        // =========================================================================
        // HEADER
        // =========================================================================
        typst.push_str(&format!(
            r#"// Character: {}
#block(
  width: 100%,
  inset: spacing.md,
  stroke: (bottom: 2pt + colors.accent),
  {{
    grid(
      columns: (1fr, auto),
      column-gutter: spacing.sm,
      text(size: sizes.xl, weight: "bold")[{}],
      {}
    )
    v(spacing.xs)
    text(size: sizes.md)[Level {} {} {}]
    {}
  }}
)

"#,
            escape_typst_string(&char.name),
            escape_typst_string(&char.name),
            if char.is_npc {
                r#"box(fill: colors.accent, radius: 4pt, inset: (x: spacing.sm, y: spacing.xs))[
        #text(fill: white, weight: "bold", size: sizes.sm)[NPC]
      ]"#
            } else {
                ""
            },
            level,
            char.race_name.as_deref().map(|r| escape_typst_string(r)).unwrap_or_default(),
            escape_typst_string(&classes),
            if char.background_name.is_some() || char.player_name.is_some() {
                let mut extra = String::new();
                extra.push_str("linebreak()\nsmall-text[");
                if let Some(ref bg) = char.background_name {
                    extra.push_str(&escape_typst_string(bg));
                }
                if let Some(ref player) = char.player_name {
                    if char.background_name.is_some() {
                        extra.push_str(" · ");
                    }
                    extra.push_str("Player: ");
                    extra.push_str(&escape_typst_string(player));
                }
                extra.push(']');
                extra
            } else {
                String::new()
            }
        ));

        // =========================================================================
        // NPC INFO (if applicable)
        // =========================================================================
        if char.is_npc && (char.role.is_some() || char.location.is_some() || char.faction.is_some()) {
            typst.push_str(r##"#v(spacing.md)

#block(
  width: 100%,
  fill: rgb("#fff8e1"),
  stroke: 1pt + colors.accent,
  radius: 4pt,
  inset: spacing.md,
)[
  #text(weight: "bold", size: sizes.md)[NPC Information]
  #v(spacing.sm)

  #grid(
    columns: (1fr, 1fr),
    column-gutter: spacing.lg,
    row-gutter: spacing.sm,
"##);

            if let Some(ref role) = char.role {
                typst.push_str(&format!(
                    "    [#label-text(\"Role\")#linebreak()#text(size: sizes.sm)[{}]],\n",
                    escape_typst_string(role)
                ));
            }
            if let Some(ref loc) = char.location {
                typst.push_str(&format!(
                    "    [#label-text(\"Location\")#linebreak()#text(size: sizes.sm)[{}]],\n",
                    escape_typst_string(loc)
                ));
            }
            if let Some(ref faction) = char.faction {
                typst.push_str(&format!(
                    "    [#label-text(\"Faction\")#linebreak()#text(size: sizes.sm)[{}]],\n",
                    escape_typst_string(faction)
                ));
            }

            typst.push_str("  )\n]\n\n");
        }

        typst.push_str("#v(spacing.md)\n\n");

        // =========================================================================
        // MAIN STATS ROW: Ability Scores + Combat/Saves
        // =========================================================================
        typst.push_str(&format!(
            r#"#grid(
  columns: (2fr, 1fr),
  column-gutter: spacing.md,

  // Ability Scores
  ability-scores(
    str: {},
    dex: {},
    con: {},
    int: {},
    wis: {},
    cha: {},
    layout: "grid"
  ),

  // Combat Stats + Saves
  {{
    info-box(title: "Combat")[
      #grid(
        columns: (1fr, 1fr),
        row-gutter: spacing.sm,
        column-gutter: spacing.md,
        labeled-value("AC", str({})),
        labeled-value("Speed", [{} ft]),
        labeled-value("Prof", [+{}]),
        labeled-value("Init", [{}]),
      )
    ]

    v(spacing.sm)

    // Saving Throws
    info-box(title: "Saving Throws")[
"#,
            char.strength, char.dexterity, char.constitution,
            char.intelligence, char.wisdom, char.charisma,
            ac, char.speed, prof,
            Self::modifier_str(char.dexterity)
        ));

        // Generate saving throws with proficiency
        let prof_saves = &char.proficiencies.saves;

        for (abbrev, full, score) in [
            ("STR", "Strength", char.strength),
            ("DEX", "Dexterity", char.dexterity),
            ("CON", "Constitution", char.constitution),
            ("INT", "Intelligence", char.intelligence),
            ("WIS", "Wisdom", char.wisdom),
            ("CHA", "Charisma", char.charisma),
        ] {
            let modifier = Self::modifier(score);
            let is_prof = prof_saves.iter().any(|s| s == full);
            let total = modifier + if is_prof { prof } else { 0 };
            let sign = if total >= 0 { "+" } else { "" };

            typst.push_str(&format!(
                "      #text(size: sizes.sm)[{}#h(1fr){}{}]\n      #linebreak()\n",
                if is_prof {
                    format!("#text(weight: \"bold\")[{}]", abbrev)
                } else {
                    abbrev.to_string()
                },
                sign,
                total
            ));
        }

        typst.push_str("    ]\n  }\n)\n\n#v(spacing.md)\n\n");

        // =========================================================================
        // PROFICIENCIES AND EQUIPMENT ROW
        // =========================================================================
        typst.push_str(r#"#grid(
  columns: (1fr, 1fr),
  column-gutter: spacing.md,

  // Proficiencies
  {
    info-box(title: "Proficiencies")[
"#);

        // Skills
        if !char.proficiencies.skills.is_empty() {
            let skill_str: Vec<String> = char.proficiencies.skills.iter()
                .map(|s| if s.expertise { format!("{}*", s.name) } else { s.name.clone() })
                .collect();
            typst.push_str(&format!(
                "      #label-text(\"Skills\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n      #v(spacing.sm)\n",
                escape_typst_string(&skill_str.join(", "))
            ));
        }

        // Languages
        if !char.proficiencies.languages.is_empty() {
            let lang_str = char.proficiencies.languages.join(", ");
            typst.push_str(&format!(
                "      #label-text(\"Languages\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n      #v(spacing.sm)\n",
                escape_typst_string(&lang_str)
            ));
        }

        // Armor/Weapons/Tools
        if !char.proficiencies.armor.is_empty() || !char.proficiencies.weapons.is_empty() {
            let mut combat_profs = Vec::new();
            combat_profs.extend(char.proficiencies.armor.iter().cloned());
            combat_profs.extend(char.proficiencies.weapons.iter().cloned());
            if !combat_profs.is_empty() {
                typst.push_str(&format!(
                    "      #label-text(\"Combat\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n      #v(spacing.sm)\n",
                    escape_typst_string(&combat_profs.join(", "))
                ));
            }
        }

        if !char.proficiencies.tools.is_empty() {
            let tool_str = char.proficiencies.tools.join(", ");
            typst.push_str(&format!(
                "      #label-text(\"Tools\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n",
                escape_typst_string(&tool_str)
            ));
        }

        typst.push_str("    ]\n  },\n\n  // Equipment\n  {\n    info-box(title: \"Equipment\")[\n");

        // Equipped items
        let equipped: Vec<_> = char.inventory.iter().filter(|i| i.equipped).collect();
        let other: Vec<_> = char.inventory.iter().filter(|i| !i.equipped).take(8).collect();

        if !equipped.is_empty() {
            typst.push_str("      #label-text(\"Equipped\")\n      #linebreak()\n");
            for item in &equipped {
                let qty = if item.quantity > 1 { format!(" (x{})", item.quantity) } else { String::new() };
                let attuned = if item.attuned { " ★" } else { "" };
                typst.push_str(&format!(
                    "      #text(size: sizes.sm)[- {}{}{}]\n      #linebreak()\n",
                    escape_typst_string(&item.name),
                    qty,
                    attuned
                ));
            }
            typst.push_str("      #v(spacing.sm)\n");
        }

        if !other.is_empty() {
            typst.push_str("      #label-text(\"Other Items\")\n      #linebreak()\n");
            for item in &other {
                let qty = if item.quantity > 1 { format!(" (x{})", item.quantity) } else { String::new() };
                typst.push_str(&format!(
                    "      #text(size: sizes.sm)[- {}{}]\n      #linebreak()\n",
                    escape_typst_string(&item.name),
                    qty
                ));
            }

            let remaining = char.inventory.iter().filter(|i| !i.equipped).count() - other.len();
            if remaining > 0 {
                typst.push_str(&format!(
                    "      #text(fill: colors.text-secondary, size: sizes.sm)[...and {} more]\n      #linebreak()\n",
                    remaining
                ));
            }
        }

        // Currency
        typst.push_str("      #v(spacing.sm)\n      #label-text(\"Currency\")\n      #linebreak()\n      #text(size: sizes.sm)[");
        let mut parts = Vec::new();
        if char.pp > 0 { parts.push(format!("{} pp", char.pp)); }
        if char.gp > 0 { parts.push(format!("{} gp", char.gp)); }
        if char.ep > 0 { parts.push(format!("{} ep", char.ep)); }
        if char.sp > 0 { parts.push(format!("{} sp", char.sp)); }
        if char.cp > 0 { parts.push(format!("{} cp", char.cp)); }
        if parts.is_empty() { parts.push("None".to_string()); }
        typst.push_str(&parts.join(" "));
        typst.push_str("]\n    ]\n  }\n)\n\n#v(spacing.md)\n\n");

        // =========================================================================
        // PERSONALITY SECTION
        // =========================================================================
        if char.traits.is_some() || char.ideals.is_some() || char.bonds.is_some() || char.flaws.is_some() {
            typst.push_str("info-box(title: \"Personality\")[\n");

            if let Some(ref traits) = char.traits {
                typst.push_str(&format!(
                    "  #label-text(\"Traits\")\n  #linebreak()\n  #text(size: sizes.sm)[{}]\n  #v(spacing.sm)\n",
                    escape_typst_string(traits)
                ));
            }
            if let Some(ref ideals) = char.ideals {
                typst.push_str(&format!(
                    "  #label-text(\"Ideals\")\n  #linebreak()\n  #text(size: sizes.sm)[{}]\n  #v(spacing.sm)\n",
                    escape_typst_string(ideals)
                ));
            }
            if let Some(ref bonds) = char.bonds {
                typst.push_str(&format!(
                    "  #label-text(\"Bonds\")\n  #linebreak()\n  #text(size: sizes.sm)[{}]\n  #v(spacing.sm)\n",
                    escape_typst_string(bonds)
                ));
            }
            if let Some(ref flaws) = char.flaws {
                typst.push_str(&format!(
                    "  #label-text(\"Flaws\")\n  #linebreak()\n  #text(size: sizes.sm)[{}]\n",
                    escape_typst_string(flaws)
                ));
            }

            typst.push_str("]\n\n");
        }

        // =========================================================================
        // FOOTER
        // =========================================================================
        typst.push_str("#v(1fr)\n#align(center)[\n  #small-text[Generated by Mimir]\n]\n");

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
                },
                InventoryItem {
                    name: "Chain Mail".to_string(),
                    quantity: 1,
                    equipped: true,
                    attuned: false,
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
        // 50cp/100 + 20sp/10 + 0ep/2 + 150gp + 5pp*10 = 0.5 + 2 + 0 + 150 + 50 = 202.5
        assert!((section.total_gold() - 202.5).abs() < 0.01);
    }

    #[test]
    fn test_toc_title() {
        let char = test_character();
        let section = CharacterSection::new(char);
        assert_eq!(section.toc_title(), Some("Aragorn".to_string()));
    }
}
