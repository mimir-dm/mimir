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

      <!-- Print Sections -->
      <div class="mode-options">
        <!-- Preview Section -->
        <div class="mode-card" :class="{ active: options.include_preview }">
          <label class="mode-header" @click.prevent="options.include_preview = !options.include_preview">
            <input type="checkbox" v-model="options.include_preview" @click.stop />
            <span class="mode-icon">&#128196;</span>
            <div class="mode-info">
              <span class="mode-label">Preview</span>
              <span class="mode-desc">Fit to single page</span>
            </div>
          </label>
          <div class="mode-checkboxes" v-if="options.include_preview">
            <label class="checkbox-option">
              <input type="checkbox" v-model="options.preview_grid" />
              <span class="checkbox-label">Grid</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" v-model="options.preview_los_walls" />
              <span class="checkbox-label">LOS Walls</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" v-model="options.preview_positions" />
              <span class="checkbox-label">Starting Positions</span>
            </label>
          </div>
        </div>

        <!-- Play Section -->
        <div class="mode-card" :class="{ active: options.include_play }">
          <label class="mode-header" @click.prevent="options.include_play = !options.include_play">
            <input type="checkbox" v-model="options.include_play" @click.stop />
            <span class="mode-icon">&#127922;</span>
            <div class="mode-info">
              <span class="mode-label">Play</span>
              <span class="mode-desc">1" = 5ft scale (tiled)</span>
            </div>
          </label>
          <div class="mode-checkboxes" v-if="options.include_play">
            <label class="checkbox-option">
              <input type="checkbox" v-model="options.play_grid" />
              <span class="checkbox-label">Grid</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" v-model="options.play_los_walls" />
              <span class="checkbox-label">LOS Walls</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" v-model="options.play_cutouts" />
              <span class="checkbox-label">Token Cutouts</span>
            </label>
          </div>
        </div>
      </div>

      <!-- Page Count Estimate (for Play mode) -->
      <div v-if="options.include_play && estimatedPages > 1" class="page-estimate">
        <span class="estimate-label">Estimated tile pages:</span>
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
        :disabled="isLoading || !mapId || !canGenerate"
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
import { PrintService, type MapPrintOptions } from '../../services/PrintService'

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

// Options - both sections can be included
const options = reactive({
  // Preview section
  include_preview: true,
  preview_grid: true,
  preview_los_walls: false,
  preview_positions: false,
  // Play section
  include_play: false,
  play_grid: true,
  play_los_walls: false,
  play_cutouts: true,
})

// Estimated page count for Play mode (1" = 5ft = 70px typical)
const estimatedPages = computed(() => {
  if (!options.include_play || !props.mapDimensions || !props.gridSizePx) {
    return 0
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

// Can generate if at least one section is selected
const canGenerate = computed(() => options.include_preview || options.include_play)

// Reset options when dialog opens
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    error.value = null
    // Preview section
    options.include_preview = true
    options.preview_grid = true
    options.preview_los_walls = false
    options.preview_positions = false
    // Play section
    options.include_play = false
    options.play_grid = true
    options.play_los_walls = false
    options.play_cutouts = true
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

  if (!canGenerate.value) {
    error.value = 'Select at least one section to include'
    return
  }

  isLoading.value = true
  error.value = null

  try {
    // Show preview modal and set loading state
    showPreview.value = true
    pdfPreviewRef.value?.setLoading(true)

    // Build options for backend
    const printOptions: MapPrintOptions = {
      include_preview: options.include_preview,
      preview_grid: options.preview_grid,
      preview_los_walls: options.preview_los_walls,
      preview_positions: options.preview_positions,
      include_play: options.include_play,
      play_grid: options.play_grid,
      play_los_walls: options.play_los_walls,
      play_cutouts: options.play_cutouts,
    }

    // Generate PDF
    const result = await PrintService.printMap(props.mapId, printOptions)

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

.mode-options {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.mode-card {
  border: 2px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  cursor: pointer;
  transition: all 0.2s ease;
  overflow: hidden;
}

.mode-card:hover:not(.active) {
  border-color: var(--color-primary-400);
  background: var(--color-surface-variant);
}

.mode-card.active {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50);
}

.theme-dark .mode-card.active,
.theme-hyper .mode-card.active {
  background: var(--color-primary-900);
}

.mode-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
}

.mode-icon {
  font-size: 1.5rem;
}

.mode-info {
  display: flex;
  flex-direction: column;
}

.mode-label {
  font-weight: 600;
  color: var(--color-text);
}

.mode-desc {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.mode-checkboxes {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-sm) var(--spacing-lg);
  padding: var(--spacing-sm) var(--spacing-md) var(--spacing-md);
  border-top: 1px solid var(--color-border);
  background: var(--color-surface);
}

.checkbox-option {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  cursor: pointer;
  padding: var(--spacing-xs);
  border-radius: var(--radius-sm);
  transition: background 0.15s ease;
}

.checkbox-option:hover {
  background: var(--color-surface-variant);
}

.checkbox-option input[type="checkbox"] {
  width: 16px;
  height: 16px;
  cursor: inherit;
}

.checkbox-label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text);
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
