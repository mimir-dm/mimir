---
id: equipment-cards-auto-attacks-for
level: task
title: "Equipment Cards & Auto-Attacks for Character PDF"
short_code: "MIMIR-T-0272"
created_at: 2026-01-03T02:15:57.662548+00:00
updated_at: 2026-01-03T02:35:11.528593+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Equipment Cards & Auto-Attacks for Character PDF

*Backlog feature request for character PDF export enhancements.*

## Objective

Add two related features for character PDF export:
1. **Equipment Cards**: Printable cards (2.5in x 3.5in poker size) for weapons, special ammo, and magic items - black/white with icons to distinguish from spell cards
2. **Auto-Attacks**: Populate the Attacks & Damage section automatically from inventory weapons

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P1 - High (important for user experience)
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Helps newer/younger players remember their cool equipment and how to use it. Equipment cards provide quick reference during gameplay.
- **Business Value**: Improves character sheet usability and print quality
- **Effort Estimate**: L (Large)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Part 1: Equipment Cards
- [ ] New `EquipmentCardsSection` in `mimir-dm-print/src/sections/equipment.rs`
- [ ] Cards are 2.5in x 3.5in (same as spell cards), 9 per page
- [ ] Black/white design with equipment type icons (sword, bow, shield, gem)
- [ ] Card shows: name, type, rarity, damage, properties, description, attunement
- [ ] Auto-detects card-worthy items: weapons (M/R), special ammo (A/AF with rarity), magic items, items with notes
- [ ] UI checkbox "Include Equipment Cards" in CharacterPrintDialog.vue

### Part 2: Auto-Attacks
- [ ] Attacks & Damage section auto-populates from inventory weapons
- [ ] Attack bonus calculated: STR/DEX mod + proficiency (finesse uses higher)
- [ ] Damage formatted from catalog data (dmg1 + modifier, versatile shows both)

## Implementation Notes

### Files to Create/Modify

| File | Change |
|------|--------|
| `mimir-dm-print/src/sections/equipment.rs` | New - EquipmentCardsSection |
| `mimir-dm-print/src/sections/mod.rs` | Export new module |
| `mimir-dm-print/src/character.rs` | Add include_equipment_cards option |
| `mimir-dm/frontend/.../CharacterPrintDialog.vue` | Add UI checkbox |
| `mimir-dm/src/print/character.rs` | Handle equipment cards generation |
| `mimir-dm-print/src/sections/character_sheet.rs` | Auto-populate attacks from inventory |

### Card-Worthy Detection Logic

Items qualify if ANY of:
- Weapons: type M (melee), R (ranged)
- Special Ammo: type A, AF with rarity != "none"
- Magic Items: rarity != "none" or null
- Has `requires_attunement`
- Has `entries` with special abilities
- Has user-added notes

### Equipment Icons

| Icon | Item Types |
|------|------------|
| Sword | Melee weapons (M) |
| Bow/Arrow | Ranged weapons (R), Ammunition (A, AF) |
| Shield | Shields (S), Armor (LA, MA, HA) |
| Gem/Star | Wondrous items, Rings, Rods, Wands |

### Dependencies
- Follows `SpellCardsSection` pattern in `sections/spells.rs`
- Uses item catalog data from `mimir-dm-core/src/models/catalog/item.rs`

## Status Updates

*To be added during implementation*