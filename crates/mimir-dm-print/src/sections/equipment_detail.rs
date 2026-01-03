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
            r#"#text(size: 14pt, weight: "bold")[{} - Inventory]

#v(8pt)
#line(length: 100%, stroke: 0.5pt + luma(200))
#v(12pt)

"#,
            escape_typst(&c.character_name),
        ));

        // ==== CURRENCY (inline) ====
        let currency = &c.currency;
        let mut coin_parts = Vec::new();
        if currency.platinum > 0 { coin_parts.push(format!("{} pp", currency.platinum)); }
        if currency.gold > 0 { coin_parts.push(format!("{} gp", currency.gold)); }
        if currency.electrum > 0 { coin_parts.push(format!("{} ep", currency.electrum)); }
        if currency.silver > 0 { coin_parts.push(format!("{} sp", currency.silver)); }
        if currency.copper > 0 { coin_parts.push(format!("{} cp", currency.copper)); }

        if !coin_parts.is_empty() {
            typst.push_str(&format!(
                "#text(weight: \"bold\")[Currency:] {}\n\n#v(8pt)\n\n",
                coin_parts.join(", ")
            ));
        }

        // ==== EQUIPPED (inline) ====
        let equipped = &c.equipped;
        let mut equipped_parts = Vec::new();
        if let Some(ref armor) = equipped.armor {
            equipped_parts.push(format!("Armor: {}", escape_typst(armor)));
        }
        if let Some(ref shield) = equipped.shield {
            equipped_parts.push(format!("Shield: {}", escape_typst(shield)));
        }
        if let Some(ref main_hand) = equipped.main_hand {
            equipped_parts.push(format!("Main: {}", escape_typst(main_hand)));
        }
        if let Some(ref off_hand) = equipped.off_hand {
            equipped_parts.push(format!("Off-hand: {}", escape_typst(off_hand)));
        }

        if !equipped_parts.is_empty() {
            typst.push_str(&format!(
                "#text(weight: \"bold\")[Equipped:] {}\n\n#v(8pt)\n\n",
                equipped_parts.join(" | ")
            ));
        }

        // ==== SIMPLE INVENTORY LIST ====
        typst.push_str("#text(weight: \"bold\")[Inventory:]\n#v(4pt)\n\n");

        if c.inventory.is_empty() {
            typst.push_str("#text(fill: luma(150))[No items]\n");
        } else {
            typst.push_str("#table(\n  columns: (auto, 1fr, auto),\n  stroke: none,\n  inset: (x: 8pt, y: 4pt),\n  align: (right, left, right),\n\n");

            for item in &c.inventory {
                let qty = if item.quantity > 1 {
                    format!("{}×", item.quantity)
                } else {
                    String::new()
                };

                let weight_str = if item.weight > 0.0 {
                    format!("{:.1} lb", item.weight * item.quantity as f64)
                } else {
                    String::new()
                };

                typst.push_str(&format!(
                    "  [{}], [{}], [{}],\n",
                    qty,
                    escape_typst(&item.name),
                    weight_str
                ));
            }

            typst.push_str(")\n\n");
        }

        // ==== FOOTER ====
        typst.push_str(&format!(
            r#"#v(1fr)
#line(length: 100%, stroke: 0.5pt + luma(200))
#v(4pt)
#text(size: 9pt)[Total: {:.1} lbs / {} lbs capacity]
"#,
            self.total_weight(),
            self.carry_capacity()
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
        AbilityScores, Appearance, ClassLevel, Currency, EquippedItems, InventoryItem, Personality,
        Proficiencies, RoleplayNotes, SpellData as CharacterSpellData,
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
            player_name: None,
            appearance: Appearance::default(),
            backstory: None,
            background_feature: None,
            roleplay_notes: RoleplayNotes::default(),
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
        // Check for items
        assert!(typst.contains("Frostbite Blade"));
        // Check for currency
        assert!(typst.contains("250 gp"));
        // Check for capacity
        assert!(typst.contains("270 lbs capacity"));
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
