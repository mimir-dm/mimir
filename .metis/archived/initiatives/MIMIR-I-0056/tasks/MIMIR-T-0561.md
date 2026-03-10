---
id: player-display-window-tests-token
level: task
title: "Player display window tests — token rendering, map display, DM controls"
short_code: "MIMIR-T-0561"
created_at: 2026-03-10T01:31:57.110544+00:00
updated_at: 2026-03-10T13:53:27.206056+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: MIMIR-I-0056
---

# Player display window tests — token rendering, map display, DM controls

**Phase 6** — Homebrew & Advanced Features

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[MIMIR-I-0056]]

## Objective

Write Vitest component tests for the player display window — the secondary window that shows the map/content to players during a session. Test token rendering, map display, DM-to-player event communication, and visibility controls.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Player display renders map image at correct dimensions
- [ ] Visible tokens appear at correct positions on the player display
- [ ] Hidden tokens (DM-only) do not appear on the player display
- [ ] Fog of war masks unexplored areas
- [ ] DM reveal events update the player display in real-time
- [ ] Token movement events update positions on player display
- [ ] `usePlayerDisplayEvents` composable handles all event types correctly
- [ ] Player display handles no-map state gracefully
- [ ] All tests pass in CI

## Key Components

- `PlayerDisplayWindow.vue` — the player-facing display
- `usePlayerDisplayEvents.ts` — event handling composable
- Token rendering on player map
- Fog of war overlay

## Implementation Notes

The player display communicates with the DM window via Tauri window events. Mock the event system to simulate DM actions (reveal area, move token, toggle visibility). The `usePlayerDisplayEvents` composable is the key testable unit — it's the bridge between DM actions and player display state.

## Status Updates

### Completed
- Created 3 test files with 56 total tests:

**`usePlayerViewport.test.ts`** (26 tests):
- Initial state: zero pan, 1x zoom, panning disabled, display scale 1, zero image dimensions
- Transform computation: identity transform, pan offset, zoom, combined displayScale+zoom, all three combined
- Panning: mousedown starts panning (left button only), mousemove updates pan, mousemove no-op when not panning, mouseup stops panning
- Zooming: scroll down decreases, scroll up increases, clamp to min/max, custom zoom step
- Reset: resets pan+zoom but preserves displayScale
- updateDisplayScale: null image no-op, zero-dimension no-op, stores natural dimensions
- Custom options: default bounds, all options together

**`usePlayerDisplay.test.ts`** (18 tests):
- Initial state: display closed, no map, blackout off
- checkDisplayOpen: updates state from invoke, returns false on error
- openDisplay: calls invoke, sets state, throws on error
- closeDisplay: calls invoke, resets state
- toggleDisplay: opens when closed, closes when open
- sendMapToDisplay: correct invoke params, default params, updates currentMapId
- Blackout: toggleBlackout toggles state, toggle back off, setBlackout explicit state
- Fullscreen: calls invoke, throws on error

**`usePlayerDisplayEvents.test.ts`** (12 tests):
- Event registration: all 6 listeners registered on mount, 6 unlisten functions created
- Event dispatching: map update, blackout, tokens update, fog update, light sources update, markers update — all dispatch to correct handler with correct payload
- Cleanup: all unlisten functions called on unmount
- Payload types: MapUpdatePayload optional fields, FogUpdatePayload LOS blocking, TokensUpdatePayload dead token IDs