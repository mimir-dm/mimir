<template>
  <AppModal
    :visible="visible"
    :title="`Token Setup - ${map.name}`"
    size="xl"
    no-padding
    @close="handleClose"
  >
    <div class="token-setup-body">
          <!-- Token Palette (left side, scrollable) -->
          <div class="palette-wrapper">
            <TokenPalette
              ref="paletteRef"
              :module-id="map.module_id"
              @token-config-change="handleTokenConfigChange"
              @light-config-change="handleLightConfigChange"
            />
          </div>

          <!-- Map Canvas (center) -->
          <div class="map-canvas-container">
            <div class="canvas-controls">
              <div class="zoom-controls">
                <span class="zoom-label">Zoom:</span>
                <button class="ctrl-btn" @click="zoomOut" :disabled="zoom <= 0.25">−</button>
                <span class="zoom-level">{{ Math.round(zoom * 100) }}%</span>
                <button class="ctrl-btn" @click="zoomIn" :disabled="zoom >= 4">+</button>
                <button class="ctrl-btn" @click="resetView">Fit</button>
                <button class="ctrl-btn" @click="showGridConfigModal = true">Grid</button>
              </div>
              <div class="token-count">
                {{ tokens.length }} tokens
              </div>
            </div>

            <div
              class="map-viewport"
              ref="viewportRef"
              :class="{ 'placement-mode': !!pendingTokenConfig || !!pendingLightType }"
              @wheel.prevent="onWheel"
              @mousedown="onMouseDown"
              @mousemove="onMouseMove"
              @mouseup="onMouseUp"
              @mouseleave="onMouseUp"
              @click="handleCanvasClick"
            >
              <!-- Map Image -->
              <img
                v-if="mapImageUrl"
                :src="mapImageUrl"
                :alt="map.name"
                class="map-image"
                :style="imageStyle"
                @load="onImageLoad"
                draggable="false"
              />
              <div v-else class="loading-map">Loading map...</div>

              <!-- Grid Overlay -->
              <svg
                v-if="mapImageUrl && imageLoaded && uvttGridSize > 0"
                class="grid-overlay"
                :style="gridOverlayStyle"
              >
                <defs>
                  <pattern
                    id="tokenGridPattern"
                    :width="displayGridSize"
                    :height="displayGridSize"
                    patternUnits="userSpaceOnUse"
                    :x="displayOffsetX"
                    :y="displayOffsetY"
                  >
                    <rect
                      :width="displayGridSize"
                      :height="displayGridSize"
                      fill="none"
                      stroke="rgba(255, 255, 255, 0.2)"
                      stroke-width="1"
                    />
                  </pattern>
                </defs>
                <rect width="100%" height="100%" fill="url(#tokenGridPattern)" />
              </svg>

              <!-- Token Layer -->
              <div
                v-if="mapImageUrl && imageLoaded"
                class="token-layer"
                :style="tokenLayerStyle"
              >
                <div
                  v-for="token in tokens"
                  :key="token.id"
                  class="token"
                  :class="{
                    'token-hidden': !token.visible_to_players,
                    'token-selected': selectedTokenId === token.id,
                    'token-dragging': draggingToken?.id === token.id
                  }"
                  :style="getTokenStyle(token)"
                  @mousedown.stop="onTokenMouseDown($event, token)"
                  @click.stop="selectToken(token)"
                  @contextmenu.prevent="showTokenContextMenu($event, token)"
                >
                  <span class="token-label">{{ getTokenInitial(token) }}</span>
                  <span
                    v-if="!token.visible_to_players"
                    class="visibility-indicator"
                    title="Hidden from players"
                  >👁️‍🗨️</span>
                </div>

                <!-- Light Sources -->
                <div
                  v-for="light in lightSources"
                  :key="'light-' + light.id"
                  class="light-dot"
                  :class="{
                    'light-inactive': !light.is_active,
                    'light-dragging': draggingLight?.id === light.id
                  }"
                  :style="getLightStyle(light)"
                  :title="`${light.name} (${light.bright_radius_ft}/${light.dim_radius_ft}ft) - Drag to move, Right-click to toggle`"
                  @mousedown.stop="onLightMouseDown($event, light)"
                  @contextmenu.prevent="toggleLight(light)"
                />

                <!-- Map Traps -->
                <div
                  v-for="trap in mapTraps"
                  :key="'trap-' + trap.id"
                  class="trap-marker"
                  :class="{
                    'trap-triggered': trap.triggered === 1,
                    'trap-visible': trap.visible === 1,
                    'trap-dragging': draggingTrap?.id === trap.id
                  }"
                  :style="getTrapStyle(trap)"
                  :title="`${trap.name}${trap.dc ? ' (DC ' + trap.dc + ')' : ''} - Drag to move`"
                  @mousedown.stop="onTrapMouseDown($event, trap)"
                  @contextmenu.prevent="showTrapContextMenu($event, trap)"
                >
                  <span class="trap-icon">⚠️</span>
                </div>

                <!-- Map POIs (Points of Interest) -->
                <div
                  v-for="poi in mapPois"
                  :key="'poi-' + poi.id"
                  class="poi-marker"
                  :class="{
                    'poi-visible': poi.visible === 1,
                    'poi-dragging': draggingPoi?.id === poi.id
                  }"
                  :style="getPoiStyle(poi)"
                  :title="`${poi.name} - Drag to move`"
                  @mousedown.stop="onPoiMouseDown($event, poi)"
                  @contextmenu.prevent="showPoiContextMenu($event, poi)"
                >
                  <span class="poi-icon">{{ getPoiIcon(poi.icon) }}</span>
                </div>
              </div>

              <!-- Placement Preview -->
              <div
                v-if="pendingTokenConfig && mousePosition"
                class="placement-preview"
                :style="getPlacementPreviewStyle()"
              />
            </div>
          </div>

          <!-- Token List (right side) -->
          <div class="token-list-panel">
            <h4>Placed Tokens</h4>
            <div v-if="tokens.length === 0" class="empty-tokens">
              No tokens placed yet.
            </div>
            <div v-else class="token-list">
              <div
                v-for="token in tokens"
                :key="token.id"
                class="token-list-item"
                :class="{ selected: selectedTokenId === token.id }"
                @click="selectToken(token)"
              >
                <div
                  class="token-list-color"
                  :style="{ background: getTokenColor(token) }"
                />
                <div class="token-list-info">
                  <span class="token-list-name">{{ token.name }}</span>
                  <span class="token-list-type">{{ token.token_type }} · {{ token.size }}</span>
                </div>
                <button
                  class="token-list-visibility"
                  :class="{ hidden: !token.visible_to_players }"
                  @click.stop="handleToggleVisibility(token)"
                  :title="token.visible_to_players ? 'Visible to players' : 'Hidden from players'"
                >
                  {{ token.visible_to_players ? '👁️' : '👁️‍🗨️' }}
                </button>
                <button
                  class="token-list-delete"
                  @click.stop="confirmDeleteToken(token)"
                  title="Delete token"
                >
                  ×
                </button>
              </div>
            </div>

            <!-- Light Sources Section -->
            <h4 v-if="lightSources.length > 0" class="section-header">Light Sources</h4>
            <div v-if="lightSources.length > 0" class="token-list">
              <div
                v-for="light in lightSources"
                :key="'light-' + light.id"
                class="token-list-item light-item"
                :class="{ 'light-inactive': !light.is_active }"
              >
                <div
                  class="token-list-color light-color"
                  :style="{ background: light.color || '#ffcc00' }"
                />
                <div class="token-list-info">
                  <span class="token-list-name">{{ light.name }}</span>
                  <span class="token-list-type">{{ light.bright_radius_ft }}/{{ light.dim_radius_ft }}ft</span>
                </div>
                <button
                  class="light-toggle-btn"
                  :class="{ lit: light.is_active }"
                  @click.stop="toggleLight(light)"
                  :title="light.is_active ? 'Click to extinguish' : 'Click to ignite'"
                >
                  {{ light.is_active ? 'Lit' : 'Unlit' }}
                </button>
                <button
                  class="token-list-delete"
                  @click.stop="confirmDeleteLight(light)"
                  title="Delete light source"
                >
                  ×
                </button>
              </div>
            </div>

            <!-- Traps Section -->
            <h4 v-if="mapTraps.length > 0" class="section-header">Traps</h4>
            <div v-if="mapTraps.length > 0" class="token-list">
              <div
                v-for="trap in mapTraps"
                :key="'trap-list-' + trap.id"
                class="token-list-item trap-item"
                :class="{
                  'trap-triggered': trap.triggered === 1,
                  'trap-visible-marker': trap.visible === 1
                }"
              >
                <div class="token-list-color trap-color">
                  <span>⚠️</span>
                </div>
                <div class="token-list-info">
                  <span class="token-list-name">{{ trap.name }}</span>
                  <span class="token-list-type">
                    {{ trap.dc ? `DC ${trap.dc}` : '' }}
                    {{ trap.triggered === 1 ? '(Triggered)' : '' }}
                  </span>
                </div>
                <button
                  class="trap-toggle-btn"
                  :class="{ visible: trap.visible === 1 }"
                  @click.stop="handleToggleTrapVisibilityDirect(trap)"
                  :title="trap.visible === 1 ? 'Hide from players' : 'Show to players'"
                >
                  {{ trap.visible === 1 ? 'Vis' : 'Hid' }}
                </button>
                <button
                  class="token-list-delete"
                  @click.stop="confirmDeleteTrapDirect(trap)"
                  title="Delete trap"
                >
                  ×
                </button>
              </div>
            </div>

            <!-- POIs Section -->
            <h4 v-if="mapPois.length > 0" class="section-header">Points of Interest</h4>
            <div v-if="mapPois.length > 0" class="token-list">
              <div
                v-for="poi in mapPois"
                :key="'poi-list-' + poi.id"
                class="token-list-item poi-item"
                :class="{ 'poi-visible-marker': poi.visible === 1 }"
              >
                <div
                  class="token-list-color poi-color"
                  :style="{ background: poi.color || '#4488ff' }"
                >
                  <span>{{ getPoiIcon(poi.icon) }}</span>
                </div>
                <div class="token-list-info">
                  <span class="token-list-name">{{ poi.name }}</span>
                  <span class="token-list-type">{{ poi.icon }}</span>
                </div>
                <button
                  class="poi-toggle-btn"
                  :class="{ visible: poi.visible === 1 }"
                  @click.stop="handleTogglePoiVisibilityDirect(poi)"
                  :title="poi.visible === 1 ? 'Hide from players' : 'Show to players'"
                >
                  {{ poi.visible === 1 ? 'Vis' : 'Hid' }}
                </button>
                <button
                  class="token-list-delete"
                  @click.stop="confirmDeletePoiDirect(poi)"
                  title="Delete POI"
                >
                  ×
                </button>
              </div>
            </div>
          </div>
        </div>

      <!-- Token Context Menu -->
      <div
        v-if="contextMenu.visible"
        class="context-menu"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
        @click.stop
      >
        <button @click="handleEditToken">Edit</button>
        <button @click="handleToggleSelectedVisibility">
          {{ contextMenu.token?.visible_to_players ? 'Hide from Players' : 'Show to Players' }}
        </button>
        <button class="danger" @click="handleDeleteFromContext">Delete</button>
      </div>

      <!-- Trap Context Menu -->
      <div
        v-if="trapContextMenu.visible"
        class="context-menu"
        :style="{ left: trapContextMenu.x + 'px', top: trapContextMenu.y + 'px' }"
        @click.stop
      >
        <button @click="handleToggleTrapVisibility">
          {{ trapContextMenu.trap?.visible === 1 ? 'Hide from Players' : 'Show to Players' }}
        </button>
        <button v-if="trapContextMenu.trap?.triggered === 0" @click="handleTriggerTrap">
          Trigger Trap
        </button>
        <button v-if="trapContextMenu.trap?.triggered === 1" @click="handleResetTrap">
          Reset (Re-arm)
        </button>
        <button class="danger" @click="handleDeleteTrap">Delete</button>
      </div>

      <!-- POI Context Menu -->
      <div
        v-if="poiContextMenu.visible"
        class="context-menu"
        :style="{ left: poiContextMenu.x + 'px', top: poiContextMenu.y + 'px' }"
        @click.stop
      >
        <button @click="handleTogglePoiVisibility">
          {{ poiContextMenu.poi?.visible === 1 ? 'Hide from Players' : 'Show to Players' }}
        </button>
        <button class="danger" @click="handleDeletePoi">Delete</button>
      </div>

    <template #footer>
      <button class="btn btn-secondary" @click="handleClose">Close</button>
    </template>
  </AppModal>

  <!-- Grid Configuration Modal -->
  <MapGridConfigModal
    :visible="showGridConfigModal"
    :map="map"
    @close="showGridConfigModal = false"
    @saved="handleGridSaved"
  />
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import TokenPalette from './TokenPalette.vue'
import MapGridConfigModal from '@/features/campaigns/components/StageLanding/MapGridConfigModal.vue'
import type { Token, CreateTokenRequest, TokenSize, TokenConfigWithMonster } from '@/types/api'
import { TOKEN_SIZE_GRID_SQUARES, TOKEN_TYPE_COLORS } from '@/types/api'
import { useTokens } from '@/composables/useTokens'

