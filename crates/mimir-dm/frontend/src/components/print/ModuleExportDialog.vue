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
      <div class="section-group">
        <div class="section-header">Content</div>
        <div class="mode-card" :class="{ active: options.includeDocuments }">
          <label class="mode-header" @click.prevent="options.includeDocuments = !options.includeDocuments">
            <input type="checkbox" v-model="options.includeDocuments" @click.stop />
            <span class="mode-icon">&#128196;</span>
            <div class="mode-info">
              <span class="mode-label">Documents</span>
              <span class="mode-desc">Module documents and notes</span>
            </div>
          </label>
          <div class="mode-checkboxes" v-if="options.includeDocuments">
            <label class="checkbox-option">
              <input type="checkbox" v-model="options.includeMonsters" />
              <span class="checkbox-label">Monsters</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" v-model="options.includeNpcs" />
              <span class="checkbox-label">NPCs</span>
            </label>
          </div>
        </div>
      </div>

      <!-- Maps Section -->
      <div class="section-group">
        <div class="section-header">Maps</div>
        <div class="mode-options">
          <!-- Preview Section -->
          <div class="mode-card" :class="{ active: options.includePreview }">
            <label class="mode-header" @click.prevent="options.includePreview = !options.includePreview">
              <input type="checkbox" v-model="options.includePreview" @click.stop />
              <span class="mode-icon">&#128196;</span>
              <div class="mode-info">
                <span class="mode-label">Preview</span>
                <span class="mode-desc">Fit to single page</span>
              </div>
            </label>
            <div class="mode-checkboxes" v-if="options.includePreview">
              <label class="checkbox-option">
                <input type="checkbox" v-model="options.previewGrid" />
                <span class="checkbox-label">Grid</span>
              </label>
              <label class="checkbox-option">
                <input type="checkbox" v-model="options.previewLosWalls" />
                <span class="checkbox-label">LOS Walls</span>
              </label>
              <label class="checkbox-option">
                <input type="checkbox" v-model="options.previewPositions" />
                <span class="checkbox-label">Starting Positions</span>
              </label>
            </div>
          </div>

          <!-- Play Section -->
          <div class="mode-card" :class="{ active: options.includePlay }">
            <label class="mode-header" @click.prevent="options.includePlay = !options.includePlay">
              <input type="checkbox" v-model="options.includePlay" @click.stop />
              <span class="mode-icon">&#127922;</span>
              <div class="mode-info">
                <span class="mode-label">Play</span>
                <span class="mode-desc">1" = 5ft scale (tiled)</span>
              </div>
            </label>
            <div class="mode-checkboxes" v-if="options.includePlay">
              <label class="checkbox-option">
                <input type="checkbox" v-model="options.playGrid" />
                <span class="checkbox-label">Grid</span>
              </label>
              <label class="checkbox-option">
                <input type="checkbox" v-model="options.playLosWalls" />
                <span class="checkbox-label">LOS Walls</span>
              </label>
              <label class="checkbox-option">
                <input type="checkbox" v-model="options.playCutouts" />
                <span class="checkbox-label">Token Cutouts</span>
              </label>
            </div>
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
  moduleId: number | null
  moduleName?: string
  moduleNumber?: number
  campaignId?: number
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
  includeNpcs: false,
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
  return options.includeDocuments || options.includePreview || options.includePlay
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
    // Content section
    options.includeDocuments = true
    options.includeMonsters = true
    options.includeNpcs = false
    // Map Preview section
    options.includePreview = true
    options.previewGrid = true
    options.previewLosWalls = false
    options.previewPositions = false
    // Map Play section
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
      include_monsters: options.includeDocuments && options.includeMonsters,
      include_npcs: options.includeDocuments && options.includeNpcs,
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

.section-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.section-header {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
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
