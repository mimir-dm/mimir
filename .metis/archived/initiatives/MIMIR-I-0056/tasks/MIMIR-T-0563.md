---
id: level-up-flow-tests-class
level: task
title: "Level-up flow tests — class selection, HP rolling, ASI/feats, subclass selection, spell picks"
short_code: "MIMIR-T-0563"
created_at: 2026-03-10T01:31:59.878920+00:00
updated_at: 2026-03-10T17:23:48.796905+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Level-up flow tests — class selection, HP rolling, ASI/feats, subclass selection, spell picks

**Phase 6** — Homebrew & Advanced Features

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for the character level-up flow — the multi-step wizard that guides class selection, HP rolling/average, ability score improvements or feat selection, subclass selection (at appropriate levels), and spell picks for spellcasters.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Class selection step shows available classes and validates selection
- [ ] HP step offers roll or average options with correct hit die for class
- [ ] ASI/feat step appears at correct levels (4, 8, 12, 16, 19)
- [ ] ASI step allows increasing two ability scores by 1 each (with cap at 20)
- [ ] Feat selection shows available feats and applies prerequisites
- [ ] Subclass selection step appears at correct level for each class (Wizard: 2, Fighter: 3, etc.)
- [ ] Spell selection step shows class spell list filtered by level
- [ ] Spell selection respects "spells known" vs "prepared" casting (Wizard vs Sorcerer)
- [ ] Cantrip selection appears when new cantrips are gained
- [ ] Level-up submit calls correct invoke commands with all selections
- [ ] Multiclass level-up validates multiclass prerequisites
- [ ] All tests pass in CI

## Key Components

- Level-up wizard/stepper component
- `ClassSelectionStep.vue`
- `SpellsStep.vue`
- `FeaturesDisplayStep.vue`
- `useSpellManagement.ts` composable
- `SpellSelector.vue`

## Implementation Notes

The level-up flow is a multi-step wizard with conditional steps. Use the invoke mock to provide class data, spell lists, and feat lists. Test the step sequencing logic — which steps appear depends on the class, level, and subclass. Key edge cases: multiclass prerequisites, cantrip vs spell selection, prepared vs known casters, subclass level varying by class.

## Status Updates

### Session 1 (2026-03-10)
- Created 3 test files with 95 passing tests covering the level-up flow composables:
  - `useLevelUp.test.ts` (45 tests) — step visibility (subclass level, ASI levels, spellcasting, feature choices per class), wizard navigation (forward/back/goToStep), class selection with invoke mock, request building with all optional fields (subclass, ASI, feat, spells, features), HP method types, submit error handling, reset
  - `useFeatureSelection.test.ts` (21 tests) — multi-select with slot limits, single-select replace, toggle/deselect/clear, callbacks, custom getKey, reactive maxSlots via computed, string variant for expertise
  - `useSpellManagement.test.ts` (29 tests) — spellcaster detection, spell slot calculation (single class, multiclass, Warlock pact magic), spell grouping by level, helper functions (school names, level display, casting time, range, components, duration), toggle behavior, loadClassSpells with source filtering and deduplication
- All 1352 tests pass across 58 test files (full suite)