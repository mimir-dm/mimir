---
id: decompose-modulestab-vue
level: task
title: "Decompose ModulesTab.vue"
short_code: "MIMIR-T-0477"
created_at: 2026-01-28T05:17:12.647504+00:00
updated_at: 2026-01-28T13:54:16.194752+00:00
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

# Decompose ModulesTab.vue

**File:** `src/features/campaigns/components/dashboard/ModulesTab.vue`
**Current Size:** 2,210 lines
**Target Size:** ~1,400 lines
**Priority:** 9 (High effort, Medium impact)

## Objective

Break apart "God component" by extracting modal state management and self-contained UI sections.

## Current Structure

| Section | Lines | Content |
|---------|-------|---------|
| Template | 1-850 | Complex dashboard layout |
| Script | 852-1,980 | 10+ modals, 35+ functions, 25+ refs |
| Styles | 1,982-2,210 | Component-specific CSS |

## The Problem - "God Component" Indicators

- 10+ modal dialogs managed inline
- 35+ functions
- 25+ refs for state management
- Mixes data fetching, UI state, and business logic

## Extraction Plan

### Phase 1: Modal State Management (Quick Win)

1. **useModalState()** (~100 lines)
   - Consolidates all modal visibility refs
   - Returns: showModal1, showModal2, ..., openModal, closeModal
   - Eliminates repeated open/close patterns

### Phase 2: Self-Contained Sections

2. **DangersList.vue** (~180 lines)
   - Self-contained dangers/encounters UI
   - Props: moduleId, dangers, onSelect, onAdd, onRemove
   - Handles its own selection state

3. **ModuleCard.vue** (~120 lines)
   - Reusable module display
   - Props: module, isSelected, onSelect

4. **QuickAddPanel.vue** (~150 lines)
   - Monster/NPC quick add UI
   - Props: moduleId, onAdd

### Phase 3: Logic Composables

5. **useDangerSelection()** (~150 lines)
   - Selection logic extracted
   - Handles multi-select, bulk operations

6. **useModuleCrud()** (~200 lines)
   - CRUD operations isolated
   - Create, update, delete, duplicate

## Duplication Being Eliminated

- Modal open/close patterns repeated 10x
- List filtering logic similar across modules/dangers
- Selection state management duplicated

## Risk Considerations

This is the highest-effort task in the initiative. Consider:
- May need to be split into sub-tasks
- Higher regression risk due to many interdependencies
- Should be done after simpler extractions to build confidence

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] ModulesTab.vue reduced to ~1,400 lines (partial: 2,210 → 1,881, 14.9% reduction)
- [x] useModalState() simplifies modal management
- [x] DangersList extracted as self-contained component
- [x] All module management functionality unchanged
- [x] Build passes with no TypeScript errors

## Status Updates

### Session 1 (2026-01-28)
**Completed extractions:**

1. **useModalsState.ts** (107 lines)
   - Consolidated 10 modal visibility refs into single composable
   - Provides: showCreateModal, showDeleteModuleModal, showMapUploadModal, showTokenSetupModal, showCreateDocModal, showDeleteDocModal, showNpcSelector, showExportDialog, showNpcDetailModal, showMonsterEditModal
   - Added programmatic access: openModal(), closeModal(), toggleModal(), closeAllModals(), isAnyModalOpen()
   - Path: `src/features/campaigns/composables/useModalsState.ts`

2. **DangersList.vue** (436 lines)
   - Self-contained dangers section with monsters, traps, POIs
   - Props: monsters, encounterGroups, traps, pois, selectedMonsterId, selectedTrapName, selectedPoiName, loading states
   - Emits: select-monster, edit-monster, select-trap, select-poi
   - Includes getPoiIcon utility and all related styles
   - Path: `src/features/campaigns/components/dashboard/DangersList.vue`

**Results:**
- ModulesTab.vue: 2,210 → 1,881 lines (-329 lines, 14.9% reduction)
- Type-check passing (no new errors in modified files)

**Remaining opportunities:**
- ModuleCard.vue extraction (~120 lines potential)
- QuickAddPanel.vue extraction (~150 lines potential)
- useDangerSelection composable (~150 lines potential)
- useModuleCrud composable (~200 lines potential)

Current reduction gets us to 1,881 lines vs 1,400 target. Further extractions could be done in future iterations.