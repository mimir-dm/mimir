//! Equipment detail section
//!
//! Full inventory list with descriptions, quantities, and status.

use crate::builder::{escape_typst_string, RenderContext, Renderable};
use crate::error::Result;
use crate::sections::character::InventoryItem;

/// Equipment detail section - comprehensive inventory list
pub struct EquipmentDetailSection {
    /// Character name for header
    character_name: String,
    /// All inventory items
    items: Vec<InventoryItem>,
}

impl EquipmentDetailSection {
    /// Create a new equipment detail section
    pub fn new(character_name: String, items: Vec<InventoryItem>) -> Self {
        Self {
            character_name,
            items,
        }
    }

    /// Categorize items for organized display
    fn categorize_items(
        items: &[InventoryItem],
    ) -> (
        Vec<&InventoryItem>,
        Vec<&InventoryItem>,
        Vec<&InventoryItem>,
        Vec<&InventoryItem>,
    ) {
        let mut weapons = Vec::new();
        let mut armor = Vec::new();
        let mut magic = Vec::new();
        let mut other = Vec::new();

        for item in items {
            let lower = item.name.to_lowercase();

            // Weapons
            if lower.contains("sword")
                || lower.contains("bow")
                || lower.contains("axe")
                || lower.contains("mace")
                || lower.contains("dagger")
                || lower.contains("crossbow")
                || lower.contains("hammer")
                || lower.contains("spear")
                || lower.contains("rapier")
                || lower.contains("scimitar")
                || lower.contains("glaive")
                || lower.contains("halberd")
                || lower.contains("javelin")
                || lower.contains("trident")
                || lower.contains("whip")
                || lower.contains("flail")
            {
                weapons.push(item);
            }
            // Armor
            else if lower.contains("armor")
                || lower.contains("mail")
                || lower.contains("shield")
                || lower.contains("plate")
                || lower.contains("leather")
                || lower.contains("hide")
                || lower.contains("breastplate")
                || lower.contains("helm")
                || lower.contains("gauntlet")
            {
                armor.push(item);
            }
            // Magic items
            else if lower.contains("+1")
                || lower.contains("+2")
                || lower.contains("+3")
                || lower.contains("magic")
                || lower.contains("ring of")
                || lower.contains("wand of")
                || lower.contains("staff of")
                || lower.contains("cloak of")
                || lower.contains("boots of")
                || lower.contains("amulet")
                || lower.contains("potion")
                || lower.contains("scroll")
            {
                magic.push(item);
            }
            // Other
            else {
                other.push(item);
            }
        }

        (weapons, armor, magic, other)
    }

    /// Render a category section
    fn render_category(title: &str, items: &[&InventoryItem], accent_color: &str) -> String {
        if items.is_empty() {
            return String::new();
        }

        let mut typst = String::new();

        typst.push_str(&format!(
            r#"#box(
  width: 100%,
  stroke: (left: 3pt + {accent_color}),
  inset: (left: 8pt, y: 4pt),
)[
  #text(size: 11pt, weight: "bold", fill: {accent_color})[{title}]
]
#v(4pt)

#table(
  columns: (auto, 1fr, auto, auto),
  stroke: 0.5pt + colors.border-light,
  inset: 6pt,
  align: (left, left, center, center),

  // Header
  table.header(
    [*Qty*],
    [*Item*],
    [*Equipped*],
    [*Attuned*],
  ),

"#,
            accent_color = accent_color,
            title = title,
        ));

        for item in items {
            let equipped = if item.equipped { "Yes" } else { "-" };
            let attuned = if item.attuned { "Yes" } else { "-" };

            typst.push_str(&format!(
                "  [{}], [{}], [{}], [{}],\n",
                item.quantity,
                escape_typst_string(&item.name),
                equipped,
                attuned
            ));
        }

        typst.push_str(")\n\n#v(8pt)\n\n");
        typst
    }
}

