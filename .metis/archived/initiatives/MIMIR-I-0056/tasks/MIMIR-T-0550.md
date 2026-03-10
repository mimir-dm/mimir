---
id: item-detail-rendering-tests
level: task
title: "Item detail rendering tests — properties, damage, armor, attunement, magic items"
short_code: "MIMIR-T-0550"
created_at: 2026-03-10T01:31:39.249726+00:00
updated_at: 2026-03-10T12:54:20.420358+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Item detail rendering tests — properties, damage, armor, attunement, magic items

**Phase 4** — Catalog & Search Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for item detail rendering. Verify that all item types (weapons, armor, wondrous items, potions, rings, rods, wands, scrolls, adventuring gear) parse and display correctly from 5etools JSON, including properties, damage dice, armor class, attunement requirements, and magical properties.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Weapon items render: damage dice, damage type, properties (finesse, versatile, thrown, etc.), range
- [ ] Armor items render: AC value, type (light/medium/heavy), strength requirement, stealth disadvantage
- [ ] Shield renders AC bonus
- [ ] Magic items render: rarity, attunement requirement (with class/alignment restrictions if any)
- [ ] Potions render with usage description
- [ ] Wondrous items render with description and properties
- [ ] Item weight and cost display correctly
- [ ] Item description entries render with formatted text
- [ ] Type code (M, R, LA, MA, HA, W, P, etc.) maps to correct display label
- [ ] All tests pass in CI

## Key Components

- Item detail/stat block component
- Item data parser (type codes, properties, damage)
- Property abbreviation expander (F→Finesse, V→Versatile, etc.)

## Implementation Notes

Use SRD item fixtures from MIMIR-T-0533. Good test cases: Longsword (versatile weapon), Longbow (ranged, ammunition), Chain Mail (heavy armor, stealth disadvantage), Shield, Cloak of Protection (wondrous, attunement), Potion of Healing. Test the `parse_item_data` helper and type code mapping.

## Status Updates

### Completed
- Created `__tests__/formatters/itemFormatting.test.ts` (35 tests)
- Tests cover: melee weapons (Longsword, Dagger, Greataxe, Quarterstaff, Rapier), ranged weapons (Longbow, Shortbow, Heavy Crossbow), magic weapons (+1 Longsword with bonus/properties/versatile), armor (Chain Mail, Breastplate, Leather, Plate, Shield, Studded Leather — AC, strength req, stealth disadvantage), magic items (Cloak of Protection, Cloak of Elvenkind, Boots of Elvenkind, Ring of Protection — rarity, attunement, AC bonus, tier, loot tables), potions (Healing, Greater Healing), containers (Backpack, Bag of Holding), light sources (Torch), cost/weight formatting, description with 5etools tags, source attribution, type code mapping, summary fallback
- Key finding: basic SRD weapons without `entries` field go through summary path (raw type codes), magic weapons with entries go through full format path (formatted damage types, weapon properties)
- All 35 tests passing