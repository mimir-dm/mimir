<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, emit, type UnlistenFn } from '@tauri-apps/api/event'
import TokenRenderer from '@/components/tokens/TokenRenderer.vue'
import LightSourceRenderer from '@/components/lighting/LightSourceRenderer.vue'
import LightOverlay from '@/components/los/LightOverlay.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import type { Token } from '@/types/api'
import type { LightSourceSummary } from '@/composables/useLightSources'
import type { Light, Wall } from '@/composables/useVisibilityPolygon'
import { useVisionCalculation, type AmbientLight } from '@/composables/useVisionCalculation'

// Types for map display
interface MapState {
  mapId: number | null
  imageUrl: string | null
  gridType: 'square' | 'hex' | 'none'
  gridSizePx: number | null
  gridOffsetX: number
  gridOffsetY: number
  viewportX: number
  viewportY: number
  zoom: number
  isBlackout: boolean
  ambientLight: AmbientLight
  mapWidth: number
  mapHeight: number
}

// Reactive state
const mapState = ref<MapState>({
  mapId: null,
  imageUrl: null,
  gridType: 'none',
  gridSizePx: null,
  gridOffsetX: 0,
  gridOffsetY: 0,
  viewportX: 0,
  viewportY: 0,
  zoom: 1,
  isBlackout: false,
  ambientLight: 'bright',
  mapWidth: 0,
  mapHeight: 0
})

const isLoading = ref(false)
const errorMessage = ref<string | null>(null)
const imageRef = ref<HTMLImageElement | null>(null)
const tokens = ref<Token[]>([])
const deadTokenIds = ref<number[]>([])
const tokenImages = ref<Map<number, string>>(new Map())

// Track actual display scale and image dimensions
const displayScale = ref(1)
const imageNaturalWidth = ref(0)
const imageNaturalHeight = ref(0)

// Fog of war state (vision-based)
interface VisionCircle {
  tokenId: number
  x: number
  y: number
  radiusPx: number
}

const revealMap = ref(false) // Master toggle: false = blackout, true = show something
const visionCircles = ref<VisionCircle[]>([])

// UVTT LOS state
const useLosBlocking = ref(false)
const visibilityPaths = ref<{ tokenId: number; path: string; polygon?: { x: number; y: number }[] }[]>([])
const blockingWalls = ref<Wall[]>([])
const uvttLights = ref<Light[]>([])
const tokenOnlyLos = ref(false) // Token LOS mode: map visible but tokens hidden outside LOS

// Point-in-polygon test using ray casting algorithm
function isPointInPolygon(point: { x: number; y: number }, polygon: { x: number; y: number }[]): boolean {
  if (polygon.length < 3) return false

  let inside = false
  const n = polygon.length

  for (let i = 0, j = n - 1; i < n; j = i++) {
    const xi = polygon[i].x, yi = polygon[i].y
    const xj = polygon[j].x, yj = polygon[j].y

    if (((yi > point.y) !== (yj > point.y)) &&
        (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi)) {
      inside = !inside
    }
  }

  return inside
}

// Tokens visible based on reveal mode and LOS settings
// revealMap ON = everything visible (show all tokens)
// revealMap OFF = hiding active (fog or token LOS mode)
const visibleTokens = computed(() => {
  // If revealMap is ON, show all tokens (everything revealed)
  if (revealMap.value) {
    console.log('visibleTokens: revealMap ON, showing all', tokens.value.length, 'tokens')
    return tokens.value
  }

  // revealMap OFF = hiding active
  // If Fog mode (tokenOnlyLos OFF), show all tokens (fog overlay hides the map, not tokens)
  if (!tokenOnlyLos.value) {
    console.log('visibleTokens: Fog mode, showing all', tokens.value.length, 'tokens')
    return tokens.value
  }

  // Token LOS mode: filter tokens by visibility polygons
  // If no visibility polygons, fall back to showing all tokens
  if (visibilityPaths.value.length === 0) {
    console.log('visibleTokens: Token LOS mode but no visibility paths, showing all', tokens.value.length, 'tokens')
    return tokens.value
  }

  // Filter tokens: show if they're within any visibility polygon
  const filtered = tokens.value.filter(token => {
    // Get the token IDs that have visibility (player tokens)
    const playerTokenIds = visibilityPaths.value.map(v => v.tokenId)

    // Always show player tokens (the ones with visibility polygons)
    if (playerTokenIds.includes(token.id)) {
      return true
    }

    // For other tokens (enemies), check if they're in any visibility polygon
    const tokenPoint = { x: token.x, y: token.y }

    return visibilityPaths.value.some(vis => {
      if (!vis.polygon || vis.polygon.length < 3) return false
      return isPointInPolygon(tokenPoint, vis.polygon)
    })
  })

  console.log('visibleTokens: Token LOS mode, filtered', tokens.value.length, '->', filtered.length, 'tokens')
  return filtered
})