interface Map {
  id: string
  name: string
  image_path: string
  width_px: number
  height_px: number
  grid_type: string
  grid_size_px: number | null
  grid_offset_x: number
  grid_offset_y: number
  campaign_id: string
  module_id?: string | null
  original_width_px: number | null
  original_height_px: number | null
}

const props = defineProps<{
  visible: boolean
  map: Map
}>()

const emit = defineEmits<{
  close: []
}>()

// Token management
const {
  tokens,
  loading,
  loadTokens,
  createToken,
  updateTokenPosition,
  toggleVisibility,
  deleteToken
} = useTokens(props.map.id)

// Refs
const paletteRef = ref<InstanceType<typeof TokenPalette> | null>(null)
const viewportRef = ref<HTMLElement | null>(null)

// Map image state
const mapImageUrl = ref<string | null>(null)
const imageLoaded = ref(false)
const uvttGridSize = ref<number>(70)  // Grid size from UVTT file (default 70px)

// View state
const zoom = ref(1)
const panX = ref(0)
const panY = ref(0)

// Drag state
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const dragStartPanX = ref(0)
const dragStartPanY = ref(0)

// Token placement state
const pendingTokenConfig = ref<TokenConfigWithMonster | null>(null)
const mousePosition = ref<{ x: number; y: number } | null>(null)
const selectedTokenId = ref<string | null>(null)

