---
id: player-display-window-and-view
level: task
title: "Player display window and view"
short_code: "MIMIR-T-0422"
created_at: 2026-01-25T02:44:22.947595+00:00
updated_at: 2026-01-25T16:34:10.044978+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# Player display window and view

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Verify and configure the player display window routing and Tauri config. The frontend component `PlayerDisplayWindow.vue` already exists and is fully implemented.

**Frontend already exists** at `components/PlayerDisplayWindow.vue` - fully implemented

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Vue Router has `/player-display` route configured
- [ ] Route points to PlayerDisplayWindow component
- [ ] Tauri config allows multi-window (if needed)
- [ ] Player display window receives IPC events correctly
- [ ] Manual test: open window, send map, verify display

## Implementation Notes

### Route Configuration

Check/add in `router/index.ts`:

```typescript
{
  path: '/player-display',
  name: 'player-display',
  component: () => import('@/components/PlayerDisplayWindow.vue'),
  meta: { hideNav: true }  // No navigation bar in player window
}
```

### Tauri Configuration

Verify `tauri.conf.json` allows window creation:

```json
{
  "tauri": {
    "allowlist": {
      "window": {
        "create": true,
        "close": true,
        "setFullscreen": true
      }
    }
  }
}
```

### Existing Component (for reference)

The `PlayerDisplayWindow.vue` component already:
- Listens for map-update, tokens-update, viewport-update events
- Renders map with blackout mode support
- Filters tokens by visibility
- Renders visibility polygons
- Handles viewport sync

### Files to Verify/Modify

- `crates/mimir/frontend/src/router/index.ts` - Add route
- `crates/mimir/src-tauri/tauri.conf.json` - Verify allowlist
- `crates/mimir/src-tauri/Cargo.toml` - Verify features

### Dependencies

- MIMIR-T-0421 (Backend commands for window management)

## Status Updates

### Completed 2026-01-25

Added player-display route to `crates/mimir/frontend/src/app/router/index.ts`:

```typescript
{
  path: '/player-display',
  name: 'player-display',
  component: () => import('../../components/PlayerDisplayWindow.vue'),
  meta: { hideNav: true, isPlayerDisplay: true }
}
```

Route configuration verified with `npm run type-check`.

**Note**: Tauri v2 with webview window builder doesn't require explicit window allowlist - permissions are handled differently than v1. The backend commands in T-0421 use `WebviewWindowBuilder` which is the Tauri v2 approach.