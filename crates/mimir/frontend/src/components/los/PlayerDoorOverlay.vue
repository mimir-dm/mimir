<template>
  <svg
    v-if="portals.length > 0"
    class="player-door-layer"
    :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
    :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
  >
    <!-- Door markers for each portal -->
    <g
      v-for="portal in portals"
      :key="`door-${portal.id}`"
      class="door-marker"
    >
      <!-- Door line -->
      <line
        class="door-line"
        :class="{ closed: portal.closed, open: !portal.closed }"
        :x1="portal.wall.p1.x"
        :y1="portal.wall.p1.y"
        :x2="portal.wall.p2.x"
        :y2="portal.wall.p2.y"
      />

      <!-- Door icon at center -->
      <g
        :transform="`translate(${(portal.wall.p1.x + portal.wall.p2.x) / 2}, ${(portal.wall.p1.y + portal.wall.p2.y) / 2})`"
      >
        <!-- Background circle -->
        <circle
          class="door-icon-bg"
          :class="{ closed: portal.closed, open: !portal.closed }"
          cx="0"
          cy="0"
          r="8"
        />
        <!-- Door state icon -->
        <g v-if="portal.closed">
          <!-- Closed door icon (horizontal bar) -->
          <rect class="door-icon" x="-5" y="-1.5" width="10" height="3" rx="1" />
        </g>
        <g v-else>
          <!-- Open door icon (vertical bar - rotated) -->
          <rect class="door-icon open" x="-1.5" y="-5" width="3" height="10" rx="1" />
        </g>
      </g>
    </g>
  </svg>
</template>

<script setup lang="ts">
import type { Portal } from '@/composables/map/useVisibilityPolygon'

interface Props {
  portals: Portal[]
  mapWidth: number
  mapHeight: number
}

defineProps<Props>()
</script>

<style scoped>
.player-door-layer {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 8; /* Below fog overlay */
  pointer-events: none;
}

.door-line {
  stroke-width: 3;
  stroke-linecap: round;
  fill: none;
}

.door-line.closed {
  stroke: #c4883a;
}

.door-line.open {
  stroke: #6b8e4e;
  stroke-dasharray: 4, 3;
}

.door-icon-bg.closed {
  fill: #c4883a;
  stroke: #2a1f0d;
  stroke-width: 1;
}

.door-icon-bg.open {
  fill: #6b8e4e;
  stroke: #1f2a15;
  stroke-width: 1;
}

.door-icon {
  fill: #2a1f0d;
}

.door-icon.open {
  fill: #1f2a15;
}
</style>