// Token dragging state
const draggingToken = ref<Token | null>(null)
const dragTokenOffsetX = ref(0)
const dragTokenOffsetY = ref(0)

// Light source dragging state
const draggingLight = ref<LightSource | null>(null)
const dragLightOffsetX = ref(0)
const dragLightOffsetY = ref(0)

// Light placement state
const pendingLightType = ref<'' | 'torch' | 'lantern' | 'candle'>('')

// Light sources on the map
interface LightSource {
  id: number
  name: string
  light_type: string
  x: number
  y: number
  bright_radius_ft: number
  dim_radius_ft: number
  color: string | null
  is_active: boolean
}
const lightSources = ref<LightSource[]>([])

// Map traps on the map
interface MapTrap {
  id: string
  map_id: string
  grid_x: number
  grid_y: number
  name: string
  description: string | null
  trigger_description: string | null
  effect_description: string | null
  dc: number | null
  triggered: number
  visible: number
}
const mapTraps = ref<MapTrap[]>([])

// Map POIs (Points of Interest)
interface MapPoi {
  id: string
  map_id: string
  grid_x: number
  grid_y: number
  name: string
  description: string | null
  icon: string
  color: string | null
  visible: number
}
const mapPois = ref<MapPoi[]>([])

// POI dragging state
const draggingPoi = ref<MapPoi | null>(null)
const dragPoiOffsetX = ref(0)
const dragPoiOffsetY = ref(0)

// Trap dragging state
const draggingTrap = ref<MapTrap | null>(null)
const dragTrapOffsetX = ref(0)
const dragTrapOffsetY = ref(0)

// Context menu
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  token: null as Token | null
})

// Trap context menu
const trapContextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  trap: null as MapTrap | null
})

// POI context menu
const poiContextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  poi: null as MapPoi | null
})

// Grid config modal
const showGridConfigModal = ref(false)

// Container dimensions
const containerWidth = 700
const containerHeight = 500

// Computed values
const baseScale = computed(() => {
  if (!props.map.width_px || !props.map.height_px) return 1
  const scaleX = containerWidth / props.map.width_px
  const scaleY = containerHeight / props.map.height_px
  return Math.min(scaleX, scaleY, 1)
})

const baseImageWidth = computed(() => props.map.width_px * baseScale.value)
const baseImageHeight = computed(() => props.map.height_px * baseScale.value)

const displayGridSize = computed(() => uvttGridSize.value * baseScale.value)
// UVTT maps don't use grid offsets - the grid starts at origin
const displayOffsetX = computed(() => 0)
const displayOffsetY = computed(() => 0)

const imageStyle = computed(() => ({
  width: baseImageWidth.value + 'px',
  height: baseImageHeight.value + 'px',
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${zoom.value})`,
  transformOrigin: '0 0'
}))

const gridOverlayStyle = computed(() => ({
  width: baseImageWidth.value + 'px',
  height: baseImageHeight.value + 'px',
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${zoom.value})`,
  transformOrigin: '0 0',
  position: 'absolute' as const,
  top: 0,
  left: 0,
  pointerEvents: 'none' as const
}))