// Light source state (database-stored lights)
const lightSources = ref<LightSourceSummary[]>([])

// Vision calculation
const ambientLightRef = computed(() => mapState.value.ambientLight)
const gridSizePxRef = computed(() => mapState.value.gridSizePx || 70)
const mapWidthRef = computed(() => mapState.value.mapWidth || imageRef.value?.naturalWidth || 0)
const mapHeightRef = computed(() => mapState.value.mapHeight || imageRef.value?.naturalHeight || 0)

const {
  visibilityCircles,
  needsVisionOverlay,
  lightZones
} = useVisionCalculation({
  tokens,
  lightSources,
  ambientLight: ambientLightRef,
  gridSizePx: gridSizePxRef,
  mapWidth: mapWidthRef,
  mapHeight: mapHeightRef
})

// Combined transform: mirror DM's viewport and fill player screen
const combinedTransform = computed(() => {
  const baseScale = displayScale.value
  const dmZoom = mapState.value.zoom
  const dmPanX = mapState.value.viewportX
  const dmPanY = mapState.value.viewportY

  // Scale to fill player screen: combine fit-to-screen scale with DM's zoom
  const finalScale = baseScale * dmZoom

  // Scale pan values to match the final scale
  const scaledPanX = dmPanX * baseScale
  const scaledPanY = dmPanY * baseScale

  return {
    transform: `translate(${scaledPanX}px, ${scaledPanY}px) scale(${finalScale})`,
    transformOrigin: 'center center'
  }
})

// Grid overlay types
interface SquareGridPattern {
  type: 'square'
  patternSize: number
  offsetX: number
  offsetY: number
}

interface HexGridPattern {
  type: 'hex'
  width: number
  height: number
  offsetX: number
  offsetY: number
}

type GridPattern = SquareGridPattern | HexGridPattern | null

// Helper to get hex points for SVG polygon
function getHexPoints(size: number): string {
  const w = size
  const h = size * Math.sqrt(3) / 2
  const points = [
    [w * 0.5, 0],
    [w, h * 0.5],
    [w, h * 1.5],
    [w * 0.5, h * 2],
    [0, h * 1.5],
    [0, h * 0.5]
  ]
  return points.map(p => p.join(',')).join(' ')
}

// Grid overlay SVG pattern
const gridPattern = computed<GridPattern>(() => {
  if (mapState.value.gridType === 'none' || !mapState.value.gridSizePx) {
    return null
  }

  const size = mapState.value.gridSizePx
  const offsetX = mapState.value.gridOffsetX
  const offsetY = mapState.value.gridOffsetY

  if (mapState.value.gridType === 'square') {
    return {
      type: 'square' as const,
      patternSize: size,
      offsetX,
      offsetY
    }
  } else if (mapState.value.gridType === 'hex') {
    // Hex grid calculations (pointy-top hexes)
    const hexWidth = size
    const hexHeight = size * Math.sqrt(3) / 2
    return {
      type: 'hex' as const,
      width: hexWidth,
      height: hexHeight,
      offsetX,
      offsetY
    }
  }

  return null
})

