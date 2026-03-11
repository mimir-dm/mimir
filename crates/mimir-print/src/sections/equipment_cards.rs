//! Equipment cards section
//!
//! Generates printable equipment cards (2.5in x 3.5in) for weapons,
//! special ammo, and magic items. Uses black/white design with
//! equipment icons to distinguish from spell cards.

use serde_json::Value;

use super::card_utils::{escape_typst, flatten_entries, split_text_natural, SMALL_CARD_DESC_BUDGET};
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

    /// Extract common fields from an item for card rendering
    fn extract_fields(item: &Value) -> CardFields {
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

        let range = item.get("range").and_then(|v| v.as_str()).unwrap_or("");
        let ac = item.get("ac").and_then(|v| v.as_i64());

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
            flatten_entries(entries)
        } else {
            String::new()
        };

        let notes = item.get("notes").and_then(|v| v.as_str()).unwrap_or("");
        let desc_text = if !notes.is_empty() && description.is_empty() {
            notes.to_string()
        } else {
            description
        };

        let icon_type = Self::get_icon(item_type);
        let type_name = Self::get_type_name(item_type);
        let rarity_display = Self::format_rarity(rarity);

        let attune_text = match attunement {
            Some("true") => "Requires Attunement",
            Some(req) if !req.is_empty() => req,
            _ => "",
        };

        // Build stats rows
        let damage_row = if !damage_str.is_empty() {
            format!("[*Damage:*], [{}],\n      ", escape_typst(&damage_str))
        } else {
            String::new()
        };
        let ac_row = if let Some(ac_val) = ac {
            format!("[*AC:*], [{}],\n      ", ac_val)
        } else {
            String::new()
        };
        let properties_row = if !properties.is_empty() {
            format!("[*Properties:*], [{}],\n      ", escape_typst(&properties))
        } else {
            String::new()
        };
        let range_row = if !range.is_empty() {
            format!("[*Range:*], [{}],\n      ", escape_typst(range))
        } else {
            String::new()
        };

        let rarity_str = if rarity_display.is_empty() {
            String::new()
        } else {
            format!("— {}", rarity_display)
        };

        CardFields {
            name: name.to_string(),
            source: source.to_string(),
            icon_type,
            type_name,
            rarity_str,
            damage_row,
            ac_row,
            properties_row,
            range_row,
            attune_text: attune_text.to_string(),
            desc_text,
        }
    }

    /// Render the front card for an item. Returns (front_card, Option<back_card>).
    fn render_cards(item: &Value) -> (String, Option<String>) {
        let f = Self::extract_fields(item);
        let split = split_text_natural(&f.desc_text, SMALL_CARD_DESC_BUDGET);

        let fold_indicator = if split.is_foldable { " ▶ continued" } else { "" };

        let front = format!(
            r#"box(
  width: 2.5in,
  height: 3.25in,
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
      {icon}(size: sizes.sm),
      [
        #text(size: 7pt, weight: "bold")[{name}]
      ]
    )
    #text(size: 5pt, fill: luma(80))[
      {type_name} {rarity_str}
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
      {damage_row}{ac_row}{properties_row}{range_row}
    )
  ]

  // Description
  #block(
    width: 100%,
    inset: 4pt,
  )[
    #text(size: 5.5pt)[{desc_display}]
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
        {attune_text}
        #h(1fr)
        {source}{fold_indicator}
      ]
    ]
  )
]"#,
            icon = f.icon_type,
            name = escape_typst(&f.name),
            type_name = f.type_name,
            rarity_str = f.rarity_str,
            damage_row = f.damage_row,
            ac_row = f.ac_row,
            properties_row = f.properties_row,
            range_row = f.range_row,
            desc_display = escape_typst(&split.front),
            attune_text = escape_typst(&f.attune_text),
            source = escape_typst(&f.source),
            fold_indicator = fold_indicator,
        );

        let back = if split.is_foldable {
            Some(format!(
                r#"box(
  width: 2.5in,
  height: 3.25in,
  stroke: 0.5pt + black,
  radius: 3pt,
  clip: true,
  inset: 0pt,
)[
  // Header - continuation
  #block(
    width: 100%,
    fill: luma(240),
    inset: (x: 4pt, y: 3pt),
  )[
    #text(size: 7pt, weight: "bold")[{name}]
    #h(1fr)
    #text(size: 5pt, style: "italic", fill: luma(100))[(continued)]
  ]

  // Description continued
  #block(
    width: 100%,
    inset: 4pt,
  )[
    #text(size: 5.5pt)[{desc_continued}]
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
        ◀ fold
        #h(1fr)
        {source}
      ]
    ]
  )
]"#,
                name = escape_typst(&f.name),
                desc_continued = escape_typst(&split.back),
                source = escape_typst(&f.source),
            ))
        } else {
            None
        };

        (front, back)
    }

    /// Render a single equipment card (front only, used by tests)
    #[cfg(test)]
    fn render_card(item: &Value) -> String {
        Self::render_cards(item).0
    }
}

