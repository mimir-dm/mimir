---
id: spell-detail-rendering-tests
level: task
title: "Spell detail rendering tests — components, duration, range, school, class lists"
short_code: "MIMIR-T-0549"
created_at: 2026-03-10T01:31:38.360659+00:00
updated_at: 2026-03-10T12:50:53.453109+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Spell detail rendering tests — components, duration, range, school, class lists

**Phase 4** — Catalog & Search Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for the spell detail renderer. Verify that all spell fields parse and display correctly from the 5etools JSON format — components (V/S/M with material details), duration, range, casting time, school, level, class lists, and description text with embedded references.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Spell header renders: name, level, school
- [ ] Casting time displays correctly (action, bonus action, reaction with trigger, ritual tag)
- [ ] Range displays correctly (self, touch, feet, miles, special)
- [ ] Components render: V, S, M with material description and cost/consumed flags
- [ ] Duration renders: instantaneous, concentration with time, rounds, until dispelled
- [ ] Spell description renders with formatted text and nested entries
- [ ] Higher level scaling ("At Higher Levels") section renders when present
- [ ] Class list shows which classes have the spell
- [ ] Source attribution displays correctly
- [ ] All tests pass in CI

## Key Components

- Spell detail/stat block component
- Spell data formatter
- Component/duration/range parsers

## Implementation Notes

Use SRD spell fixtures from MIMIR-T-0533. Good test cases: Fireball (simple damage), Shield (reaction trigger), Find Familiar (ritual, material with cost), Detect Magic (concentration), Wish (complex description). Test edge cases like spells with no material component vs expensive consumed materials.

## Status Updates

### Completed
- Created `__tests__/formatters/spellFormatting.test.ts` (46 tests)
- Tests cover: header (level/school), casting time (action/bonus/reaction with trigger), range (point/touch/self), components (V/S/M with material text and cost/consumed), duration (instant/concentration/timed), description with 5etools tags, at higher levels, combat mechanics (save/attack/damage/conditions/area), cantrip scaling, misc tags, source attribution, summary fallback
- Integration tests for: Fireball, Shield, Detect Magic, Wish, Misty Step, Power Word Kill, Eldritch Blast, Fire Bolt, Cure Wounds, Guidance, Healing Word, Counterspell, Hold Person, Light, Mage Armor, Revivify
- All 46 tests passing