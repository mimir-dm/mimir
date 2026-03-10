---
id: map-feature-tests-fog-of-war-light
level: task
title: "Map feature tests — fog of war, light sources, traps, points of interest"
short_code: "MIMIR-T-0562"
created_at: 2026-03-10T01:31:58.429772+00:00
updated_at: 2026-03-10T01:31:58.429772+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*