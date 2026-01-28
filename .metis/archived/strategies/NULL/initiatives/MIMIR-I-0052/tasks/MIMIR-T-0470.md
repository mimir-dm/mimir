---
id: decompose-dmmapviewer-vue
level: task
title: "Decompose DmMapViewer.vue"
short_code: "MIMIR-T-0470"
created_at: 2026-01-28T05:17:11.146515+00:00
updated_at: 2026-01-28T13:45:49.299696+00:00
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

# Decompose DmMapViewer.vue

**File:** `src/components/DmMapViewer.vue`
**Current Size:** 2,621 lines
**Target Size:** ~1,400 lines
**Priority:** 2 (High effort, High impact)

## Objective

Extract complex canvas logic, vision systems, and token management into focused composables to improve testability and reusability.

## Current Structure

| Section | Lines | Content |
|---------|-------|---------|
| Template | 1-450 | Canvas + UI controls |
| Script | 452-2,350 | Complex vision/token/lighting logic |
| Styles | 2,352-2,621 | Component-specific CSS |

## Extraction Plan

### Composables to Extract

1. **useFogOfWar()** (~250 lines) - PRIORITY
   - Vision/fog logic isolated
   - LOS calculations
   - Fog reveal/hide operations

2. **useLightingSystem()** (~220 lines) - PRIORITY
   - Dynamic shadow calculations
   - Light source management
   - Ambient lighting

3. **useTokenDragHandler()** (~200 lines)
   - Drag/drop logic
   - Token placement
   - Snap-to-grid behavior

4. **usePlayerDisplaySync()** (~150 lines)
   - WebSocket sync extracted
   - Player window communication

5. **useMapDataLoader()** (~180 lines)
   - Data fetching centralized
   - Map/token loading

6. **useMapContextMenu()** (~120 lines)
   - Context menu state/actions

## Duplication to Address

- Canvas coordinate transformation logic repeated 4x
- Event handler setup patterns similar across drag operations
- Token/marker selection logic nearly identical

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] DmMapViewer.vue reduced to ~1,400 lines
- [ ] Fog of war functionality unchanged
- [ ] Token drag/drop works correctly
- [ ] Player display sync operational
- [ ] Build passes with no TypeScript errors

## Status Updates

### Session 1 (2026-01-28)

**Composables extracted:**

1. **useTokenDrag.ts** (~250 lines)
   - BackendToken interface (shared across components)
   - transformToken() utility function
   - Token drag handling logic (partially integrated - full integration needs ref restructuring)

2. **useMapMarkers.ts** (~250 lines)
   - MapTrap and MapPoi interfaces
   - State: mapTraps, mapPois, selectedTrapId, selectedPoiId, poiContextMenu, showPoiEditModal, poiToEdit
   - Loading: loadMapTraps(), loadMapPois()
   - Actions: toggleTrapVisibility(), togglePoiVisibility(), showPoiContextMenuAt(), openPoiEditModal(), etc.
   - Utility: getPoiIcon()

**Results:**
- DmMapViewer.vue reduced from 2,621 → 2,402 lines (-219 lines, 8.4% reduction)
- Build passes with no TypeScript errors

**Observations:**
- The component already uses several composables well (useVisionCalculation, useUvttMap, useMultiTokenVisibility)
- Remaining logic is tightly integrated with the component's specific requirements
- The 1,400 line target would require significant restructuring of:
  - Player display sync functions (depend on many local refs for fog/vision state)
  - Token drag handlers (zoom ref defined late in file, complex state dependencies)
  - Context menu handlers (multiple menu types with different behaviors)

**Recommendation:**
- Consider the current reduction (219 lines) as meaningful progress
- Further reduction would require restructuring the component's state management approach
- The extracted composables (useMapMarkers, useTokenDrag) improve code organization and reusability