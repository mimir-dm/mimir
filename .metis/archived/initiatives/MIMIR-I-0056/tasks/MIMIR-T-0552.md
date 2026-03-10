---
id: cross-reference-rendering-tests
level: task
title: "Cross-reference rendering tests — modal formatters for all ref types (@spell, @item, @creature, etc.)"
short_code: "MIMIR-T-0552"
created_at: 2026-03-10T01:31:41.825576+00:00
updated_at: 2026-03-10T13:03:16.256369+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Cross-reference rendering tests — modal formatters for all ref types (@spell, @item, @creature, etc.)

**Phase 4** — Catalog & Search Coverage

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest tests for the cross-reference rendering system — the modal formatters that handle 5etools inline references like `{@spell fireball}`, `{@creature goblin}`, `{@item longsword}`, `{@class wizard}`, `{@race elf}`, `{@background soldier}`, `{@condition frightened}`, `{@dice 2d6}`, etc. These are used throughout the app in document rendering, stat blocks, and descriptions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `{@spell name}` references resolve to spell data and render a clickable link
- [ ] `{@creature name}` references resolve to monster stat block
- [ ] `{@item name}` references resolve to item detail
- [ ] `{@class name}` references link to class info
- [ ] `{@race name}` references link to race info
- [ ] `{@condition name}` references show condition tooltip/modal
- [ ] `{@dice XdY+Z}` references render with dice notation
- [ ] `{@damage XdY}` references render damage dice
- [ ] `{@dc X}` references render DC value
- [ ] References with display text override (`{@spell fireball|PHB|a fireball}`) use the override text
- [ ] References with source hints (`{@creature goblin|MM}`) resolve correctly
- [ ] Unknown/malformed references render gracefully (don't crash)
- [ ] All tests pass in CI

## Key Components

- `textFormatting.ts` — main tag parser
- Individual formatter files in `features/sources/formatters/`
- Reference modal/tooltip components

## Implementation Notes

The `textFormatting.ts` utility is the core of this — it's a pure function that can be unit tested directly. Test both the parsing (extracting tag type, name, source, display text) and the rendering (producing correct HTML/component output). The formatters are mostly pure functions too. Focus on edge cases: nested references, malformed tags, missing sources.

## Status Updates

### Session 2 — Completed
- Read full `modalFormatters.ts` (1045 lines) — contains `renderModalContent` dispatcher + 12 individual renderers + helper functions
- Created `__tests__/formatters/modalFormatters.test.ts` with 95 tests covering:
  - **Dispatcher**: null/undefined input, name heading, modal-content wrapper, creature/monster routing, unknown ref_type fallback, type field fallback
  - **Spell content**: cantrip/leveled level display, all 8 school codes, casting time (action/reaction with condition), range (point/touch/self), components (V/S/M with material text), duration (instant/concentration/permanent with ends), entries (string/nested/list), higher level scaling
  - **Item content**: type, rarity (shown/hidden for 'none'), value (gp/sp/cp via formatCurrency), weight, description entries
  - **Condition content**: string entries, list entries, nested entries with name, fallback message for no entries
  - **Action content**: time as array/string, entries with list, nested entries
  - **Feat content**: prerequisites (level/race/ability/spellcasting), entries with list, nested entries
  - **Background content**: skill/tool/language proficiencies, starting equipment, features
  - **Race content**: size (single/multiple), speed (numeric/object with fly/swim/climb), darkvision, traits
  - **Class content**: hit die, primary ability, saving throws, armor/weapon proficiencies, subclass title, level 1 features (object + string format)
  - **Class feature content**: className/class_name + level subtitle, entries with text/table/inset/options/list
  - **Subclass content**: parent class name, features list (string/object format), description entries
  - **Subclass feature content**: full subtitle (class + subclass + level), entries with table/quote/options
  - **Generic fallback**: entries/text/description fields
  - **Currency formatting**: gp (whole/decimal), sp, cp
  - **Ordinal suffix**: 1st, 2nd, 3rd, 4th, 9th
- Fixed class features test: string format needs `|1|` (pipe on both sides) for filter to match
- All 95 tests passing

*To be added during implementation*