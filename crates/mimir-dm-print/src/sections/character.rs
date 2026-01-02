//! Character sheet section
//!
//! Generates a full character sheet using shared Typst components.

use mimir_dm_core::models::catalog::Spell;
use mimir_dm_core::models::character::data::{AbilityScores, CharacterData};

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Character sheet section - generates a full character sheet
pub struct CharacterSheetSection {
    /// Character data (typed struct from mimir-dm-core)
    character: CharacterData,
    /// Full spell details from catalog (for spell cards)
    spell_details: Vec<Spell>,
    /// Whether to include spell cards after the sheet
    include_spell_cards: bool,
}

impl CharacterSheetSection {
    /// Create a new character sheet section
    pub fn new(character: CharacterData) -> Self {
        Self {
            character,
            spell_details: Vec::new(),
            include_spell_cards: false,
        }
    }

    /// Add spell details for spell cards
    pub fn with_spells(mut self, spells: Vec<Spell>) -> Self {
        self.include_spell_cards = !spells.is_empty();
        self.spell_details = spells;
        self
    }

    /// Build class string from classes array
    fn build_class_string(&self) -> String {
        if self.character.classes.is_empty() {
            return "No Class".to_string();
        }

        self.character
            .classes
            .iter()
            .map(|c| {
                if let Some(ref sub) = c.subclass {
                    format!("{} ({}) {}", c.class_name, sub, c.level)
                } else {
                    format!("{} {}", c.class_name, c.level)
                }
            })
            .collect::<Vec<_>>()
            .join(" / ")
    }

    /// Check if this is an NPC
    fn is_npc(&self) -> bool {
        self.character.npc_role.is_some()
            || self.character.npc_location.is_some()
            || self.character.npc_faction.is_some()
    }
}

