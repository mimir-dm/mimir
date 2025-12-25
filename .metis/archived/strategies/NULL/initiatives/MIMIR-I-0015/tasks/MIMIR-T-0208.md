---
id: render-tokens-on-dm-and-player
level: task
title: "Render tokens on DM and player display"
short_code: "MIMIR-T-0208"
created_at: 2025-12-21T22:15:21.212549+00:00
updated_at: 2025-12-22T02:28:03.736419+00:00
parent: MIMIR-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0015
---

# Render tokens on DM and player display

## Parent Initiative
[[MIMIR-I-0015]] - Visual Display System

## Objective
Render tokens on both the DM's map viewer and the player display window, with proper sizing, positioning, and visibility handling.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Tokens render on DM map viewer at correct grid positions
- [x] Tokens render on player display window
- [x] Token size matches D&D size category (Tiny=0.5, Medium=1, Large=2x2, etc.)
- [x] Hidden tokens shown with transparency on DM view, invisible on player view
- [x] Tokens scale correctly with map zoom
- [x] Token colors/images display properly
- [x] Tokens update in real-time when changed via IPC

## Implementation Summary

### Files Created
- `crates/mimir-dm/frontend/src/components/tokens/TokenRenderer.vue` - Shared component for rendering tokens on map canvas

### Files Modified
- `crates/mimir-dm/frontend/src/components/DmMapViewer.vue` - Added token layer, token loading, IPC emission
- `crates/mimir-dm/frontend/src/components/PlayerDisplayWindow.vue` - Added token layer, IPC listener for token updates

### IPC Events
- `player-display:tokens-update` - Emitted from DmMapViewer with visible tokens when map loads or display opens

## Implementation Notes

### Token Rendering

```typescript
interface TokenRenderData {
  id: number
  name: string
  x: number  // Grid position
  y: number
  size: TokenSize
  visible: boolean
  color?: string
  imagePath?: string
}

// Size to grid squares mapping
const SIZE_GRID_SQUARES = {
  tiny: 0.5,
  small: 1,
  medium: 1,
  large: 2,
  huge: 3,
  gargantuan: 4,
}
```

### DM View
- Show all tokens
- Hidden tokens: 50% opacity with "eye-slash" indicator
- Clickable for selection/editing
- Drag to move (connects to T-0209)

### Player Display View
- Only show tokens where `visible_to_players = true`
- No interaction (display only)
- Smooth transitions when tokens appear/move

### IPC Token Updates
When DM changes tokens, send update to player display:
```typescript
// From main window
emit('tokens-updated', { mapId, tokens: visibleTokens })

// Player display listens
listen('tokens-updated', (event) => {
  updateTokenLayer(event.payload.tokens)
})
```

### Files to Create/Modify
- `crates/mimir-dm/frontend/src/components/DmMapViewer.vue` (add token layer)
- `crates/mimir-dm/frontend/src/components/PlayerDisplayWindow.vue` (add token layer)
- `crates/mimir-dm/frontend/src/components/TokenRenderer.vue` (shared component)

### Dependencies
- T-0206 (TokenService to fetch tokens)
- T-0199 (IPC infrastructure already exists)