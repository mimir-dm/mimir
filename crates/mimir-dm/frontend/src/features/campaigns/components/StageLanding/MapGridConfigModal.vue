<template>
  <AppModal
    :visible="visible"
    :title="`Configure Grid - ${map.name}`"
    :size="gridType !== 'none' ? 'lg' : 'sm'"
    :closable="!saving"
    :close-on-overlay="!saving"
    :close-on-escape="!saving"
    @close="handleClose"
  >
          <!-- Grid Type Selection -->
          <div class="form-group">
            <label>Grid Type</label>
            <div class="radio-group">
              <label class="radio-option">
                <input type="radio" v-model="gridType" value="none" @change="onGridTypeChange" />
                <span>No Grid</span>
              </label>
              <label class="radio-option">
                <input type="radio" v-model="gridType" value="square" @change="onGridTypeChange" />
                <span>Square</span>
              </label>
              <label class="radio-option">
                <input type="radio" v-model="gridType" value="hex" @change="onGridTypeChange" />
                <span>Hexagonal</span>
              </label>
            </div>
          </div>

          <!-- Visual Grid Editor (when grid enabled) -->
          <div v-if="gridType !== 'none'" class="grid-editor-section">

            <!-- Step indicator -->
            <div class="step-indicator">
              <div class="step" :class="{ active: editorMode === 'position' }">
                <span class="step-num">1</span>
                <span class="step-label">Position View</span>
              </div>
              <div class="step-arrow">→</div>
              <div class="step" :class="{ active: editorMode === 'grid' }">
                <span class="step-num">2</span>
                <span class="step-label">Align Grid</span>
              </div>
            </div>

            <!-- Position Mode Controls -->
            <div v-if="editorMode === 'position'" class="mode-controls">
              <div class="zoom-controls">
                <span class="zoom-label">Zoom:</span>
                <button class="ctrl-btn" @click="zoomOut" :disabled="zoom <= 0.5">−</button>
                <span class="zoom-level">{{ Math.round(zoom * 100) }}%</span>
                <button class="ctrl-btn" @click="zoomIn" :disabled="zoom >= 4">+</button>
                <button class="ctrl-btn" @click="resetView">Fit</button>
              </div>
              <p class="mode-hint">Pan and zoom to frame the area where you want to align the grid.</p>
            </div>

            <!-- Grid Mode Controls -->
            <div v-if="editorMode === 'grid'" class="mode-controls">
              <div class="grid-controls">
                <div class="control-group">
                  <label>Cell Size (px)</label>
                  <div class="size-input-group">
                    <button class="ctrl-btn" @click="adjustGridSize(-5)">−</button>
                    <input
                      v-model.number="gridSizePx"
                      type="number"
                      class="size-input"
                      min="10"
                      max="500"
                    />
                    <button class="ctrl-btn" @click="adjustGridSize(5)">+</button>
                  </div>
                </div>
                <div class="control-group">
                  <label>Offset</label>
                  <div class="offset-display">X: {{ gridOffsetX }}, Y: {{ gridOffsetY }}</div>
                </div>
                <button class="ctrl-btn reset-btn" @click="resetOffset">Reset Offset</button>
              </div>
              <p class="mode-hint">Drag on the map to position the grid origin. Adjust cell size to match your map's grid.</p>
            </div>

            <!-- Map Preview -->
            <div
              class="preview-container"
              ref="previewContainer"
              @wheel.prevent="onWheel"
            >
              <div
                class="preview-viewport"
                :style="viewportStyle"
                @mousedown="onMouseDown"
                @mousemove="onMouseMove"
                @mouseup="onMouseUp"
                @mouseleave="onMouseUp"
              >
                <!-- Wrapper scales both image and SVG together -->
                <div class="map-wrapper" :style="mapWrapperStyle">
                  <!-- Map Image -->
                  <img
                    v-if="mapImageUrl"
                    :src="mapImageUrl"
                    :alt="map.name"
                    class="preview-image"
                    :style="imageStyle"
                    @load="onImageLoad"
                    ref="previewImage"
                    draggable="false"
                  />

                  <!-- Grid Overlay (only in grid mode) -->
                  <!-- Uses explicit lines at natural coords, wrapper transform handles scaling -->
                  <svg
                    v-if="editorMode === 'grid' && mapImageUrl && imageLoaded"
                    class="grid-overlay"
                    :style="gridOverlayStyle"
                  >
                  <!-- Square grid: explicit lines -->
                  <g v-if="gridType === 'square'">
                    <!-- Vertical lines -->
                    <line
                      v-for="x in verticalGridLines"
                      :key="'v' + x"
                      :x1="x"
                      :y1="0"
                      :x2="x"
                      :y2="props.map.height_px"
                      stroke="rgba(255, 0, 0, 0.7)"
                      vector-effect="non-scaling-stroke"
                      stroke-width="1"
                    />
                    <!-- Horizontal lines -->
                    <line
                      v-for="y in horizontalGridLines"
                      :key="'h' + y"
                      :x1="0"
                      :y1="y"
                      :x2="props.map.width_px"
                      :y2="y"
                      stroke="rgba(255, 0, 0, 0.7)"
                      vector-effect="non-scaling-stroke"
                      stroke-width="1"
                    />
                  </g>
                  <!-- Hex grid: keep using pattern for now -->
                  <g v-if="gridType === 'hex'">
                    <defs>
                      <pattern
                        id="gridPattern"
                        :width="displayGridSize * 1.5"
                        :height="displayGridSize * 1.732"
                        patternUnits="userSpaceOnUse"
                        :x="displayOffsetX"
                        :y="displayOffsetY"
                      >
                        <polygon
                          :points="hexPoints"
                          fill="none"
                          stroke="rgba(255, 0, 0, 0.7)"
                          stroke-width="1"
                        />
                      </pattern>
                    </defs>
                    <rect width="100%" height="100%" fill="url(#gridPattern)" />
                  </g>
                    <!-- Origin marker -->
                    <circle
                      :cx="displayOffsetX"
                      :cy="displayOffsetY"
                      r="6"
                      fill="var(--color-primary-500)"
                      stroke="white"
                      stroke-width="2"
                    />
                  </svg>
                </div>
                <div v-if="!mapImageUrl" class="loading-preview">Loading map preview...</div>
              </div>
            </div>

            <!-- Mode Toggle Button -->
            <div class="mode-toggle">
              <button
                v-if="editorMode === 'position'"
                class="btn-primary"
                @click="enterGridMode"
                :disabled="!imageLoaded"
              >
                Overlay Grid →
              </button>
              <button
                v-if="editorMode === 'grid'"
                class="btn-secondary"
                @click="enterPositionMode"
              >
                ← Adjust View
              </button>
            </div>
          </div>

          <!-- Grid Size Calculator -->
          <div v-if="gridType !== 'none' && compressionRatio !== 1" class="calculator-section">
            <details open>
              <summary>Grid Size Calculator</summary>
              <div class="calculator-content">
                <p class="calculator-info">
                  Image was compressed to {{ Math.round(compressionRatio * 100) }}% of original size.
                </p>
                <div class="calculator-row">
                  <div class="form-group">
                    <label for="original-grid">Original Grid Size (px)</label>
                    <input
                      id="original-grid"
                      v-model.number="originalGridSize"
                      type="number"
                      class="form-input"
                      placeholder="e.g., 256"
                      min="1"
                    />
                  </div>
                  <div class="calculator-result" v-if="calculatedGridSize">
                    <span class="result-label">Scaled:</span>
                    <span class="result-value">{{ calculatedGridSize }}px</span>
                    <button class="btn-apply" @click="applyCalculatedSize">Apply</button>
                  </div>
                </div>
              </div>
            </details>
          </div>

          <!-- Manual offset inputs -->
          <div v-if="gridType !== 'none'" class="manual-offset-section">
            <details>
              <summary>Manual Offset Input</summary>
              <div class="form-row">
                <div class="form-group">
                  <label for="offset-x">X Offset</label>
                  <input id="offset-x" v-model.number="gridOffsetX" type="number" class="form-input" />
                </div>
                <div class="form-group">
                  <label for="offset-y">Y Offset</label>
                  <input id="offset-y" v-model.number="gridOffsetY" type="number" class="form-input" />
                </div>
              </div>
            </details>
          </div>

    <template #footer>
      <button class="btn btn-secondary" @click="handleClose" :disabled="saving">Cancel</button>
      <button class="btn btn-primary" @click="handleSave" :disabled="saving">
        {{ saving ? 'Saving...' : 'Save Grid Settings' }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'

interface Map {
  id: number
  name: string
  width_px: number
  height_px: number
  grid_type: string
  grid_size_px: number | null
  grid_offset_x: number
  grid_offset_y: number
  original_width_px: number | null
  original_height_px: number | null
}

const props = defineProps<{
  visible: boolean
  map: Map
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

// Form state
const gridType = ref(props.map.grid_type || 'none')
const gridSizePx = ref(props.map.grid_size_px || 70)
const gridOffsetX = ref(props.map.grid_offset_x || 0)
const gridOffsetY = ref(props.map.grid_offset_y || 0)
const saving = ref(false)

// Grid calculator state
const originalGridSize = ref<number | null>(null)

// Compression ratio (current / original)
const compressionRatio = computed(() => {
  if (!props.map.original_width_px || props.map.original_width_px === props.map.width_px) {
    return 1
  }
  return props.map.width_px / props.map.original_width_px
})

// Calculated grid size based on compression ratio
const calculatedGridSize = computed(() => {
  if (!originalGridSize.value || originalGridSize.value <= 0) return null
  return Math.round(originalGridSize.value * compressionRatio.value)
})

function applyCalculatedSize() {
  if (calculatedGridSize.value) {
    gridSizePx.value = calculatedGridSize.value
  }
}

// Editor mode: 'position' (pan/zoom image) or 'grid' (adjust grid)
const editorMode = ref<'position' | 'grid'>('position')

// Image state
const mapImageUrl = ref<string | null>(null)
const imageLoaded = ref(false)
const previewContainer = ref<HTMLElement | null>(null)
const previewImage = ref<HTMLImageElement | null>(null)

// View state (for position mode)
const zoom = ref(1)
const panX = ref(0)
const panY = ref(0)

// Drag state
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const dragStartPanX = ref(0)
const dragStartPanY = ref(0)
const dragStartOffsetX = ref(0)
const dragStartOffsetY = ref(0)

// Container dimensions
const containerWidth = 700
const containerHeight = 400

// Computed display values
const baseScale = computed(() => {
  if (!props.map.width_px || !props.map.height_px) return 1
  const scaleX = containerWidth / props.map.width_px
  const scaleY = containerHeight / props.map.height_px
  return Math.min(scaleX, scaleY, 1)
})

// Base image dimensions (fit to container, before zoom)
const baseImageWidth = computed(() => props.map.width_px * baseScale.value)
const baseImageHeight = computed(() => props.map.height_px * baseScale.value)

// Grid display calculations - use UNSCALED values, SVG viewBox handles scaling
const displayGridSize = computed(() => gridSizePx.value)
const displayOffsetX = computed(() => gridOffsetX.value)
const displayOffsetY = computed(() => gridOffsetY.value)

// Styles
const viewportStyle = computed(() => ({
  width: containerWidth + 'px',
  height: containerHeight + 'px',
  cursor: editorMode.value === 'position'
    ? (isDragging.value ? 'grabbing' : 'grab')
    : (isDragging.value ? 'crosshair' : 'crosshair'),
  overflow: 'hidden'
}))

// Combined scale factor for uniform transform
const combinedScale = computed(() => baseScale.value * zoom.value)

// Wrapper transform - scales and positions both image and SVG together
const mapWrapperStyle = computed(() => ({
  width: props.map.width_px + 'px',
  height: props.map.height_px + 'px',
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${combinedScale.value})`,
  transformOrigin: '0 0',
  willChange: 'transform'
}))

// Image uses natural dimensions, wrapper handles transform
const imageStyle = computed(() => ({
  width: props.map.width_px + 'px',
  height: props.map.height_px + 'px'
}))

// SVG overlay matches image dimensions exactly, wrapper handles transform
const gridOverlayStyle = computed(() => ({
  width: props.map.width_px + 'px',
  height: props.map.height_px + 'px'
}))

// Hex points for pattern
const hexPoints = computed(() => {
  const size = displayGridSize.value
  const h = size * 0.866
  return `${size * 0.5},0 ${size},${h * 0.5} ${size},${h * 1.5} ${size * 0.5},${h * 2} 0,${h * 1.5} 0,${h * 0.5}`
})

// Generate explicit grid lines to avoid pattern tiling rounding errors
const verticalGridLines = computed(() => {
  const lines: number[] = []
  const gridSize = gridSizePx.value
  const offset = gridOffsetX.value
  const width = props.map.width_px

  // Start from offset and go in both directions
  for (let x = offset; x <= width; x += gridSize) {
    lines.push(x)
  }
  for (let x = offset - gridSize; x >= 0; x -= gridSize) {
    lines.push(x)
  }
  return lines.sort((a, b) => a - b)
})

const horizontalGridLines = computed(() => {
  const lines: number[] = []
  const gridSize = gridSizePx.value
  const offset = gridOffsetY.value
  const height = props.map.height_px

  // Start from offset and go in both directions
  for (let y = offset; y <= height; y += gridSize) {
    lines.push(y)
  }
  for (let y = offset - gridSize; y >= 0; y -= gridSize) {
    lines.push(y)
  }
  return lines.sort((a, b) => a - b)
})

// Watchers
watch(() => props.map, (newMap) => {
  gridType.value = newMap.grid_type || 'none'
  gridSizePx.value = newMap.grid_size_px || 70
  gridOffsetX.value = newMap.grid_offset_x || 0
  gridOffsetY.value = newMap.grid_offset_y || 0
  imageLoaded.value = false
  editorMode.value = 'position'
  resetView()
}, { immediate: true })

watch(() => props.visible, async (visible) => {
  if (visible && props.map.id) {
    await loadMapImage()
  }
})

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

function onImageLoad() {
  imageLoaded.value = true
  resetView()
}

function onGridTypeChange() {
  editorMode.value = 'position'
  resetView()
}

function enterPositionMode() {
  editorMode.value = 'position'
}

function enterGridMode() {
  editorMode.value = 'grid'
}

// Zoom functions
function zoomIn() {
  zoom.value = Math.min(zoom.value * 1.5, 4)
}

function zoomOut() {
  zoom.value = Math.max(zoom.value / 1.5, 0.5)
}

function resetView() {
  zoom.value = 1
  panX.value = 0
  panY.value = 0
}

function onWheel(event: WheelEvent) {
  if (editorMode.value !== 'position') return

  const delta = event.deltaY > 0 ? 0.9 : 1.1
  zoom.value = Math.max(0.5, Math.min(4, zoom.value * delta))
}

// Mouse handlers
function onMouseDown(event: MouseEvent) {
  isDragging.value = true
  dragStartX.value = event.clientX
  dragStartY.value = event.clientY

  if (editorMode.value === 'position') {
    dragStartPanX.value = panX.value
    dragStartPanY.value = panY.value
  } else {
    // Grid mode - set offset immediately on click
    updateGridOffset(event)
    dragStartOffsetX.value = gridOffsetX.value
    dragStartOffsetY.value = gridOffsetY.value
  }
}

function onMouseMove(event: MouseEvent) {
  if (!isDragging.value) return

  if (editorMode.value === 'position') {
    // Pan the image
    const deltaX = event.clientX - dragStartX.value
    const deltaY = event.clientY - dragStartY.value
    panX.value = dragStartPanX.value + deltaX
    panY.value = dragStartPanY.value + deltaY
  } else {
    // Update grid offset
    updateGridOffset(event)
  }
}

function onMouseUp() {
  isDragging.value = false
}

function updateGridOffset(event: MouseEvent) {
  const target = event.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()

  // Mouse position relative to viewport
  const mouseX = event.clientX - rect.left
  const mouseY = event.clientY - rect.top

  // Convert to image coordinates (accounting for pan and combined scale)
  const imageX = (mouseX - panX.value) / combinedScale.value
  const imageY = (mouseY - panY.value) / combinedScale.value

  // Set offset (clamped to image bounds)
  gridOffsetX.value = Math.round(Math.max(0, Math.min(imageX, props.map.width_px)))
  gridOffsetY.value = Math.round(Math.max(0, Math.min(imageY, props.map.height_px)))
}

function adjustGridSize(delta: number) {
  const newSize = gridSizePx.value + delta
  if (newSize >= 10 && newSize <= 500) {
    gridSizePx.value = newSize
  }
}

function resetOffset() {
  gridOffsetX.value = 0
  gridOffsetY.value = 0
}

async function handleSave() {
  saving.value = true
  try {
    const response = await invoke<{ success: boolean; error?: string }>('update_map_grid', {
      id: props.map.id,
      gridType: gridType.value,
      gridSizePx: gridType.value !== 'none' ? gridSizePx.value : null,
      offsetX: gridOffsetX.value,
      offsetY: gridOffsetY.value
    })
    if (response.success) {
      emit('saved')
    } else {
      alert(`Failed to save: ${response.error}`)
    }
  } catch (e) {
    console.error('Failed to save grid settings:', e)
    alert('Failed to save grid settings')
  } finally {
    saving.value = false
  }
}

function handleClose() {
  if (!saving.value) {
    emit('close')
  }
}

onMounted(() => {
  if (props.visible && props.map.id) {
    loadMapImage()
  }
})
</script>

<style scoped>
/* Grid editor form styles */
.form-group {
  margin-bottom: var(--spacing-md);
}

.form-group label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: var(--spacing-xs);
}

.radio-group {
  display: flex;
  gap: var(--spacing-md);
}

.radio-option {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  cursor: pointer;
  font-size: 0.875rem;
}

/* Step Indicator */
.step-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
  padding: var(--spacing-sm);
  background: var(--color-base-100);
  border-radius: var(--radius-md);
}

.step {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  opacity: 0.5;
  transition: all 0.2s;
}

.step.active {
  opacity: 1;
  background: var(--color-primary-100);
  color: var(--color-primary-700);
}

.step-num {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--color-base-300);
  font-size: 0.75rem;
  font-weight: 600;
}

.step.active .step-num {
  background: var(--color-primary-500);
  color: white;
}

.step-label {
  font-size: 0.875rem;
  font-weight: 500;
}

.step-arrow {
  color: var(--color-text-muted);
}

/* Mode Controls */
.mode-controls {
  margin-bottom: var(--spacing-md);
}

.zoom-controls, .grid-controls {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
}

.control-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.control-group label {
  font-size: 0.75rem;
  color: var(--color-text-muted);
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
  transition: all 0.15s;
}

.ctrl-btn:hover:not(:disabled) {
  background: var(--color-base-200);
  border-color: var(--color-primary-500);
}

.ctrl-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.size-input-group {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.size-input {
  width: 70px;
  padding: var(--spacing-xs);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  text-align: center;
  font-size: 0.875rem;
}

.offset-display {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-base-200);
  border-radius: var(--radius-sm);
  font-family: monospace;
  font-size: 0.875rem;
}

.reset-btn {
  margin-left: auto;
}

.mode-hint {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-top: var(--spacing-sm);
  margin-bottom: 0;
}

/* Preview Container */
.preview-container {
  background: #1a1a1a;
  border-radius: var(--radius-md);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-viewport {
  position: relative;
  background: #222;
}

.map-wrapper {
  position: relative;
}

.preview-image {
  display: block;
  user-select: none;
  -webkit-user-drag: none;
}

.loading-preview {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 300px;
  color: var(--color-text-muted);
}

.grid-overlay {
  position: absolute;
  top: 0;
  left: 0;
  pointer-events: none;
  overflow: visible;
}

/* Mode Toggle */
.mode-toggle {
  display: flex;
  justify-content: center;
  margin-top: var(--spacing-md);
}

/* Grid Calculator */
.calculator-section {
  margin-top: var(--spacing-md);
}

.calculator-section details {
  border: 1px solid var(--color-primary-200);
  border-radius: var(--radius-md);
  background: var(--color-primary-50);
}

.calculator-section summary {
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-primary-700);
}

.calculator-content {
  padding: var(--spacing-md);
  padding-top: 0;
}

.calculator-info {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin: 0 0 var(--spacing-sm) 0;
}

.calculator-row {
  display: flex;
  align-items: flex-end;
  gap: var(--spacing-md);
}

.calculator-row .form-group {
  flex: 1;
  margin-bottom: 0;
}

.calculator-result {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm);
  background: var(--color-background);
  border-radius: var(--radius-sm);
}

.result-label {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.result-value {
  font-weight: 600;
  font-family: monospace;
  color: var(--color-primary-600);
}

.btn-apply {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.75rem;
  font-weight: 500;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--color-primary-500);
  color: white;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-apply:hover {
  background: var(--color-primary-600);
}

/* Manual Offset */
.manual-offset-section {
  margin-top: var(--spacing-md);
}

.manual-offset-section details {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.manual-offset-section summary {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-base-200);
  cursor: pointer;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.manual-offset-section .form-row {
  padding: var(--spacing-md);
}

.form-row {
  display: flex;
  gap: var(--spacing-md);
}

.form-row .form-group {
  flex: 1;
  margin-bottom: 0;
}

.form-input {
  width: 100%;
  padding: var(--spacing-sm);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  background: var(--color-background);
}
</style>
