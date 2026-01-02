//! Equipment detail section
//!
//! Detailed inventory list with item descriptions and special rules.

use mimir_dm_core::models::character::data::CharacterData;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Equipment detail section - full inventory with descriptions
pub struct EquipmentDetailSection {
    character: CharacterData,
}

impl EquipmentDetailSection {
    pub fn new(character: CharacterData) -> Self {
        Self { character }
    }

    /// Calculate total weight of all inventory items
    fn total_weight(&self) -> f64 {
        self.character
            .inventory
            .iter()
            .map(|item| item.weight * item.quantity as f64)
            .sum()
    }

    /// Calculate carry capacity (STR * 15)
    fn carry_capacity(&self) -> i32 {
        self.character.abilities.strength * 15
    }
}

impl Renderable for EquipmentDetailSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        let c = &self.character;
        let mut typst = String::new();

        // Page setup
        typst.push_str(r#"#set page(paper: "us-letter", margin: (x: 0.75in, y: 0.75in))
"#);

        // ==== HEADER ====
        typst.push_str(&format!(
            r#"#align(center)[
  #text(size: 14pt, weight: "bold", fill: colors.accent)[EQUIPMENT & INVENTORY]
]

#v(8pt)

#grid(
  columns: (1fr, auto),
  column-gutter: 16pt,
  [#text(size: 12pt, weight: "bold")[{}]],
  [#text(size: 10pt)[Carry Capacity: STR {} × 15 = {} lbs]],
)

#line(length: 100%, stroke: 1pt + colors.border)
#v(16pt)

"#,
            escape_typst(&c.character_name),
            c.abilities.strength,
            self.carry_capacity()
        ));

        // ==== CURRENCY ====
        let currency = &c.currency;
        typst.push_str(r#"#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 10pt, weight: "bold")[CURRENCY]
  #v(8pt)
  #grid(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr),
    column-gutter: 8pt,
"#);

        for (label, value) in [
            ("CP", currency.copper),
            ("SP", currency.silver),
            ("EP", currency.electrum),
            ("GP", currency.gold),
            ("PP", currency.platinum),
        ] {
            typst.push_str(&format!(
                r#"    box(
      stroke: 0.5pt + luma(200),
      radius: 2pt,
      inset: 8pt,
      width: 100%,
      align(center)[
        #text(size: 8pt, fill: luma(100))[{}]
        #v(2pt)
        #text(size: 14pt, weight: "bold")[{}]
      ]
    ),
"#,
                label, value
            ));
        }

        typst.push_str("  )\n]\n\n#v(16pt)\n\n");

        // ==== EQUIPMENT LIST ====
        typst.push_str(r#"#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 10pt, weight: "bold")[EQUIPMENT]
  #v(8pt)

"#);

        if c.inventory.is_empty() {
            // Empty inventory placeholder
            typst.push_str(r#"  #align(center)[
    #text(size: 10pt, fill: luma(150))[No items in inventory]
  ]
  #v(40pt)
"#);
        } else {
            // Table header
            typst.push_str(r#"  #table(
    columns: (1fr, auto, 1fr),
    stroke: none,
    inset: 6pt,
    fill: (_, row) => if row == 0 { luma(240) } else { none },
    align: (left, center, left),

    // Header
    [#text(size: 9pt, weight: "bold")[Item]],
    [#text(size: 9pt, weight: "bold")[Weight]],
    [#text(size: 9pt, weight: "bold")[Properties / Notes]],

"#);

            for item in &c.inventory {
                let weight_str = if item.weight > 0.0 {
                    format!("{} lbs", item.weight * item.quantity as f64)
                } else {
                    "—".to_string()
                };

                let name_with_qty = if item.quantity > 1 {
                    format!("{} (×{})", item.name, item.quantity)
                } else {
                    item.name.clone()
                };

                // Standard row
                typst.push_str(&format!(
                    "    [{}],\n    [{}],\n    [],\n",
                    escape_typst(&name_with_qty),
                    weight_str
                ));

                // If item has notes, add a full-width description row
                if let Some(ref notes) = item.notes {
                    if !notes.is_empty() {
                        typst.push_str(&format!(
                            r#"    // Description for {}
    table.cell(colspan: 3)[
      #box(
        width: 100%,
        fill: luma(250),
        inset: 8pt,
        radius: 2pt,
      )[
        #text(size: 8pt, style: "italic")[{}]
      ]
    ],
"#,
                            escape_typst(&item.name),
                            escape_typst(notes)
                        ));
                    }
                }
            }

            typst.push_str("  )\n");
        }

        typst.push_str("]\n\n");

        // ==== EQUIPPED ITEMS ====
        let equipped = &c.equipped;
        let has_equipped = equipped.armor.is_some()
            || equipped.shield.is_some()
            || equipped.main_hand.is_some()
            || equipped.off_hand.is_some();

        if has_equipped {
            typst.push_str(r#"#v(16pt)

#box(
  width: 100%,
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #text(size: 10pt, weight: "bold")[EQUIPPED]
  #v(8pt)

  #grid(
    columns: (auto, 1fr),
    column-gutter: 16pt,
    row-gutter: 4pt,
"#);

            if let Some(ref armor) = equipped.armor {
                typst.push_str(&format!(
                    "    [#text(size: 9pt, fill: luma(100))[Armor:]], [#text(size: 9pt)[{}]],\n",
                    escape_typst(armor)
                ));
            }
            if let Some(ref shield) = equipped.shield {
                typst.push_str(&format!(
                    "    [#text(size: 9pt, fill: luma(100))[Shield:]], [#text(size: 9pt)[{}]],\n",
                    escape_typst(shield)
                ));
            }
            if let Some(ref main_hand) = equipped.main_hand {
                typst.push_str(&format!(
                    "    [#text(size: 9pt, fill: luma(100))[Main Hand:]], [#text(size: 9pt)[{}]],\n",
                    escape_typst(main_hand)
                ));
            }
            if let Some(ref off_hand) = equipped.off_hand {
                typst.push_str(&format!(
                    "    [#text(size: 9pt, fill: luma(100))[Off Hand:]], [#text(size: 9pt)[{}]],\n",
                    escape_typst(off_hand)
                ));
            }

            typst.push_str("  )\n]\n\n");
        }

        // ==== TOTAL WEIGHT ====
        typst.push_str(&format!(
            r#"#v(1fr)

#line(length: 100%, stroke: 1pt + colors.border)
#v(8pt)

#grid(
  columns: (1fr, auto),
  [#text(size: 10pt, fill: luma(100))[Total Weight]],
  [#text(size: 12pt, weight: "bold")[{:.1} lbs]],
)

#v(8pt)
#align(center)[
  #text(size: 8pt, fill: luma(150))[Generated by Mimir]
]
"#,
            self.total_weight()
        ));

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        Some(format!("{} - Equipment", self.character.character_name))
    }

    fn page_break_before(&self) -> bool {
        true
    }
}

/// Escape special Typst characters
fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mimir_dm_core::models::character::data::{
        AbilityScores, ClassLevel, Currency, EquippedItems, InventoryItem, Personality,
        Proficiencies, SpellData as CharacterSpellData,
    };

    fn sample_character() -> CharacterData {
        CharacterData {
            character_name: "Thorin Ironforge".to_string(),
            player_id: Some(1),
            level: 8,
            experience_points: 34000,
            version: 1,
            snapshot_reason: None,
            created_at: "2025-01-01".to_string(),
            race: "Dwarf".to_string(),
            subrace: Some("Mountain".to_string()),
            classes: vec![ClassLevel {
                class_name: "Fighter".to_string(),
                level: 8,
                subclass: Some("Battlemaster".to_string()),
                hit_dice_type: "d10".to_string(),
                hit_dice_remaining: 8,
            }],
            background: "Soldier".to_string(),
            alignment: Some("Lawful Good".to_string()),
            abilities: AbilityScores {
                strength: 18,
                dexterity: 12,
                constitution: 16,
                intelligence: 10,
                wisdom: 14,
                charisma: 8,
            },
            max_hp: 76,
            current_hp: 76,
            speed: 25,
            proficiencies: Proficiencies::default(),
            class_features: vec![],
            feats: vec![],
            spells: CharacterSpellData::default(),
            inventory: vec![
                InventoryItem {
                    name: "Frostbite Blade".to_string(),
                    source: None,
                    quantity: 1,
                    weight: 3.0,
                    value: 5000.0,
                    notes: Some("A longsword forged in the frozen peaks. Once per day, create 5ft radius of icy difficult terrain around a struck target.".to_string()),
                },
                InventoryItem {
                    name: "Chain Mail".to_string(),
                    source: None,
                    quantity: 1,
                    weight: 55.0,
                    value: 75.0,
                    notes: None,
                },
                InventoryItem {
                    name: "Rations".to_string(),
                    source: None,
                    quantity: 10,
                    weight: 2.0,
                    value: 0.5,
                    notes: None,
                },
                InventoryItem {
                    name: "Bedroll".to_string(),
                    source: None,
                    quantity: 1,
                    weight: 7.0,
                    value: 1.0,
                    notes: None,
                },
            ],
            currency: Currency {
                copper: 23,
                silver: 45,
                electrum: 0,
                gold: 250,
                platinum: 5,
            },
            equipped: EquippedItems {
                armor: Some("Chain Mail".to_string()),
                shield: Some("Shield".to_string()),
                main_hand: Some("Frostbite Blade".to_string()),
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
    fn test_equipment_detail_basic() {
        let character = sample_character();
        let section = EquipmentDetailSection::new(character);
        assert_eq!(
            section.toc_title(),
            Some("Thorin Ironforge - Equipment".to_string())
        );
    }

    #[test]
    fn test_equipment_detail_generates_typst() {
        let character = sample_character();
        let section = EquipmentDetailSection::new(character);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // Check for character name
        assert!(typst.contains("Thorin Ironforge"));
        // Check for section headers
        assert!(typst.contains("EQUIPMENT & INVENTORY"));
        assert!(typst.contains("CURRENCY"));
        // Check for items
        assert!(typst.contains("Frostbite Blade"));
        assert!(typst.contains("frozen peaks"));
        // Check for currency
        assert!(typst.contains("GP"));
        assert!(typst.contains("250"));
    }

    #[test]
    fn test_total_weight_calculation() {
        let character = sample_character();
        let section = EquipmentDetailSection::new(character);
        // 3 + 55 + 20 + 7 = 85
        assert!((section.total_weight() - 85.0).abs() < 0.01);
    }

    #[test]
    fn test_carry_capacity() {
        let character = sample_character();
        let section = EquipmentDetailSection::new(character);
        // STR 18 * 15 = 270
        assert_eq!(section.carry_capacity(), 270);
    }
}