impl Renderable for EquipmentCardsSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.items.is_empty() {
            return Ok("// No equipment to display\n".to_string());
        }

        let mut typst = String::new();

        // Equipment icons defined as simple Typst functions
        typst.push_str(r#"// Equipment icons (black/white)
#let sword-icon(size: 12pt) = text(size: size)[⚔]
#let bow-icon(size: 12pt) = text(size: size)[🏹]
#let shield-icon(size: 12pt) = text(size: size)[🛡]
#let gem-icon(size: 12pt) = text(size: size)[💎]
#let gear-icon(size: 12pt) = text(size: size)[⚙]

"#);

        // Set page margins for equipment cards (centered with gutters for cutting)
        typst.push_str("#set page(paper: \"us-letter\", margin: 0.25in)\n");

        // Pre-render all cards, collecting front + continuation cards into a flat list.
        // Continuation cards are placed immediately after their front card so they
        // end up adjacent in the grid (easy to fold together after cutting).
        let mut all_cards: Vec<String> = Vec::new();
        for item in &self.items {
            let (front, back) = Self::render_cards(item);
            all_cards.push(front);
            if let Some(back_card) = back {
                all_cards.push(back_card);
            }
        }

        let cards_per_page = 9;
        let total_pages = (all_cards.len() + cards_per_page - 1) / cards_per_page;
        let has_foldable = all_cards.len() > self.items.len();

        for page_num in 0..total_pages {
            let start_idx = page_num * cards_per_page;
            let end_idx = std::cmp::min(start_idx + cards_per_page, all_cards.len());
            let page_cards = &all_cards[start_idx..end_idx];

            if page_num > 0 {
                typst.push_str("\n#pagebreak()\n");
            }

            // Card grid (3x3) - cards sized to fit with gutters for cutting
            // align(center) centers the grid on the page; set align(left) inside
            // to prevent center alignment from cascading into card text
            typst.push_str("#align(center)[#align(left)[\n  #grid(\n");
            typst.push_str("    columns: (2.5in,) * 3,\n");
            typst.push_str("    rows: (3.25in,) * 3,\n");
            typst.push_str("    column-gutter: 0.25in,\n");
            typst.push_str("    row-gutter: 0.25in,\n\n");

            // Render each card in this page
            for (i, card) in page_cards.iter().enumerate() {
                typst.push_str("    ");
                typst.push_str(card);
                if i < page_cards.len() - 1 || page_cards.len() < 9 {
                    typst.push(',');
                }
                typst.push('\n');
            }

            // Fill remaining slots with empty boxes
            for _ in page_cards.len()..9 {
                typst.push_str("    box(width: 2.5in, height: 3.25in),\n");
            }

            typst.push_str("  )\n]]\n");

            // Cut lines indicator
            if self.show_cut_lines && !page_cards.is_empty() {
                let hint = if has_foldable {
                    "Cut along card borders — fold adjacent cards for extended descriptions"
                } else {
                    "Cut along card borders"
                };
                typst.push_str(&format!(
                    "#place(\n  bottom + center,\n  dy: 0.1in,\n  text(size: 6pt, fill: luma(150))[{}]\n)\n",
                    hint
                ));
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

    fn page_break_before(&self) -> bool {
        true
    }
}

/// Extracted fields for equipment card rendering
struct CardFields {
    name: String,
    source: String,
    icon_type: &'static str,
    type_name: &'static str,
    rarity_str: String,
    damage_row: String,
    ac_row: String,
    properties_row: String,
    range_row: String,
    attune_text: String,
    desc_text: String,
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

    #[test]
    fn test_get_icon_all_types() {
        assert_eq!(EquipmentCardsSection::get_icon("A"), "bow-icon");
        assert_eq!(EquipmentCardsSection::get_icon("AF"), "bow-icon");
        assert_eq!(EquipmentCardsSection::get_icon("LA"), "shield-icon");
        assert_eq!(EquipmentCardsSection::get_icon("MA"), "shield-icon");
        assert_eq!(EquipmentCardsSection::get_icon("HA"), "shield-icon");
        assert_eq!(EquipmentCardsSection::get_icon("RD"), "gem-icon");
        assert_eq!(EquipmentCardsSection::get_icon("WD"), "gem-icon");
        assert_eq!(EquipmentCardsSection::get_icon("W"), "gem-icon");
        assert_eq!(EquipmentCardsSection::get_icon("G"), "gear-icon");
        assert_eq!(EquipmentCardsSection::get_icon("P"), "gear-icon");
    }

    #[test]
    fn test_get_type_name_all() {
        assert_eq!(EquipmentCardsSection::get_type_name("M"), "Melee Weapon");
        assert_eq!(EquipmentCardsSection::get_type_name("R"), "Ranged Weapon");
        assert_eq!(EquipmentCardsSection::get_type_name("A"), "Ammunition");
        assert_eq!(EquipmentCardsSection::get_type_name("AF"), "Special Ammunition");
        assert_eq!(EquipmentCardsSection::get_type_name("S"), "Shield");
        assert_eq!(EquipmentCardsSection::get_type_name("LA"), "Light Armor");
        assert_eq!(EquipmentCardsSection::get_type_name("MA"), "Medium Armor");
        assert_eq!(EquipmentCardsSection::get_type_name("HA"), "Heavy Armor");
        assert_eq!(EquipmentCardsSection::get_type_name("RG"), "Ring");
        assert_eq!(EquipmentCardsSection::get_type_name("RD"), "Rod");
        assert_eq!(EquipmentCardsSection::get_type_name("WD"), "Wand");
        assert_eq!(EquipmentCardsSection::get_type_name("W"), "Wondrous Item");
        assert_eq!(EquipmentCardsSection::get_type_name("P"), "Potion");
        assert_eq!(EquipmentCardsSection::get_type_name("SC"), "Scroll");
        assert_eq!(EquipmentCardsSection::get_type_name("XYZ"), "Equipment");
    }

    #[test]
    fn test_format_rarity() {
        assert_eq!(EquipmentCardsSection::format_rarity("common"), "Common");
        assert_eq!(EquipmentCardsSection::format_rarity("uncommon"), "Uncommon");
        assert_eq!(EquipmentCardsSection::format_rarity("rare"), "Rare");
        assert_eq!(EquipmentCardsSection::format_rarity("very rare"), "Very Rare");
        assert_eq!(EquipmentCardsSection::format_rarity("veryrare"), "Very Rare");
        assert_eq!(EquipmentCardsSection::format_rarity("legendary"), "Legendary");
        assert_eq!(EquipmentCardsSection::format_rarity("artifact"), "Artifact");
        assert_eq!(EquipmentCardsSection::format_rarity("none"), "");
        assert_eq!(EquipmentCardsSection::format_rarity(""), "");
    }

    #[test]
    fn test_render_card_weapon_with_damage() {
        let item = json!({
            "name": "Longsword",
            "type": "M",
            "rarity": "none",
            "dmg1": "1d8",
            "dmg2": "1d10",
            "dmg_type": "S",
            "property": ["V"],
            "source": "PHB"
        });
        let card = EquipmentCardsSection::render_card(&item);

        assert!(card.contains("Longsword"));
        assert!(card.contains("Melee Weapon"));
        assert!(card.contains("sword-icon"));
        assert!(card.contains("1d8/1d10 slashing"));
        assert!(card.contains("Versatile"));
        assert!(card.contains("PHB"));
    }

    #[test]
    fn test_render_card_armor_with_ac() {
        let item = json!({
            "name": "Chain Mail",
            "type": "HA",
            "rarity": "none",
            "ac": 16,
            "source": "PHB"
        });
        let card = EquipmentCardsSection::render_card(&item);

        assert!(card.contains("Chain Mail"));
        assert!(card.contains("Heavy Armor"));
        assert!(card.contains("shield-icon"));
        assert!(card.contains("*AC:*"));
        assert!(card.contains("16"));
    }

    #[test]
    fn test_render_card_magic_item_with_rarity() {
        let item = json!({
            "name": "Ring of Protection",
            "type": "RG",
            "rarity": "rare",
            "reqAttune": true,
            "entries": ["You gain a +1 bonus to AC and saving throws while wearing this ring."],
            "source": "DMG"
        });
        let card = EquipmentCardsSection::render_card(&item);

        assert!(card.contains("Ring of Protection"));
        assert!(card.contains("Ring"));
        assert!(card.contains("Rare"));
        assert!(card.contains("gem-icon"));
        assert!(card.contains("Requires Attunement"));
        assert!(card.contains("+1 bonus to AC"));
    }

    #[test]
    fn test_render_card_ranged_weapon_with_range() {
        let item = json!({
            "name": "Longbow",
            "type": "R",
            "rarity": "none",
            "dmg1": "1d8",
            "dmg_type": "P",
            "range": "150/600",
            "property": ["A", "H", "2H"],
            "source": "PHB"
        });
        let card = EquipmentCardsSection::render_card(&item);

        assert!(card.contains("Longbow"));
        assert!(card.contains("Ranged Weapon"));
        assert!(card.contains("bow-icon"));
        assert!(card.contains("1d8 piercing"));
        assert!(card.contains("150/600"));
        assert!(card.contains("Ammunition"));
        assert!(card.contains("Heavy"));
        assert!(card.contains("Two-Handed"));
    }

    #[test]
    fn test_render_card_attunement_with_requirement() {
        let item = json!({
            "name": "Staff of Fire",
            "type": "W",
            "rarity": "very rare",
            "reqAttune": "by a druid, sorcerer, warlock, or wizard"
        });
        let card = EquipmentCardsSection::render_card(&item);

        assert!(card.contains("by a druid"));
    }

    #[test]
    fn test_render_card_missing_type_defaults_to_gear() {
        let item = json!({
            "name": "Mystery Object"
        });
        let card = EquipmentCardsSection::render_card(&item);

        assert!(card.contains("Mystery Object"));
        assert!(card.contains("gear-icon"));
        assert!(card.contains("Equipment"));
    }

    #[test]
    fn test_render_card_short_description_fits() {
        let item = json!({
            "name": "Simple Item",
            "type": "W",
            "entries": ["A short description."]
        });
        let (front, back) = EquipmentCardsSection::render_cards(&item);

        assert!(front.contains("A short description."));
        assert!(back.is_none());
    }

    #[test]
    fn test_render_card_long_description_foldable() {
        // Create text well over the budget
        let long_desc = "This item does amazing things. ".repeat(40);
        let item = json!({
            "name": "Verbose Item",
            "type": "W",
            "rarity": "rare",
            "entries": [long_desc]
        });
        let (front, back) = EquipmentCardsSection::render_cards(&item);

        // Front should have fold indicator
        assert!(front.contains("continued"));
        // Back card should exist with continuation header
        let back = back.expect("should have continuation card");
        assert!(back.contains("continued"));
        assert!(back.contains("fold"));
    }

    #[test]
    fn test_render_card_notes_as_description() {
        let item = json!({
            "name": "Noted Item",
            "type": "G",
            "notes": "Found in the dragon's hoard"
        });
        let card = EquipmentCardsSection::render_card(&item);

        assert!(card.contains("Found in the dragon"));
    }

    #[test]
    fn test_to_typst_grid_layout() {
        let items = vec![json!({"name": "Sword", "type": "M"})];
        let section = EquipmentCardsSection::new(items);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        // 3x3 grid
        assert!(typst.contains("columns: (2.5in,) * 3"));
        assert!(typst.contains("rows: (3.25in,) * 3"));
        // 1 item = 8 empty slots
        assert_eq!(typst.matches("box(width: 2.5in, height: 3.25in)").count(), 8);
    }

    #[test]
    fn test_to_typst_cut_lines() {
        let items = vec![json!({"name": "Sword", "type": "M"})];
        let section = EquipmentCardsSection::new(items);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        assert!(typst.contains("Cut along card borders"));
    }

    #[test]
    fn test_to_typst_no_cut_lines() {
        let items = vec![json!({"name": "Sword", "type": "M"})];
        let section = EquipmentCardsSection::new(items).with_cut_lines(false);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        assert!(!typst.contains("Cut along card borders"));
    }

    #[test]
    fn test_to_typst_empty_returns_comment() {
        let section = EquipmentCardsSection::new(vec![]);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        assert!(typst.contains("No equipment to display"));
    }

    #[test]
    fn test_to_typst_page_break_after_nine() {
        let items: Vec<Value> = (0..10)
            .map(|i| json!({"name": format!("Item {}", i), "type": "M"}))
            .collect();
        let section = EquipmentCardsSection::new(items);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        assert!(typst.contains("#pagebreak()"));
    }

    #[test]
    fn test_to_typst_icon_definitions() {
        let items = vec![json!({"name": "Sword", "type": "M"})];
        let section = EquipmentCardsSection::new(items);
        let ctx = RenderContext::default();
        let typst = section.to_typst(&ctx).unwrap();

        assert!(typst.contains("sword-icon"));
        assert!(typst.contains("bow-icon"));
        assert!(typst.contains("shield-icon"));
        assert!(typst.contains("gem-icon"));
        assert!(typst.contains("gear-icon"));
    }

    #[test]
    fn test_format_damage_type_all() {
        assert_eq!(format_damage_type("C"), "cold");
        assert_eq!(format_damage_type("L"), "lightning");
        assert_eq!(format_damage_type("A"), "acid");
        assert_eq!(format_damage_type("T"), "thunder");
        assert_eq!(format_damage_type("N"), "necrotic");
        assert_eq!(format_damage_type("R"), "radiant");
        assert_eq!(format_damage_type("O"), "force");
        assert_eq!(format_damage_type("Y"), "psychic");
        assert_eq!(format_damage_type("I"), "poison");
        assert_eq!(format_damage_type("Z"), "Z"); // unknown passes through
    }

    #[test]
    fn test_format_property_all() {
        assert_eq!(format_property("A"), "Ammunition");
        assert_eq!(format_property("H"), "Heavy");
        assert_eq!(format_property("L"), "Light");
        assert_eq!(format_property("LD"), "Loading");
        assert_eq!(format_property("R"), "Reach");
        assert_eq!(format_property("S"), "Special");
        assert_eq!(format_property("T"), "Thrown");
        assert_eq!(format_property("RLD"), "Reload");
        assert_eq!(format_property("BF"), "Burst Fire");
        assert_eq!(format_property("??"), "Special"); // unknown
    }

    #[test]
    fn test_is_card_worthy_attunement() {
        let item = json!({
            "name": "Cloak",
            "type": "W",
            "rarity": "none",
            "reqAttune": true
        });
        assert!(is_card_worthy(&item));
    }

    #[test]
    fn test_is_card_worthy_entries() {
        let item = json!({
            "name": "Magic Gem",
            "type": "G",
            "rarity": "none",
            "entries": ["This gem glows faintly."]
        });
        assert!(is_card_worthy(&item));
    }

    #[test]
    fn test_is_card_worthy_pipe_type() {
        // Type with pipe separator (e.g., "M|PHB")
        let item = json!({
            "name": "Sword",
            "type": "M|PHB",
            "rarity": "none"
        });
        assert!(is_card_worthy(&item));
    }

    #[test]
    fn test_page_break_before() {
        let section = EquipmentCardsSection::new(vec![json!({"name": "X"})]);
        assert!(section.page_break_before());
    }

    #[test]
    fn test_from_json() {
        let data = json!([
            {"name": "Sword", "type": "M"},
            {"name": "Shield", "type": "S"}
        ]);
        let section = EquipmentCardsSection::from_json(data);
        assert_eq!(section.items.len(), 2);
    }

    #[test]
    fn test_from_json_non_array() {
        let data = json!({"name": "Not an array"});
        let section = EquipmentCardsSection::from_json(data);
        assert_eq!(section.items.len(), 0);
    }

    #[test]
    fn test_render_card_damage_single_no_type() {
        let item = json!({
            "name": "Club",
            "type": "M",
            "dmg1": "1d4"
        });
        let card = EquipmentCardsSection::render_card(&item);
        assert!(card.contains("1d4"));
        assert!(!card.contains("slashing"));
    }

    #[test]
    fn test_render_card_no_damage() {
        let item = json!({
            "name": "Shield",
            "type": "S",
            "ac": 2
        });
        let card = EquipmentCardsSection::render_card(&item);
        // Should have AC but no damage row
        assert!(card.contains("*AC:*"));
        assert!(!card.contains("*Damage:*"));
    }
}
