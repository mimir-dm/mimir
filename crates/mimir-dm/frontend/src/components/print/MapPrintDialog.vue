<template>
  <AppModal
    :visible="visible"
    title="Print Map"
    size="md"
    @close="handleClose"
  >
    <div class="map-print-dialog">
      <!-- Map Info -->
      <div class="map-info" v-if="mapName">
        <h3 class="map-name">{{ mapName }}</h3>
        <p class="map-dimensions" v-if="mapDimensions">
          {{ mapDimensions.width }} x {{ mapDimensions.height }} px
        </p>
      </div>

      <!-- Print Mode -->
      <div class="option-section">
        <label class="section-label">Print Mode</label>
        <div class="mode-selector">
          <button
            type="button"
            class="mode-btn"
            :class="{ active: options.mode === 'preview' }"
            @click="options.mode = 'preview'"
          >
            <span class="mode-icon">&#128196;</span>
            <span class="mode-label">Preview</span>
            <span class="mode-desc">Fit to single page</span>
          </button>
          <button
            type="button"
            class="mode-btn"
            :class="{ active: options.mode === 'play' }"
            @click="options.mode = 'play'"
          >
            <span class="mode-icon">&#127922;</span>
            <span class="mode-label">Play</span>
            <span class="mode-desc">1" = 5ft scale (tiled)</span>
          </button>
        </div>
      </div>

      <!-- Overlay Options (both modes) -->
      <div class="option-section">
        <label class="section-label">Overlays</label>
        <div class="checkbox-group">
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.show_grid" />
            <span class="checkbox-label">Grid Overlay</span>
            <span class="checkbox-desc">Show grid lines on the map</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.show_los_walls" />
            <span class="checkbox-label">LOS Walls</span>
            <span class="checkbox-desc">Show line-of-sight blocking walls</span>
          </label>
          <!-- Starting Positions: Preview mode only -->
          <label v-if="options.mode === 'preview'" class="checkbox-option">
            <input type="checkbox" v-model="options.show_positions" />
            <span class="checkbox-label">Starting Positions</span>
            <span class="checkbox-desc">Show numbered position markers for tokens</span>
          </label>
        </div>
      </div>

      <!-- Token Cutouts: Play mode only -->
      <div v-if="options.mode === 'play'" class="option-section">
        <label class="section-label">Extras</label>
        <div class="checkbox-group">
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.include_cutouts" />
            <span class="checkbox-label">Token Cutouts</span>
            <span class="checkbox-desc">Include printable paper standee cutouts</span>
          </label>
        </div>
      </div>

      <!-- Page Count Estimate (for Play mode) -->
      <div v-if="options.mode === 'play' && estimatedPages > 1" class="page-estimate">
        <span class="estimate-label">Estimated pages:</span>
        <span class="estimate-value">{{ estimatedPages }}</span>
      </div>

      <!-- Error Message -->
      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <template #footer>
      <button
        @click="handleClose"
        class="btn btn-secondary"
        :disabled="isLoading"
      >
        Cancel
      </button>
      <button
        @click="handlePrint"
        class="btn btn-primary"
        :disabled="isLoading || !mapId"
      >
        <span v-if="isLoading" class="spinner-sm"></span>
        {{ isLoading ? 'Generating...' : 'Generate PDF' }}
      </button>
    </template>
  </AppModal>

  <!-- PDF Preview Modal -->
  <PdfPreviewModal
    ref="pdfPreviewRef"
    :visible="showPreview"
    :title="`Map: ${mapName}`"
    :default-file-name="`${mapName || 'map'}.pdf`"
    @close="showPreview = false"
    @retry="handlePrint"
  />
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import AppModal from '@/components/shared/AppModal.vue'
import PdfPreviewModal from './PdfPreviewModal.vue'
import { PrintService, type MapPrintOptions as PrintOptions } from '../../services/PrintService'

interface Props {
  visible: boolean
  mapId: number | null
  mapName?: string
  mapDimensions?: { width: number; height: number }
  gridSizePx?: number
}

const props = withDefaults(defineProps<Props>(), {
  mapName: '',
  gridSizePx: 70
})

const emit = defineEmits<{
  close: []
}>()

