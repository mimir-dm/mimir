<template>
  <svg
    v-if="portals.length > 0"
    class="door-interaction-layer"
    :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
    :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
  >
    <defs>
      <!-- Drop shadow for hover effect -->
      <filter id="doorGlow" x="-50%" y="-50%" width="200%" height="200%">
        <feGaussianBlur stdDeviation="3" result="blur" />
        <feMerge>
          <feMergeNode in="blur" />
          <feMergeNode in="SourceGraphic" />
        </feMerge>
      </filter>
    </defs>

    <!-- Door interaction zones -->
    <g
      v-for="portal in portals"
      :key="`door-${portal.id}`"
      class="door-group"
      :class="{ hovered: hoveredDoorId === portal.id }"
      @mouseenter="hoveredDoorId = portal.id"
      @mouseleave="hoveredDoorId = null"
      @click="handleDoorClick(portal)"
    >
      <!-- Invisible hit area (wider than visual) -->
      <line
        class="door-hit-area"
        :x1="portal.wall.p1.x"
        :y1="portal.wall.p1.y"
        :x2="portal.wall.p2.x"
        :y2="portal.wall.p2.y"
      />

      <!-- Visual door line -->
      <line
        class="door-line"
        :class="{ closed: portal.closed, open: !portal.closed }"
        :x1="portal.wall.p1.x"
        :y1="portal.wall.p1.y"
        :x2="portal.wall.p2.x"
        :y2="portal.wall.p2.y"
        :filter="hoveredDoorId === portal.id ? 'url(#doorGlow)' : ''"
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
          r="10"
        />
        <!-- Door state icon -->
        <g v-if="portal.closed">
          <!-- Closed door icon (horizontal bar) -->
          <rect class="door-icon" x="-6" y="-2" width="12" height="4" rx="1" />
        </g>
        <g v-else>
          <!-- Open door icon (vertical bar - rotated) -->
          <rect class="door-icon open" x="-2" y="-6" width="4" height="12" rx="1" />
        </g>
      </g>

      <!-- State label on hover -->
      <text
        v-if="hoveredDoorId === portal.id"
        class="door-label"
        :x="(portal.wall.p1.x + portal.wall.p2.x) / 2"
        :y="(portal.wall.p1.y + portal.wall.p2.y) / 2 + 22"
        text-anchor="middle"
      >
        {{ portal.closed ? 'Click to open' : 'Click to close' }}
      </text>
    </g>
  </svg>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Portal } from '@/composables/map/useVisibilityPolygon'

interface Props {
  portals: Portal[]
  mapWidth: number
  mapHeight: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'toggle-door': [portalId: string]
}>()

// Hovered door state
const hoveredDoorId = ref<string | null>(null)

// Handle door click
function handleDoorClick(portal: Portal) {
  emit('toggle-door', portal.id)
}
</script>

<style scoped>
.door-interaction-layer {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 12; /* Above tokens, below context menus */
  pointer-events: none; /* Allow clicks to pass through to tokens */
}

.door-group {
  cursor: pointer;
  pointer-events: auto; /* Re-enable for door groups only */
}

.door-hit-area {
  stroke: transparent;
  stroke-width: 20;
  fill: none;
  pointer-events: stroke;
}

.door-line {
  stroke-width: 4;
  stroke-linecap: round;
  fill: none;
  transition: all 0.15s ease;
}

.door-line.closed {
  stroke: #c4883a;
}

.door-line.open {
  stroke: #6b8e4e;
  stroke-dasharray: 5, 4;
}

.door-group.hovered .door-line {
  stroke-width: 5;
}

.door-group.hovered .door-line.closed {
  stroke: #e5a44a;
}

.door-group.hovered .door-line.open {
  stroke: #8ab861;
}

.door-icon-bg {
  transition: all 0.15s ease;
}

.door-icon-bg.closed {
  fill: #c4883a;
  stroke: #2a1f0d;
  stroke-width: 1.5;
}

.door-icon-bg.open {
  fill: #6b8e4e;
  stroke: #1f2a15;
  stroke-width: 1.5;
}

.door-group.hovered .door-icon-bg {
  r: 12;
}

.door-group.hovered .door-icon-bg.closed {
  fill: #e5a44a;
}

.door-group.hovered .door-icon-bg.open {
  fill: #8ab861;
}

.door-icon {
  fill: #2a1f0d;
  transition: all 0.15s ease;
}

.door-icon.open {
  fill: #1f2a15;
}

.door-label {
  fill: white;
  font-size: 11px;
  font-family: system-ui, sans-serif;
  font-weight: 500;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.9);
  pointer-events: none;
}
</style>
