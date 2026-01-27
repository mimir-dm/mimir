<template>
  <AppModal
    :visible="visible"
    title="Export Module"
    size="md"
    @close="handleClose"
  >
    <div class="export-dialog">
      <!-- Module Info -->
      <div class="module-info" v-if="moduleName">
        <h3 class="module-name">{{ moduleName }}</h3>
      </div>

      <!-- Content Section -->
      <div class="option-section">
        <label class="section-label">Content</label>
        <div class="checkbox-group">
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeDocuments" />
            <span class="checkbox-label">Documents</span>
            <span class="checkbox-desc">Module documents and notes</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeMonsters" />
            <span class="checkbox-label">Monster Stat Blocks</span>
            <span class="checkbox-desc">Full stat blocks for tagged monsters</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeTraps" />
            <span class="checkbox-label">Traps &amp; Hazards</span>
            <span class="checkbox-desc">Trap and hazard reference cards</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includePois" />
            <span class="checkbox-label">Points of Interest</span>
            <span class="checkbox-desc">Location notes and descriptions</span>
          </label>
        </div>
      </div>

      <!-- Maps Section -->
      <div class="option-section">
        <label class="section-label">Maps</label>
        <div class="checkbox-group">
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includePreview" />
            <span class="checkbox-label">Map Previews</span>
            <span class="checkbox-desc">Maps scaled to fit one page</span>
          </label>
          <!-- Preview sub-options -->
          <div v-if="options.includePreview" class="nested-options">
            <label class="checkbox-option sub-option">
              <input type="checkbox" v-model="options.previewGrid" />
              <span class="checkbox-label">Show Grid</span>
            </label>
            <label class="checkbox-option sub-option">
              <input type="checkbox" v-model="options.previewLosWalls" />
              <span class="checkbox-label">Show LOS Walls</span>
            </label>
            <label class="checkbox-option sub-option">
              <input type="checkbox" v-model="options.previewPositions" />
              <span class="checkbox-label">Show Starting Positions</span>
            </label>
          </div>

          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includePlay" />
            <span class="checkbox-label">Tiled Maps</span>
            <span class="checkbox-desc">At 1"=5ft scale for tabletop play</span>
          </label>
          <!-- Play sub-options -->
          <div v-if="options.includePlay" class="nested-options">
            <label class="checkbox-option sub-option">
              <input type="checkbox" v-model="options.playGrid" />
              <span class="checkbox-label">Show Grid</span>
            </label>
            <label class="checkbox-option sub-option">
              <input type="checkbox" v-model="options.playLosWalls" />
              <span class="checkbox-label">Show LOS Walls</span>
            </label>
            <label class="checkbox-option sub-option">
              <input type="checkbox" v-model="options.playCutouts" />
              <span class="checkbox-label">Token Cutouts</span>
              <span class="checkbox-desc">Printable paper standees</span>
            </label>
          </div>
        </div>
      </div>

      <!-- Validation Warning -->
      <div v-if="!hasAnySelection" class="warning-message">
        Select at least one option to export.
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
        @click="handleExport"
        class="btn btn-primary"
        :disabled="isLoading || !moduleId || !hasAnySelection"
      >
        <span v-if="isLoading" class="spinner-sm"></span>
        {{ isLoading ? 'Generating...' : 'Export PDF' }}
      </button>
    </template>
  </AppModal>

  <!-- PDF Preview Modal -->
  <PdfPreviewModal
    ref="pdfPreviewRef"
    :visible="showPreview"
    :title="`Module: ${moduleName}`"
    :default-file-name="defaultFileName"
    @close="showPreview = false"
    @retry="handleExport"
  />
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import AppModal from '@/components/shared/AppModal.vue'
import PdfPreviewModal from './PdfPreviewModal.vue'
import { PrintService } from '../../services/PrintService'

interface Props {
  visible: boolean
  moduleId: string | null
  moduleName?: string
  moduleNumber?: number
  campaignId?: string
}

const props = withDefaults(defineProps<Props>(), {
  moduleName: '',
  moduleNumber: 1
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
const options = reactive({
  // Content section
  includeDocuments: true,
  includeMonsters: true,
  includeTraps: true,
  includePois: true,
  // Map Preview section
  includePreview: true,
  previewGrid: true,
  previewLosWalls: false,
  previewPositions: false,
  // Map Play section
  includePlay: false,
  playGrid: true,
  playLosWalls: false,
  playCutouts: true,
})

// Computed
const hasAnySelection = computed(() => {
  return options.includeDocuments ||
    options.includeMonsters ||
    options.includeTraps ||
    options.includePois ||
    options.includePreview ||
    options.includePlay
})

const defaultFileName = computed(() => {
  const name = props.moduleName || 'module'
  const safeName = name.replace(/[^a-z0-9\s\-_.]/gi, '').replace(/\s+/g, '_')
  return `Module_${props.moduleNumber}_${safeName}.pdf`
})

// Reset options when dialog opens
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    error.value = null
    // Reset to defaults
    options.includeDocuments = true
    options.includeMonsters = true
    options.includeTraps = true
    options.includePois = true
    options.includePreview = true
    options.previewGrid = true
    options.previewLosWalls = false
    options.previewPositions = false
    options.includePlay = false
    options.playGrid = true
    options.playLosWalls = false
    options.playCutouts = true
  }
})

function handleClose() {
  if (!isLoading.value) {
    emit('close')
  }
}

async function handleExport() {
  if (!props.moduleId || !hasAnySelection.value) {
    error.value = 'Select at least one option to export'
    return
  }

  isLoading.value = true
  error.value = null

  try {
    // Show preview modal and set loading state
    showPreview.value = true
    pdfPreviewRef.value?.setLoading(true)

    // Generate PDF with options
    const result = await PrintService.exportModuleDocuments(props.moduleId, {
      include_documents: options.includeDocuments,
      include_monsters: options.includeMonsters,
      include_traps: options.includeTraps,
      include_pois: options.includePois,
      // Map options
      include_preview: options.includePreview,
      preview_grid: options.previewGrid,
      preview_los_walls: options.previewLosWalls,
      preview_positions: options.previewPositions,
      include_play: options.includePlay,
      play_grid: options.playGrid,
      play_los_walls: options.playLosWalls,
      play_cutouts: options.playCutouts,
    })

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
.export-dialog {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.module-info {
  text-align: center;
  padding-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
}

.module-name {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
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

.checkbox-option:hover {
  background: var(--color-surface-variant);
}

.nested-options {
  margin-left: var(--spacing-lg);
  padding-left: var(--spacing-md);
  border-left: 2px solid var(--color-border);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.checkbox-option.sub-option {
  padding: var(--spacing-xs) var(--spacing-sm);
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

.warning-message {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-warning-50);
  border-radius: var(--radius-sm);
  color: var(--color-warning-700);
  font-size: 0.875rem;
}

.theme-dark .warning-message {
  background: var(--color-warning-900);
  color: var(--color-warning-300);
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
