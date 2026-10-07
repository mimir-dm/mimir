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
          <Plus :size="2" aria-hidden="true" />
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
          <FileText class="document-icon-svg" :stroke-width="1.5" aria-hidden="true" />
          <span class="document-title">{{ doc.title }}</span>
          <button
            class="delete-btn"
            @click.stop="confirmDeleteDocument(doc)"
            title="Delete document"
          >
            <Trash2 :stroke-width="1.5" aria-hidden="true" />
          </button>
        </div>

        <!-- Divider between template and user documents -->
        <div v-if="userDocuments.length > 0" class="document-divider">
          <span>Your Documents</span>
        </div>

        <!-- User Documents -->
        <div
          v-for="(doc, index) in userDocuments"
          :key="doc.id"
          class="document-item"
          :class="{ selected: selectedDocument?.id === doc.id }"
          @click="selectDocument(doc)"
        >
          <FileText class="document-icon-svg" :stroke-width="1.5" aria-hidden="true" />
          <span class="document-title">{{ doc.title }}</span>
          <span class="document-reorder-buttons">
            <button
              class="btn-reorder"
              :disabled="index === 0"
              title="Move up"
              @click.stop="moveDocument(doc.id, userDocuments[index - 1].id)"
            >&#9650;</button>
            <button
              class="btn-reorder"
              :disabled="index === userDocuments.length - 1"
              title="Move down"
              @click.stop="moveDocument(doc.id, userDocuments[index + 1].id)"
            >&#9660;</button>
          </span>
          <button
            class="delete-btn"
            @click.stop="confirmDeleteDocument(doc)"
            title="Delete document"
          >
            <Trash2 :stroke-width="1.5" aria-hidden="true" />
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
          <ImageIcon class="document-icon-svg asset-icon" :stroke-width="1.5" aria-hidden="true" />
          <span class="document-title">{{ asset.description || asset.filename }}</span>
          <button
            class="delete-btn"
            @click.stop="confirmDeleteAsset(asset)"
            title="Delete image"
          >
            <Trash2 :stroke-width="1.5" aria-hidden="true" />
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
import { FileText, Image as ImageIcon, Plus, Trash2 } from '@lucide/vue'
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

// User-created documents (ordered by sort_order)
const userDocuments = computed(() => {
  return [...documents.value]
    .filter(d => !(d.doc_type in DOCUMENT_SORT_ORDER))
    .sort((a, b) => a.sort_order - b.sort_order || a.title.localeCompare(b.title))
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

// Reorder a user document by swapping with neighbor
const moveDocument = async (documentId: string, swapWithId: string) => {
  try {
    const updatedDocs = await DocumentService.reorder(documentId, swapWithId)
    documents.value = updatedDocs
  } catch (e) {
    console.error('Failed to reorder document:', e)
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
  color: var(--color-text-secondary);
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
  background: var(--color-primary-500);
  color: white;
  border-color: var(--color-primary-500);
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
  background: var(--color-border);
}

.document-divider span {
  font-size: 0.7rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--legacy-text-muted);
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
  background: var(--color-surface-variant);
}

.document-item.selected {
  background: var(--color-primary-100);
}

.document-icon-svg {
  width: 16px;
  height: 16px;
  opacity: 0.7;
  flex-shrink: 0;
  color: var(--legacy-text-muted);
}

.document-icon-svg.asset-icon {
  color: var(--color-primary-400);
}

.document-title {
  flex: 1;
  font-size: 0.875rem;
  color: var(--color-text);
}

/* Loading/Empty states */
.loading-state,
.empty-state {
  padding: var(--spacing-lg, 16px);
  text-align: center;
  color: var(--legacy-text-muted);
  font-size: 0.875rem;
}

/* Reorder buttons */
.document-reorder-buttons {
  display: none;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}

.document-item:hover .document-reorder-buttons {
  display: flex;
}

.btn-reorder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  padding: 0;
  background: transparent;
  color: var(--color-text-secondary);
  border: none;
  border-radius: 0.25rem;
  cursor: pointer;
  font-size: 0.5rem;
  line-height: 1;
  transition: all 0.15s;
}

.btn-reorder:hover:not(:disabled) {
  background: var(--color-surface-variant);
  color: var(--color-text);
}

.btn-reorder:disabled {
  opacity: 0.3;
  cursor: default;
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
  color: var(--color-text-secondary);
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
  background: var(--color-error-100);
  color: var(--color-error);
}

.delete-btn svg {
  width: 14px;
  height: 14px;
}

/* Delete modal styles */
.delete-warning {
  font-size: 0.875rem;
  color: var(--color-error);
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
  background: var(--color-error);
  color: white;
  border: none;
}

.btn-danger:hover {
  background: var(--color-error-dark);
}
</style>
