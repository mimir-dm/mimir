---
id: character-sheet-component-tests
level: task
title: "Character sheet component tests — EquipmentSection and SpellsSection"
short_code: "MIMIR-T-0538"
created_at: 2026-03-09T14:25:14.683382+00:00
updated_at: 2026-03-10T01:17:33.180525+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Character sheet component tests — EquipmentSection and SpellsSection

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Create comprehensive Vue component tests for EquipmentSection and SpellsSection character sheet components, covering rendering, user interactions, invoke pipeline mocking, and edge cases.

## Acceptance Criteria

## Acceptance Criteria

- [x] EquipmentSection tests: currency display, equipped items, inventory, item detail caching
- [x] SpellsSection tests: spellcasting stats, spell slots, available spells, spell formatting, non-spellcaster
- [x] All tests pass (43 tests across 2 files)
- [x] Full suite passes (325 tests across 15 files)

## Implementation

### EquipmentSection.test.ts (20 tests)
- **Currency**: renders all 5 types, correct values, PP/GP as large items
- **Equipped items**: empty state, equipped filtering, attuned badge, homebrew badge, source badge, expand/collapse with invoke, loading behavior (item only expands after details load)
- **Full inventory**: empty/loading states, renders all items, quantity badges, equipped badge, item notes on expand, Manage button emits openInventory
- **Item detail caching**: does not re-fetch on re-expand

### SpellsSection.test.ts (23 tests)
- **Spellcasting stats**: save DC calculation, attack bonus, ability abbreviation, multiclass display
- **Spell slots**: cantrips as Unlimited, slot boxes per level, tracking note
- **Available spells**: loading/empty states, level grouping, spell count in header, collapse toggle, ritual/concentration badges, school name, expand/collapse with full details
- **Spell detail formatting**: Self range, Touch range, concentration duration, permanent duration, V-only components
- **Non-spellcaster**: Fighter shows no spells

### Key patterns
- Factory functions: `makeWizard()`, `makeCharacter()`, `makeInventoryItem()`, `makeSpellResponse()`
- Mock setup: `setupSpellMocks()` helper for list_character_sources + get_spells_by_class
- Invoke mocks: `mockCommandHandler` for async invoke responses, `mockCommand` for static responses
- Item expansion awaits invoke before showing (no intermediate loading state visible)

## Status Updates

### 2026-03-09
- Created EquipmentSection.test.ts (20 tests) and SpellsSection.test.ts (23 tests)
- Fixed 1 test: loading state test assumed item expands before invoke resolves, but component awaits invoke before expanding — changed to test correct behavior
- All 43 new tests pass, full suite of 325 tests passes