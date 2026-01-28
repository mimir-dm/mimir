---
id: los-debug-visualization
level: task
title: "LOS debug visualization"
short_code: "MIMIR-T-0426"
created_at: 2026-01-25T02:44:32.489016+00:00
updated_at: 2026-01-25T16:04:45.025972+00:00
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

# LOS debug visualization

## Parent Initiative

[[MIMIR-I-0046]] - Map & Token VTT System

## Objective

Create a debug overlay component for DMs to visualize UVTT walls, portals, and visibility polygons to help troubleshoot LOS issues.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Toggle button in DM toolbar to show/hide debug overlay
- [ ] Walls rendered as blue lines
- [ ] Portals rendered as red lines (with open/closed state indicated)
- [ ] Visibility polygons rendered as semi-transparent orange fill
- [ ] Light source ranges rendered as yellow circles
- [ ] DM-only feature (not sent to player display)

## Implementation Notes

### LosDebugOverlay Component

```vue
<template>
  <svg class="los-debug-overlay" :width="mapWidth" :height="mapHeight">
    <!-- Walls (blue) -->
    <g class="walls">
      <line
        v-for="(wall, idx) in walls"
        :key="'wall-' + idx"
        :x1="wall.start.x"
        :y1="wall.start.y"
        :x2="wall.end.x"
        :y2="wall.end.y"
        stroke="blue"
        stroke-width="2"
      />
    </g>
    
    <!-- Portals (red, dashed if open) -->
    <g class="portals">
      <line
        v-for="portal in portals"
        :key="'portal-' + portal.id"
        :x1="portal.wall.start.x"
        :y1="portal.wall.start.y"
        :x2="portal.wall.end.x"
        :y2="portal.wall.end.y"
        stroke="red"
        stroke-width="3"
        :stroke-dasharray="portal.closed ? 'none' : '5,5'"
      />
    </g>
    
    <!-- Visibility polygons (orange, semi-transparent) -->
    <g class="visibility">
      <path
        v-for="polygon in visibilityPolygons"
        :key="'vis-' + polygon.tokenId"
        :d="polygon.path"
        fill="rgba(255, 165, 0, 0.3)"
        stroke="orange"
        stroke-width="1"
      />
    </g>
    
    <!-- Light ranges (yellow circles) -->
    <g class="lights">
      <circle
        v-for="light in activeLights"
        :key="'light-' + light.id"
        :cx="light.x"
        :cy="light.y"
        :r="feetToPixels(light.bright_radius_ft + light.dim_radius_ft)"
        fill="none"
        stroke="yellow"
        stroke-width="1"
        stroke-dasharray="3,3"
      />
    </g>
  </svg>
</template>
```

### Toolbar Integration

```vue
<button 
  @click="showLosDebug = !showLosDebug"
  :class="{ active: showLosDebug }"
  title="Toggle LOS Debug"
>
  🔍
</button>

<LosDebugOverlay v-if="showLosDebug" ... />
```

### Files to Create/Modify

- `crates/mimir/frontend/src/components/map/LosDebugOverlay.vue`
- `crates/mimir/frontend/src/components/DmMapViewer.vue` (toggle & integration)

### Dependencies

- MIMIR-T-0424 (useVisibilityPolygon)
- useUvttMap (walls and portals data)

## Status Updates

*To be added during implementation*