// Type-narrowed computed properties for template use
const isSquareGrid = computed(() => gridPattern.value?.type === 'square')
const isHexGrid = computed(() => gridPattern.value?.type === 'hex')
const squarePattern = computed(() => gridPattern.value?.type === 'square' ? gridPattern.value : null)
const hexPattern = computed(() => gridPattern.value?.type === 'hex' ? gridPattern.value : null)

// Event listeners for IPC from main window
let unlistenMapUpdate: UnlistenFn | null = null
let unlistenViewportUpdate: UnlistenFn | null = null
let unlistenBlackout: UnlistenFn | null = null
let unlistenTokensUpdate: UnlistenFn | null = null
let unlistenFogUpdate: UnlistenFn | null = null
let unlistenLightSourcesUpdate: UnlistenFn | null = null

onMounted(async () => {
  console.log('PlayerDisplayWindow: Setting up event listeners')

  // Listen for map updates from main window
  unlistenMapUpdate = await listen<{
    mapId: number
    gridType: string
    gridSizePx: number | null
    gridOffsetX: number
    gridOffsetY: number
    ambientLight?: string
    mapWidth?: number
    mapHeight?: number
  }>('player-display:map-update', async (event) => {
    console.log('PlayerDisplayWindow: Received map-update event:', event.payload)
    const data = event.payload
    mapState.value.mapId = data.mapId
    mapState.value.gridType = data.gridType as 'square' | 'hex' | 'none'
    mapState.value.gridSizePx = data.gridSizePx
    mapState.value.gridOffsetX = data.gridOffsetX
    mapState.value.gridOffsetY = data.gridOffsetY
    // Handle ambient light if provided
    if (data.ambientLight) {
      mapState.value.ambientLight = data.ambientLight as AmbientLight
    }
    // Handle map dimensions if provided
    if (data.mapWidth) {
      mapState.value.mapWidth = data.mapWidth
    }
    if (data.mapHeight) {
      mapState.value.mapHeight = data.mapHeight
    }

    // Load the map image
    await loadMapImage(data.mapId)

    // Request current state from DM window now that we're ready
    console.log('PlayerDisplayWindow: Requesting state for map', data.mapId)
    await emit('player-display:request-state', { mapId: data.mapId })
  })

  // Listen for viewport updates (pan/zoom)
  unlistenViewportUpdate = await listen<{
    x: number
    y: number
    zoom: number
  }>('player-display:viewport-update', (event) => {
    mapState.value.viewportX = event.payload.x
    mapState.value.viewportY = event.payload.y
    mapState.value.zoom = event.payload.zoom
  })

  // Listen for blackout toggle
  unlistenBlackout = await listen<{ isBlackout: boolean }>('player-display:blackout', (event) => {
    mapState.value.isBlackout = event.payload.isBlackout
  })

  // Listen for token updates
  unlistenTokensUpdate = await listen<{
    mapId: number
    tokens: Token[]
    deadTokenIds?: number[]
  }>('player-display:tokens-update', async (event) => {
    console.log('PlayerDisplayWindow: Received tokens-update event:', event.payload.tokens.length, 'tokens')
    // Accept tokens if they're for the current map OR if we don't have a map yet (initial load)
    if (mapState.value.mapId === null || event.payload.mapId === mapState.value.mapId) {
      tokens.value = event.payload.tokens
      deadTokenIds.value = event.payload.deadTokenIds || []

      // Load token images for tokens that have image_path
      const tokensWithImages = event.payload.tokens.filter(t => t.image_path)
      for (const token of tokensWithImages) {
        if (!tokenImages.value.has(token.id)) {
          try {
            const response = await invoke<{ success: boolean; data?: string }>('serve_token_image', { tokenId: token.id })
            if (response.success && response.data) {
              tokenImages.value.set(token.id, response.data)
            }
          } catch (e) {
            console.error(`Failed to load token image for ${token.id}:`, e)
          }
        }
      }
    }
  })

  // Listen for fog of war updates (vision-based, with optional LOS data)
  unlistenFogUpdate = await listen<{
    mapId: number
    revealMap: boolean
    tokenOnlyLos: boolean
    visionCircles: VisionCircle[]
    useLosBlocking?: boolean
    visibilityPaths?: { tokenId: number; path: string; polygon?: { x: number; y: number }[] }[]
    blockingWalls?: Wall[]
    uvttLights?: Light[]
    ambientLight?: 'bright' | 'dim' | 'darkness'
  }>('player-display:fog-update', (event) => {
    const payload = event.payload
    console.log('PlayerDisplayWindow: Received fog-update event:',
      'revealMap:', payload.revealMap,
      'tokenOnlyLos:', payload.tokenOnlyLos,
      'circles:', payload.visionCircles?.length || 0,
      'los:', payload.useLosBlocking,
      'paths:', payload.visibilityPaths?.length || 0,
      'walls:', payload.blockingWalls?.length || 0,
      'lights:', payload.uvttLights?.length || 0,
      'ambient:', payload.ambientLight
    )
    // Accept if it's for the current map OR if we don't have a map yet (initial load)
    if (mapState.value.mapId === null || payload.mapId === mapState.value.mapId) {
      revealMap.value = payload.revealMap ?? false
      tokenOnlyLos.value = payload.tokenOnlyLos ?? false
      visionCircles.value = payload.visionCircles || []
      // UVTT LOS data
      useLosBlocking.value = payload.useLosBlocking || false
      visibilityPaths.value = payload.visibilityPaths || []
      blockingWalls.value = payload.blockingWalls || []
      uvttLights.value = payload.uvttLights || []
      // Ambient light
      if (payload.ambientLight) {
        mapState.value.ambientLight = payload.ambientLight
      }
      console.log('PlayerDisplayWindow: State updated - revealMap:', revealMap.value, 'tokenOnlyLos:', tokenOnlyLos.value, 'visibilityPaths:', visibilityPaths.value.length)
    } else {
      console.log('PlayerDisplayWindow: Ignoring fog update for different map', payload.mapId, 'vs', mapState.value.mapId)
    }
  })

  // Listen for light source updates
  unlistenLightSourcesUpdate = await listen<{
    mapId: number
    lightSources: LightSourceSummary[]
  }>('player-display:light-sources-update', (event) => {
    console.log('PlayerDisplayWindow: Received light-sources-update event:', event.payload.lightSources.length, 'lights')
    // Accept lights if they're for the current map OR if we don't have a map yet (initial load)
    if (mapState.value.mapId === null || event.payload.mapId === mapState.value.mapId) {
      lightSources.value = event.payload.lightSources
    }
  })

  // Handle keyboard shortcuts
  window.addEventListener('keydown', handleKeydown)

  // Handle window resize to recalculate scale
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  unlistenMapUpdate?.()
  unlistenViewportUpdate?.()
  unlistenBlackout?.()
  unlistenTokensUpdate?.()
  unlistenFogUpdate?.()
  unlistenLightSourcesUpdate?.()
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('resize', handleResize)
})

