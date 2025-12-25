---
id: add-token-drag-and-drop-movement
level: task
title: "Add token drag-and-drop movement during play"
short_code: "MIMIR-T-0209"
created_at: 2025-12-21T22:15:21.325782+00:00
updated_at: 2025-12-22T02:38:38.548731+00:00
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

# Add token drag-and-drop movement during play

## Parent Initiative
[[MIMIR-I-0015]] - Visual Display System

## Objective
Enable the DM to drag tokens on the map during play mode, with real-time updates to the player display.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] DM can click and drag tokens on the map viewer
- [ ] Tokens snap to grid during/after drag (when grid enabled)
- [ ] Hold Shift to disable grid snapping
- [ ] Token position updates in database on drag end
- [ ] Player display updates in real-time as token moves
- [ ] Visual feedback during drag (shadow, highlight)
- [ ] Multi-select and drag multiple tokens together

## Implementation Notes

### Drag Implementation

```typescript
// Token drag state
const dragState = ref<{
  tokenId: number
  startX: number
  startY: number
  offsetX: number
  offsetY: number
} | null>(null)

function onTokenMouseDown(token: Token, event: MouseEvent) {
  dragState.value = {
    tokenId: token.id,
    startX: token.x,
    startY: token.y,
    offsetX: event.clientX - tokenScreenX,
    offsetY: event.clientY - tokenScreenY,
  }
}

function onMouseMove(event: MouseEvent) {
  if (!dragState.value) return
  // Update token visual position
  // Optionally send interim updates to player display
}

function onMouseUp(event: MouseEvent) {
  if (!dragState.value) return
  // Calculate final grid position
  // Snap to grid if enabled and Shift not held
  // Call update_token_position command
  // Token position persists to DB
  dragState.value = null
}
```

### Grid Snapping
- Calculate nearest grid intersection based on token size
- Large tokens snap to top-left grid cell of their footprint
- Respect grid offset from map configuration

### Real-time Player Display Updates
- During drag: optionally send throttled position updates
- On drop: send final position via IPC
- Player display animates token movement smoothly

### Multi-Select
- Ctrl+click to add/remove from selection
- Drag any selected token to move all
- Calculate relative positions to maintain formation

### Files to Modify
- `crates/mimir-dm/frontend/src/components/DmMapViewer.vue`
- `crates/mimir-dm/frontend/src/components/TokenRenderer.vue`

### Dependencies
- T-0208 (token rendering)
- T-0206 (update_token_position command)