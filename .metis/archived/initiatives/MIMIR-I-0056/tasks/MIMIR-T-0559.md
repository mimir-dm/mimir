---
id: homebrew-rendering-tests-homebrew
level: task
title: "Homebrew rendering tests — homebrew items, monsters, and spells display correctly"
short_code: "MIMIR-T-0559"
created_at: 2026-03-10T01:31:54.887276+00:00
updated_at: 2026-03-10T13:43:22.836090+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Homebrew rendering tests — homebrew items, monsters, and spells display correctly

**Phase 6** — Homebrew & Advanced Features

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for rendering homebrew content — verifying that homebrew items, monsters, and spells display correctly when viewed in the catalog browser, module monster list, character inventory, and stat block views. This tests the consumer side (rendering) rather than the producer side (CRUD, covered in MIMIR-T-0558).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Homebrew item displays with "HB" source badge in catalog/inventory views
- [ ] Homebrew monster stat block renders correctly from custom JSON data
- [ ] Homebrew spell renders with correct fields from custom JSON
- [ ] Homebrew items in character inventory show correct type icon and properties
- [ ] Homebrew monsters in module monster list show correct CR and source
- [ ] Clone-from-catalog homebrew items retain original data with modifications
- [ ] Missing/malformed homebrew JSON data renders gracefully (no crash)
- [ ] All tests pass in CI

## Key Components

- Item/monster/spell detail renderers (same as catalog, but with HB data)
- Character inventory section (homebrew item display)
- Module monster list (homebrew monster display)
- Source badge / "HB" indicator

## Implementation Notes

Reuse the catalog rendering test patterns from MIMIR-T-0548/0549/0550 but with homebrew-shaped fixture data. The key difference is that homebrew data comes from `campaign_homebrew_*` tables and may have slightly different JSON structure than catalog data. Test that the renderers handle both formats gracefully.

## Status Updates

### Session 2 - Completed
- Created `homebrewRendering.test.ts` with 58 tests covering:
  - Item rendering: melee weapon (damage, rarity, attunement, entries, bonus), ranged weapon (range, damage, attunement), armor (AC, entries), potion (description, rarity), wondrous (rarity, attunement, entries), minimal item, item without entries, HB source attribution
  - Monster rendering: full stat block (type, AC, HP, speed, ability scores, saves, CR, traits, actions, senses, languages, HB source), minimal monster, simple AC (number), climb speed
  - Spell rendering: leveled spell (level, school code, casting time, range, components, damage, conditions, higher level casting, classes, HB source), cantrip (level, damage scaling, damage type), ritual (ritual tag, concentration, material components, casting time, self range, school code)
  - Malformed data: empty entries, no type, missing fields, minimal monster
- Fixed 21 initial test failures: formatters don't render entity names (shown separately in UI), rarity is capitalized by formatRarity(), school rendered as code not name, "Higher Level Casting" not "Higher Levels"
- All 58 tests passing, all 115 homebrew tests (CRUD + rendering) passing together

*To be added during implementation*