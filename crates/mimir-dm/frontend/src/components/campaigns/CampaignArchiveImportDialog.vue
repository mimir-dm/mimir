<template>
  <AppModal
    :visible="visible"
    title="Import Campaign Archive"
    size="lg"
    @close="handleClose"
  >
    <div class="import-dialog">
      <!-- File Selection -->
      <div v-if="importState === 'idle' || importState === 'previewing'" class="file-selection">
        <div class="form-group">
          <label for="archive-file">Campaign Archive File</label>
          <div class="directory-input-group">
            <input
              id="archive-file"
              v-model="archivePath"
              type="text"
              class="form-input"
              placeholder="Select a .mimir-campaign.tar.gz file"
              readonly
            />
            <button type="button" class="browse-button" @click="selectArchive">
              Browse...
            </button>
          </div>
        </div>
      </div>

      <!-- Loading Preview -->
      <div v-if="importState === 'previewing'" class="preview-loading">
        <div class="progress-spinner"></div>
        <p>Reading archive...</p>
      </div>

      <!-- Archive Preview -->
      <div v-if="importState === 'previewed' && archivePreview" class="archive-preview">
        <div class="preview-header">
          <div class="preview-icon">&#128230;</div>
          <div class="preview-title">Archive Contents</div>
        </div>

        <div class="preview-stats">
          <div class="stat-item">
            <span class="stat-value">{{ archivePreview.file_count }}</span>
            <span class="stat-label">Documents</span>
          </div>
          <div class="stat-item">
            <span class="stat-value">{{ archivePreview.asset_count }}</span>
            <span class="stat-label">Assets</span>
          </div>
        </div>

        <div class="preview-meta">
          <div class="meta-item">
            <span class="meta-label">Created with:</span>
            <span class="meta-value">Mimir {{ archivePreview.mimir_version }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">Export date:</span>
            <span class="meta-value">{{ formatDate(archivePreview.created_at) }}</span>
          </div>
        </div>

        <!-- Catalog References -->
        <div v-if="archivePreview.catalog_references.length > 0" class="catalog-refs">
          <div class="refs-header">
            <span class="refs-title">Catalog References</span>
            <span class="refs-count">{{ archivePreview.catalog_references.length }}</span>
          </div>
          <div class="refs-note">
            These items are referenced from source books. Ensure you have imported the necessary books.
          </div>
          <div class="refs-list">
            <div
              v-for="(ref, index) in visibleRefs"
              :key="index"
              class="ref-item"
            >
              <span class="ref-type">{{ ref.type }}</span>
              <span class="ref-name">{{ ref.name }}</span>
              <span class="ref-source">{{ ref.source }}</span>
            </div>
            <button
              v-if="archivePreview.catalog_references.length > 5 && !showAllRefs"
              class="show-more-btn"
              @click="showAllRefs = true"
            >
              Show {{ archivePreview.catalog_references.length - 5 }} more...
            </button>
          </div>
        </div>

        <!-- Campaign Name Input -->
        <div class="form-group">
          <label for="campaign-name">Campaign Name</label>
          <input
            id="campaign-name"
            v-model="campaignName"
            type="text"
            class="form-input"
            placeholder="Enter campaign name"
          />
        </div>
      </div>

      <!-- Importing State -->
      <div v-if="importState === 'importing'" class="import-progress">
        <div class="progress-spinner"></div>
        <p>Importing campaign...</p>
      </div>

      <!-- Success State -->
      <div v-if="importState === 'success'" class="import-success">
        <div class="success-icon">&#10003;</div>
        <p class="success-message">Campaign imported successfully!</p>
        <p class="success-detail">{{ importedCampaign?.name }} is now ready to use.</p>
      </div>

      <!-- Error State -->
      <div v-if="importState === 'error'" class="import-error">
        <p class="error-message">{{ errorMessage }}</p>
      </div>
    </div>

    <template #footer>
      <button
        v-if="importState === 'idle' || importState === 'previewing' || importState === 'previewed'"
        @click="handleClose"
        class="btn btn-secondary"
      >
        Cancel
      </button>
      <button
        v-if="importState === 'previewed'"
        @click="handleImport"
        class="btn btn-primary"
        :disabled="!campaignName.trim()"
      >
        Import Campaign
      </button>

      <button
        v-if="importState === 'success'"
        @click="handleOpenCampaign"
        class="btn btn-primary"
      >
        Open Campaign
      </button>
      <button
        v-if="importState === 'success'"
        @click="handleClose"
        class="btn btn-secondary"
      >
        Close
      </button>

      <button
        v-if="importState === 'error'"
        @click="resetState"
        class="btn btn-secondary"
      >
        Try Again
      </button>
      <button
        v-if="importState === 'error'"
        @click="handleClose"
        class="btn btn-primary"
      >
        Close
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import AppModal from '@/components/shared/AppModal.vue'
import { useCampaignStore } from '@/stores/campaigns'
import type { Campaign, ApiResponse } from '@/types/api'

interface Props {
  visible: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'imported', campaign: Campaign): void
}