// Load map image from backend
async function loadMapImage(mapId: number) {
  isLoading.value = true
  errorMessage.value = null
  tokens.value = [] // Clear tokens when loading a new map
  lightSources.value = [] // Clear light sources when loading a new map
  // Clear UVTT LOS data
  useLosBlocking.value = false
  visibilityPaths.value = []
  blockingWalls.value = []
  uvttLights.value = []

  try {
    const response = await invoke<{ success: boolean; data?: string; error?: string }>(
      'serve_map_image',
      { id: mapId }
    )

    if (response.success && response.data) {
      mapState.value.imageUrl = response.data
    } else {
      errorMessage.value = response.error || 'Failed to load map image'
    }
  } catch (err) {
    errorMessage.value = `Error loading map: ${err}`
  } finally {
    isLoading.value = false
  }
}

// Calculate the scale needed to fit the image in the viewport
function updateDisplayScale() {
  if (!imageRef.value) return

  const naturalWidth = imageRef.value.naturalWidth
  const naturalHeight = imageRef.value.naturalHeight

  if (naturalWidth === 0 || naturalHeight === 0) return

  // Store for other components
  imageNaturalWidth.value = naturalWidth
  imageNaturalHeight.value = naturalHeight

  // Get viewport dimensions
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight

  // Calculate scale to fill (same as object-fit: cover logic)
  const scaleX = viewportWidth / naturalWidth
  const scaleY = viewportHeight / naturalHeight

  // Use the larger scale to fill the viewport (no black bars)
  displayScale.value = Math.max(scaleX, scaleY)
  console.log('PlayerDisplayWindow: Updated display scale to', displayScale.value,
    `(natural: ${naturalWidth}x${naturalHeight}, viewport: ${viewportWidth}x${viewportHeight})`)
}