const tokenLayerStyle = computed(() => ({
  width: baseImageWidth.value + 'px',
  height: baseImageHeight.value + 'px',
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${zoom.value})`,
  transformOrigin: '0 0',
  position: 'absolute' as const,
  top: 0,
  left: 0
}))

// Watch for visibility changes
watch(() => props.visible, async (visible) => {
  if (visible && props.map.id) {
    await Promise.all([loadMapImage(), loadUvttData(), loadTokens(), loadLightSources(), loadMapTraps(), loadMapPois()])
  }
}, { immediate: true })

watch(() => props.map.id, () => {
  if (props.visible) {
    loadTokens()
  }
})

// UVTT data interface
interface UvttData {
  resolution: {
    pixels_per_grid: number
    map_size: { x: number; y: number }
  }
}

// Functions
async function loadMapImage() {
  try {
    const response = await invoke<{ success: boolean; data?: string }>('serve_map_image', { id: props.map.id })
    if (response.success && response.data) {
      mapImageUrl.value = response.data
    }
  } catch (e) {
    console.error('Failed to load map image:', e)
  }
}

async function loadLightSources() {
  try {
    const response = await invoke<{ success: boolean; data?: LightSource[] }>('list_light_sources', {
      mapId: props.map.id
    })
    if (response.success && response.data) {
      lightSources.value = response.data
    }
  } catch (e) {
    console.error('Failed to load light sources:', e)
  }
}

async function loadMapTraps() {
  try {
    const response = await invoke<{ success: boolean; data?: MapTrap[] }>('list_map_traps', {
      mapId: props.map.id
    })
    if (response.success && response.data) {
      mapTraps.value = response.data
    }
  } catch (e) {
    console.error('Failed to load map traps:', e)
  }
}

async function loadMapPois() {
  try {
    const response = await invoke<{ success: boolean; data?: MapPoi[] }>('list_map_pois', {
      mapId: props.map.id
    })
    if (response.success && response.data) {
      mapPois.value = response.data
    }
  } catch (e) {
    console.error('Failed to load map POIs:', e)
  }
}

async function loadUvttData() {
  // Load UVTT to get grid size
  try {
    const response = await invoke<{ success: boolean; data?: UvttData }>('get_uvtt_map', {
      id: props.map.id
    })
    if (response.success && response.data) {
      uvttGridSize.value = response.data.resolution.pixels_per_grid
    }
  } catch (e) {
    console.error('Failed to load UVTT data:', e)
    // Fall back to database value or default
    uvttGridSize.value = props.map.grid_size_px || 70
  }
}

function onImageLoad() {
  imageLoaded.value = true
  resetView()
}

// View controls
function zoomIn() {
  zoom.value = Math.min(zoom.value * 1.5, 4)
}

function zoomOut() {
  zoom.value = Math.max(zoom.value / 1.5, 0.25)
}

function resetView() {
  zoom.value = 1
  panX.value = 0
  panY.value = 0
}

function onWheel(event: WheelEvent) {
  const delta = event.deltaY > 0 ? 0.9 : 1.1
  zoom.value = Math.max(0.25, Math.min(4, zoom.value * delta))
}

// Mouse handlers for panning
function onMouseDown(event: MouseEvent) {
  // Right-click is for context menu
  if (event.button === 2) return

  // Don't start panning if we're dragging a token, light, trap, or POI
  if (draggingToken.value || draggingLight.value || draggingTrap.value || draggingPoi.value) return

  // Only pan with middle mouse or when holding space
  if (event.button === 1 || (event.button === 0 && !pendingTokenConfig.value && !pendingLightType.value)) {
    isDragging.value = true
    dragStartX.value = event.clientX
    dragStartY.value = event.clientY
    dragStartPanX.value = panX.value
    dragStartPanY.value = panY.value
  }
}

// Start dragging a token
function onTokenMouseDown(event: MouseEvent, token: Token) {
  // Only left-click to drag
  if (event.button !== 0) return

  event.preventDefault()
  draggingToken.value = token
  selectedTokenId.value = token.id

  // Calculate offset from mouse to token center
  if (viewportRef.value) {
    const rect = viewportRef.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left
    const mouseY = event.clientY - rect.top

    // Token position in viewport coordinates
    const effectiveScale = baseScale.value * zoom.value
    const tokenViewportX = token.x * baseScale.value + panX.value
    const tokenViewportY = token.y * baseScale.value + panY.value

    // Store offset so token moves smoothly from where we clicked
    dragTokenOffsetX.value = mouseX - tokenViewportX * zoom.value
    dragTokenOffsetY.value = mouseY - tokenViewportY * zoom.value
  }
}

// Start dragging a light source
function onLightMouseDown(event: MouseEvent, light: LightSource) {
  // Only left-click to drag
  if (event.button !== 0) return

  event.preventDefault()
  event.stopPropagation()
  draggingLight.value = light

  // Calculate offset from mouse to light center
  if (viewportRef.value) {
    const rect = viewportRef.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left
    const mouseY = event.clientY - rect.top

    // Light position in viewport coordinates
    const lightViewportX = light.x * baseScale.value + panX.value
    const lightViewportY = light.y * baseScale.value + panY.value

    // Store offset so light moves smoothly from where we clicked
    dragLightOffsetX.value = mouseX - lightViewportX * zoom.value
    dragLightOffsetY.value = mouseY - lightViewportY * zoom.value
  }
}

// Start dragging a trap
function onTrapMouseDown(event: MouseEvent, trap: MapTrap) {
  // Only left-click to drag
  if (event.button !== 0) return

  event.preventDefault()
  event.stopPropagation()
  draggingTrap.value = trap

  // Calculate offset from mouse to trap center (traps use grid coordinates)
  if (viewportRef.value) {
    const rect = viewportRef.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left
    const mouseY = event.clientY - rect.top

    // Convert grid coords to pixel coords (center of cell)
    const trapPixelX = (trap.grid_x + 0.5) * uvttGridSize.value
    const trapPixelY = (trap.grid_y + 0.5) * uvttGridSize.value

    // Trap position in viewport coordinates
    const trapViewportX = trapPixelX * baseScale.value + panX.value
    const trapViewportY = trapPixelY * baseScale.value + panY.value

    // Store offset so trap moves smoothly from where we clicked
    dragTrapOffsetX.value = mouseX - trapViewportX * zoom.value
    dragTrapOffsetY.value = mouseY - trapViewportY * zoom.value
  }
}

// Start dragging a POI
function onPoiMouseDown(event: MouseEvent, poi: MapPoi) {
  // Only left-click to drag
  if (event.button !== 0) return

  event.preventDefault()
  event.stopPropagation()
  draggingPoi.value = poi

  // Calculate offset from mouse to POI center (POIs use grid coordinates)
  if (viewportRef.value) {
    const rect = viewportRef.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left
    const mouseY = event.clientY - rect.top

    // Convert grid coords to pixel coords (center of cell)
    const poiPixelX = (poi.grid_x + 0.5) * uvttGridSize.value
    const poiPixelY = (poi.grid_y + 0.5) * uvttGridSize.value

    // POI position in viewport coordinates
    const poiViewportX = poiPixelX * baseScale.value + panX.value
    const poiViewportY = poiPixelY * baseScale.value + panY.value

    // Store offset so POI moves smoothly from where we clicked
    dragPoiOffsetX.value = mouseX - poiViewportX * zoom.value
    dragPoiOffsetY.value = mouseY - poiViewportY * zoom.value
  }
}

function onMouseMove(event: MouseEvent) {
  // Handle token dragging
  if (draggingToken.value && viewportRef.value) {
    const rect = viewportRef.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left
    const mouseY = event.clientY - rect.top

    // Convert to image coordinates
    const effectiveScale = baseScale.value * zoom.value
    const imageX = (mouseX - dragTokenOffsetX.value - panX.value * zoom.value) / effectiveScale
    const imageY = (mouseY - dragTokenOffsetY.value - panY.value * zoom.value) / effectiveScale

    // Update token position locally (will be saved on mouse up)
    const tokenIndex = tokens.value.findIndex(t => t.id === draggingToken.value?.id)
    if (tokenIndex !== -1) {
      tokens.value[tokenIndex] = {
        ...tokens.value[tokenIndex],
        x: imageX,
        y: imageY
      }
    }
    return
  }

  // Handle light source dragging
  if (draggingLight.value && viewportRef.value) {
    const rect = viewportRef.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left
    const mouseY = event.clientY - rect.top

    // Convert to image coordinates
    const effectiveScale = baseScale.value * zoom.value
    const imageX = (mouseX - dragLightOffsetX.value - panX.value * zoom.value) / effectiveScale
    const imageY = (mouseY - dragLightOffsetY.value - panY.value * zoom.value) / effectiveScale

    // Update light position locally (will be saved on mouse up)
    const lightIndex = lightSources.value.findIndex(l => l.id === draggingLight.value?.id)
    if (lightIndex !== -1) {
      lightSources.value[lightIndex] = {
        ...lightSources.value[lightIndex],
        x: imageX,
        y: imageY
      }
    }
    return
  }

  // Handle trap dragging
  if (draggingTrap.value && viewportRef.value) {
    const rect = viewportRef.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left
    const mouseY = event.clientY - rect.top

    // Convert to image coordinates
    const effectiveScale = baseScale.value * zoom.value
    const imageX = (mouseX - dragTrapOffsetX.value - panX.value * zoom.value) / effectiveScale
    const imageY = (mouseY - dragTrapOffsetY.value - panY.value * zoom.value) / effectiveScale

    // Convert to grid coordinates for local update
    const gridX = Math.floor(imageX / uvttGridSize.value)
    const gridY = Math.floor(imageY / uvttGridSize.value)

    // Update trap position locally (will be saved on mouse up)
    const trapIndex = mapTraps.value.findIndex(t => t.id === draggingTrap.value?.id)
    if (trapIndex !== -1) {
      mapTraps.value[trapIndex] = {
        ...mapTraps.value[trapIndex],
        grid_x: gridX,
        grid_y: gridY
      }
    }
    return
  }

  // Handle POI dragging
  if (draggingPoi.value && viewportRef.value) {
    const rect = viewportRef.value.getBoundingClientRect()
    const mouseX = event.clientX - rect.left
    const mouseY = event.clientY - rect.top

    // Convert to image coordinates
    const effectiveScale = baseScale.value * zoom.value
    const imageX = (mouseX - dragPoiOffsetX.value - panX.value * zoom.value) / effectiveScale
    const imageY = (mouseY - dragPoiOffsetY.value - panY.value * zoom.value) / effectiveScale

    // Convert to grid coordinates for local update
    const gridX = Math.floor(imageX / uvttGridSize.value)
    const gridY = Math.floor(imageY / uvttGridSize.value)

    // Update POI position locally (will be saved on mouse up)
    const poiIndex = mapPois.value.findIndex(p => p.id === draggingPoi.value?.id)
    if (poiIndex !== -1) {
      mapPois.value[poiIndex] = {
        ...mapPois.value[poiIndex],
        grid_x: gridX,
        grid_y: gridY
      }
    }
    return
  }

  // Update mouse position for placement preview
  if (pendingTokenConfig.value && viewportRef.value) {
    const rect = viewportRef.value.getBoundingClientRect()
    mousePosition.value = {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top
    }
  }

  // Handle panning
  if (isDragging.value) {
    const deltaX = event.clientX - dragStartX.value
    const deltaY = event.clientY - dragStartY.value
    panX.value = dragStartPanX.value + deltaX
    panY.value = dragStartPanY.value + deltaY
  }
}

async function onMouseUp() {
  // Save token position if we were dragging
  if (draggingToken.value) {
    const token = tokens.value.find(t => t.id === draggingToken.value?.id)
    if (token) {
      // Snap to grid
      const gridSize = uvttGridSize.value
      const snappedX = Math.round(token.x / gridSize) * gridSize + gridSize / 2
      const snappedY = Math.round(token.y / gridSize) * gridSize + gridSize / 2

      // Convert pixel coordinates to grid coordinates for the backend
      const gridX = Math.floor(snappedX / gridSize)
      const gridY = Math.floor(snappedY / gridSize)

      // Update local state with snapped position
      const tokenIndex = tokens.value.findIndex(t => t.id === token.id)
      if (tokenIndex !== -1) {
        tokens.value[tokenIndex] = {
          ...tokens.value[tokenIndex],
          x: snappedX,
          y: snappedY
        }
      }

      // Save to backend
      try {
        await invoke('update_token_position', {
          id: token.id,
          gridX,
          gridY
        })
      } catch (e) {
        console.error('Failed to save token position:', e)
        // Reload tokens to restore original position on error
        await loadTokens()
      }
    }
    draggingToken.value = null
  }

  // Save light position if we were dragging
  if (draggingLight.value) {
    const light = lightSources.value.find(l => l.id === draggingLight.value?.id)
    if (light) {
      // Light sources don't snap to grid - they can be positioned freely
      // Convert to integer pixel coordinates for backend
      const pixelX = Math.round(light.x)
      const pixelY = Math.round(light.y)

      // Save to backend
      try {
        await invoke('move_light_source', {
          id: String(light.id),
          x: pixelX,
          y: pixelY
        })
      } catch (e) {
        console.error('Failed to save light position:', e)
        // Reload lights to restore original position on error
        await loadLightSources()
      }
    }
    draggingLight.value = null
  }

  // Save trap position if we were dragging
  if (draggingTrap.value) {
    const trap = mapTraps.value.find(t => t.id === draggingTrap.value?.id)
    if (trap) {
      // Save to backend (trap already has grid coordinates)
      try {
        await invoke('move_map_trap', {
          id: trap.id,
          gridX: trap.grid_x,
          gridY: trap.grid_y
        })
      } catch (e) {
        console.error('Failed to save trap position:', e)
        // Reload traps to restore original position on error
        await loadMapTraps()
      }
    }
    draggingTrap.value = null
  }

  // Save POI position if we were dragging
  if (draggingPoi.value) {
    const poi = mapPois.value.find(p => p.id === draggingPoi.value?.id)
    if (poi) {
      // Save to backend (POI already has grid coordinates)
      try {
        await invoke('move_map_poi', {
          id: poi.id,
          gridX: poi.grid_x,
          gridY: poi.grid_y
        })
      } catch (e) {
        console.error('Failed to save POI position:', e)
        // Reload POIs to restore original position on error
        await loadMapPois()
      }
    }
    draggingPoi.value = null
  }

  isDragging.value = false
}

// Token config from palette
function handleTokenConfigChange(config: TokenConfigWithMonster | null) {
  pendingTokenConfig.value = config
  // Clear light selection when selecting token
  if (config) {
    pendingLightType.value = ''
  }
}

// Light config from palette
function handleLightConfigChange(lightType: 'torch' | 'lantern' | 'candle' | null) {
  pendingLightType.value = lightType || ''
  // Clear token selection when selecting light
  if (lightType) {
    pendingTokenConfig.value = null
  }
}

// Canvas click for token or light placement
async function handleCanvasClick(event: MouseEvent) {
  // Close context menus if open
  if (contextMenu.value.visible) {
    contextMenu.value.visible = false
    return
  }
  if (trapContextMenu.value.visible) {
    trapContextMenu.value.visible = false
    return
  }
  if (poiContextMenu.value.visible) {
    poiContextMenu.value.visible = false
    return
  }

  // Handle light placement
  if (pendingLightType.value && viewportRef.value) {
    await handleLightPlacement(event)
    return
  }

  // Only place tokens if we have a pending config
  if (!pendingTokenConfig.value || !viewportRef.value) return

  const rect = viewportRef.value.getBoundingClientRect()
  const clickX = event.clientX - rect.left
  const clickY = event.clientY - rect.top

  // Convert to image coordinates
  const effectiveScale = baseScale.value * zoom.value
  const imageX = (clickX - panX.value) / effectiveScale
  const imageY = (clickY - panY.value) / effectiveScale

  // Snap to grid (all UVTT maps have a grid)
  const gridSize = uvttGridSize.value

  // Calculate grid coordinates
  const gridXVal = Math.floor(imageX / gridSize)
  const gridYVal = Math.floor(imageY / gridSize)

  // Snap to grid cell center (for pixel coordinates)
  const finalX = gridXVal * gridSize + gridSize / 2
  const finalY = gridYVal * gridSize + gridSize / 2

  // Extract monster/trap info
  const tokenType = pendingTokenConfig.value.token_type
  const monsterName = pendingTokenConfig.value.monster_name
  const monsterSource = pendingTokenConfig.value.monster_source

  // For trap tokens, create a map trap
  if (tokenType === 'trap') {
    try {
      const trapResponse = await invoke<{ success: boolean; data?: MapTrap }>('create_map_trap', {
        request: {
          mapId: props.map.id,
          name: pendingTokenConfig.value.name || 'Trap',
          gridX: gridXVal,
          gridY: gridYVal,
          visible: pendingTokenConfig.value.visible_to_players
        }
      })

      if (trapResponse.success && trapResponse.data) {
        mapTraps.value.push(trapResponse.data)
      } else {
        console.error('Failed to create trap:', trapResponse)
      }
    } catch (e) {
      console.error('Failed to place trap:', e)
    }
    return
  }

  // For marker tokens, create a map POI
  if (tokenType === 'marker') {
    try {
      const poiResponse = await invoke<{ success: boolean; data?: MapPoi }>('create_map_poi', {
        request: {
          mapId: props.map.id,
          name: pendingTokenConfig.value.name || 'Point of Interest',
          gridX: gridXVal,
          gridY: gridYVal,
          icon: 'pin',
          color: pendingTokenConfig.value.color || '#4488ff',
          visible: pendingTokenConfig.value.visible_to_players
        }
      })

      if (poiResponse.success && poiResponse.data) {
        mapPois.value.push(poiResponse.data)
      } else {
        console.error('Failed to create POI:', poiResponse)
      }
    } catch (e) {
      console.error('Failed to place POI:', e)
    }
    return
  }

  // For monster tokens, we need to first add to module_monsters, then create token placement
  if (tokenType === 'monster' && monsterName && monsterSource && props.map.module_id) {
    try {
      // First, add monster to module_monsters (or get existing)
      const addResponse = await invoke<{ success: boolean; data?: { id: string } }>('add_module_monster', {
        request: {
          moduleId: props.map.module_id,
          monsterName: monsterName,
          monsterSource: monsterSource,
          quantity: 1
        }
      })
      console.log('add_module_monster response:', addResponse)

      if (!addResponse.success || !addResponse.data) {
        console.error('Failed to add monster to module:', addResponse)
        return
      }

      const moduleMonsterIdStr = addResponse.data.id

      // Now create the token placement with the module_monster_id
      const tokenResponse = await invoke<{ success: boolean; data?: Token }>('create_token', {
        request: {
          mapId: props.map.id,
          moduleMonsterId: moduleMonsterIdStr,
          gridX: gridXVal,
          gridY: gridYVal,
          label: pendingTokenConfig.value.name || null,
          factionColor: pendingTokenConfig.value.color || null,
          hidden: !pendingTokenConfig.value.visible_to_players
        }
      })
      console.log('create_token response:', tokenResponse)

      if (tokenResponse.success && tokenResponse.data) {
        tokens.value.push(tokenResponse.data)
        // Refresh the palette's module monsters list
        paletteRef.value?.loadModuleMonsters()
      } else {
        console.error('Failed to create token:', tokenResponse)
      }
    } catch (e) {
      console.error('Failed to place monster token:', e)
    }
  } else {
    // For non-monster/non-trap tokens (markers, NPCs)
    console.warn('This token type is not yet fully supported by the backend')
  }
}

// Handle light placement on canvas click
async function handleLightPlacement(event: MouseEvent) {
  if (!viewportRef.value || !pendingLightType.value) return

  const rect = viewportRef.value.getBoundingClientRect()
  const clickX = event.clientX - rect.left
  const clickY = event.clientY - rect.top

  // Convert to image coordinates
  const effectiveScale = baseScale.value * zoom.value
  const imageX = (clickX - panX.value) / effectiveScale
  const imageY = (clickY - panY.value) / effectiveScale

  // Light defaults based on type
  const lightDefaults: Record<string, { name: string; bright: number; dim: number; color: string }> = {
    torch: { name: 'Torch', bright: 20, dim: 40, color: '#ff9933' },
    lantern: { name: 'Lantern', bright: 30, dim: 60, color: '#ffcc66' },
    candle: { name: 'Candle', bright: 5, dim: 10, color: '#ffaa44' }
  }

  const lightConfig = lightDefaults[pendingLightType.value]
  if (!lightConfig) return

  try {
    await invoke('create_light_source', {
      request: {
        map_id: props.map.id,
        name: lightConfig.name,
        light_type: pendingLightType.value,
        x: imageX,
        y: imageY,
        bright_radius_ft: lightConfig.bright,
        dim_radius_ft: lightConfig.dim,
        color: lightConfig.color,
        is_active: true
      }
    })

    // Reload lights and reset selection
    await loadLightSources()
    pendingLightType.value = ''
  } catch (e) {
    console.error('Failed to create light source:', e)
  }
}

// Add monster to module_monsters if not already present
async function addMonsterToModule(monsterName: string, monsterSource: string, moduleId: string) {
  try {
    // Check if already in module_monsters by checking the palette's current list
    const existingMonsters = paletteRef.value?.currentConfig
    // Actually, we need to check the moduleMonsters from the palette
    // For now, just try to add - the backend handles duplicates by incrementing quantity
    await invoke('add_module_monster', {
      request: {
        module_id: moduleId,
        monster_name: monsterName,
        monster_source: monsterSource,
        quantity: 1
      }
    })
    // Refresh the palette's module monsters list
    paletteRef.value?.loadModuleMonsters()
  } catch (e) {
    console.error('Failed to add monster to module:', e)
  }
}

// Token selection
function selectToken(token: Token) {
  selectedTokenId.value = token.id
}

// Context menu
function showTokenContextMenu(event: MouseEvent, token: Token) {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    token
  }
  selectedTokenId.value = token.id
}

function handleEditToken() {
  // TODO: Open token editor modal
  contextMenu.value.visible = false
}

async function handleToggleSelectedVisibility() {
  if (contextMenu.value.token) {
    await toggleVisibility(contextMenu.value.token.id)
  }
  contextMenu.value.visible = false
}

async function handleToggleVisibility(token: Token) {
  await toggleVisibility(token.id)
}

async function handleDeleteFromContext() {
  if (contextMenu.value.token) {
    await deleteToken(contextMenu.value.token.id)
  }
  contextMenu.value.visible = false
}

async function confirmDeleteToken(token: Token) {
  if (confirm(`Delete token "${token.name}"?`)) {
    await deleteToken(token.id)
    if (selectedTokenId.value === token.id) {
      selectedTokenId.value = null
    }
  }
}

async function confirmDeleteLight(light: LightSource) {
  if (confirm(`Delete light source "${light.name}"?`)) {
    try {
      await invoke('delete_light_source', { id: light.id })
      await loadLightSources()
    } catch (e) {
      console.error('Failed to delete light source:', e)
    }
  }
}

// Trap context menu
function showTrapContextMenu(event: MouseEvent, trap: MapTrap) {
  trapContextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    trap
  }
}

async function handleToggleTrapVisibility() {
  if (trapContextMenu.value.trap) {
    try {
      await invoke('toggle_map_trap_visibility', { id: trapContextMenu.value.trap.id })
      await loadMapTraps()
    } catch (e) {
      console.error('Failed to toggle trap visibility:', e)
    }
  }
  trapContextMenu.value.visible = false
}

async function handleTriggerTrap() {
  if (trapContextMenu.value.trap) {
    try {
      await invoke('trigger_map_trap', { id: trapContextMenu.value.trap.id })
      await loadMapTraps()
    } catch (e) {
      console.error('Failed to trigger trap:', e)
    }
  }
  trapContextMenu.value.visible = false
}

async function handleResetTrap() {
  if (trapContextMenu.value.trap) {
    try {
      await invoke('reset_map_trap', { id: trapContextMenu.value.trap.id })
      await loadMapTraps()
    } catch (e) {
      console.error('Failed to reset trap:', e)
    }
  }
  trapContextMenu.value.visible = false
}

async function handleDeleteTrap() {
  if (trapContextMenu.value.trap) {
    if (confirm(`Delete trap "${trapContextMenu.value.trap.name}"?`)) {
      try {
        await invoke('delete_map_trap', { id: trapContextMenu.value.trap.id })
        await loadMapTraps()
      } catch (e) {
        console.error('Failed to delete trap:', e)
      }
    }
  }
  trapContextMenu.value.visible = false
}

// Direct trap actions (for list panel buttons)
async function handleToggleTrapVisibilityDirect(trap: MapTrap) {
  try {
    await invoke('toggle_map_trap_visibility', { id: trap.id })
    await loadMapTraps()
  } catch (e) {
    console.error('Failed to toggle trap visibility:', e)
  }
}

async function confirmDeleteTrapDirect(trap: MapTrap) {
  if (confirm(`Delete trap "${trap.name}"?`)) {
    try {
      await invoke('delete_map_trap', { id: trap.id })
      await loadMapTraps()
    } catch (e) {
      console.error('Failed to delete trap:', e)
    }
  }
}

// POI context menu
function showPoiContextMenu(event: MouseEvent, poi: MapPoi) {
  poiContextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    poi
  }
}

async function handleTogglePoiVisibility() {
  if (poiContextMenu.value.poi) {
    try {
      await invoke('toggle_map_poi_visibility', { id: poiContextMenu.value.poi.id })
      await loadMapPois()
    } catch (e) {
      console.error('Failed to toggle POI visibility:', e)
    }
  }
  poiContextMenu.value.visible = false
}

async function handleDeletePoi() {
  if (poiContextMenu.value.poi) {
    if (confirm(`Delete POI "${poiContextMenu.value.poi.name}"?`)) {
      try {
        await invoke('delete_map_poi', { id: poiContextMenu.value.poi.id })
        await loadMapPois()
      } catch (e) {
        console.error('Failed to delete POI:', e)
      }
    }
  }
  poiContextMenu.value.visible = false
}

// Direct POI actions (for list panel buttons)
async function handleTogglePoiVisibilityDirect(poi: MapPoi) {
  try {
    await invoke('toggle_map_poi_visibility', { id: poi.id })
    await loadMapPois()
  } catch (e) {
    console.error('Failed to toggle POI visibility:', e)
  }
}

async function confirmDeletePoiDirect(poi: MapPoi) {
  if (confirm(`Delete POI "${poi.name}"?`)) {
    try {
      await invoke('delete_map_poi', { id: poi.id })
      await loadMapPois()
    } catch (e) {
      console.error('Failed to delete POI:', e)
    }
  }
}

// Token display helpers
function getTokenColor(token: Token): string {
  return token.color || TOKEN_TYPE_COLORS[token.token_type as keyof typeof TOKEN_TYPE_COLORS] || '#666666'
}

function getTokenStyle(token: Token) {
  const gridSquares = TOKEN_SIZE_GRID_SQUARES[token.size as TokenSize] || 1
  const tokenSize = gridSquares * uvttGridSize.value * baseScale.value
  const color = getTokenColor(token)

  return {
    left: (token.x * baseScale.value - tokenSize / 2) + 'px',
    top: (token.y * baseScale.value - tokenSize / 2) + 'px',
    width: tokenSize + 'px',
    height: tokenSize + 'px',
    background: color,
    borderColor: color
  }
}

function getTokenInitial(token: Token): string {
  return token.name.charAt(0).toUpperCase()
}

function getLightStyle(light: LightSource) {
  const dotSize = 12
  return {
    left: (light.x * baseScale.value - dotSize / 2) + 'px',
    top: (light.y * baseScale.value - dotSize / 2) + 'px',
    width: dotSize + 'px',
    height: dotSize + 'px',
    background: light.color || '#ffcc00'
  }
}

function getTrapStyle(trap: MapTrap) {
  const gridSize = uvttGridSize.value
  // Convert grid coordinates to pixel coordinates (center of cell)
  const pixelX = (trap.grid_x + 0.5) * gridSize
  const pixelY = (trap.grid_y + 0.5) * gridSize
  const markerSize = gridSize * 0.6 // Trap marker is 60% of grid cell

  return {
    left: (pixelX * baseScale.value - markerSize * baseScale.value / 2) + 'px',
    top: (pixelY * baseScale.value - markerSize * baseScale.value / 2) + 'px',
    width: (markerSize * baseScale.value) + 'px',
    height: (markerSize * baseScale.value) + 'px'
  }
}

function getPoiStyle(poi: MapPoi) {
  const gridSize = uvttGridSize.value
  // Convert grid coordinates to pixel coordinates (center of cell)
  const pixelX = (poi.grid_x + 0.5) * gridSize
  const pixelY = (poi.grid_y + 0.5) * gridSize
  const markerSize = gridSize * 0.5 // POI marker is 50% of grid cell

  return {
    left: (pixelX * baseScale.value - markerSize * baseScale.value / 2) + 'px',
    top: (pixelY * baseScale.value - markerSize * baseScale.value / 2) + 'px',
    width: (markerSize * baseScale.value) + 'px',
    height: (markerSize * baseScale.value) + 'px',
    background: poi.color || '#4488ff'
  }
}

function getPoiIcon(iconType: string): string {
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
  return icons[iconType] || '📍'
}

async function toggleLight(light: LightSource) {
  try {
    await invoke('toggle_light_source', { id: light.id })
    await loadLightSources()
  } catch (e) {
    console.error('Failed to toggle light:', e)
  }
}

function getPlacementPreviewStyle() {
  if (!mousePosition.value || !pendingTokenConfig.value) return {}

  const gridSize = uvttGridSize.value
  const size = pendingTokenConfig.value.size as TokenSize
  const gridSquares = TOKEN_SIZE_GRID_SQUARES[size] || 1
  const tokenSize = gridSquares * gridSize * baseScale.value * zoom.value
  const color = pendingTokenConfig.value.color || '#666666'

  return {
    left: (mousePosition.value.x - tokenSize / 2) + 'px',
    top: (mousePosition.value.y - tokenSize / 2) + 'px',
    width: tokenSize + 'px',
    height: tokenSize + 'px',
    background: color
  }
}

function handleClose() {
  contextMenu.value.visible = false
  emit('close')
}

// Handle grid config saved - refresh grid data
async function handleGridSaved() {
  showGridConfigModal.value = false
  // Reload UVTT data to get updated grid settings
  await loadUvttData()
}

// Close context menus on click outside
function handleClickOutside() {
  contextMenu.value.visible = false
  trapContextMenu.value.visible = false
  poiContextMenu.value.visible = false
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  if (props.visible && props.map.id) {
    Promise.all([loadMapImage(), loadUvttData(), loadTokens()])
  }
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
/* Token Setup Body Layout */
.token-setup-body {
  display: flex;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  overflow: hidden;
  flex: 1;
  min-height: 0;
  height: 70vh;
}

.palette-wrapper {
  overflow-y: auto;
  flex-shrink: 0;
}

/* Map Canvas */
.map-canvas-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.canvas-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm);
  background: var(--color-base-100);
  border-radius: var(--radius-md) var(--radius-md) 0 0;
}

.zoom-controls {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.zoom-label {
  font-size: 0.875rem;
  font-weight: 500;
}

.zoom-level {
  font-size: 0.875rem;
  font-family: monospace;
  min-width: 50px;
  text-align: center;
}

.ctrl-btn {
  padding: var(--spacing-xs) var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-background);
  cursor: pointer;
  font-size: 0.875rem;
}

.ctrl-btn:hover:not(:disabled) {
  background: var(--color-base-200);
}

.ctrl-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.divider {
  color: var(--color-border);
  margin: 0 var(--spacing-xs);
}

.token-count {
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.map-viewport {
  position: relative;
  flex: 1;
  background: #1a1a1a;
  border-radius: 0 0 var(--radius-md) var(--radius-md);
  overflow: hidden;
  cursor: grab;
}

.map-viewport:active {
  cursor: grabbing;
}

.map-viewport.placement-mode {
  cursor: crosshair;
}

.map-image {
  display: block;
  user-select: none;
  -webkit-user-drag: none;
}

.loading-map {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  color: var(--color-text-muted);
}

.grid-overlay {
  pointer-events: none;
}

/* Token Layer */
.token-layer {
  pointer-events: none;
}

.token {
  position: absolute;
  border-radius: 50%;
  border: 3px solid;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: auto;
  cursor: pointer;
  transition: transform 0.1s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
}

.token:hover {
  transform: scale(1.1);
  z-index: 10;
}

.token-selected {
  box-shadow: 0 0 0 3px var(--color-primary-500), 0 2px 8px rgba(0, 0, 0, 0.4);
}

.token-hidden {
  opacity: 0.5;
}

.token-dragging {
  cursor: grabbing;
  transform: scale(1.15);
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  transition: none;
}

.token-label {
  font-weight: 700;
  color: white;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
  font-size: 0.875em;
}

.visibility-indicator {
  position: absolute;
  top: -4px;
  right: -4px;
  font-size: 0.75rem;
}

/* Light Sources */
.light-dot {
  position: absolute;
  border-radius: 50%;
  border: 2px solid #fff;
  box-shadow: 0 0 8px 2px rgba(255, 204, 0, 0.6);
  cursor: pointer;
  pointer-events: auto;
  transition: transform 0.1s, box-shadow 0.1s;
}

.light-dot:hover {
  transform: scale(1.3);
  box-shadow: 0 0 12px 4px rgba(255, 204, 0, 0.8);
}

.light-dot.light-inactive {
  opacity: 0.4;
  box-shadow: none;
  border-color: #888;
}

.light-dot.light-dragging {
  cursor: grabbing;
  transform: scale(1.5);
  z-index: 100;
  box-shadow: 0 0 16px 6px rgba(255, 204, 0, 0.9);
  transition: none;
}

/* Placement Preview */
.placement-preview {
  position: absolute;
  border-radius: 50%;
  border: 3px dashed white;
  opacity: 0.6;
  pointer-events: none;
}

/* Token List Panel */
.token-list-panel {
  width: 200px;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
  overflow-y: auto;
}

.token-list-panel h4 {
  margin: 0 0 var(--spacing-md) 0;
  font-size: 0.875rem;
  font-weight: 600;
}

.token-list-panel h4.section-header {
  margin-top: var(--spacing-md);
  padding-top: var(--spacing-md);
  border-top: 1px solid var(--color-border);
}

.token-list-item.light-item {
  background: rgba(255, 200, 0, 0.05);
}

.token-list-item.light-item.light-inactive {
  opacity: 0.5;
}

.light-color {
  box-shadow: 0 0 4px rgba(255, 200, 0, 0.5);
}

.light-toggle-btn {
  padding: 2px 6px;
  font-size: 0.625rem;
  font-weight: 600;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-base-200);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.light-toggle-btn.lit {
  background: rgba(255, 180, 0, 0.2);
  border-color: #ffb400;
  color: #b38000;
}

.empty-tokens {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  text-align: center;
  padding: var(--spacing-md);
}

.token-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.token-list-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs);
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.token-list-item:hover {
  background: var(--color-base-200);
}

.token-list-item.selected {
  background: var(--color-primary-100);
}

.token-list-color {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  flex-shrink: 0;
}

.token-list-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.token-list-name {
  font-size: 0.875rem;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.token-list-type {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  text-transform: capitalize;
}

.token-list-visibility,
.token-list-delete {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
}

.token-list-visibility:hover,
.token-list-delete:hover {
  background: var(--color-base-300);
}

.token-list-visibility.hidden {
  opacity: 0.5;
}

.token-list-delete {
  color: var(--color-text-muted);
}

.token-list-delete:hover {
  color: var(--color-error);
}

/* Context Menu */
.context-menu {
  position: fixed;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1001;
  min-width: 150px;
}

.context-menu button {
  display: block;
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: none;
  background: transparent;
  text-align: left;
  cursor: pointer;
  font-size: 0.875rem;
}

.context-menu button:hover {
  background: var(--color-base-200);
}

.context-menu button.danger {
  color: var(--color-error);
}

.context-menu button.danger:hover {
  background: var(--color-error-100);
}

/* Trap Markers */
.trap-marker {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: auto;
  cursor: pointer;
  background: rgba(255, 100, 100, 0.3);
  border: 2px dashed #ff4444;
  border-radius: var(--radius-sm);
  transition: transform 0.1s, box-shadow 0.1s;
}

.trap-marker:hover {
  transform: scale(1.1);
  z-index: 10;
  box-shadow: 0 0 8px rgba(255, 68, 68, 0.5);
}

.trap-marker .trap-icon {
  font-size: 1rem;
}

.trap-marker.trap-triggered {
  background: rgba(100, 100, 100, 0.3);
  border-color: #888;
  opacity: 0.6;
}

.trap-marker.trap-visible {
  border-style: solid;
  background: rgba(255, 100, 100, 0.5);
}

.trap-marker.trap-dragging {
  cursor: grabbing;
  transform: scale(1.2);
  z-index: 100;
  box-shadow: 0 4px 12px rgba(255, 68, 68, 0.6);
  transition: none;
}

/* Trap List Items */
.trap-item {
  background: rgba(255, 100, 100, 0.05);
}

.trap-item.trap-triggered {
  opacity: 0.6;
}

.trap-item.trap-visible-marker {
  border-left: 3px solid #ff4444;
}

.trap-color {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
}

.trap-toggle-btn {
  padding: 2px 6px;
  font-size: 0.625rem;
  font-weight: 600;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-base-200);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.trap-toggle-btn.visible {
  background: rgba(255, 100, 100, 0.2);
  border-color: #ff4444;
  color: #cc3333;
}

/* POI (Point of Interest) Markers */
.poi-marker {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: auto;
  cursor: pointer;
  background: rgba(68, 136, 255, 0.3);
  border: 2px solid #4488ff;
  border-radius: 50%;
  transition: transform 0.1s, box-shadow 0.1s;
}

.poi-marker:hover {
  transform: scale(1.2);
  z-index: 10;
  box-shadow: 0 0 8px rgba(68, 136, 255, 0.6);
}

.poi-marker .poi-icon {
  font-size: 1rem;
}

.poi-marker.poi-visible {
  border-width: 3px;
  background: rgba(68, 136, 255, 0.5);
  box-shadow: 0 0 6px rgba(68, 136, 255, 0.4);
}

.poi-marker.poi-dragging {
  cursor: grabbing;
  transform: scale(1.3);
  z-index: 100;
  box-shadow: 0 4px 12px rgba(68, 136, 255, 0.7);
  transition: none;
}

/* POI List Items */
.poi-item {
  background: rgba(68, 136, 255, 0.05);
}

.poi-item.poi-visible-marker {
  border-left: 3px solid #4488ff;
}

.poi-color {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
}

.poi-toggle-btn {
  padding: 2px 6px;
  font-size: 0.625rem;
  font-weight: 600;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-base-200);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.poi-toggle-btn.visible {
  background: rgba(68, 136, 255, 0.2);
  border-color: #4488ff;
  color: #2266cc;
}
</style>
