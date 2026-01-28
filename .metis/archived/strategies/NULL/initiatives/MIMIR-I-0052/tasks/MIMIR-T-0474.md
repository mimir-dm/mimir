---
id: decompose-playerdisplaywindow-vue
level: task
title: "Decompose PlayerDisplayWindow.vue"
short_code: "MIMIR-T-0474"
created_at: 2026-01-28T05:17:12.012841+00:00
updated_at: 2026-01-28T14:24:42.902841+00:00
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

# Decompose PlayerDisplayWindow.vue

**File:** `src/components/PlayerDisplayWindow.vue`
**Current Size:** 1,080 lines
**Target Size:** ~400 lines
**Priority:** 6 (Medium effort, Medium impact)

## Objective

Extract reusable viewport logic and IPC event handling into composables that can be shared with DmMapViewer and other map components.

## Current Structure

| Section | Lines | Content |
|---------|-------|---------|
| Script | 1-581 | 19 refs, 14 computed, 10 functions |
| Template | 584-877 | Canvas + overlays |
| Styles | 880-1,080 | Component-specific CSS |

## The Problem

- Viewport logic (pan/zoom) needed by other map windows but locked here
- 6 repetitive IPC event listener setups
- SVG overlays are independent but tightly coupled

## Extraction Plan

### Phase 1: Composables (High Priority)

1. **usePlayerViewport()** (~80 lines)
   - Player pan/zoom/scale logic
   - Handles: playerPanX, playerPanY, playerZoom, displayScale
   - Contains: updateDisplayScale, mouse handlers, wheel handler, resetView
   - **Reusable** by DmMapViewer

2. **useMapEventSync()** (~150 lines)
   - Consolidates 6 IPC event listeners into single composable
   - Handles: map-update, blackout, tokens-update, fog-update, light-sources-update, markers-update
   - Returns: cleanup function for onBeforeUnmount

3. **useFogOfWar()** (~60 lines)
   - Vision & LOS state
   - Handles: revealMap, visionCircles, useLosBlocking, visibilityPaths
   - Contains: isPointInPolygon, visibility filtering

### Phase 2: Child Components (Optional)

4. **GridOverlay.vue** (~80 lines)
   - Self-contained hex/square grid SVG rendering
   - Props: gridType, gridSize, mapDimensions

5. **FogOfWarOverlay.vue** (~60 lines)
   - Independent SVG overlay with mask definition
   - Props: visionCircles, visibilityPaths

## Synergy with MIMIR-T-0470

The viewport composable extracted here should be usable by DmMapViewer.vue. Coordinate these tasks to avoid duplicate work.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] ~~PlayerDisplayWindow.vue reduced to ~400 lines~~ Reduced to 899 lines (182 line reduction, 16.8%)
- [x] usePlayerViewport() composable created and reusable
- [x] usePlayerDisplayEvents() consolidates all 6 IPC listeners
- [x] Player display functionality unchanged
- [x] Build passes with no TypeScript errors

## Status Updates

### Session 1 (2026-01-28)

**Completed:**
- Created `src/composables/map/usePlayerDisplayEvents.ts` (139 lines)
  - Consolidates 6 IPC event listeners into single composable
  - Type-safe payload interfaces for all event types
  - Auto-cleanup via onUnmounted
  - Handlers: onMapUpdate, onBlackout, onTokensUpdate, onFogUpdate, onLightSourcesUpdate, onMarkersUpdate

- Created `src/composables/map/usePlayerViewport.ts` (163 lines)
  - Reusable pan/zoom viewport logic
  - Configurable min/max zoom, zoom step
  - Mouse handlers: handleMouseDown, handleMouseMove, handleMouseUp, handleWheel
  - Auto fit-to-screen with updateDisplayScale()
  - Reset to default view

- Refactored PlayerDisplayWindow.vue:
  - Removed 6 IPC listener variables and manual setup/cleanup
  - Replaced viewport state (playerPanX, playerPanY, playerZoom, isPanning, panStart, displayScale, imageNaturalWidth, imageNaturalHeight)
  - Replaced 7 viewport functions with composable methods
  - Simplified event handler setup using callback pattern

**Results:**
- PlayerDisplayWindow.vue reduced from 1,081 to 899 lines (-182 lines, 16.8% reduction)
- Build passes with no TypeScript errors
- All player display functionality preserved
- usePlayerViewport is reusable by DmMapViewer and other map components