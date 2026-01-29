//! Character battle cards section
//!
//! Generates half-page (4" x 5.5") character cards for combat reference.
//! Works for both PCs and NPCs - shows AC, HP, speed, attacks, saves, skills, etc.

use crate::builder::{escape_typst_string, RenderContext, Renderable};
use crate::error::Result;
use crate::sections::character::CharacterData;

/// Character battle cards - half-page combat reference cards (2x2 layout)
/// Works for both player characters and NPCs
pub struct CharacterBattleCardSection {
    /// Character data
    characters: Vec<CharacterData>,
    /// Show cut lines between cards
    show_cut_lines: bool,
}

impl CharacterBattleCardSection {
    /// Create a new character battle cards section
    pub fn new(characters: Vec<CharacterData>) -> Self {
        Self {
            characters,
            show_cut_lines: true,
        }
    }

    /// Create from a single character
    pub fn from_single(character: CharacterData) -> Self {
        Self::new(vec![character])
    }

    /// Set whether to show cut lines
    pub fn with_cut_lines(mut self, show: bool) -> Self {
        self.show_cut_lines = show;
        self
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

    /// Calculate total character level
    fn total_level(character: &CharacterData) -> i32 {
        character.classes.iter().map(|c| c.level).sum::<i32>().max(1)
    }

    /// Calculate proficiency bonus from total level
    fn prof_bonus(character: &CharacterData) -> i32 {
        let level = Self::total_level(character);
        if level <= 4 {
            2
        } else if level <= 8 {
            3
        } else if level <= 12 {
            4
        } else if level <= 16 {
            5
        } else {
            6
        }
    }

    /// Build class string
    fn class_string(character: &CharacterData) -> String {
        if character.classes.is_empty() {
            if let Some(ref role) = character.role {
                return role.clone();
            }
            return "Adventurer".to_string();
        }

        character
            .classes
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

    /// Calculate HP (estimate from level and CON if not stored)
    fn estimate_hp(character: &CharacterData) -> i32 {
        let level = Self::total_level(character);
        let con_mod = Self::modifier(character.constitution);
        // Rough estimate: 8 + con_mod at level 1, then 5 + con_mod per level
        8 + con_mod + (level - 1) * (5 + con_mod)
    }

    /// Estimate AC (10 + DEX mod as base)
    fn estimate_ac(character: &CharacterData) -> i32 {
        let dex_mod = Self::modifier(character.dexterity);
        // Check for equipped armor in inventory
        let has_armor = character
            .inventory
            .iter()
            .any(|i| i.equipped && i.name.to_lowercase().contains("armor"));
        let has_shield = character
            .inventory
            .iter()
            .any(|i| i.equipped && i.name.to_lowercase().contains("shield"));

        let base_ac = if has_armor {
            // Rough estimate for armor
            14 + dex_mod.min(2)
        } else {
            10 + dex_mod
        };

        if has_shield {
            base_ac + 2
        } else {
            base_ac
        }
    }

    /// Render a single character battle card (half-page format)
    fn render_card(character: &CharacterData) -> String {
        let name = escape_typst_string(&character.name);
        let subtitle = escape_typst_string(&Self::class_string(character));
        let level = Self::total_level(character);
        let prof_bonus = Self::prof_bonus(character);

        // Determine card color based on type
        let (header_color, accent_color) = if character.is_npc {
            ("rgb(\"#dbeafe\")", "rgb(\"#2563eb\")") // Blue for NPCs
        } else {
            ("rgb(\"#dcfce7\")", "rgb(\"#16a34a\")") // Green for PCs
        };

        let ac = Self::estimate_ac(character);
        let hp = Self::estimate_hp(character);
        let init = Self::modifier_str(character.dexterity);

        // HP tracker
        let hp_tracker = render_hp_tracker(hp);

        // Ability modifiers
        let str_mod = Self::modifier_str(character.strength);
        let dex_mod = Self::modifier_str(character.dexterity);
        let con_mod = Self::modifier_str(character.constitution);
        let int_mod = Self::modifier_str(character.intelligence);
        let wis_mod = Self::modifier_str(character.wisdom);
        let cha_mod = Self::modifier_str(character.charisma);

        // Attacks from equipped weapons
        let attacks = Self::get_attacks(character);

        // Footer text
        let footer_text = if character.is_npc { "NPC" } else { "Player Character" };

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
      [*AC* {ac}],
      [*HP* {hp}],
      [*Prof* +{prof_bonus}],
      [*Init* {init}],
    )
    #v(2pt)
    #text(size: 6pt)[*Speed* 30 ft.]
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

  // Attacks
  {attacks_block}

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
            name = name,
            subtitle = subtitle,
            level = level,
            ac = ac,
            hp = hp,
            prof_bonus = prof_bonus,
            init = init,
            str_score = character.strength,
            str_mod = str_mod,
            dex_score = character.dexterity,
            dex_mod = dex_mod,
            con_score = character.constitution,
            con_mod = con_mod,
            int_score = character.intelligence,
            int_mod = int_mod,
            wis_score = character.wisdom,
            wis_mod = wis_mod,
            cha_score = character.charisma,
            cha_mod = cha_mod,
            attacks_block = attacks,
            footer_text = footer_text,
        )
    }

