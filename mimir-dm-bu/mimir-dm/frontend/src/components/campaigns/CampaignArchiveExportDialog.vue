<template>
  <AppModal
    :visible="visible"
    title="Export Campaign Archive"
    size="md"
    @close="handleClose"
  >
    <div class="export-dialog">
      <!-- Campaign Info -->
      <div class="campaign-info">
        <div class="campaign-label">Campaign</div>
        <div class="campaign-name">{{ campaign?.name }}</div>
      </div>

      <!-- Status Messages -->
      <div v-if="exportState === 'idle'" class="export-description">
        <p>Export this campaign as a portable archive that can be shared and imported into other Mimir instances.</p>
        <p class="export-includes">The archive will include:</p>
        <ul class="includes-list">
          <li>All campaign documents and content</li>
          <li>Maps, images, and other assets</li>
          <li>Module structure and organization</li>
        </ul>
      </div>

      <div v-if="exportState === 'exporting'" class="export-progress">
        <div class="progress-spinner"></div>
        <p>Exporting campaign...</p>
      </div>

      <div v-if="exportState === 'success'" class="export-success">
        <div class="success-icon">&#10003;</div>
        <p class="success-message">Campaign exported successfully!</p>
        <div class="archive-path">
          <span class="path-label">Saved to:</span>
          <code class="path-value">{{ exportResult?.archive_path }}</code>
        </div>
      </div>

      <div v-if="exportState === 'error'" class="export-error">
        <p class="error-message">{{ errorMessage }}</p>
      </div>

      <!-- Directory Selection (only shown before export) -->
      <div v-if="exportState === 'idle'" class="form-group">
        <label for="export-directory">Save Location</label>
        <div class="directory-input-group">
          <input
            id="export-directory"
            v-model="outputDirectory"
            type="text"
            class="form-input"
            readonly
          />
          <button type="button" class="browse-button" @click="selectDirectory">
            Browse...
          </button>
        </div>
      </div>
    </div>

    <template #footer>
      <button
        v-if="exportState === 'idle'"
        @click="handleClose"
        class="btn btn-secondary"
      >
        Cancel
      </button>
      <button
        v-if="exportState === 'idle'"
        @click="handleExport"
        class="btn btn-primary"
        :disabled="!outputDirectory"
      >
        Export
      </button>

      <button
        v-if="exportState === 'success' || exportState === 'error'"
        @click="handleClose"
        class="btn btn-primary"
      >
        Close
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import AppModal from '@/components/shared/AppModal.vue'
import { useCampaignStore } from '@/stores/campaigns'
import type { Campaign, ApiResponse } from '@/types/api'

interface Props {
  visible: boolean
  campaign: Campaign | null
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const campaignStore = useCampaignStore()

type ExportState = 'idle' | 'exporting' | 'success' | 'error'

const exportState = ref<ExportState>('idle')
const outputDirectory = ref('')
const exportResult = ref<{ archive_path: string; file_name: string } | null>(null)
const errorMessage = ref<string | null>(null)

// Load default directory when dialog opens
watch(() => props.visible, async (newVisible) => {
  if (newVisible) {
    // Reset state
    exportState.value = 'idle'
    exportResult.value = null
    errorMessage.value = null

    // Get default directory (Downloads or Documents)
    try {
      const response = await invoke<ApiResponse<string>>('get_default_campaigns_directory')
      if (response.success && response.data) {
        // Use the parent of campaigns directory as default export location
        const parts = response.data.split('/')
        parts.pop() // Remove 'Mimir Campaigns' to get parent
        outputDirectory.value = parts.join('/') || response.data
      }
    } catch {
      // Fall back to empty - user must select
    }
  }
})

async function selectDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: outputDirectory.value
    })

    if (selected && typeof selected === 'string') {
      outputDirectory.value = selected
    }
  } catch {
    // User cancelled or error
  }
}

async function handleExport() {
  if (!props.campaign || !outputDirectory.value) return

  exportState.value = 'exporting'
  errorMessage.value = null

  const result = await campaignStore.exportCampaign(props.campaign.id, outputDirectory.value)

  if (result) {
    exportResult.value = result
    exportState.value = 'success'
  } else {
    errorMessage.value = campaignStore.error || 'Failed to export campaign'
    exportState.value = 'error'
  }
}

function handleClose() {
  emit('close')
}
</script>

<style scoped>
.export-dialog {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.campaign-info {
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.campaign-label {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--spacing-xs);
}

.campaign-name {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
}

.export-description p {
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
}

.export-includes {
  font-weight: 500;
  color: var(--color-text);
}

.includes-list {
  margin: var(--spacing-sm) 0 0 var(--spacing-lg);
  color: var(--color-text-secondary);
  list-style-type: disc;
}

.includes-list li {
  margin-bottom: var(--spacing-xs);
}

.export-progress {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-xl);
  text-align: center;
}

.progress-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.export-success {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-lg);
  text-align: center;
}

.success-icon {
  width: 48px;
  height: 48px;
  background: var(--color-success-100);
  color: var(--color-success-600);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  font-weight: bold;
}

.theme-dark .success-icon {
  background: var(--color-success-900);
  color: var(--color-success-400);
}

.success-message {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-success-600);
}

.theme-dark .success-message {
  color: var(--color-success-400);
}

.archive-path {
  width: 100%;
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  text-align: left;
}

.path-label {
  display: block;
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-xs);
}

.path-value {
  display: block;
  font-size: 0.875rem;
  color: var(--color-text);
  word-break: break-all;
  background: none;
  padding: 0;
}

.export-error {
  padding: var(--spacing-lg);
  background: var(--color-error-100);
  border: 1px solid var(--color-error-200);
  border-radius: var(--radius-md);
  text-align: center;
}

.theme-dark .export-error {
  background: var(--color-error-900);
  border-color: var(--color-error-800);
}

.export-error .error-message {
  color: var(--color-error-600);
  font-weight: 500;
}

.theme-dark .export-error .error-message {
  color: var(--color-error-400);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.form-group label {
  font-weight: 500;
  color: var(--color-text);
}

.directory-input-group {
  display: flex;
  gap: var(--spacing-sm);
}

.form-input {
  flex: 1;
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 0.875rem;
}

.browse-button {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: background-color var(--transition-fast);
}

.browse-button:hover {
  background: var(--color-gray-100);
}

.theme-dark .browse-button:hover {
  background: var(--color-gray-700);
}
</style>
