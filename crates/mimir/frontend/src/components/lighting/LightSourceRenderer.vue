<template>
  <svg
    v-if="displayLights.length > 0"
    class="light-source-layer"
    :viewBox="`0 0 ${mapWidth} ${mapHeight}`"
    :style="{ width: mapWidth + 'px', height: mapHeight + 'px' }"
  >
    <defs>
      <!-- Create radial gradients for each light source -->
      <radialGradient
        v-for="light in displayLights"
        :key="`gradient-${light.id}`"
        :id="`lightGradient-${light.id}`"
      >
        <!-- Bright center -->
        <stop offset="0%" :stop-color="getLightColor(light)" stop-opacity="0.4" />
        <!-- Bright edge (transition point) -->
        <stop
          :offset="getBrightRadiusPercent(light) + '%'"
          :stop-color="getLightColor(light)"
          stop-opacity="0.3"
        />
        <!-- Dim light area -->
        <stop
          :offset="getBrightRadiusPercent(light) + '%'"
          :stop-color="getLightColor(light)"
          stop-opacity="0.15"
        />
        <!-- Outer edge (fade to transparent) -->
        <stop offset="100%" :stop-color="getLightColor(light)" stop-opacity="0" />
      </radialGradient>
    </defs>

    <!-- Render dim radius (outer circle) -->
    <g v-for="light in displayLights" :key="`light-${light.id}`">
      <!-- Dim light circle (outer) -->
      <circle
        class="light-dim"
        :class="{ 'light-inactive': !light.is_active }"
        :cx="getLightX(light)"
        :cy="getLightY(light)"
        :r="getDimRadiusPx(light)"
        :fill="`url(#lightGradient-${light.id})`"
      />

      <!-- Bright light circle border (inner) - for visual reference -->
      <circle
        v-if="showBrightBorder"
        class="light-bright-border"
        :class="{ 'light-inactive': !light.is_active }"
        :cx="getLightX(light)"
        :cy="getLightY(light)"
        :r="getBrightRadiusPx(light)"
        fill="none"
        :stroke="getLightColor(light)"
        stroke-width="1"
        stroke-dasharray="4,4"
        stroke-opacity="0.5"
      />

      <!-- Light source center indicator (small dot) -->
      <circle
        v-if="showCenterDot"
        class="light-center"
        :class="{ 'light-inactive': !light.is_active }"
        :cx="getLightX(light)"
        :cy="getLightY(light)"
        r="6"
        :fill="getLightColor(light)"
        stroke="white"
        stroke-width="2"
        :title="`${light.name} - Right-click to toggle`"
        @contextmenu.prevent.stop="$emit('light-context', $event, light)"
      />

      <!-- Light name label (DM only) -->
      <text
        v-if="showLabels"
        class="light-label"
        :class="{ 'light-inactive': !light.is_active }"
        :x="getLightX(light)"
        :y="getLightY(light) + 12"
        text-anchor="middle"
        fill="white"
        font-size="10"
        font-weight="500"
      >
        {{ light.name }}
      </text>
    </g>
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { LightSourceSummary } from '@/composables/useLightSources'

interface Props {
  lights: LightSourceSummary[]
  tokens?: { id: string; x: number; y: number }[]
  gridSizePx: number
  mapWidth: number
  mapHeight: number
  showInactive?: boolean
  showBrightBorder?: boolean
  showCenterDot?: boolean
  showLabels?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  tokens: () => [],
  showInactive: true,
  showBrightBorder: true,
  showCenterDot: true,
  showLabels: false
})

defineEmits<{
  'light-context': [event: MouseEvent, light: LightSourceSummary]
}>()

// Default light color (warm white)
const DEFAULT_LIGHT_COLOR = '#ffcc66'

// Filter lights based on showInactive prop
const displayLights = computed(() => {
  if (props.showInactive) {
    return props.lights
  }
  return props.lights.filter(l => l.is_active)
})

// Convert feet to pixels (1 grid square = 5 feet)
function feetToPixels(feet: number): number {
  return (feet / 5) * props.gridSizePx
}

// Get X position for a light (use token position if attached)
function getLightX(light: LightSourceSummary): number {
  if (light.token_id) {
    const token = props.tokens.find(t => t.id === light.token_id)
    if (token) {
      return token.x
    }
  }
  return light.x
}

// Get Y position for a light (use token position if attached)
function getLightY(light: LightSourceSummary): number {
  if (light.token_id) {
    const token = props.tokens.find(t => t.id === light.token_id)
    if (token) {
      return token.y
    }
  }
  return light.y
}

// Get bright radius in pixels
function getBrightRadiusPx(light: LightSourceSummary): number {
  return feetToPixels(light.bright_radius_ft)
}

// Get dim radius in pixels
function getDimRadiusPx(light: LightSourceSummary): number {
  return feetToPixels(light.dim_radius_ft)
}

// Get bright radius as percentage of dim radius (for gradient)
function getBrightRadiusPercent(light: LightSourceSummary): number {
  if (light.dim_radius_ft === 0) return 100
  return (light.bright_radius_ft / light.dim_radius_ft) * 100
}

// Get light color (with fallback)
function getLightColor(light: LightSourceSummary): string {
  return light.color || DEFAULT_LIGHT_COLOR
}
</script>

<style scoped>
.light-source-layer {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
  will-change: transform;
  backface-visibility: hidden;
  z-index: 5; /* Above map, below tokens */
}

.light-dim {
  mix-blend-mode: screen;
}

.light-inactive {
  opacity: 0.3;
}

.light-bright-border {
  pointer-events: none;
}

.light-center {
  cursor: pointer;
  pointer-events: auto;
}

.light-center:hover {
  r: 6;
}

.light-label {
  pointer-events: none;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
}
</style>
