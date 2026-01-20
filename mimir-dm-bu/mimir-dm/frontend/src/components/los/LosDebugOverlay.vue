<template>
  <svg
    v-if="visible && (walls.length > 0 || portals.length > 0)"
    class="los-debug-layer"
    :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
    :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
  >
    <!-- Wall segments -->
    <g class="walls-group">
      <line
        v-for="(wall, idx) in walls"
        :key="`wall-${idx}`"
        class="wall-line"
        :x1="wall.p1.x"
        :y1="wall.p1.y"
        :x2="wall.p2.x"
        :y2="wall.p2.y"
      />
      <!-- Wall endpoints -->
      <circle
        v-for="(point, idx) in uniqueWallEndpoints"
        :key="`wall-point-${idx}`"
        class="wall-endpoint"
        :cx="point.x"
        :cy="point.y"
        r="3"
      />
    </g>

    <!-- Portal (door) segments -->
    <g class="portals-group">
      <line
        v-for="portal in portals"
        :key="`portal-${portal.id}`"
        class="portal-line"
        :class="{ closed: portal.closed, open: !portal.closed }"
        :x1="portal.wall.p1.x"
        :y1="portal.wall.p1.y"
        :x2="portal.wall.p2.x"
        :y2="portal.wall.p2.y"
      />
      <!-- Portal center indicators -->
      <circle
        v-for="portal in portals"
        :key="`portal-center-${portal.id}`"
        class="portal-center"
        :class="{ closed: portal.closed, open: !portal.closed }"
        :cx="(portal.wall.p1.x + portal.wall.p2.x) / 2"
        :cy="(portal.wall.p1.y + portal.wall.p2.y) / 2"
        r="5"
      />
    </g>

    <!-- Legend -->
    <g v-if="showLegend" class="legend-group" transform="translate(10, 10)">
      <rect class="legend-bg" x="0" y="0" width="120" height="60" rx="4" />
      <line class="wall-line" x1="10" y1="15" x2="30" y2="15" />
      <text class="legend-text" x="35" y="18">Walls ({{ walls.length }})</text>
      <line class="portal-line closed" x1="10" y1="35" x2="30" y2="35" />
      <text class="legend-text" x="35" y="38">Doors ({{ closedPortalCount }})</text>
      <line class="portal-line open" x1="10" y1="50" x2="30" y2="50" />
      <text class="legend-text" x="35" y="53">Open ({{ openPortalCount }})</text>
    </g>
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Wall, Portal, Point } from '@/composables/useVisibilityPolygon'

interface Props {
  walls: Wall[]
  portals: Portal[]
  mapWidth: number
  mapHeight: number
  visible?: boolean
  showLegend?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  visible: true,
  showLegend: true
})

// Compute unique wall endpoints for display
const uniqueWallEndpoints = computed(() => {
  const seen = new Set<string>()
  const points: Point[] = []

  for (const wall of props.walls) {
    const k1 = `${wall.p1.x},${wall.p1.y}`
    const k2 = `${wall.p2.x},${wall.p2.y}`
    if (!seen.has(k1)) {
      seen.add(k1)
      points.push(wall.p1)
    }
    if (!seen.has(k2)) {
      seen.add(k2)
      points.push(wall.p2)
    }
  }

  return points
})

// Portal counts for legend
const closedPortalCount = computed(() =>
  props.portals.filter(p => p.closed).length
)
const openPortalCount = computed(() =>
  props.portals.filter(p => !p.closed).length
)
</script>

<style scoped>
.los-debug-layer {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
  z-index: 15; /* Above most overlays */
}

.wall-line {
  stroke: #ff4444;
  stroke-width: 2;
  stroke-linecap: round;
  opacity: 0.8;
}

.wall-endpoint {
  fill: #ff4444;
  opacity: 0.7;
}

.portal-line {
  stroke-width: 3;
  stroke-linecap: round;
}

.portal-line.closed {
  stroke: #ff9900;
  stroke-dasharray: none;
  opacity: 0.9;
}

.portal-line.open {
  stroke: #44ff44;
  stroke-dasharray: 6, 4;
  opacity: 0.7;
}

.portal-center {
  stroke: white;
  stroke-width: 1;
  cursor: pointer;
  pointer-events: auto;
}

.portal-center.closed {
  fill: #ff9900;
}

.portal-center.open {
  fill: #44ff44;
}

.legend-bg {
  fill: rgba(0, 0, 0, 0.75);
  stroke: rgba(255, 255, 255, 0.2);
  stroke-width: 1;
}

.legend-text {
  fill: white;
  font-size: 10px;
  font-family: system-ui, sans-serif;
}
</style>