impl Renderable for CharacterSheetSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let mut typst = String::new();
        let c = &self.character;
        let abilities = &c.abilities;

        // Build display strings
        let race_str = if let Some(ref sub) = c.subrace {
            format!("{} {}", sub, c.race)
        } else {
            c.race.clone()
        };
        let class_str = self.build_class_string();

        // =========================================================================
        // HEADER
        // =========================================================================
        typst.push_str(&format!(
            r#"#block(
  width: 100%,
  inset: spacing.md,
  stroke: (bottom: 2pt + colors.accent),
  {{
    grid(
      columns: (1fr, auto),
      column-gutter: spacing.sm,
      title-text[{}],
      {}
    )
    v(spacing.xs)
    text(size: sizes.md)[Level {} {} {}]
    {}
  }}
)

"#,
            escape_typst(&c.character_name),
            if self.is_npc() {
                r#"box(fill: colors.accent, radius: 4pt, inset: (x: spacing.sm, y: spacing.xs))[
        #text(fill: white, weight: "bold", size: sizes.sm)[NPC]
      ]"#
            } else {
                ""
            },
            c.level,
            escape_typst(&race_str),
            escape_typst(&class_str),
            if c.background != "Unknown" || c.alignment.is_some() {
                format!(
                    "linebreak()\nsmall-text[{}{}]",
                    if c.background != "Unknown" {
                        escape_typst(&c.background)
                    } else {
                        String::new()
                    },
                    if let Some(ref align) = c.alignment {
                        format!(", {}", escape_typst(align))
                    } else {
                        String::new()
                    }
                )
            } else {
                String::new()
            }
        ));

        // =========================================================================
        // NPC INFO (if applicable)
        // =========================================================================
        if self.is_npc() {
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

            if let Some(ref role) = c.npc_role {
                typst.push_str(&format!(
                    "    [#label-text(\"Role\")#linebreak()#text(size: sizes.sm)[{}]],\n",
                    escape_typst(role)
                ));
            }
            if let Some(ref loc) = c.npc_location {
                typst.push_str(&format!(
                    "    [#label-text(\"Location\")#linebreak()#text(size: sizes.sm)[{}]],\n",
                    escape_typst(loc)
                ));
            }
            if let Some(ref faction) = c.npc_faction {
                typst.push_str(&format!(
                    "    [#label-text(\"Faction\")#linebreak()#text(size: sizes.sm)[{}]],\n",
                    escape_typst(faction)
                ));
            }

            typst.push_str("  )\n");

            if let Some(ref notes) = c.npc_notes {
                typst.push_str(&format!(
                    "  #v(spacing.sm)\n  #label-text(\"DM Notes\")\n  #linebreak()\n  #text(size: sizes.sm)[{}]\n",
                    escape_typst(notes)
                ));
            }

            typst.push_str("]\n\n");
        }

        typst.push_str("#v(spacing.md)\n\n");

        // =========================================================================
        // MAIN STATS ROW
        // =========================================================================
        let str_score = abilities.strength;
        let dex_score = abilities.dexterity;
        let con_score = abilities.constitution;
        let int_score = abilities.intelligence;
        let wis_score = abilities.wisdom;
        let cha_score = abilities.charisma;

        let dex_mod = AbilityScores::modifier(dex_score);
        let ac = 10 + dex_mod;
        let prof_bonus = c.proficiency_bonus();

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

  // Combat Stats
  {{
    info-box(title: "Combat")[
      #grid(
        columns: (1fr, 1fr),
        row-gutter: spacing.sm,
        column-gutter: spacing.md,
        labeled-value("HP", [{} / {}]),
        labeled-value("AC", str({})),
        labeled-value("Speed", [{} ft]),
        labeled-value("Prof", [+{}]),
      )
    ]

    v(spacing.sm)

    // Saving Throws
    info-box(title: "Saving Throws")[
"#,
            str_score, dex_score, con_score, int_score, wis_score, cha_score,
            c.current_hp, c.max_hp, ac, c.speed, prof_bonus
        ));

        // Saving throws
        let prof_saves = &c.proficiencies.saves;

        for (abbrev, full, score) in [
            ("STR", "Strength", str_score),
            ("DEX", "Dexterity", dex_score),
            ("CON", "Constitution", con_score),
            ("INT", "Intelligence", int_score),
            ("WIS", "Wisdom", wis_score),
            ("CHA", "Charisma", cha_score),
        ] {
            let modifier = AbilityScores::modifier(score);
            let is_prof = prof_saves.iter().any(|s| s == full);
            let total = modifier + if is_prof { prof_bonus } else { 0 };
            let sign = if total >= 0 { "+" } else { "" };

            // FIX: Add # prefix to text() and linebreak()
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
        // PROFICIENCIES AND EQUIPMENT
        // =========================================================================
        typst.push_str(r#"#grid(
  columns: (1fr, 1fr),
  column-gutter: spacing.md,

  // Proficiencies
  {
    info-box(title: "Proficiencies")[
"#);

        // Skills
        if !c.proficiencies.skills.is_empty() {
            let skill_str = c.proficiencies.skills.join(", ");
            typst.push_str(&format!(
                "      #label-text(\"Skills\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n      #v(spacing.sm)\n",
                escape_typst(&skill_str)
            ));
        }

        // Languages
        if !c.proficiencies.languages.is_empty() {
            let lang_str = c.proficiencies.languages.join(", ");
            typst.push_str(&format!(
                "      #label-text(\"Languages\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n",
                escape_typst(&lang_str)
            ));
        }

        typst.push_str("    ]\n  },\n\n  // Equipment\n  {\n    info-box(title: \"Equipment\")[\n");

        // Equipped items
        if let Some(ref armor) = c.equipped.armor {
            typst.push_str(&format!(
                "      #inline-labeled(\"Armor\", [{}])\n      #linebreak()\n",
                escape_typst(armor)
            ));
        }
        if let Some(ref shield) = c.equipped.shield {
            typst.push_str(&format!(
                "      #inline-labeled(\"Shield\", [{}])\n      #linebreak()\n",
                escape_typst(shield)
            ));
        }
        if let Some(ref main_hand) = c.equipped.main_hand {
            typst.push_str(&format!(
                "      #inline-labeled(\"Main Hand\", [{}])\n      #linebreak()\n",
                escape_typst(main_hand)
            ));
        }
        if let Some(ref off_hand) = c.equipped.off_hand {
            typst.push_str(&format!(
                "      #inline-labeled(\"Off Hand\", [{}])\n      #linebreak()\n",
                escape_typst(off_hand)
            ));
        }

        // Currency
        typst.push_str("      #v(spacing.sm)\n      #label-text(\"Currency\")\n      #linebreak()\n      #text(size: sizes.sm)[");
        let currency = &c.currency;
        let mut parts = Vec::new();
        if currency.platinum > 0 {
            parts.push(format!("{} pp", currency.platinum));
        }
        if currency.gold > 0 {
            parts.push(format!("{} gp", currency.gold));
        }
        if currency.electrum > 0 {
            parts.push(format!("{} ep", currency.electrum));
        }
        if currency.silver > 0 {
            parts.push(format!("{} sp", currency.silver));
        }
        if currency.copper > 0 {
            parts.push(format!("{} cp", currency.copper));
        }
        typst.push_str(&parts.join(" "));
        typst.push_str("]\n    ]\n  }\n)\n\n#v(spacing.md)\n\n");

        // =========================================================================
        // FEATURES AND TRAITS
        // =========================================================================
        typst.push_str(r#"#grid(
  columns: (1fr, 1fr),
  column-gutter: spacing.md,

  // Features & Traits
  {
    info-box(title: "Features & Traits")[
"#);

        for feature in &c.class_features {
            typst.push_str(&format!(
                "      #text(size: sizes.sm)[- {}]\n      #linebreak()\n",
                escape_typst(&feature.name)
            ));
        }

        typst.push_str("    ]\n  },\n\n  // Spells or Personality\n  {\n");

        // Spells section (if character has spells)
        let has_spells = !c.spells.cantrips.is_empty()
            || !c.spells.prepared_spells.is_empty()
            || !c.spells.known_spells.is_empty();

        if has_spells && !self.is_npc() {
            typst.push_str("    info-box(title: \"Spellcasting\")[\n");

            // Cantrips
            if !c.spells.cantrips.is_empty() {
                let cantrip_names: Vec<&str> = c
                    .spells
                    .cantrips
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect();
                typst.push_str(&format!(
                    "      #label-text(\"Cantrips\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n      #v(spacing.sm)\n",
                    escape_typst(&cantrip_names.join(", "))
                ));
            }

            // Prepared or Known spells
            let (label, spells) = if !c.spells.prepared_spells.is_empty() {
                ("Prepared Spells", &c.spells.prepared_spells)
            } else {
                ("Known Spells", &c.spells.known_spells)
            };

            if !spells.is_empty() {
                let spell_names: Vec<&str> = spells.iter().map(|s| s.name.as_str()).collect();
                typst.push_str(&format!(
                    "      #label-text(\"{}\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n",
                    label,
                    escape_typst(&spell_names.join(", "))
                ));
            }

            typst.push_str("    ]\n");
        } else {
            // Personality traits for non-spellcasters or NPCs
            typst.push_str("    info-box(title: \"Personality\")[\n");

            if let Some(ref traits) = c.personality.traits {
                typst.push_str(&format!(
                    "      #label-text(\"Traits\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n      #v(spacing.sm)\n",
                    escape_typst(traits)
                ));
            }
            if let Some(ref ideals) = c.personality.ideals {
                typst.push_str(&format!(
                    "      #label-text(\"Ideals\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n      #v(spacing.sm)\n",
                    escape_typst(ideals)
                ));
            }
            if let Some(ref bonds) = c.personality.bonds {
                typst.push_str(&format!(
                    "      #label-text(\"Bonds\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n      #v(spacing.sm)\n",
                    escape_typst(bonds)
                ));
            }
            if let Some(ref flaws) = c.personality.flaws {
                typst.push_str(&format!(
                    "      #label-text(\"Flaws\")\n      #linebreak()\n      #text(size: sizes.sm)[{}]\n",
                    escape_typst(flaws)
                ));
            }

            typst.push_str("    ]\n");
        }

        typst.push_str("  }\n)\n");

        // =========================================================================
        // FOOTER
        // =========================================================================
        typst.push_str("\n#v(1fr)\n#align(center)[\n  #small-text[Generated by Mimir]\n]\n");

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        Some(format!("Character: {}", self.character.character_name))
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
    use mimir_dm_core::models::character::data::{
        AbilityScores, ClassLevel, Currency, EquippedItems, Personality, Proficiencies,
        SpellData as CharacterSpellData,
    };

    fn sample_character() -> CharacterData {
        CharacterData {
            character_name: "Test Fighter".to_string(),
            player_id: Some(1),
            level: 5,
            experience_points: 6500,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-01".to_string(),
            race: "Human".to_string(),
            subrace: None,
            classes: vec![ClassLevel {
                class_name: "Fighter".to_string(),
                level: 5,
                subclass: Some("Champion".to_string()),
                hit_dice_type: "d10".to_string(),
                hit_dice_remaining: 5,
            }],
            background: "Soldier".to_string(),
            alignment: Some("Lawful Good".to_string()),
            abilities: AbilityScores {
                strength: 16,
                dexterity: 14,
                constitution: 14,
                intelligence: 10,
                wisdom: 12,
                charisma: 8,
            },
            max_hp: 44,
            current_hp: 44,
            speed: 30,
            proficiencies: Proficiencies {
                skills: vec!["Athletics".to_string(), "Intimidation".to_string()],
                saves: vec!["Strength".to_string(), "Constitution".to_string()],
                armor: vec!["All armor".to_string(), "Shields".to_string()],
                weapons: vec!["Simple weapons".to_string(), "Martial weapons".to_string()],
                tools: vec![],
                languages: vec!["Common".to_string()],
            },
            class_features: vec![],
            feats: vec![],
            spells: CharacterSpellData::default(),
            inventory: vec![],
            currency: Currency::default(),
            equipped: EquippedItems {
                armor: Some("Chain Mail".to_string()),
                shield: Some("Shield".to_string()),
                main_hand: Some("Longsword".to_string()),
                off_hand: None,
            },
            personality: Personality::default(),
            npc_role: None,
            npc_location: None,
            npc_faction: None,
            npc_notes: None,
            legendary_actions: vec![],
            legendary_action_count: None,
        }
    }

    #[test]
    fn test_character_sheet_basic() {
        let character = sample_character();
        let section = CharacterSheetSection::new(character);
        assert_eq!(
            section.toc_title(),
            Some("Character: Test Fighter".to_string())
        );
    }

    #[test]
    fn test_character_sheet_generates_valid_typst() {
        let character = sample_character();
        let section = CharacterSheetSection::new(character);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Verify character name appears
        assert!(typst.contains("Test Fighter"));
        // Verify class string
        assert!(typst.contains("Fighter (Champion) 5"));
        // Verify no "Unknown Character"
        assert!(!typst.contains("Unknown Character"));
        // Verify ability scores
        assert!(typst.contains("str: 16"));
        // Verify # prefix on text() calls (the bug fix)
        assert!(typst.contains("#text(size: sizes.sm)"));
    }

    #[test]
    fn test_npc_detection() {
        let mut character = sample_character();
        character.npc_role = Some("Friendly Merchant".to_string());

        let section = CharacterSheetSection::new(character);
        assert!(section.is_npc());

        let pc_character = sample_character();
        let pc_section = CharacterSheetSection::new(pc_character);
        assert!(!pc_section.is_npc());
    }

    #[test]
    fn test_class_string_multiclass() {
        let mut character = sample_character();
        character.classes = vec![
            ClassLevel {
                class_name: "Fighter".to_string(),
                level: 3,
                subclass: None,
                hit_dice_type: "d10".to_string(),
                hit_dice_remaining: 3,
            },
            ClassLevel {
                class_name: "Wizard".to_string(),
                level: 2,
                subclass: None,
                hit_dice_type: "d6".to_string(),
                hit_dice_remaining: 2,
            },
        ];

        let section = CharacterSheetSection::new(character);
        assert_eq!(section.build_class_string(), "Fighter 3 / Wizard 2");
    }
}
