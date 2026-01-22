<template>
  <AppModal
    :visible="visible"
    title="Export Campaign"
    size="md"
    @close="handleClose"
  >
    <div class="export-dialog">
      <!-- Campaign Info -->
      <div class="campaign-info" v-if="campaignName">
        <h3 class="campaign-name">{{ campaignName }}</h3>
      </div>

      <!-- Reference Document Section -->
      <div class="option-section">
        <label class="section-label">Reference Document</label>
        <div class="checkbox-group">
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeCampaignDocs" />
            <span class="checkbox-label">Campaign Documents</span>
            <span class="checkbox-desc">Campaign-level planning and world documents</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeModuleContent" />
            <span class="checkbox-label">Module Content</span>
            <span class="checkbox-desc">Module documents and monster stat blocks</span>
          </label>
          <!-- Module Maps - nested under Module Content -->
          <div v-if="options.includeModuleContent" class="nested-options">
            <span class="nested-label">Module Maps</span>
            <label class="checkbox-option sub-option">
              <input type="checkbox" v-model="options.includeModuleMapPreviews" />
              <span class="checkbox-label">Map Previews</span>
              <span class="checkbox-desc">Maps scaled to fit one page</span>
            </label>
            <label class="checkbox-option sub-option">
              <input type="checkbox" v-model="options.includeModuleTiledMaps" />
              <span class="checkbox-label">Tiled Maps</span>
              <span class="checkbox-desc">At 1"=5ft scale for tabletop play</span>
            </label>
            <label class="checkbox-option sub-option nested" :class="{ disabled: !options.includeModuleTiledMaps }">
              <input
                type="checkbox"
                v-model="options.includeTokenCutouts"
                :disabled="!options.includeModuleTiledMaps"
              />
              <span class="checkbox-label">Token Cutouts</span>
              <span class="checkbox-desc">Printable paper standees</span>
            </label>
          </div>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeNpcs" />
            <span class="checkbox-label">NPCs</span>
            <span class="checkbox-desc">Campaign NPC sheets</span>
          </label>
        </div>
      </div>

      <!-- Campaign Maps Section -->
      <div class="option-section">
        <label class="section-label">Campaign Maps</label>
        <span class="section-hint">Regional and world maps</span>
        <div class="checkbox-group">
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeCampaignMapPreviews" />
            <span class="checkbox-label">Map Previews</span>
            <span class="checkbox-desc">Maps scaled to fit one page</span>
          </label>
          <label class="checkbox-option">
            <input type="checkbox" v-model="options.includeCampaignTiledMaps" />
            <span class="checkbox-label">Tiled Maps</span>
            <span class="checkbox-desc">Maps at 1"=5ft scale for tabletop play</span>
          </label>
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
        :disabled="isLoading || !campaignId || !hasAnySelection"
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
    :title="`Campaign: ${campaignName}`"
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
  campaignId: string | null
  campaignName?: string
}

const props = withDefaults(defineProps<Props>(), {
  campaignName: ''
})

const emit = defineEmits<{
  close: []
}>()

// State
const isLoading = ref(false)
const error = ref<string | null>(null)
const showPreview = ref(false)
const pdfPreviewRef = ref<InstanceType<typeof PdfPreviewModal> | null>(null)

// Options with defaults
const options = reactive({
  // Reference Document
  includeCampaignDocs: true,
  includeModuleContent: true,
  includeNpcs: false,
  // Module Maps
  includeModuleMapPreviews: false,
  includeModuleTiledMaps: false,
  includeTokenCutouts: false,
  // Campaign Maps
  includeCampaignMapPreviews: true,
  includeCampaignTiledMaps: false,
})

// Computed
const hasAnySelection = computed(() => {
  return options.includeCampaignDocs ||
    options.includeModuleContent ||
    options.includeNpcs ||
    options.includeModuleMapPreviews ||
    options.includeModuleTiledMaps ||
    options.includeCampaignMapPreviews ||
    options.includeCampaignTiledMaps
})

const defaultFileName = computed(() => {
  const name = props.campaignName || 'campaign'
  const safeName = name.replace(/[^a-z0-9\s\-_.]/gi, '').replace(/\s+/g, '_')
  return `${safeName}_export.pdf`
})

// Auto-disable token cutouts when tiled maps is unchecked
watch(() => options.includeModuleTiledMaps, (newValue) => {
  if (!newValue) {
    options.includeTokenCutouts = false
  }
})

// Reset options when dialog opens
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    error.value = null
    // Reset to defaults
    options.includeCampaignDocs = true
    options.includeModuleContent = true
    options.includeNpcs = false
    options.includeModuleMapPreviews = false
    options.includeModuleTiledMaps = false
    options.includeTokenCutouts = false
    options.includeCampaignMapPreviews = true
    options.includeCampaignTiledMaps = false
  }
})

function handleClose() {
  if (!isLoading.value) {
    emit('close')
  }
}

async function handleExport() {
  if (!props.campaignId || !hasAnySelection.value) {
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
    const result = await PrintService.exportCampaignDocuments(props.campaignId, {
      include_campaign_docs: options.includeCampaignDocs,
      include_module_content: options.includeModuleContent,
      include_npcs: options.includeNpcs,
      include_module_map_previews: options.includeModuleMapPreviews,
      include_module_tiled_maps: options.includeModuleTiledMaps,
      include_token_cutouts: options.includeTokenCutouts,
      include_campaign_map_previews: options.includeCampaignMapPreviews,
      include_campaign_tiled_maps: options.includeCampaignTiledMaps,
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

.campaign-info {
  text-align: center;
  padding-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
}

.campaign-name {
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

.section-hint {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin-top: -4px;
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

.nested-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--spacing-xs);
}

.checkbox-option.sub-option {
  padding: var(--spacing-xs) var(--spacing-sm);
}

.checkbox-option.sub-option.nested {
  margin-left: var(--spacing-md);
}

.checkbox-option.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.checkbox-option.disabled:hover {
  background: transparent;
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