// Keyboard shortcuts
function handleKeydown(event: KeyboardEvent) {
  // F11 to toggle fullscreen
  if (event.key === 'F11') {
    event.preventDefault()
    invoke('toggle_player_display_fullscreen')
  }
  // Escape to exit blackout or close window
  if (event.key === 'Escape') {
    if (mapState.value.isBlackout) {
      // Just visual feedback, main window controls blackout
    }
  }
}

// Handle image load to calculate scale
function handleImageLoad() {
  console.log('PlayerDisplayWindow: Image loaded')
  updateDisplayScale()
}

// Handle window resize to recalculate scale
function handleResize() {
  updateDisplayScale()
}
</script>

<template>
  <div class="player-display" :class="{ blackout: mapState.isBlackout }">
    <!-- Blackout overlay (manual pause) -->
    <div v-if="mapState.isBlackout" class="blackout-overlay">
      <div class="blackout-text">Display Paused</div>
    </div>

    <!-- Map display area -->
    <div v-else class="map-viewport">
      <!-- Loading state -->
      <div v-if="isLoading" class="loading-state">
        <div class="loading-spinner"></div>
        <div class="loading-text">Loading map...</div>
      </div>

      <!-- Error state -->
      <div v-else-if="errorMessage" class="error-state">
        <div class="error-icon">!</div>
        <div class="error-text">{{ errorMessage }}</div>
      </div>

      <!-- No map selected -->
      <EmptyState
        v-else-if="!mapState.imageUrl"
        variant="campaigns"
        title="Waiting for map selection..."
        description="Select a map from the DM window to display"
      />

      <!-- Map with grid overlay - synced with DM viewport -->
      <div
        v-else
        class="map-container"
        :style="combinedTransform"
      >
        <img
          ref="imageRef"
          :src="mapState.imageUrl"
          alt="Battle Map"
          class="map-image"
          draggable="false"
          @load="handleImageLoad"
        />

        <!-- Grid overlay -->
        <svg
          v-if="gridPattern"
          class="grid-overlay"
          :style="{
            width: imageNaturalWidth + 'px',
            height: imageNaturalHeight + 'px'
          }"
        >
          <defs>
            <!-- Square grid pattern -->
            <pattern
              v-if="squarePattern"
              id="grid-pattern"
              :width="squarePattern.patternSize"
              :height="squarePattern.patternSize"
              patternUnits="userSpaceOnUse"
              :patternTransform="`translate(${squarePattern.offsetX}, ${squarePattern.offsetY})`"
            >
              <path
                :d="`M ${squarePattern.patternSize} 0 L 0 0 0 ${squarePattern.patternSize}`"
                fill="none"
                stroke="rgba(255, 255, 255, 0.3)"
                stroke-width="1"
              />
            </pattern>

            <!-- Hex grid pattern (pointy-top) -->
            <pattern
              v-if="hexPattern"
              id="grid-pattern"
              :width="hexPattern.width * 1.5"
              :height="hexPattern.height * 2"
              patternUnits="userSpaceOnUse"
              :patternTransform="`translate(${hexPattern.offsetX}, ${hexPattern.offsetY})`"
            >
              <!-- Hex paths would go here -->
              <polygon
                :points="getHexPoints(hexPattern.width)"
                fill="none"
                stroke="rgba(255, 255, 255, 0.3)"
                stroke-width="1"
              />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid-pattern)" />
        </svg>

        <!-- UVTT Map Lights (embedded in map file, with shadow casting) -->
        <LightOverlay
          v-if="uvttLights.length > 0 && imageNaturalWidth > 0"
          :lights="uvttLights"
          :walls="blockingWalls"
          :map-width="imageNaturalWidth"
          :map-height="imageNaturalHeight"
          :show-debug="false"
          blend-mode="soft-light"
        />

        <!-- Light Source Layer (only active lights) -->
        <LightSourceRenderer
          v-if="lightSources.length > 0 && mapState.gridSizePx && imageNaturalWidth > 0"
          :lights="lightSources"
          :tokens="tokens"
          :grid-size-px="mapState.gridSizePx"
          :map-width="imageNaturalWidth"
          :map-height="imageNaturalHeight"
          :show-inactive="false"
          :show-bright-border="false"
          :show-center-dot="false"
          :show-labels="false"
        />

        <!-- Token Layer (filtered by Token LOS mode if enabled) -->
        <TokenRenderer
          v-if="visibleTokens.length > 0 && mapState.gridSizePx"
          :key="`tokens-${mapState.mapId}-${mapState.gridSizePx}-${tokenOnlyLos}`"
          :tokens="visibleTokens"
          :grid-size-px="mapState.gridSizePx"
          :base-scale="1"
          :show-hidden="false"
          :interactive="false"
          :dead-token-ids="deadTokenIds"
          :token-images="tokenImages"
        />

        <!-- Fog of War Overlay (Fog mode: revealMap OFF + tokenOnlyLos OFF) -->
        <svg
          v-if="!revealMap && !tokenOnlyLos && imageNaturalWidth > 0"
          class="fog-overlay"
          :style="{
            width: imageNaturalWidth + 'px',
            height: imageNaturalHeight + 'px'
          }"
          :viewBox="`0 0 ${imageNaturalWidth} ${imageNaturalHeight}`"
        >
          <defs>
            <!-- Blur filter for soft vision edges (only used for circle fallback) -->
            <filter id="playerVisionBlur" x="-50%" y="-50%" width="200%" height="200%">
              <feGaussianBlur in="SourceGraphic" stdDeviation="12" />
            </filter>
            <mask id="playerFogMask">
              <!-- White = visible (fog), Black = hidden (revealed) -->
              <rect width="100%" height="100%" fill="white" />
              <!-- Use visibility polygons when LOS blocking is enabled (no blur for sharp wall edges) -->
              <g v-if="useLosBlocking && visibilityPaths.length > 0">
                <path
                  v-for="vis in visibilityPaths"
                  :key="'vis-' + vis.tokenId"
                  :d="vis.path"
                  fill="black"
                />
              </g>
              <!-- Fall back to circles when no LOS data (with blur for soft edges) -->
              <g v-else filter="url(#playerVisionBlur)">
                <circle
                  v-for="circle in visionCircles"
                  :key="'vision-' + circle.tokenId"
                  :cx="circle.x"
                  :cy="circle.y"
                  :r="circle.radiusPx"
                  fill="black"
                />
              </g>
            </mask>
          </defs>
          <!-- Fully opaque fog for player view -->
          <rect
            width="100%"
            height="100%"
            fill="#000000"
            mask="url(#playerFogMask)"
          />
        </svg>

        <!-- Vision/Lighting Overlay (darkness with vision cutouts) -->
        <svg
          v-if="needsVisionOverlay && imageNaturalWidth > 0"
          class="vision-overlay"
          :style="{
            width: imageNaturalWidth + 'px',
            height: imageNaturalHeight + 'px'
          }"
          :viewBox="`0 0 ${imageNaturalWidth} ${imageNaturalHeight}`"
        >
          <defs>
            <!-- Mask for darkness (white = show darkness, black = hide darkness) -->
            <mask id="darknessMask">
              <!-- White background = darkness everywhere by default -->
              <rect width="100%" height="100%" fill="white" />
              <!-- Cut out (hide darkness) in dim vision areas with gray -->
              <circle
                v-for="zone in lightZones"
                :key="`dark-light-dim-${zone.lightSourceId}`"
                :cx="zone.x"
                :cy="zone.y"
                :r="zone.dimRadiusPx"
                fill="#666"
              />
              <circle
                v-for="circle in visibilityCircles"
                :key="`dark-vision-dim-${circle.tokenId}`"
                :cx="circle.x"
                :cy="circle.y"
                :r="circle.dimRadiusPx"
                fill="#666"
              />
              <!-- Fully cut out (no darkness) in bright vision areas -->
              <circle
                v-for="zone in lightZones"
                :key="`dark-light-bright-${zone.lightSourceId}`"
                :cx="zone.x"
                :cy="zone.y"
                :r="zone.brightRadiusPx"
                fill="black"
              />
              <circle
                v-for="circle in visibilityCircles"
                :key="`dark-vision-bright-${circle.tokenId}`"
                :cx="circle.x"
                :cy="circle.y"
                :r="circle.brightRadiusPx"
                fill="black"
              />
            </mask>

            <!-- Radial gradients for soft vision edges -->
            <radialGradient
              v-for="circle in visibilityCircles"
              :key="`gradient-${circle.tokenId}`"
              :id="`visionGradient-${circle.tokenId}`"
            >
              <stop offset="70%" stop-color="black" />
              <stop offset="100%" stop-color="white" />
            </radialGradient>
          </defs>

          <!-- Main darkness overlay -->
          <rect
            width="100%"
            height="100%"
            :fill="mapState.ambientLight === 'darkness' ? 'rgba(0, 0, 0, 0.92)' : 'rgba(0, 0, 0, 0.75)'"
            mask="url(#darknessMask)"
          />
        </svg>

      </div>
    </div>

    <!-- Minimal status bar (only visible on hover in fullscreen) -->
    <!-- Minimal status bar - hidden by default, shows on hover -->
    <div class="status-bar">
      <span v-if="mapState.mapId">Map loaded</span>
    </div>
  </div>
