<template>
  <svg
    v-if="traps.length > 0 || pois.length > 0"
    class="player-marker-layer"
    :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
    :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
  >
    <!-- Traps -->
    <g class="trap-markers">
      <g
        v-for="trap in traps"
        :key="'trap-' + trap.id"
        class="trap-marker"
        :transform="`translate(${trap.grid_x * gridSizePx + gridSizePx / 2}, ${trap.grid_y * gridSizePx + gridSizePx / 2})`"
      >
        <!-- Trap icon (warning triangle) -->
        <polygon
          points="-10,-8 10,-8 0,10"
          fill="#ef4444"
          stroke="#fff"
          stroke-width="1.5"
        />
        <text
          y="2"
          text-anchor="middle"
          fill="#fff"
          font-size="10"
          font-weight="bold"
        >!</text>
        <!-- Label -->
        <text
          y="-14"
          text-anchor="middle"
          fill="#fff"
          font-size="9"
          class="marker-label"
        >{{ trap.name }}</text>
      </g>
    </g>

    <!-- POIs -->
    <g class="poi-markers">
      <g
        v-for="poi in pois"
        :key="'poi-' + poi.id"
        class="poi-marker"
        :transform="`translate(${poi.grid_x * gridSizePx + gridSizePx / 2}, ${poi.grid_y * gridSizePx + gridSizePx / 2})`"
      >
        <!-- POI icon (circle with icon) -->
        <circle
          r="12"
          :fill="poi.color || '#3b82f6'"
          stroke="#fff"
          stroke-width="1.5"
        />
        <text
          y="4"
          text-anchor="middle"
          fill="#fff"
          font-size="12"
        >{{ getPoiIcon(poi.icon) }}</text>
        <!-- Label -->
        <text
          y="-18"
          text-anchor="middle"
          fill="#fff"
          font-size="9"
          class="marker-label"
        >{{ poi.name }}</text>
      </g>
    </g>
  </svg>
</template>

<script setup lang="ts">
interface MapTrap {
  id: string
  grid_x: number
  grid_y: number
  name: string
}

interface MapPoi {
  id: string
  grid_x: number
  grid_y: number
  name: string
  icon: string
  color: string | null
}

interface Props {
  traps: MapTrap[]
  pois: MapPoi[]
  gridSizePx: number
  mapWidth: number
  mapHeight: number
}

defineProps<Props>()

// Get icon character for POI type (matches PoiEditModal icons)
function getPoiIcon(icon: string): string {
  const icons: Record<string, string> = {
    'pin': '📍',
    'star': '⭐',
    'skull': '💀',
    'chest': '📦',
    'door': '🚪',
    'secret': '🔮',
    'question': '❓',
    'exclamation': '❗'
  }
  return icons[icon] || '📍'
}
</script>

<style scoped>
.player-marker-layer {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 7; /* Below doors and fog */
  pointer-events: none;
}

.marker-label {
  text-shadow:
    -1px -1px 0 #000,
    1px -1px 0 #000,
    -1px 1px 0 #000,
    1px 1px 0 #000;
  font-family: system-ui, sans-serif;
}

.trap-marker,
.poi-marker {
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.5));
}
</style>
