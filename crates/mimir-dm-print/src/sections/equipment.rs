//! Equipment cards section
//!
//! Generates printable equipment cards (2.5in x 3.5in) for weapons,
//! special ammo, and magic items. Uses black/white design with
//! equipment icons to distinguish from spell cards.

use serde_json::Value;

use crate::builder::{RenderContext, Renderable};
use crate::error::Result;

/// Equipment cards section - generates multi-up equipment cards for printing
pub struct EquipmentCardsSection {
    /// Equipment data (JSON array of catalog items)
    items: Vec<Value>,
    /// Show cut lines between cards
    show_cut_lines: bool,
}

impl EquipmentCardsSection {
    /// Create a new equipment cards section
    pub fn new(items: Vec<Value>) -> Self {
        Self {
            items,
            show_cut_lines: true,
        }
    }

    /// Create from a JSON value (expects array)
    pub fn from_json(items: Value) -> Self {
        let item_vec = items
            .as_array()
            .map(|arr| arr.clone())
            .unwrap_or_default();
        Self::new(item_vec)
    }

    /// Set whether to show cut lines
    pub fn with_cut_lines(mut self, show: bool) -> Self {
        self.show_cut_lines = show;
        self
    }

    /// Get equipment icon based on item type
    fn get_icon(item_type: &str) -> &'static str {
        // Extract base type before any | separator
        let base_type = if let Some(pipe_pos) = item_type.find('|') {
            &item_type[..pipe_pos]
        } else {
            item_type
        };

