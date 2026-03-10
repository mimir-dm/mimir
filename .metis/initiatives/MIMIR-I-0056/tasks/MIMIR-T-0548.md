---
id: monster-detail-rendering-tests
level: task
title: "Monster detail rendering tests — stat blocks, actions, legendary actions, lair actions"
short_code: "MIMIR-T-0548"
created_at: 2026-03-10T01:31:37.031341+00:00
updated_at: 2026-03-10T01:31:37.031341+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Monster detail rendering tests — stat blocks, actions, legendary actions, lair actions

**Phase 4** — Catalog & Search Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for the monster stat block renderer. The 5etools monster JSON format is complex — abilities, actions, legendary actions, lair actions, regional effects, traits, and special abilities all need to parse and render correctly.

## Acceptance Criteria

- [ ] Basic stat block renders: name, size, type, alignment, AC, HP, speed
- [ ] Ability scores render with correct modifiers
- [ ] Skills, saving throws, senses, languages, CR display correctly
- [ ] Traits/special abilities render with formatted descriptions
- [ ] Actions render with attack bonus, damage, and reach/range
- [ ] Multiattack action renders correctly
- [ ] Legendary actions render with point costs and descriptions
- [ ] Lair actions render when present
- [ ] Regional effects render when present
- [ ] Spellcasting trait renders spell list by level
- [ ] Innate spellcasting renders at-will / per-day groupings
- [ ] Damage immunities, resistances, vulnerabilities, condition immunities display
- [ ] All tests pass in CI

## Key Components

- Monster stat block component (SpellStatBlock.vue or equivalent)
- 5etools monster data parser
- Action/trait description formatter

## Implementation Notes

Use SRD monster fixtures from MIMIR-T-0533. Good test cases: Goblin (simple), Adult Red Dragon (legendary actions, lair), Lich (spellcasting), Aboleth (legendary + lair + regional). The 5etools JSON uses nested arrays for formatted text — test the recursive renderer.

## Status Updates

*To be added during implementation*