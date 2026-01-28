---
id: decompose-mapgridconfigmodal-vue
level: task
title: "Decompose MapGridConfigModal.vue"
short_code: "MIMIR-T-0476"
created_at: 2026-01-28T05:17:12.423822+00:00
updated_at: 2026-01-28T05:17:12.423822+00:00
parent: MIMIR-I-0052
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0052
---

# Decompose MapGridConfigModal.vue

**File:** `src/features/campaigns/components/StageLanding/MapGridConfigModal.vue`
**Current Size:** 939 lines
**Target Size:** ~250 lines
**Priority:** 8 (Low effort, Medium impact)

## Objective

Extract semi-independent features (calculator, preview editor, grid configurator) into focused composables and child components.

## Current Structure

| Section | Lines | Content |
|---------|-------|---------|
| Template | 1-256 | Modal with form, editor, preview, calculator |
| Script | 258-602 | 29+ refs, 13+ computed, 18 functions |
| Styles | 604-939 | 335 lines of CSS |

## The Problem

- Multiple semi-independent features in one file
- 29+ refs create cognitive load
- Preview logic (zoom/pan/transformation) hard to test
- Calculator could be reused elsewhere

## Extraction Plan

### Phase 1: Composables

1. **useMapPreview()** (~120 lines)
   - Zoom, pan, viewport calculations
   - Image loading state
   - Mouse handlers for pan/zoom
   - Returns: zoom, pan, handlers, viewportStyle

2. **useMapGridCalculator()** (~40 lines)
   - Compression ratio calculation
   - Grid size calculation from original
   - Returns: compressionRatio, calculatedGridSize, applyCalculatedSize

3. **useGridLines()** (~30 lines)
   - Line generation utility
   - Unify vertical/horizontal logic
   - Returns: generateGridLines function

### Phase 2: Child Components

4. **GridOverlay.vue** (~80 lines)
   - SVG grid rendering (square/hex)
   - Props: gridType, gridSize, offset, mapDimensions

5. **GridCalculator.vue** (~60 lines)
   - Calculator section UI
   - Props: originalGridSize, compressionRatio, onApply

6. **GridModeControls.vue** (~50 lines)
   - Cell size + offset input controls
   - Props: gridSize, offset, onAdjust

7. **PositionModeControls.vue** (~40 lines)
   - Zoom controls
   - Props: zoom, onZoomIn, onZoomOut, onReset

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] MapGridConfigModal.vue reduced to ~250 lines
- [ ] useMapPreview() composable reusable for other previews
- [ ] Grid calculator functionality unchanged
- [ ] Grid configuration works correctly
- [ ] Build passes with no TypeScript errors

## Status Updates

### 2026-01-28: Task Cancelled - Component Removed

**MapGridConfigModal.vue has been completely deleted** rather than decomposed.

**Reason:** The component was dead code:
- Called `update_map_grid` backend command which never existed
- UVTT files provide grid configuration via `pixels_per_grid`
- Manual grid configuration is no longer needed

**Removed from:**
- MapTokenSetupModal.vue (Grid button + modal)
- ModuleMaps.vue (Configure Grid button + modal)
- WorldTab.vue (Grid button + modal)
- CampaignMaps.vue (Configure Grid button + modal)

**Lines removed:** 939 (component) + ~100 (references) = ~1,039 lines total