        match base_type {
            // Melee weapons - sword icon
            "M" => "sword-icon",
            // Ranged weapons and ammunition - bow icon
            "R" | "A" | "AF" => "bow-icon",
            // Armor and shields - shield icon
            "S" | "LA" | "MA" | "HA" => "shield-icon",
            // Wondrous items, rings, rods, wands - gem/star icon
            "RG" | "RD" | "WD" | "W" => "gem-icon",
            // Default to gear icon for other types
            _ => "gear-icon",
        }
    }

    /// Get human-readable type name
    fn get_type_name(item_type: &str) -> &'static str {
        let base_type = if let Some(pipe_pos) = item_type.find('|') {
            &item_type[..pipe_pos]
        } else {
            item_type
        };

        match base_type {
            "M" => "Melee Weapon",
            "R" => "Ranged Weapon",
            "A" => "Ammunition",
            "AF" => "Special Ammunition",
            "S" => "Shield",
            "LA" => "Light Armor",
            "MA" => "Medium Armor",
            "HA" => "Heavy Armor",
            "RG" => "Ring",
            "RD" => "Rod",
            "WD" => "Wand",
            "W" => "Wondrous Item",
            "P" => "Potion",
            "SC" => "Scroll",
            _ => "Equipment",
        }
    }

    /// Format rarity for display
    fn format_rarity(rarity: &str) -> &str {
        match rarity.to_lowercase().as_str() {
            "common" => "Common",
            "uncommon" => "Uncommon",
            "rare" => "Rare",
            "very rare" | "veryrare" => "Very Rare",
            "legendary" => "Legendary",
            "artifact" => "Artifact",
            "none" | "" => "",
            _ => rarity,
        }
    }

    /// Render a single equipment card
    fn render_card(item: &Value) -> String {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Item");
        let item_type = item
            .get("type")
            .or_else(|| item.get("item_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("G");
        let rarity = item
            .get("rarity")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        let source = item
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Damage info
        let dmg1 = item.get("dmg1").and_then(|v| v.as_str());
        let dmg2 = item.get("dmg2").and_then(|v| v.as_str());
        let dmg_type = item.get("dmg_type").or_else(|| item.get("dmgType")).and_then(|v| v.as_str());

        let damage_str = match (dmg1, dmg2, dmg_type) {
            (Some(d1), Some(d2), Some(dt)) => format!("{}/{} {}", d1, d2, format_damage_type(dt)),
            (Some(d1), None, Some(dt)) => format!("{} {}", d1, format_damage_type(dt)),
            (Some(d1), Some(d2), None) => format!("{}/{}", d1, d2),
            (Some(d1), None, None) => d1.to_string(),
            _ => String::new(),
        };

        // Properties
        let properties = item
            .get("property")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| p.as_str())
                    .map(format_property)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        // Range
        let range = item.get("range").and_then(|v| v.as_str()).unwrap_or("");

        // AC
        let ac = item.get("ac").and_then(|v| v.as_i64());

        // Attunement
        let attunement = item
            .get("requires_attunement")
            .or_else(|| item.get("reqAttune"))
            .and_then(|v| {
                if v.is_string() {
                    Some(v.as_str().unwrap())
                } else if v.as_bool() == Some(true) {
                    Some("true")
                } else {
                    None
                }
            });

        // Description from entries
        let description = if let Some(entries) = item.get("entries").and_then(|v| v.as_array()) {
            entries
                .iter()
                .filter_map(|e| e.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        };

        // User notes (if present)
        let notes = item.get("notes").and_then(|v| v.as_str()).unwrap_or("");

        // Truncate description for card
        let desc_text = if !notes.is_empty() && description.is_empty() {
            notes.to_string()
        } else {
            description
        };
        let desc_truncated: String = desc_text.chars().take(350).collect();
        let desc_display = if desc_text.len() > 350 {
            format!("{}...", desc_truncated)
        } else {
            desc_truncated
        };

        let icon_type = Self::get_icon(item_type);
        let type_name = Self::get_type_name(item_type);
        let rarity_display = Self::format_rarity(rarity);

        // Build attunement footer text
        let attune_text = match attunement {
            Some("true") => "Requires Attunement",
            Some(req) if !req.is_empty() => req,
            _ => "",
        };

        format!(
            r##"box(
  width: 2.5in,
  height: 3.5in,
  stroke: 0.5pt + black,
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // Header - black/white with icon
  #block(
    width: 100%,
    fill: luma(240),
    inset: (x: 4pt, y: 3pt),
  )[
    #grid(
      columns: (auto, 1fr),
      column-gutter: 4pt,
      {{icon}}(size: sizes.sm),
      [
        #text(size: 7pt, weight: "bold")[{}]
      ]
    )
    #text(size: 5pt, fill: luma(80))[
      {} {}
    ]
  ]

  // Stats
  #block(
    width: 100%,
    inset: 4pt,
    stroke: (bottom: 0.5pt + luma(200)),
  )[
    #set text(size: 6pt)
    #grid(
      columns: (auto, 1fr),
      row-gutter: 1pt,
      {}{}{}{}
    )
  ]

  // Description
  #block(
    width: 100%,
    inset: 4pt,
  )[
    #text(size: 5.5pt)[{}]
  ]

  // Footer
  #place(
    bottom + left,
    block(
      width: 100%,
      fill: luma(240),
      inset: (x: 4pt, y: 2pt),
    )[
      #text(size: 4pt, fill: luma(80))[
        {}
        #h(1fr)
        {}
      ]
    ]
  )
]"##,
            escape_typst(name),
            type_name,
            if rarity_display.is_empty() { "".to_string() } else { format!("— {}", rarity_display) },
            // Stats rows
            if !damage_str.is_empty() { format!("[*Damage:*], [{}],\n      ", escape_typst(&damage_str)) } else { String::new() },
            if let Some(ac_val) = ac { format!("[*AC:*], [{}],\n      ", ac_val) } else { String::new() },
            if !properties.is_empty() { format!("[*Properties:*], [{}],\n      ", escape_typst(&properties)) } else { String::new() },
            if !range.is_empty() { format!("[*Range:*], [{}],\n      ", escape_typst(range)) } else { String::new() },
            escape_typst(&desc_display),
            escape_typst(attune_text),
            escape_typst(source),
        ).replace("{icon}", icon_type)
    }
}

