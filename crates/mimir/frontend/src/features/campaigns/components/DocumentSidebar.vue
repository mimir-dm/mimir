<template>
  <div class="document-sidebar">
    <div class="sidebar-header">
      <h3>Documents</h3>
      <div class="header-actions">
        <button
          class="add-btn"
          @click="showCreateModal = true"
          title="Create document"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Document List -->
    <div class="document-content">
      <div v-if="loading" class="loading-state">
        Loading documents...
      </div>

      <div v-else-if="documents.length === 0" class="empty-state">
        <p>No documents yet</p>
      </div>

      <div v-else class="document-items">
        <!-- Template Documents -->
        <div
          v-for="doc in templateDocuments"
          :key="doc.id"
          class="document-item"
          :class="{ selected: selectedDocument?.id === doc.id }"
          @click="selectDocument(doc)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="document-icon-svg">
            <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
          </svg>
          <span class="document-title">{{ doc.title }}</span>
          <button
            class="delete-btn"
            @click.stop="confirmDeleteDocument(doc)"
            title="Delete document"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
            </svg>
          </button>
        </div>

        <!-- Divider between template and user documents -->
        <div v-if="userDocuments.length > 0" class="document-divider">
          <span>Your Documents</span>
        </div>

        <!-- User Documents -->
        <div
          v-for="doc in userDocuments"
          :key="doc.id"
          class="document-item"
          :class="{ selected: selectedDocument?.id === doc.id }"
          @click="selectDocument(doc)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="document-icon-svg">
            <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
          </svg>
          <span class="document-title">{{ doc.title }}</span>
          <button
            class="delete-btn"
            @click.stop="confirmDeleteDocument(doc)"
            title="Delete document"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
            </svg>
          </button>
        </div>

        <!-- Divider for assets -->
        <div v-if="imageAssets.length > 0" class="document-divider">
          <span>Images</span>
        </div>

        <!-- Assets (Images) -->
        <div
          v-for="asset in imageAssets"
          :key="asset.id"
          class="document-item"
          :class="{ selected: selectedAsset?.id === asset.id }"
          @click="selectAsset(asset)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="document-icon-svg asset-icon">
            <path stroke-linecap="round" stroke-linejoin="round" d="m2.25 15.75 5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-1.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5Zm10.5-11.25h.008v.008h-.008V8.25Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z" />
          </svg>
          <span class="document-title">{{ asset.description || asset.filename }}</span>
          <button
            class="delete-btn"
            @click.stop="confirmDeleteAsset(asset)"
            title="Delete image"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Create Document Modal -->
    <CreateDocumentModal
      :visible="showCreateModal"
      :campaign-id="campaignId"
      @close="showCreateModal = false"
      @created="handleDocumentCreated"
    />

    <!-- Delete Document Confirmation Modal -->
    <AppModal
      :visible="showDeleteModal"
      title="Delete Document"
      size="sm"
      @close="showDeleteModal = false"
    >
      <p>Are you sure you want to delete "{{ documentToDelete?.title }}"?</p>
      <p class="delete-warning">This action cannot be undone.</p>
      <template #footer>
        <button class="btn btn-secondary" @click="showDeleteModal = false">Cancel</button>
        <button class="btn btn-danger" @click="deleteDocument">Delete</button>
      </template>
    </AppModal>

    <!-- Delete Asset Confirmation Modal -->
    <AppModal
      :visible="showDeleteAssetModal"
      title="Delete Image"
      size="sm"
      @close="showDeleteAssetModal = false"
    >
      <p>Are you sure you want to delete "{{ assetToDelete?.filename }}"?</p>
      <p class="delete-warning">This action cannot be undone.</p>
      <template #footer>
        <button class="btn btn-secondary" @click="showDeleteAssetModal = false">Cancel</button>
        <button class="btn btn-danger" @click="deleteAsset">Delete</button>
      </template>
    </AppModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { DocumentService } from '@/services/DocumentService'
