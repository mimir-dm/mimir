---
id: character-sheet-integration-tests
level: task
title: "Character sheet integration tests — catalog lookups and character enrichment"
short_code: "MIMIR-T-0539"
created_at: 2026-03-09T14:25:15.601680+00:00
updated_at: 2026-03-10T01:17:33.731798+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Character sheet integration tests — catalog lookups and character enrichment

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Create integration tests for CharacterSheetView that verify the full data flow: character loading → catalog lookups (race, class, background, subclass, features, inventory) → enriched data passed to child components.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Integration test for full character load flow (character + all catalog lookups)
- [x] Tests verify correct parameters passed to each catalog command
- [x] Speed enrichment from race data (numeric, object, fallback)
- [x] Class feature parsing and level filtering
- [x] Subclass feature loading and header exclusion
- [x] Multiclass support (multiple class lookups)
- [x] Tab navigation (Character, Equipment, Spells, Details)
- [x] Inventory integration with Equipment tab
- [x] Error handling (character, race, class, background failures)
- [x] Loading state
- [x] All 25 tests pass, full suite of 350 tests passes

## Implementation

### File: `__tests__/integration/CharacterSheetIntegration.test.ts` (25 tests)

**Full character load flow (4 tests)**
- Loads character and all catalog data on mount
- Passes correct parameters to each catalog lookup
- Renders header with enriched character info (level, race, class, background)
- Shows NPC badge for NPC characters

**Speed enrichment from race catalog (3 tests)**
- Uses numeric speed from race data
- Extracts walk speed from object-format `{ walk: 25 }`
- Falls back to 30ft when race data has no speed

**Class feature enrichment (4 tests)**
- Parses `classFeatures` array (string + object formats) and filters by level
- Includes subclass features at/below character level
- Excludes subclass header feature (name matches subclass, header=1)
- Excludes features above character level

**Multiclass character enrichment (1 test)**
- Loads class data for each class with dynamic handler

**Tab navigation (4 tests)**
- Shows Character tab by default
- Shows Spells tab for spellcaster (Wizard)
- Hides Spells tab for non-spellcaster (Fighter)
- Switches to Equipment tab on click

**Inventory integration (2 tests)**
- Passes loaded inventory to Equipment tab
- Verifies inventory invoke was called

**Error handling (4 tests)**
- Shows error state when character load fails
- Degrades gracefully when race/class/background lookups fail

**Loading state & navigation (2 tests)**
- Shows "Loading character..." while loading
- Back button calls router.back()

### Key patterns
- vue-router mocked via `vi.mock('vue-router')` with `useRoute`/`useRouter` stubs
- MainLayout stubbed as pass-through `<slot />` wrapper
- Heavy child components (InventoryManager, PrintDialog, SourcesModal) stubbed
- `setupFullCharacterMocks()` helper wires up all 9 invoke commands for a complete load
- Factory functions for realistic 5etools-format catalog data

## Status Updates

### 2026-03-09
- Explored all invoke commands in CharacterSheetView (9 commands total for full load)
- Created `__tests__/integration/CharacterSheetIntegration.test.ts` with 25 tests
- All tests passed on first run — no fixes needed
- Full suite: 350 tests across 16 files, all passing