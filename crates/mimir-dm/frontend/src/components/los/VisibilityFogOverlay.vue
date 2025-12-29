<template>
  <svg
    v-if="enabled && mapWidth > 0 && mapHeight > 0"
    class="visibility-fog-layer"
    :class="{ 'dm-view': isDmView }"
    :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
    :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
  >
    <defs>
      <!-- Blur filter for soft vision edges -->
      <filter id="visionEdgeBlur" x="-20%" y="-20%" width="140%" height="140%">
        <feGaussianBlur in="SourceGraphic" :stdDeviation="blurAmount" />
      </filter>

      <!-- Visibility mask: white = fog, black = visible -->
      <mask id="losVisibilityMask">
        <!-- Start with full fog (white) -->
        <rect width="100%" height="100%" fill="white" />

        <!-- Cut out visibility polygons (black) with blur for soft edges -->
        <g filter="url(#visionEdgeBlur)">
          <path
            v-for="(vis, idx) in visibilityData"
            :key="`vis-${idx}`"
            :d="vis.path"
            fill="black"
          />
        </g>
      </mask>
    </defs>

    <!-- Fog rectangle with visibility mask -->
    <rect
      width="100%"
      height="100%"
      :fill="fogColor"
      mask="url(#losVisibilityMask)"
    />

    <!-- Debug: show visibility polygon outlines -->
    <g v-if="showPolygonOutlines">
      <path
        v-for="(vis, idx) in visibilityData"
        :key="`outline-${idx}`"
        :d="vis.path"
        fill="none"
        stroke="rgba(0, 255, 255, 0.5)"
        stroke-width="2"
      />
    </g>
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface VisibilityPolygon {
  tokenId: number
  path: string
}

interface Props {
  /** Whether fog is enabled */
  enabled: boolean
  /** Map width in pixels */
  mapWidth: number
  /** Map height in pixels */
  mapHeight: number
  /** Visibility polygon data for each token */
  visibilityData: VisibilityPolygon[]
  /** Whether this is the DM view (semi-transparent) or player view (opaque) */
  isDmView?: boolean
  /** Blur amount for soft edges */
  blurAmount?: number
  /** Show polygon outlines for debugging */
  showPolygonOutlines?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isDmView: false,
  blurAmount: 15,
  showPolygonOutlines: false
})

// Fog color based on view type
const fogColor = computed(() => {
  return props.isDmView
    ? 'rgba(0, 0, 0, 0.5)'  // Semi-transparent for DM
    : '#000000'             // Opaque for players
})
</script>

<style scoped>
.visibility-fog-layer {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
  z-index: 10;
  will-change: transform;
  backface-visibility: hidden;
}

.visibility-fog-layer.dm-view {
  /* DM view is semi-transparent */
  opacity: 1;
}
</style>
