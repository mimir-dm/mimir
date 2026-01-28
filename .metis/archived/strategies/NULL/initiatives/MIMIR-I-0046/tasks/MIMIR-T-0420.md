---
id: dmmapviewer-token-and-light-wiring
level: task
title: "DmMapViewer token and light wiring"
short_code: "MIMIR-T-0420"
created_at: 2026-01-25T02:44:22.153245+00:00
updated_at: 2026-01-25T16:04:44.080842+00:00
parent: MIMIR-I-0046
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: MIMIR-I-0046
---

# DmMapViewer token and light wiring

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Wire up the DmMapViewer component to use the useTokens and useLightSources composables, enabling token/light rendering and interaction.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Tokens load and render when map is opened
- [ ] Token images display (or colored circle fallback)
- [ ] Token drag and drop updates runtime position
- [ ] Token selection works (click to select)
- [ ] Light sources render with bright/dim radii
- [ ] Grid snapping works for token placement
- [ ] Toolbar buttons functional (Add Token, etc.)

## Implementation Notes

### Integration Points

```vue
<script setup>
const { tokens, loadTokens, moveToken, ... } = useTokens()
const { lightSources, loadLightSources, ... } = useLightSources()
const { uvttData, loadUvttData, ... } = useUvttMap()

// Load data when map changes
watch(() => props.mapId, async (mapId) => {
  if (mapId) {
    await loadTokens(mapId)
    await loadLightSources(mapId)
    await loadUvttData(mapId)
  }
})
</script>
```

### Token Rendering

```vue
<template>
  <div class="token-layer">
    <TokenRenderer
      v-for="token in tokens"
      :key="token.id"
      :token="token"
      :selected="selectedTokenId === token.id"
      :grid-size="gridSize"
      @click="selectToken(token.id)"
      @drag-end="handleTokenDragEnd"
    />
  </div>
</template>
```

### Token Drag Handling

```typescript
function handleTokenDragEnd(tokenId: number, x: number, y: number) {
  // Snap to grid
  const snappedX = snapToGrid(x)
  const snappedY = snapToGrid(y)
  
  // Update runtime position (not DB)
  moveToken(tokenId, snappedX, snappedY)
  
  // Emit for player display sync
  emitTokensUpdate()
}
```

### Files to Modify

- `crates/mimir/frontend/src/components/DmMapViewer.vue`
- `crates/mimir/frontend/src/components/tokens/TokenRenderer.vue`
- `crates/mimir/frontend/src/components/lighting/LightSourceRenderer.vue`

### Dependencies

- MIMIR-T-0418 (useTokens composable)
- MIMIR-T-0419 (useLightSources composable)

## Status Updates

*To be added during implementation*