    /// Get attacks from equipped weapons
    fn get_attacks(character: &CharacterData) -> String {
        let prof_bonus = Self::prof_bonus(character);
        let str_mod = Self::modifier(character.strength);
        let dex_mod = Self::modifier(character.dexterity);

        let mut attacks = Vec::new();

        for item in character.inventory.iter().filter(|i| i.equipped) {
            let lower = item.name.to_lowercase();
            if lower.contains("sword")
                || lower.contains("bow")
                || lower.contains("axe")
                || lower.contains("mace")
                || lower.contains("dagger")
                || lower.contains("staff")
                || lower.contains("crossbow")
                || lower.contains("hammer")
                || lower.contains("spear")
                || lower.contains("rapier")
                || lower.contains("scimitar")
            {
                // Determine if finesse or ranged
                let is_finesse =
                    lower.contains("rapier") || lower.contains("dagger") || lower.contains("scimitar");
                let is_ranged = lower.contains("bow") || lower.contains("crossbow");
                let ability_mod = if is_finesse || is_ranged {
                    dex_mod
                } else {
                    str_mod
                };
                let to_hit = prof_bonus + ability_mod;

                attacks.push(format!(
                    "*{}.* {:+} to hit",
                    escape_typst_string(&item.name),
                    to_hit
                ));
            }
        }

        if attacks.is_empty() {
            // Default unarmed strike
            attacks.push(format!("*Unarmed Strike.* {:+} to hit", prof_bonus + str_mod));
        }

        format!(
            r#"#block(
    width: 100%,
    inset: (x: 6pt, y: 3pt),
    stroke: (bottom: 0.5pt + colors.border-light),
  )[
    #set text(size: 6pt)
    #text(size: 7pt, weight: "bold")[Attacks]
    #v(2pt)
    {}
  ]"#,
            attacks.join("\n#v(2pt)\n    ")
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
        } else if self.characters.len() == 1 {
            Some(format!("{} - Battle Card", self.characters[0].name))
        } else {
            Some("Character Battle Cards".to_string())
        }
    }

    fn page_break_before(&self) -> bool {
        true
    }
}

/// Render HP tracker based on HP value
/// - 1-20 HP: Individual boxes with styled 10th box
/// - 21-100 HP: 5s and 1s grouping
/// - 101+ HP: 10s and 1s grouping
fn render_hp_tracker(hp: i32) -> String {
    match hp {
        1..=20 => render_individual_boxes(hp),
        21..=100 => render_fives_and_ones(hp),
        _ => render_tens_and_ones(hp),
    }
}

/// Render individual HP boxes (1-20 HP)
/// Every 10th box is styled differently for easier counting
fn render_individual_boxes(hp: i32) -> String {
    let mut boxes = Vec::new();
    for i in 1..=hp {
        if i % 10 == 0 {
            // Styled 10th box - slightly larger with fill
            boxes.push(
                r##"#box(width: 7pt, height: 7pt, stroke: 1pt + black, fill: rgb("#e5e5e5"))"##
                    .to_string(),
            );
        } else {
            // Regular box
            boxes.push(r##"#box(width: 6pt, height: 6pt, stroke: 0.5pt + black)"##.to_string());
        }
    }
    boxes.join("#h(1pt)")
}

/// Render 5-HP boxes plus 1-HP boxes (for 21-100 HP)
fn render_fives_and_ones(hp: i32) -> String {
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
fn render_tens_and_ones(hp: i32) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections::character::{ClassInfo, InventoryItem, Proficiencies};

    fn test_character() -> CharacterData {
        CharacterData {
            name: "Thorin".to_string(),
            player_name: Some("Player 1".to_string()),
            is_npc: false,
            race_name: Some("Dwarf".to_string()),
            background_name: Some("Soldier".to_string()),
            strength: 16,
            dexterity: 12,
            constitution: 16,
            intelligence: 10,
            wisdom: 12,
            charisma: 8,
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 100,
            pp: 0,
            traits: None,
            ideals: None,
            bonds: None,
            flaws: None,
            role: None,
            location: None,
            faction: None,
            classes: vec![ClassInfo {
                class_name: "Fighter".to_string(),
                level: 5,
                subclass_name: Some("Champion".to_string()),
                is_starting: true,
            }],
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
                InventoryItem {
                    name: "Shield".to_string(),
                    quantity: 1,
                    equipped: true,
                    attuned: false,
                    item_type: Some("S".to_string()),
                    damage: None,
                    damage_type: None,
                    armor_ac: Some(2),
                    finesse: false,
                },
            ],
            proficiencies: Proficiencies::default(),
            speed: 25,
            ac: 18, // Chain Mail 16 + Shield 2
            hit_points_max: 49,
            hit_die: "5d10".to_string(),
            spellcasting_ability: None,
            spell_save_dc: None,
            spell_attack_bonus: None,
            spell_slots: vec![0; 9],
        }
    }

    #[test]
    fn test_modifier() {
        assert_eq!(CharacterBattleCardSection::modifier(10), 0);
        assert_eq!(CharacterBattleCardSection::modifier(14), 2);
        assert_eq!(CharacterBattleCardSection::modifier(8), -1);
    }

    #[test]
    fn test_battle_card_empty() {
        let section = CharacterBattleCardSection::new(vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_battle_card_single() {
        let char = test_character();
        let section = CharacterBattleCardSection::from_single(char);
        assert_eq!(
            section.toc_title(),
            Some("Thorin - Battle Card".to_string())
        );
    }

    #[test]
    fn test_battle_card_renders() {
        let char = test_character();
        let section = CharacterBattleCardSection::from_single(char);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Check for character name
        assert!(typst.contains("Thorin"));
        // Check for PC color (green)
        assert!(typst.contains("dcfce7"));
        // Check for card dimensions
        assert!(typst.contains("width: 3.875in"));
    }

    #[test]
    fn test_npc_card_colors() {
        let mut char = test_character();
        char.is_npc = true;
        char.name = "Bartender Bob".to_string();

        let section = CharacterBattleCardSection::from_single(char);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // NPC should have blue header
        assert!(typst.contains("dbeafe"));
    }
}
