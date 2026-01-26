<template>
  <svg
    v-if="lights.length > 0"
    class="light-overlay"
    :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
    :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
  >
    <defs>
      <!-- Define radial gradients for each light -->
      <radialGradient
        v-for="light in lights"
        :key="`gradient-${light.id}`"
        :id="`light-gradient-${light.id}`"
        cx="50%"
        cy="50%"
        r="50%"
        fx="50%"
        fy="50%"
      >
        <!-- Core (0-20%) -->
        <stop offset="0%" :stop-color="getLightColor(light, 0.5)" />
        <!-- Falloff zone (20-60%) -->
        <stop offset="20%" :stop-color="getLightColor(light, 0.35)" />
        <!-- Dim edge (60-90%) -->
        <stop offset="60%" :stop-color="getLightColor(light, 0.15)" />
        <!-- Fade to transparent -->
        <stop offset="100%" stop-color="rgba(0, 0, 0, 0)" />
      </radialGradient>

      <!-- Clip paths for shadow-casting lights -->
      <clipPath
        v-for="(polygon, idx) in lightVisibilityPolygons"
        :key="`clip-${idx}`"
        :id="`light-clip-${polygon.lightId}`"
      >
        <path :d="polygon.path" />
      </clipPath>
    </defs>

    <!-- Render each light -->
    <g class="lights-group" :style="{ mixBlendMode: blendMode }">
      <!-- All lights - using clip path for shadow casters if available -->
      <g
        v-for="light in lights"
        :key="`light-${light.id}`"
        :clip-path="light.shadows && hasClipPath(light.id) ? `url(#light-clip-${light.id})` : undefined"
      >
        <circle
          :cx="light.position.x"
          :cy="light.position.y"
          :r="light.range"
          :fill="`url(#light-gradient-${light.id})`"
          class="light-circle"
        />
      </g>
    </g>

    <!-- Debug: show light centers if debug mode -->
    <g v-if="showDebug" class="light-debug">
      <g v-for="light in lights" :key="`debug-${light.id}`">
        <!-- Light visibility polygon outline (for shadow casters) -->
        <path
          v-if="light.shadows && getLightPolygonPath(light.id)"
          :d="getLightPolygonPath(light.id) ?? undefined"
          fill="none"
          stroke="orange"
          stroke-width="2"
          stroke-dasharray="4,4"
          opacity="0.6"
        />
        <!-- Light center marker -->
        <circle
          :cx="light.position.x"
          :cy="light.position.y"
          r="8"
          fill="yellow"
          stroke="#333"
          stroke-width="2"
        />
        <!-- Light range circle -->
        <circle
          :cx="light.position.x"
          :cy="light.position.y"
          :r="light.range"
          fill="none"
          stroke="yellow"
          stroke-width="2"
          stroke-dasharray="8,4"
          opacity="0.5"
        />
        <!-- Label -->
        <text
          :x="light.position.x"
          :y="light.position.y - 15"
          text-anchor="middle"
          class="debug-label"
        >
          {{ light.id }} (r={{ Math.round(light.range) }}{{ light.shadows ? ', shadows' : '' }})
        </text>
      </g>
    </g>
  </svg>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import {
  type Light,
  type Wall,
  calculateVisibilityPolygon,
  polygonToSvgPath
} from '@/composables/useVisibilityPolygon'

interface Props {
  lights: Light[]
  walls: Wall[]
  mapWidth: number
  mapHeight: number
  showDebug?: boolean
  blendMode?: 'screen' | 'lighten' | 'soft-light' | 'normal' | 'overlay'
}

const props = withDefaults(defineProps<Props>(), {
  showDebug: false,
  blendMode: 'soft-light',
  walls: () => []
})

// Debug: log when props change
watch(() => [props.lights, props.walls], () => {
  console.log('LightOverlay: lights=', props.lights.length, 'walls=', props.walls.length, 'map=', props.mapWidth, 'x', props.mapHeight)
  props.lights.forEach(l => console.log('  Light:', l.id, 'pos:', l.position, 'range:', l.range, 'shadows:', l.shadows))
}, { immediate: true })

// Split lights into shadow-casting and non-shadow-casting
const shadowCastingLights = computed(() =>
  props.lights.filter(l => l.shadows && props.walls.length > 0)
)

const nonShadowLights = computed(() =>
  props.lights.filter(l => !l.shadows || props.walls.length === 0)
)

// Calculate visibility polygons for shadow-casting lights
const lightVisibilityPolygons = computed(() => {
  if (props.walls.length === 0) {
    console.log('LightOverlay: No walls, skipping visibility calculation')
    return []
  }

  console.log('LightOverlay: Calculating visibility for', shadowCastingLights.value.length, 'lights with', props.walls.length, 'walls')

  return shadowCastingLights.value.map(light => {
    const polygon = calculateVisibilityPolygon(
      light.position,
      props.walls,
      light.range,
      props.mapWidth,
      props.mapHeight
    )
    const path = polygonToSvgPath(polygon)
    console.log(`LightOverlay: Light ${light.id} at (${light.position.x}, ${light.position.y}) - polygon has ${polygon.length} points, path: ${path.substring(0, 50)}...`)
    return {
      lightId: light.id,
      polygon,
      path
    }
  })
})

// Helper to get polygon path for a specific light
function getLightPolygonPath(lightId: string): string | null {
  const poly = lightVisibilityPolygons.value.find(p => p.lightId === lightId)
  return poly?.path || null
}

// Helper to check if a clip path exists and is valid
function hasClipPath(lightId: string): boolean {
  const path = getLightPolygonPath(lightId)
  return path !== null && path.length > 10
}

/**
 * Get light color with adjusted opacity.
 */
function getLightColor(light: Light, baseOpacity: number): string {
  // Parse the rgba color
  const match = light.color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/)
  if (match) {
    const r = parseInt(match[1])
    const g = parseInt(match[2])
    const b = parseInt(match[3])
    // Scale by intensity, cap at 0.7 to avoid washing out
    const alpha = Math.min(0.7, Math.max(0, baseOpacity * Math.min(light.intensity, 2)))
    return `rgba(${r}, ${g}, ${b}, ${alpha})`
  }
  // Fallback warm light
  return `rgba(255, 200, 100, ${Math.min(0.7, baseOpacity)})`
}
</script>

<style scoped>
.light-overlay {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 5; /* Below fog - purely visual decoration */
  pointer-events: none;
}

.lights-group {
  /* Blend mode set via prop */
}

.light-circle {
  /* Smooth rendering */
}

.light-debug {
  pointer-events: none;
}

.debug-label {
  fill: yellow;
  font-size: 12px;
  font-family: system-ui, sans-serif;
  font-weight: 600;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.9);
}
</style>
