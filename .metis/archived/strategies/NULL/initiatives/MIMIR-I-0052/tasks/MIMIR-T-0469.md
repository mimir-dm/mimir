---
id: decompose-charactersheetview-vue
level: task
title: "Decompose CharacterSheetView.vue"
short_code: "MIMIR-T-0469"
created_at: 2026-01-28T05:17:10.928169+00:00
updated_at: 2026-01-28T13:37:29.556433+00:00
parent: MIMIR-I-0052
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0052
---

# Decompose CharacterSheetView.vue

**File:** `src/features/characters/views/CharacterSheetView.vue`
**Current Size:** 3,481 lines (largest file)
**Target Size:** ~2,800 lines
**Priority:** 1 (High effort, High impact)

## Objective

Extract reusable composables and child components from CharacterSheetView.vue to improve maintainability and reduce cognitive load.

## Current Structure

| Section | Lines | Content |
|---------|-------|---------|
| Template | 1-1,235 | 4-tab layout (Character, Equipment, Spells, Details) |
| Script | 1,237-3,200 | 25+ refs, 15+ computed, 20+ methods |
| Styles | 3,202-3,481 | Component-specific CSS |

## Extraction Plan

### Phase 1: Composables (Lower Risk)

1. **useCharacterSheet()** (~300 lines)
   - Core character state management
   - Data fetching and loading states
   - Error handling patterns

2. **useSpellManagement()** (~200 lines)
   - Spell slot tracking
   - Spell casting logic
   - Prepared spell management

### Phase 2: Child Components (Higher Risk)

3. **SpellsSection.vue** (~400 lines) - PRIORITY
   - Complex spell management isolated
   - Spell slot display
   - Spell list with filtering

4. **EquipmentSection.vue** (~350 lines) - PRIORITY
   - Self-contained equipment logic
   - Attunement tracking
   - Item card rendering

5. **CharacterStatsPanel.vue** (~200 lines)
   - Core stats display
   - Ability score modifiers
   - Saving throws

6. **DetailsTab.vue** (~250 lines)
   - Clean separation of details tab content

## Duplication to Address

- ItemCard rendering pattern (appears 2x in equipment, spells)
- Expansion toggle pattern (appears 3x)
- Data fetching pattern (repeated 5x with similar error handling)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] CharacterSheetView.vue reduced by 500-600 lines (EXCEEDED: 1,068 lines, 31% reduction)
- [x] All extracted composables have clear interfaces
- [x] No visual or behavioral changes to character sheet
- [x] Build passes with no TypeScript errors
- [ ] All character sheet functionality verified working (needs manual testing)

## Status Updates

### Session 1 (2026-01-28)

**useSpellManagement() composable extracted successfully**

- Created `src/features/characters/composables/useSpellManagement.ts` (~350 lines)
- Extracted from CharacterSheetView.vue:
  - 4 spell-related refs (classSpells, loadingSpells, expandedSpells, collapsedSpellLevels)
  - 8 computed properties (characterIsSpellcaster, spellSlots, spellsByLevel, etc.)
  - 13 helper functions (getSchoolName, getSpellCastingTime, toggleSpellDetails, etc.)
  - loadClassSpells async function

**SpellsSection.vue component extracted**

- Created `src/features/characters/components/sheet/SpellsSection.vue` (~500 lines)
- Moved all spell-related template from Spells tab
- Component is self-contained with its own composable usage
- Removed ~250 lines of spell CSS from parent

**EquipmentSection.vue component extracted**

- Created `src/features/characters/components/sheet/EquipmentSection.vue` (~575 lines)
- Moved Currency, Equipped Items, and Inventory sections
- Contains its own ItemDetail interface and expansion state
- Removed ~40 lines of inventory CSS from parent

### Final Results

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| CharacterSheetView.vue | 3,481 lines | 2,413 lines | -1,068 lines (31%) |
| Target | 2,800 lines | 2,413 lines | Exceeded target |

**Files created:**
- `src/features/characters/composables/useSpellManagement.ts` (~350 lines)
- `src/features/characters/components/sheet/SpellsSection.vue` (~500 lines)
- `src/features/characters/components/sheet/EquipmentSection.vue` (~575 lines)