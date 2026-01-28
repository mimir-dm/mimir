---
id: viewport-synchronization
level: task
title: "Viewport synchronization"
short_code: "MIMIR-T-0423"
created_at: 2026-01-25T02:44:23.395010+00:00
updated_at: 2026-01-25T02:44:23.395010+00:00
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

# Viewport synchronization

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Integration testing for the player display system. The frontend components are already implemented - this task verifies the complete system works end-to-end once backend commands are in place.

**Note**: DmMapViewer already has "Push View" button and viewport sync code. PlayerDisplayWindow already handles viewport events.

## Acceptance Criteria

## Acceptance Criteria

- [ ] Open player display window from DmMapViewer toolbar
- [ ] Load map in DM view, verify it appears in player display
- [ ] Place tokens, verify visible tokens appear in player display
- [ ] Toggle token visibility, verify hidden tokens disappear from player view
- [ ] Push viewport, verify player display pans/zooms
- [ ] Toggle blackout mode, verify player display goes dark
- [ ] Close player display window

## Test Scenarios

### Scenario 1: Basic Window Lifecycle
1. Click "Open Player Display" in DmMapViewer toolbar
2. Verify window opens with correct title
3. Click "Close Player Display"
4. Verify window closes

### Scenario 2: Map Sync
1. Open player display window
2. Load a map in DmMapViewer
3. Verify map image appears in player display
4. Verify grid overlay matches (if enabled)

### Scenario 3: Token Sync
1. Add monster tokens to map
2. Verify tokens appear in player display
3. Toggle a token hidden
4. Verify token disappears from player display
5. Toggle token visible again
6. Verify token reappears

### Scenario 4: Viewport Sync
1. Pan/zoom DM view to specific location
2. Click "Push View" button
3. Verify player display smoothly transitions to same view
4. Pan DM view elsewhere
5. Verify player display doesn't move (manual sync only)

### Scenario 5: Blackout Mode
1. Toggle blackout on
2. Verify player display shows black screen
3. Toggle blackout off
4. Verify map reappears

### Files to Verify

- `crates/mimir/frontend/src/components/DmMapViewer.vue` - Toolbar controls
- `crates/mimir/frontend/src/components/PlayerDisplayWindow.vue` - Event handling
- `crates/mimir/frontend/src/composables/usePlayerDisplay.ts` - API calls

### Dependencies

- MIMIR-T-0421 (Backend commands)
- MIMIR-T-0422 (Route configuration)

## Status Updates

*To be added during implementation*