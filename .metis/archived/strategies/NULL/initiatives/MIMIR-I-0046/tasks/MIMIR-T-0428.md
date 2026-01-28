---
id: end-to-end-testing-and-bug-fixes
level: task
title: "End-to-end testing and bug fixes"
short_code: "MIMIR-T-0428"
created_at: 2026-01-25T02:44:33.312794+00:00
updated_at: 2026-01-25T02:44:33.312794+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# End-to-end testing and bug fixes

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Comprehensive manual testing of the complete map/token VTT system once all backend commands are implemented. The frontend is already complete - this tests the frontend-backend integration.

**Prerequisites**: T-0414, T-0415, T-0416, T-0417, T-0421, T-0422, T-0423 must be completed.

## Acceptance Criteria

## Acceptance Criteria

- [ ] All test scenarios pass
- [ ] Critical bugs fixed
- [ ] Performance acceptable (smooth pan/zoom, responsive token drag)
- [ ] Player display sync works reliably

## Test Scenarios

### Basic Map

- [ ] Load a UVTT map - image displays with grid overlay
- [ ] Pan map with mouse drag
- [ ] Zoom with mouse wheel
- [ ] Grid snapping visual works

### Token System

- [ ] Create token from monster catalog - appears on map
- [ ] Token image displays (from catalog) or colored circle fallback
- [ ] Drag token - snaps to grid
- [ ] Context menu appears on right-click
- [ ] Toggle visibility - token shows/hides indicator
- [ ] Toggle dead - skull overlay appears
- [ ] Toggle light - light source created/removed
- [ ] Delete token - removed from map

### Light Sources

- [ ] Create light source (torch preset)
- [ ] Light renders with bright/dim radii
- [ ] Toggle light on/off
- [ ] Light affects vision in darkness

### Player View Settings

- [ ] Lighting dropdown changes behavior (Bright/Dim/Darkness)
- [ ] Mask Unexplored toggle - fog overlay appears/disappears
- [ ] Mask Hidden Tokens toggle - affects token filtering

### Fog of War (when Mask Unexplored = on)

- [ ] Fog overlay renders
- [ ] PC tokens reveal area around them
- [ ] Vision blocked by walls
- [ ] Open/close door - visibility updates
- [ ] LOS debug mode shows walls/portals/polygons

### Player Display

- [ ] Open player display window
- [ ] Map syncs to player window
- [ ] Only visible tokens appear
- [ ] Fog/vision mask applied correctly
- [ ] Push View syncs viewport
- [ ] Token movements sync in real-time

### State Persistence

- [ ] Close and reopen map - tokens at starting positions
- [ ] Play session changes (positions, dead, lights) reset on reload
- [ ] Token Placement Panel can save current layout as new starting positions

## Bug Tracking

Document bugs found during testing here, then fix them:

### Found Issues

*To be filled during testing*

### Resolved Issues

*To be updated as bugs are fixed*

## Dependencies

- All other tasks in this initiative must be completed

## Status Updates

*To be added during implementation*