import type { Document, ApiResponse } from '@/types/api'
import CreateDocumentModal from '@/components/dialogs/CreateDocumentModal.vue'
import AppModal from '@/components/shared/AppModal.vue'

// Asset type matching backend CampaignAsset
interface CampaignAsset {
  id: string
  campaign_id: string | null
  module_id: string | null
  filename: string
  description: string | null
  mime_type: string
  blob_path: string
  file_size: number | null
  uploaded_at: string
}

const props = defineProps<{
  campaignId: string
  campaignName: string
}>()

const emit = defineEmits<{
  selectDocument: [document: Document]
  selectAsset: [asset: CampaignAsset]
}>()

// Sort order for campaign documents by doc_type
const DOCUMENT_SORT_ORDER: Record<string, number> = {
  'campaign_pitch': 1,
  'world_primer': 2,
  'starting_scenario': 3,
  'character_guidelines': 4,
  'faction_overview': 5,
  'character_integration': 6,
  'player_secrets': 7,
  'campaign_bible': 8,
  'table_expectations': 9,
  'safety_tools': 10,
  'house_rules': 11,
}

// State
const documents = ref<Document[]>([])
const allAssets = ref<CampaignAsset[]>([])
const selectedDocument = ref<Document | null>(null)
const selectedAsset = ref<CampaignAsset | null>(null)
const loading = ref(false)
const showCreateModal = ref(false)
const showDeleteModal = ref(false)
const showDeleteAssetModal = ref(false)
const documentToDelete = ref<Document | null>(null)
const assetToDelete = ref<CampaignAsset | null>(null)

// Template documents (have a defined sort order)
const templateDocuments = computed(() => {
  return [...documents.value]
    .filter(d => d.doc_type in DOCUMENT_SORT_ORDER)
    .sort((a, b) => {
      const orderA = DOCUMENT_SORT_ORDER[a.doc_type]
      const orderB = DOCUMENT_SORT_ORDER[b.doc_type]
      return orderA - orderB
    })
})

// User-created documents (no defined sort order)
const userDocuments = computed(() => {
  return [...documents.value]
    .filter(d => !(d.doc_type in DOCUMENT_SORT_ORDER))
    .sort((a, b) => a.title.localeCompare(b.title))
})

// Filter assets to only show images (exclude map UVTT files which are application/octet-stream)
const imageAssets = computed(() => {
  return allAssets.value.filter(a => a.mime_type.startsWith('image/'))
})

// Load all documents and assets for the campaign
const loadDocuments = async () => {
  loading.value = true

  try {
    // Load documents and assets in parallel
    const [docs, assetResponse] = await Promise.all([
      DocumentService.listForCampaign(props.campaignId),
      invoke<ApiResponse<CampaignAsset[]>>('list_campaign_assets', { campaignId: props.campaignId })
    ])

    documents.value = docs
    allAssets.value = assetResponse.success && assetResponse.data ? assetResponse.data : []
  } catch (e) {
    console.error('Failed to load documents:', e)
  } finally {
    loading.value = false
  }
}

// Select a document
const selectDocument = (doc: Document) => {
  selectedDocument.value = doc
  selectedAsset.value = null
  emit('selectDocument', doc)
}

// Select an asset
const selectAsset = (asset: CampaignAsset) => {
  selectedAsset.value = asset
  selectedDocument.value = null
  emit('selectAsset', asset)
}

// Handle document created from modal
const handleDocumentCreated = async () => {
  showCreateModal.value = false
  await loadDocuments()
  // Select the most recently created document (by created_at timestamp)
  if (documents.value.length > 0) {
    const newestDoc = [...documents.value].sort((a, b) =>
      new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    )[0]
    selectDocument(newestDoc)
  }
}

// Confirm delete document
const confirmDeleteDocument = (doc: Document) => {
  documentToDelete.value = doc
  showDeleteModal.value = true
}

