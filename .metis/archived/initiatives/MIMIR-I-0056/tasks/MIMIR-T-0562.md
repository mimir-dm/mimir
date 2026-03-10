---
id: map-feature-tests-fog-of-war-light
level: task
title: "Map feature tests — fog of war, light sources, traps, points of interest"
short_code: "MIMIR-T-0562"
created_at: 2026-03-10T01:31:58.429772+00:00
updated_at: 2026-03-10T15:09:35.316218+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Map feature tests — fog of war, light sources, traps, points of interest

**Phase 6** — Homebrew & Advanced Features

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest tests for the DM map features — fog of war, light sources, traps, and points of interest. These are complex interactive features on the DM map viewer that involve spatial calculations and state management.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Fog of war toggle reveals/hides map areas
- [ ] Fog of war state persists across map reloads
- [ ] Light sources calculate correct bright/dim radius areas
- [ ] Multiple light sources combine correctly (overlapping areas)
- [ ] Darkvision interacts correctly with light source calculations
- [ ] Trap placement stores correct position and properties
- [ ] Trap visibility toggle (DM-only vs player-visible)
- [ ] Points of interest render with correct icons and labels
- [ ] POI click/hover shows detail panel
- [ ] `useUvttMap` composable parses UVTT format correctly (walls, doors, lights)
- [ ] All tests pass in CI

## Key Components

- `DmMapViewer.vue` — main DM map view
- `useUvttMap.ts` — UVTT map format parser
- `useVisionCalculation.ts` — vision/light area calculation
- `useLightSources.ts` — light source management
- Trap and POI overlay components

## Implementation Notes

The composables (`useUvttMap`, `useVisionCalculation`, `useLightSources`) are the most testable units — they're pure logic that can be tested with known inputs and expected outputs. Create test UVTT map data with known wall positions, then verify vision calculations produce correct results. For fog of war, test the state management (which cells are revealed) rather than canvas rendering.

## Status Updates

### Session 2 (2026-03-10)
- Created 8 test files with 164 passing tests covering all map feature composables:
  - `useVisionCalculation.test.ts` (36 tests) — light zones, light levels, token vision in bright/dim/darkness, darkvision, party visibility, overlay detection
  - `usePlayerViewport.test.ts` (26 tests) — pan/zoom state, transform computation, mouse interactions, zoom clamping, reset, display scale
  - `useVisibilityPolygon.test.ts` (18 tests) — UVTT wall/portal/light coordinate conversion, ambient light ARGB parsing
  - `usePlayerDisplayEvents.test.ts` (12 tests) — event registration, dispatching all 6 event types, cleanup on unmount
  - `useFog.test.ts` (19 tests) — fog state CRUD, reveal rect/circle, isPointRevealed spatial logic, error handling
  - `useLightSources.test.ts` (18 tests) — light source CRUD, computed properties, toggle/move/delete, unit conversion, LIGHT_PRESETS
  - `useMapMarkers.test.ts` (17 tests) — trap/POI loading, visibility filtering for display, context menu, icon mapping, clear
  - `usePlayerDisplay.test.ts` (18 tests) — window open/close/toggle, blackout, sendMapToDisplay params, fullscreen, error handling
- All 164 tests pass in 1.8s