</template>

<style scoped>
.player-display {
  position: fixed;
  inset: 0;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  cursor: none;
}

.player-display:not(.blackout):hover {
  cursor: default;
}

/* Blackout mode */
.blackout-overlay {
  position: absolute;
  inset: 0;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.blackout-text {
  color: #333;
  font-size: 1.5rem;
  font-family: system-ui, sans-serif;
}

/* Map viewport */
.map-viewport {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.map-container {
  position: relative;
  /* Container is sized to natural image dimensions and scaled via transform */
}

.map-image {
  display: block;
  max-width: none;
  user-select: none;
  -webkit-user-drag: none;
  /* Image renders at natural size, container handles scaling via transform */
}

.grid-overlay {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
}

.fog-overlay {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
  z-index: 10;
}

.vision-overlay {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
  z-index: 9; /* Below fog overlay */
}

/* Loading state */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  color: #666;
}

.loading-spinner {
  width: 48px;
  height: 48px;
  border: 3px solid #333;
  border-top-color: #666;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.loading-text {
  font-size: 1rem;
  font-family: system-ui, sans-serif;
}

/* Error state */
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  color: #cc4444;
}

.error-icon {
  width: 48px;
  height: 48px;
  border: 3px solid currentColor;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  font-weight: bold;
}

.error-text {
  font-size: 1rem;
  font-family: system-ui, sans-serif;
  max-width: 400px;
  text-align: center;
}

/* Status bar */
.status-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.5rem 1rem;
  background: rgba(0, 0, 0, 0.8);
  color: #666;
  font-size: 0.75rem;
  font-family: system-ui, sans-serif;
  display: flex;
  gap: 1rem;
  opacity: 0;
  transition: opacity 0.3s;
}

.player-display:hover .status-bar {
  opacity: 1;
}
</style>