impl Renderable for EquipmentCardsSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.items.is_empty() {
            return Ok("// No equipment to display\n".to_string());
        }

        let mut typst = String::new();
        let cards_per_page = 9;
        let total_pages = (self.items.len() + cards_per_page - 1) / cards_per_page;

        // Equipment icons defined as simple Typst functions
        typst.push_str(r#"// Equipment icons (black/white)
#let sword-icon(size: 12pt) = text(size: size)[⚔]
#let bow-icon(size: 12pt) = text(size: size)[🏹]
#let shield-icon(size: 12pt) = text(size: size)[🛡]
#let gem-icon(size: 12pt) = text(size: size)[💎]
#let gear-icon(size: 12pt) = text(size: size)[⚙]

"#);

        // Set tight margins to fit 3x3 grid of 2.5in x 3.5in cards
        typst.push_str("#set page(paper: \"us-letter\", margin: 0.25in)\n");

        for page_num in 0..total_pages {
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, self.items.len());
            let page_items = &self.items[start_idx..end_idx];

            if page_num > 0 {
                typst.push_str("\n#pagebreak()\n");
            }

            // Center the card grid
            typst.push_str("#align(center)[\n  #grid(\n");
            typst.push_str("    columns: (2.5in,) * 3,\n");
            typst.push_str("    rows: (3.5in,) * 3,\n");
            typst.push_str(&format!(
                "    column-gutter: {},\n",
                if self.show_cut_lines { "0pt" } else { "4pt" }
            ));
            typst.push_str(&format!(
                "    row-gutter: {},\n\n",
                if self.show_cut_lines { "0pt" } else { "4pt" }
            ));

            // Render each card in this page
            for (i, item) in page_items.iter().enumerate() {
                typst.push_str("    ");
                typst.push_str(&Self::render_card(item));
                if i < page_items.len() - 1 || page_items.len() < 9 {
                    typst.push(',');
                }
                typst.push('\n');
            }

            // Fill remaining slots with empty boxes
            for _ in page_items.len()..9 {
                typst.push_str("    box(width: 2.5in, height: 3.5in),\n");
            }

            typst.push_str("  )\n]\n");

            // Cut lines indicator
            if self.show_cut_lines && !page_items.is_empty() {
                typst.push_str("#place(\n  bottom + center,\n  dy: 0.1in,\n");
                typst.push_str("  text(size: 6pt, fill: luma(150))[Cut along card borders]\n)\n");
            }
        }

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.items.is_empty() {
            None
        } else {
            Some("Equipment Cards".to_string())
        }
    }
}

/// Format damage type abbreviation to full name
fn format_damage_type(dt: &str) -> &str {
    match dt.to_uppercase().as_str() {
        "S" => "slashing",
        "P" => "piercing",
        "B" => "bludgeoning",
        "F" => "fire",
        "C" => "cold",
        "L" => "lightning",
        "A" => "acid",
        "T" => "thunder",
        "N" => "necrotic",
        "R" => "radiant",
        "O" => "force",
        "Y" => "psychic",
        "I" => "poison",
        _ => dt,
    }
}

/// Format property abbreviation to full name
fn format_property(prop: &str) -> &'static str {
    match prop.to_uppercase().as_str() {
        "A" => "Ammunition",
        "F" => "Finesse",
        "H" => "Heavy",
        "L" => "Light",
        "LD" => "Loading",
        "R" => "Reach",
        "S" => "Special",
        "T" => "Thrown",
        "2H" => "Two-Handed",
        "V" => "Versatile",
        "RLD" => "Reload",
        "BF" => "Burst Fire",
        _ => "Special",
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
        .replace('_', "\\_")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('@', "\\@")
}