impl Renderable for EquipmentDetailSection {
    fn to_typst(&self, _ctx: &RenderContext) -> Result<String> {
        if self.items.is_empty() {
            return Ok("// No equipment to render\n".to_string());
        }

        let mut typst = String::new();

        // Page break and header
        typst.push_str("#pagebreak(weak: true)\n");
        typst.push_str(&format!(
            r#"#text(size: 10pt, fill: luma(100))[EQUIPMENT DETAIL]

#grid(
  columns: (1fr, auto),
  column-gutter: 16pt,
  [#text(size: 20pt, weight: "bold")[{}]],
  [#text(size: 12pt)[{} items]],
)

#line(length: 100%, stroke: 2pt + colors.accent)
#v(16pt)

"#,
            escape_typst_string(&self.character_name),
            self.items.len()
        ));

        // Categorize and render
        let (weapons, armor, magic, other) = Self::categorize_items(&self.items);

        // Weapons (red)
        typst.push_str(&Self::render_category(
            "WEAPONS",
            &weapons,
            "rgb(\"#dc2626\")",
        ));

        // Armor (blue)
        typst.push_str(&Self::render_category(
            "ARMOR & SHIELDS",
            &armor,
            "rgb(\"#2563eb\")",
        ));

        // Magic Items (purple)
        typst.push_str(&Self::render_category(
            "MAGIC ITEMS",
            &magic,
            "rgb(\"#9333ea\")",
        ));

        // Other (gray)
        typst.push_str(&Self::render_category(
            "OTHER EQUIPMENT",
            &other,
            "rgb(\"#4b5563\")",
        ));

        // Summary
        let equipped_count = self.items.iter().filter(|i| i.equipped).count();
        let attuned_count = self.items.iter().filter(|i| i.attuned).count();
        let total_items: i32 = self.items.iter().map(|i| i.quantity).sum();

        typst.push_str(&format!(
            r##"#v(1fr)

#box(
  width: 100%,
  fill: rgb("#f9fafb"),
  stroke: 1pt + colors.border,
  radius: 4pt,
  inset: 12pt,
)[
  #grid(
    columns: (1fr, 1fr, 1fr),
    align: center,
    [
      #text(size: 8pt, fill: luma(100))[TOTAL ITEMS]
      #linebreak()
      #text(size: 14pt, weight: "bold")[{total_items}]
    ],
    [
      #text(size: 8pt, fill: luma(100))[EQUIPPED]
      #linebreak()
      #text(size: 14pt, weight: "bold")[{equipped_count}]
    ],
    [
      #text(size: 8pt, fill: luma(100))[ATTUNED]
      #linebreak()
      #text(size: 14pt, weight: "bold")[{attuned_count}]
    ],
  )
]

#v(8pt)
#align(center)[
  #text(size: 8pt, fill: luma(150))[Generated by Mimir]
]
"##,
            total_items = total_items,
            equipped_count = equipped_count,
            attuned_count = attuned_count
        ));

        Ok(typst)
    }

    fn toc_title(&self) -> Option<String> {
        if self.items.is_empty() {
            None
        } else {
            Some(format!("{} - Equipment Detail", self.character_name))
        }
    }

    fn page_break_before(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_items() -> Vec<InventoryItem> {
        vec![
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
            InventoryItem {
                name: "Ring of Protection".to_string(),
                quantity: 1,
                equipped: true,
                attuned: true,
            },
            InventoryItem {
                name: "Rope (50 ft)".to_string(),
                quantity: 1,
                equipped: false,
                attuned: false,
            },
        ]
    }

    #[test]
    fn test_categorize_items() {
        let items = test_items();
        let (weapons, armor, magic, other) = EquipmentDetailSection::categorize_items(&items);

        assert_eq!(weapons.len(), 1);
        assert_eq!(armor.len(), 1);
        assert_eq!(magic.len(), 1);
        assert_eq!(other.len(), 1);
    }

    #[test]
    fn test_empty_section() {
        let section = EquipmentDetailSection::new("Test".to_string(), vec![]);
        assert!(section.toc_title().is_none());
    }

    #[test]
    fn test_toc_title() {
        let section = EquipmentDetailSection::new("Thorin".to_string(), test_items());
        assert_eq!(
            section.toc_title(),
            Some("Thorin - Equipment Detail".to_string())
        );
    }
}
