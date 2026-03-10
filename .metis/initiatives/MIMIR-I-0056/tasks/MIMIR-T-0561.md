---
id: player-display-window-tests-token
level: task
title: "Player display window tests — token rendering, map display, DM controls"
short_code: "MIMIR-T-0561"
created_at: 2026-03-10T01:31:57.110544+00:00
updated_at: 2026-03-10T01:31:57.110544+00:00
parent: MIMIR-I-0056
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*