/// Check if an item is "card-worthy" based on type, rarity, and content
pub fn is_card_worthy(item: &Value) -> bool {
    let item_type = item
        .get("type")
        .or_else(|| item.get("item_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let base_type = if let Some(pipe_pos) = item_type.find('|') {
        &item_type[..pipe_pos]
    } else {
        item_type
    };

    let rarity = item
        .get("rarity")
        .and_then(|v| v.as_str())
        .unwrap_or("none")
        .to_lowercase();

    let has_attunement = item
        .get("requires_attunement")
        .or_else(|| item.get("reqAttune"))
        .map(|v| v.as_bool() == Some(true) || v.is_string())
        .unwrap_or(false);

    let has_entries = item
        .get("entries")
        .and_then(|v| v.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    let has_notes = item
        .get("notes")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    // Weapons are always card-worthy
    if base_type == "M" || base_type == "R" {
        return true;
    }

    // Special ammo (A, AF) with rarity
    if (base_type == "A" || base_type == "AF") && rarity != "none" && !rarity.is_empty() {
        return true;
    }

    // Magic items (rarity != "none")
    if rarity != "none" && !rarity.is_empty() {
        return true;
    }

    // Items requiring attunement
    if has_attunement {
        return true;
    }

    // Items with special abilities
    if has_entries {
        return true;
    }

    // Items with user notes
    if has_notes {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_equipment_cards_empty() {
        let section = EquipmentCardsSection::new(vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_equipment_cards_with_items() {
        let items = vec![json!({
            "name": "Longsword",
            "type": "M",
            "rarity": "none",
            "dmg1": "1d8",
            "dmg_type": "S"
        })];
        let section = EquipmentCardsSection::new(items);
        assert_eq!(section.toc_title(), Some("Equipment Cards".to_string()));
    }

    #[test]
    fn test_is_card_worthy_weapon() {
        let weapon = json!({
            "name": "Longsword",
            "type": "M",
            "rarity": "none"
        });
        assert!(is_card_worthy(&weapon));

        let ranged = json!({
            "name": "Longbow",
            "type": "R",
            "rarity": "none"
        });
        assert!(is_card_worthy(&ranged));
    }

    #[test]
    fn test_is_card_worthy_magic_item() {
        let magic = json!({
            "name": "Ring of Protection",
            "type": "RG",
            "rarity": "rare"
        });
        assert!(is_card_worthy(&magic));
    }

    #[test]
    fn test_is_card_worthy_special_ammo() {
        let ammo = json!({
            "name": "Arrow +1",
            "type": "A",
            "rarity": "uncommon"
        });
        assert!(is_card_worthy(&ammo));

        // Non-magic ammo is not card-worthy
        let normal_ammo = json!({
            "name": "Arrow",
            "type": "A",
            "rarity": "none"
        });
        assert!(!is_card_worthy(&normal_ammo));
    }

    #[test]
    fn test_is_card_worthy_with_notes() {
        let with_notes = json!({
            "name": "Custom Item",
            "type": "G",
            "rarity": "none",
            "notes": "Found in dungeon level 3"
        });
        assert!(is_card_worthy(&with_notes));
    }

    #[test]
    fn test_is_card_worthy_mundane() {
        let mundane = json!({
            "name": "Rope",
            "type": "G",
            "rarity": "none"
        });
        assert!(!is_card_worthy(&mundane));
    }

    #[test]
    fn test_format_damage_type() {
        assert_eq!(format_damage_type("S"), "slashing");
        assert_eq!(format_damage_type("P"), "piercing");
        assert_eq!(format_damage_type("B"), "bludgeoning");
        assert_eq!(format_damage_type("F"), "fire");
    }

    #[test]
    fn test_format_property() {
        assert_eq!(format_property("V"), "Versatile");
        assert_eq!(format_property("F"), "Finesse");
        assert_eq!(format_property("2H"), "Two-Handed");
    }

    #[test]
    fn test_get_icon() {
        assert_eq!(EquipmentCardsSection::get_icon("M"), "sword-icon");
        assert_eq!(EquipmentCardsSection::get_icon("R"), "bow-icon");
        assert_eq!(EquipmentCardsSection::get_icon("S"), "shield-icon");
        assert_eq!(EquipmentCardsSection::get_icon("RG"), "gem-icon");
        assert_eq!(EquipmentCardsSection::get_icon("M|PHB"), "sword-icon");
    }
}
