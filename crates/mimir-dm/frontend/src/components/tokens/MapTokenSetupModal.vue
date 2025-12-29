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
              </div>
              <div class="token-count">
                {{ tokens.length }} tokens
              </div>
            </div>

            <div
              class="map-viewport"
              ref="viewportRef"
              :class="{ 'placement-mode': !!pendingTokenConfig }"
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
                    'token-selected': selectedTokenId === token.id
                  }"
                  :style="getTokenStyle(token)"
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
          </div>
        </div>

      <!-- Context Menu -->
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

    <template #footer>
      <button class="btn btn-secondary" @click="handleClose">Close</button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import TokenPalette from './TokenPalette.vue'
import type { Token, CreateTokenRequest, TokenSize } from '@/types/api'
import { TOKEN_SIZE_GRID_SQUARES, TOKEN_TYPE_COLORS } from '@/types/api'
import { useTokens } from '@/composables/useTokens'

interface Map {
  id: number
  name: string
  image_path: string
  width_px: number
  height_px: number
  grid_type: string
  grid_size_px: number | null
  grid_offset_x: number
  grid_offset_y: number
  campaign_id: number
  module_id?: number | null
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
const pendingTokenConfig = ref<CreateTokenRequest | null>(null)
const mousePosition = ref<{ x: number; y: number } | null>(null)
const selectedTokenId = ref<number | null>(null)

// Context menu
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  token: null as Token | null
})

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
    await Promise.all([loadMapImage(), loadUvttData(), loadTokens()])
  }
})

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

async function loadUvttData() {
  // Load UVTT to get grid size
  try {
    const response = await invoke<{ success: boolean; data?: UvttData }>('get_uvtt_map', {
      campaignId: props.map.campaign_id,
      moduleId: props.map.module_id ?? null,
      filePath: props.map.image_path
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

  // Only pan with middle mouse or when holding space
  if (event.button === 1 || (event.button === 0 && !pendingTokenConfig.value)) {
    isDragging.value = true
    dragStartX.value = event.clientX
    dragStartY.value = event.clientY
    dragStartPanX.value = panX.value
    dragStartPanY.value = panY.value
  }
}

function onMouseMove(event: MouseEvent) {
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

function onMouseUp() {
  isDragging.value = false
}

// Token config from palette
function handleTokenConfigChange(config: CreateTokenRequest | null) {
  pendingTokenConfig.value = config
}

// Canvas click for token placement
async function handleCanvasClick(event: MouseEvent) {
  // Close context menu if open
  if (contextMenu.value.visible) {
    contextMenu.value.visible = false
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
  let finalX = imageX
  let finalY = imageY

  const gridSize = uvttGridSize.value
  // UVTT maps don't use grid offsets
  const offsetX = 0
  const offsetY = 0

  // Snap to grid cell center
  finalX = Math.round((imageX - offsetX) / gridSize) * gridSize + offsetX + gridSize / 2
  finalY = Math.round((imageY - offsetY) / gridSize) * gridSize + offsetY + gridSize / 2

  // Create the token
  const request: CreateTokenRequest = {
    ...pendingTokenConfig.value,
    map_id: props.map.id,
    x: finalX,
    y: finalY
  }

  const token = await createToken(request)
  if (token) {
    // Optionally clear selection after placing
    // paletteRef.value?.clearSelection()
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

// Close context menu on click outside
function handleClickOutside() {
  contextMenu.value.visible = false
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
</style>