interface ArchivePreview {
  campaign_name: string
  file_count: number
  asset_count: number
  catalog_references: Array<{ type: string; name: string; source: string }>
  mimir_version: string
  created_at: string
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const router = useRouter()
const campaignStore = useCampaignStore()

type ImportState = 'idle' | 'previewing' | 'previewed' | 'importing' | 'success' | 'error'

const importState = ref<ImportState>('idle')
const archivePath = ref('')
const archivePreview = ref<ArchivePreview | null>(null)
const campaignName = ref('')
const campaignsDirectory = ref('')
const showAllRefs = ref(false)
const importedCampaign = ref<Campaign | null>(null)
const errorMessage = ref<string | null>(null)

const visibleRefs = computed(() => {
  if (!archivePreview.value) return []
  if (showAllRefs.value) return archivePreview.value.catalog_references
  return archivePreview.value.catalog_references.slice(0, 5)
})

// Reset state when dialog opens
watch(() => props.visible, async (newVisible) => {
  if (newVisible) {
    resetState()

    // Get default campaigns directory
    try {
      const response = await invoke<ApiResponse<string>>('get_default_campaigns_directory')
      if (response.success && response.data) {
        campaignsDirectory.value = response.data
      }
    } catch {
      // Fall back - will error on import
    }
  }
})

function resetState() {
  importState.value = 'idle'
  archivePath.value = ''
  archivePreview.value = null
  campaignName.value = ''
  showAllRefs.value = false
  importedCampaign.value = null
  errorMessage.value = null
}

async function selectArchive() {
  try {
    const extension = await campaignStore.getArchiveExtension()
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Mimir Campaign Archive',
        extensions: [extension.replace(/^\./, '')]
      }]
    })

    if (selected && typeof selected === 'string') {
      archivePath.value = selected
      await previewArchive()
    }
  } catch {
    // User cancelled or error
  }
}

async function previewArchive() {
  if (!archivePath.value) return

  importState.value = 'previewing'
  errorMessage.value = null

  const preview = await campaignStore.previewArchive(archivePath.value)

  if (preview) {
    archivePreview.value = preview
    campaignName.value = preview.campaign_name
    importState.value = 'previewed'
  } else {
    errorMessage.value = campaignStore.error || 'Failed to read archive'
    importState.value = 'error'
  }
}

async function handleImport() {
  if (!archivePath.value || !campaignName.value.trim() || !campaignsDirectory.value) return

  importState.value = 'importing'
  errorMessage.value = null

  const campaign = await campaignStore.importCampaign(
    archivePath.value,
    campaignName.value.trim(),
    campaignsDirectory.value
  )

  if (campaign) {
    importedCampaign.value = campaign
    importState.value = 'success'
    emit('imported', campaign)
  } else {
    errorMessage.value = campaignStore.error || 'Failed to import campaign'
    importState.value = 'error'
  }
}

function handleOpenCampaign() {
  if (importedCampaign.value) {
    router.push(`/campaigns/${importedCampaign.value.id}/dashboard`)
    handleClose()
  }
}

function handleClose() {
  emit('close')
}

function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
}
</script>

<style scoped>
.import-dialog {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
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

.preview-loading,
.import-progress {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-xl);
  text-align: center;
  color: var(--color-text-secondary);
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

.archive-preview {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
}

.preview-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.preview-icon {
  font-size: 2rem;
}

.preview-title {
  font-weight: 600;
  font-size: 1.125rem;
  color: var(--color-text);
}

.preview-stats {
  display: flex;
  gap: var(--spacing-lg);
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-md) var(--spacing-xl);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  flex: 1;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-primary);
}

.stat-label {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.preview-meta {
  display: flex;
  gap: var(--spacing-lg);
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.meta-item {
  display: flex;
  gap: var(--spacing-xs);
}

.meta-label {
  color: var(--color-text-secondary);
}

.meta-value {
  color: var(--color-text);
  font-weight: 500;
}

.catalog-refs {
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.refs-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--spacing-sm);
}

.refs-title {
  font-weight: 600;
  color: var(--color-text);
}

.refs-count {
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-size: 0.75rem;
  font-weight: 600;
}

.theme-dark .refs-count {
  background: var(--color-primary-900);
  color: var(--color-primary-300);
}

.refs-note {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-md);
}

.refs-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.ref-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
}

.ref-type {
  background: var(--color-gray-100);
  color: var(--color-text-secondary);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  text-transform: capitalize;
}

.theme-dark .ref-type {
  background: var(--color-gray-700);
}

.ref-name {
  flex: 1;
  color: var(--color-text);
}

.ref-source {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
}

.show-more-btn {
  background: none;
  border: none;
  color: var(--color-primary);
  cursor: pointer;
  font-size: 0.875rem;
  padding: var(--spacing-xs);
  text-align: left;
}

.show-more-btn:hover {
  text-decoration: underline;
}

.import-success {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-xl);
  text-align: center;
}

.success-icon {
  width: 64px;
  height: 64px;
  background: var(--color-success-100);
  color: var(--color-success-600);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 2rem;
  font-weight: bold;
}

.theme-dark .success-icon {
  background: var(--color-success-900);
  color: var(--color-success-400);
}

.success-message {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--color-success-600);
}

.theme-dark .success-message {
  color: var(--color-success-400);
}

.success-detail {
  color: var(--color-text-secondary);
}

.import-error {
  padding: var(--spacing-lg);
  background: var(--color-error-100);
  border: 1px solid var(--color-error-200);
  border-radius: var(--radius-md);
  text-align: center;
}

.theme-dark .import-error {
  background: var(--color-error-900);
  border-color: var(--color-error-800);
}

.import-error .error-message {
  color: var(--color-error-600);
  font-weight: 500;
}

.theme-dark .import-error .error-message {
  color: var(--color-error-400);
}
</style>
