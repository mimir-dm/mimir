---
id: fog-overlay-svg-mask-rendering
level: task
title: "Fog overlay SVG mask rendering"
short_code: "MIMIR-T-0425"
created_at: 2026-01-25T02:44:31.941551+00:00
updated_at: 2026-01-25T16:04:44.731475+00:00
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

# Fog overlay SVG mask rendering

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Implement the fog overlay component that masks unexplored areas using SVG masks with visibility polygons.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] FogOverlay component renders SVG mask over map
- [ ] Uses visibility polygons to reveal explored areas
- [ ] Different opacity for DM view (semi-transparent) vs player view (opaque)
- [ ] Soft edges on visibility boundaries (Gaussian blur)
- [ ] Respects `maskUnexplored` setting

## Implementation Notes

### SVG Mask Technique

```vue
<template>
  <svg class="fog-overlay" :width="mapWidth" :height="mapHeight">
    <defs>
      <mask id="fogMask">
        <!-- White = fogged (hidden), Black = revealed -->
        <rect width="100%" height="100%" fill="white" />
        
        <!-- Cut out visibility polygons (black reveals) -->
        <g v-for="polygon in visibilityPolygons" :key="polygon.tokenId">
          <path :d="polygon.path" fill="black" />
        </g>
      </mask>
      
      <!-- Soft edge filter -->
      <filter id="fogBlur">
        <feGaussianBlur stdDeviation="10" />
      </filter>
    </defs>
    
    <!-- Fog rectangle with mask applied -->
    <rect 
      width="100%" 
      height="100%" 
      :fill="fogColor"
      mask="url(#fogMask)"
      filter="url(#fogBlur)"
    />
  </svg>
</template>

<script setup>
const props = defineProps<{
  mapWidth: number
  mapHeight: number
  visibilityPolygons: VisibilityPolygon[]
  isDmView: boolean
}>()

const fogColor = computed(() => 
  props.isDmView ? 'rgba(0, 0, 0, 0.5)' : 'rgba(0, 0, 0, 1)'
)
</script>
```

### Integration with Player View Settings

```typescript
// Only render if maskUnexplored is enabled
<FogOverlay 
  v-if="playerViewSettings.maskUnexplored"
  :visibility-polygons="visibilityPolygons"
  :is-dm-view="true"
/>
```

### Visibility Polygon Structure

```typescript
interface VisibilityPolygon {
  tokenId: number
  path: string  // SVG path string from useVisibilityPolygon
}
```

### Files to Create/Modify

- `crates/mimir/frontend/src/components/map/FogOverlay.vue`
- `crates/mimir/frontend/src/components/DmMapViewer.vue` (integrate)
- `crates/mimir/frontend/src/components/PlayerMapViewer.vue` (integrate)

### Dependencies

- MIMIR-T-0424 (useVisibilityPolygon raycasting)

## Status Updates

*To be added during implementation*