// State
const isLoading = ref(false)
const error = ref<string | null>(null)
const showPreview = ref(false)
const pdfPreviewRef = ref<InstanceType<typeof PdfPreviewModal> | null>(null)

// Options
const options = reactive<PrintOptions>({
  mode: 'preview',
  show_grid: true,
  show_los_walls: false,
  show_positions: false,
  include_cutouts: false
})

// Estimated page count for Play mode (1" = 5ft = 70px typical)
const estimatedPages = computed(() => {
  if (options.mode !== 'play' || !props.mapDimensions || !props.gridSizePx) {
    return 1
  }

  // Letter size printable area: ~7.5" x 10" (landscape)
  const printableWidthIn = 10
  const printableHeightIn = 7.5

  // Map size in inches at 1"=5ft scale
  const gridWidthCells = props.mapDimensions.width / props.gridSizePx
  const gridHeightCells = props.mapDimensions.height / props.gridSizePx
  const mapWidthIn = gridWidthCells / 5 // 5ft per inch
  const mapHeightIn = gridHeightCells / 5

  // Calculate pages needed
  const pagesWide = Math.ceil(mapWidthIn / printableWidthIn)
  const pagesHigh = Math.ceil(mapHeightIn / printableHeightIn)

  return pagesWide * pagesHigh
})

// Reset options when dialog opens
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    error.value = null
    options.mode = 'preview'
    options.show_grid = true
    options.show_los_walls = false
    options.show_positions = false
    options.include_cutouts = false
  }
})

function handleClose() {
  if (!isLoading.value) {
    emit('close')
  }
}

async function handlePrint() {
  if (!props.mapId) {
    error.value = 'No map selected'
    return
  }

  isLoading.value = true
  error.value = null

  try {
    // Show preview modal and set loading state
    showPreview.value = true
    pdfPreviewRef.value?.setLoading(true)

    // Generate PDF
    const result = await PrintService.printMap(props.mapId, options)

    // Display result
    pdfPreviewRef.value?.setPdfResult(result)

    // Close this dialog
    emit('close')
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : 'Failed to generate PDF'
    error.value = errorMessage
    pdfPreviewRef.value?.setError(errorMessage)
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.map-print-dialog {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.map-info {
  text-align: center;
  padding-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
}

.map-name {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
}

.map-dimensions {
  margin: var(--spacing-xs) 0 0;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.option-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.section-label {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.mode-selector {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-md);
}

.mode-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-md);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  cursor: pointer;
  transition: all 0.2s ease;
}

.mode-btn:hover:not(.disabled) {
  border-color: var(--color-primary-400);
  background: var(--color-surface-variant);
}

.mode-btn.active {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50);
}

.theme-dark .mode-btn.active {
  background: var(--color-primary-900);
}

.mode-btn.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.mode-icon {
  font-size: 1.5rem;
}

.mode-label {
  font-weight: 600;
  color: var(--color-text);
}

.mode-desc {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-align: center;
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.checkbox-option {
  display: grid;
  grid-template-columns: auto 1fr;
  grid-template-rows: auto auto;
  gap: 0 var(--spacing-sm);
  align-items: start;
  cursor: pointer;
  padding: var(--spacing-sm);
  border-radius: var(--radius-sm);
  transition: background 0.15s ease;
}

.checkbox-option:hover:not(.disabled) {
  background: var(--color-surface-variant);
}

.checkbox-option.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.checkbox-option input[type="checkbox"] {
  grid-row: span 2;
  margin-top: 2px;
  width: 16px;
  height: 16px;
  cursor: inherit;
}

.checkbox-label {
  font-weight: 500;
  color: var(--color-text);
}

.checkbox-desc {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.page-estimate {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-warning-50);
  border-radius: var(--radius-sm);
  color: var(--color-warning-700);
}

.theme-dark .page-estimate {
  background: var(--color-warning-900);
  color: var(--color-warning-300);
}

.estimate-label {
  font-size: 0.875rem;
}

.estimate-value {
  font-weight: 600;
}

.error-message {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-error-50);
  border-radius: var(--radius-sm);
  color: var(--color-error-700);
  font-size: 0.875rem;
}

.theme-dark .error-message {
  background: var(--color-error-900);
  color: var(--color-error-300);
}

.spinner-sm {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-right: var(--spacing-xs);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