// Delete document
const deleteDocument = async () => {
  if (!documentToDelete.value) return

  try {
    await DocumentService.delete(documentToDelete.value.id)

    // Remove from list
    documents.value = documents.value.filter((d: Document) => d.id !== documentToDelete.value!.id)

    // Clear selection if deleted doc was selected
    if (selectedDocument.value?.id === documentToDelete.value.id) {
      selectedDocument.value = null
    }

    showDeleteModal.value = false
    documentToDelete.value = null
  } catch (e) {
    console.error('Failed to delete document:', e)
  }
}

// Confirm delete asset
const confirmDeleteAsset = (asset: CampaignAsset) => {
  assetToDelete.value = asset
  showDeleteAssetModal.value = true
}

// Delete asset
const deleteAsset = async () => {
  if (!assetToDelete.value) return

  try {
    await invoke('delete_asset', { id: assetToDelete.value.id })

    // Remove from list
    allAssets.value = allAssets.value.filter((a: CampaignAsset) => a.id !== assetToDelete.value!.id)

    // Clear selection if deleted asset was selected
    if (selectedAsset.value?.id === assetToDelete.value.id) {
      selectedAsset.value = null
    }

    showDeleteAssetModal.value = false
    assetToDelete.value = null
  } catch (e) {
    console.error('Failed to delete asset:', e)
  }
}

// Watch for campaign changes
watch(() => props.campaignId, () => {
  loadDocuments()
})

onMounted(() => {
  loadDocuments()
})
</script>

<style scoped>
.document-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-surface);
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--color-border);
}

.sidebar-header h3 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  background: var(--color-surface);
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  cursor: pointer;
  transition: all 0.2s;
}

.add-btn svg {
  width: 14px;
  height: 14px;
}

.add-btn:hover {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

/* Document content area */
.document-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-sm, 8px);
}

/* Document items */
.document-items {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs, 4px);
}

/* Divider between template and user documents */
.document-divider {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm, 8px);
  margin: var(--spacing-md, 12px) 0 var(--spacing-sm, 8px);
  padding: 0 var(--spacing-sm, 8px);
}

.document-divider::before,
.document-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--color-border, #333);
}

.document-divider span {
  font-size: 0.7rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted, #888);
  white-space: nowrap;
}

.document-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm, 8px);
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  border-radius: var(--radius-sm, 4px);
  cursor: pointer;
  transition: background 0.15s;
}

.document-item:hover {
  background: var(--color-surface-variant, #252525);
}

.document-item.selected {
  background: var(--color-primary-900, #1e3a5f);
}

.document-icon-svg {
  width: 16px;
  height: 16px;
  opacity: 0.7;
  flex-shrink: 0;
  color: var(--color-text-muted, #888);
}

.document-icon-svg.asset-icon {
  color: var(--color-primary-400, #60a5fa);
}

.document-title {
  flex: 1;
  font-size: 0.875rem;
  color: var(--color-text, #e0e0e0);
}

/* Loading/Empty states */
.loading-state,
.empty-state {
  padding: var(--spacing-lg, 16px);
  text-align: center;
  color: var(--color-text-muted, #888);
  font-size: 0.875rem;
}

/* Delete button */
.delete-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  padding: 0;
  background: transparent;
  color: var(--color-text-muted);
  border: none;
  border-radius: 0.25rem;
  cursor: pointer;
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}

.document-item:hover .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  background: var(--color-error-100, rgba(239, 68, 68, 0.1));
  color: var(--color-error, #ef4444);
}

.delete-btn svg {
  width: 14px;
  height: 14px;
}

/* Delete modal styles */
.delete-warning {
  font-size: 0.875rem;
  color: var(--color-error, #ef4444);
  margin-top: 0.5rem;
}

.btn {
  padding: 0.5rem 1rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-secondary {
  background: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover {
  background: var(--color-surface-variant);
}

.btn-danger {
  background: var(--color-error, #ef4444);
  color: white;
  border: none;
}

.btn-danger:hover {
  background: var(--color-error-dark, #dc2626);
}
</style>
