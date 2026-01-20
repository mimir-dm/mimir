<template>
  <div class="document-sidebar">
    <div class="sidebar-header">
      <h3>Documents</h3>
      <div class="header-actions">
        <button
          class="add-btn"
          @click="showCreateModal = true"
          title="Create or upload document"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
        </button>
        <button
          v-if="documents.length > 0"
          class="export-btn"
          @click="openExportDialog"
          title="Export campaign as PDF"
        >
          PDF
        </button>
      </div>
    </div>

    <!-- Document List Grouped by Stage -->
    <div class="document-content">
      <div v-if="loading" class="loading-state">
        Loading documents...
      </div>
      
      <!-- Document list grouped by stage with dividers -->
      <div v-else class="stage-groups">
        <template v-for="(stage, index) in stagesWithDocuments" :key="stage.key">
          <div class="stage-group">
            <div class="stage-header">
              <h4>{{ stage.display_name }}</h4>
            </div>
            <div class="document-items">
              <div
                v-for="doc in getStageDocuments(stage.key).documents"
                :key="doc.instance?.id || doc.templateId"
                class="document-item"
                :class="{ selected: selectedDocument?.id === doc.instance?.id }"
                @click="handleDocumentClick(doc)"
              >
                <img
                  :src="getEditIcon()"
                  alt="Edit"
                  class="document-icon"
                />
                <span class="document-title">
                  {{ doc.title }}
                  <span v-if="!doc.required" class="optional-label">(Optional)</span>
                </span>
                <button
                  class="delete-btn"
                  @click="confirmDeleteDocument(doc.instance, $event)"
                  title="Delete document"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
          <!-- Divider between stage groups -->
          <hr v-if="index < stagesWithDocuments.length - 1" class="stage-divider" />
        </template>
      </div>

      <!-- User Documents Section -->
      <div v-if="userDocuments.length > 0" class="user-documents-section">
        <hr class="stage-divider" />
        <div class="stage-group">
          <div class="stage-header">
            <h4>My Documents</h4>
          </div>
          <div class="document-items">
            <div
              v-for="doc in userDocuments"
              :key="doc.id"
              class="document-item user-document"
              :class="{ selected: selectedDocument?.id === doc.id }"
              @click="handleUserDocumentClick(doc)"
            >
              <svg v-if="isImageDocument(doc)" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="document-icon-svg">
                <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 19.5h16.5a2.25 2.25 0 002.25-2.25V6.75a2.25 2.25 0 00-2.25-2.25H3.75A2.25 2.25 0 001.5 6.75v10.5A2.25 2.25 0 003.75 19.5z" />
              </svg>
              <svg v-else xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="document-icon-svg">
                <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
              </svg>
              <span class="document-title">{{ doc.title }}</span>
              <button
                class="delete-btn"
                @click="confirmDeleteDocument(doc, $event)"
                title="Delete document"
              >
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Campaign Export Dialog -->
    <CampaignExportDialog
      :visible="showExportDialog"
      :campaign-id="campaignId"
      :campaign-name="campaignName"
      @close="showExportDialog = false"
    />

    <!-- Create Document Modal -->
    <CreateDocumentModal
      :visible="showCreateModal"
      :campaign-id="campaignId"
      @close="showCreateModal = false"
      @created="handleDocumentCreated"
    />

    <!-- Delete Confirmation Modal -->
    <AppModal
      :visible="showDeleteModal"
      title="Delete Document"
      size="sm"
      @close="showDeleteModal = false"
    >
      <p>Are you sure you want to delete "{{ documentToDelete?.title }}"?</p>
      <p class="delete-warning">This will permanently remove the document and its file from disk.</p>
      <template #footer>
        <button class="btn btn-secondary" @click="showDeleteModal = false">Cancel</button>
        <button class="btn btn-danger" @click="deleteDocument">Delete</button>
      </template>
    </AppModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { DocumentService, type Document } from '@/services/DocumentService'
import { useThemeStore } from '../../../stores/theme'
import { debugDocument } from '../../../shared/utils/debug'
import CampaignExportDialog from '../../../components/print/CampaignExportDialog.vue'
import CreateDocumentModal from '../../../components/CreateDocumentModal.vue'
import AppModal from '../../../components/shared/AppModal.vue'

// Import icon images
import lightEditIcon from '../../../assets/images/light-edit.png'
import darkEditIcon from '../../../assets/images/dark-edit.png'
import hyperEditIcon from '../../../assets/images/hyper-edit.png'

const props = defineProps<{
  campaignId: number
  campaignName: string
  campaignStage: string
  boardConfig: any
}>()

const emit = defineEmits<{
  selectDocument: [document: Document]
  createDocument: []
  documentCompletionChanged: [document: Document]
}>()

// Build a map of template IDs to their stage and metadata from board config
const templateMetadata = computed(() => {
  if (!props.boardConfig) return {}

  const metadata: Record<string, { stage: string; required: boolean; displayName: string }> = {}

  for (const stage of props.boardConfig.stages) {
    for (const docId of stage.required_documents) {
      metadata[docId] = {
        stage: stage.key,
        required: true,
        displayName: docId.replace(/[-_]/g, ' ').split(' ').map((word: string) =>
          word.charAt(0).toUpperCase() + word.slice(1)
        ).join(' ')
      }
    }
    for (const docId of stage.optional_documents) {
      metadata[docId] = {
        stage: stage.key,
        required: false,
        displayName: docId.replace(/[-_]/g, ' ').split(' ').map((word: string) =>
          word.charAt(0).toUpperCase() + word.slice(1)
        ).join(' ')
      }
    }
  }

  return metadata
})

// State
const documents = ref<Document[]>([])
const userDocuments = ref<Document[]>([])
const selectedDocument = ref<Document | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const showExportDialog = ref(false)
const showCreateModal = ref(false)
const showDeleteModal = ref(false)
const documentToDelete = ref<Document | null>(null)

// Theme store for icon selection
const themeStore = useThemeStore()

// Icon mapping
const iconMap = {
  light: { edit: lightEditIcon },
  dark: { edit: darkEditIcon },
  hyper: { edit: hyperEditIcon }
}

// Group existing documents by stage - only shows documents that exist in database
const documentsByStage = computed(() => {
  const grouped: Record<string, Document[]> = {}

  for (const doc of documents.value) {
    const templateId = doc.template_id
    if (templateId && templateMetadata.value[templateId]) {
      const stage = templateMetadata.value[templateId].stage
      if (!grouped[stage]) {
        grouped[stage] = []
      }
      grouped[stage].push(doc)
    }
  }

  return grouped
})

// Filter stages to only those with actual documents that exist
const stagesWithDocuments = computed(() => {
  if (!props.boardConfig?.stages) return []
  return props.boardConfig.stages.filter((stage: any) =>
    documentsByStage.value[stage.key]?.length > 0
  )
})

// Get documents for a specific stage - only returns documents that exist
const getStageDocuments = (stage: string) => {
  const stageDocs = documentsByStage.value[stage] || []
  const stageDocumentList = stageDocs.map((doc: Document) => {
    const templateId = doc.template_id
    const meta = templateId ? templateMetadata.value[templateId] : null
    return {
      templateId: templateId,
      title: meta?.displayName || doc.title,
      required: meta?.required ?? true,
      instance: doc
    }
  })

  // Only count required documents for completion tracking
  const requiredDocs = stageDocumentList.filter((doc: any) => doc.required)
  const completed = requiredDocs.filter((doc: any) => doc.instance?.completed_at).length
  const total = requiredDocs.length
  const percentage = total > 0 ? Math.round((completed / total) * 100) : 0

  return {
    documents: stageDocumentList,
    completed,
    total,
    percentage
  }
}

// Get edit icon for current theme
const getEditIcon = (): string => {
  const theme = themeStore.currentTheme as 'light' | 'dark' | 'hyper'
  return iconMap[theme]?.edit || lightEditIcon
}

// Get which stage a template belongs to
const getDocumentStage = (templateId: string): string => {
  return templateMetadata.value[templateId]?.stage || 'concept'
}

// Get all documents for active/concluding/completed campaigns (simplified view)
// Only returns documents that actually exist in the database
const getAllDocumentsForActive = () => {
  const allDocs: any[] = []

  // Only include documents that exist in the database
  for (const doc of documents.value) {
    const templateId = doc.template_id
    const meta = templateId ? templateMetadata.value[templateId] : null

    allDocs.push({
      templateId: templateId,
      title: meta?.displayName || doc.title,
      required: meta?.required ?? true,
      instance: doc
    })
  }

  // Sort alphabetically by title
  return allDocs.sort((a, b) => a.title.localeCompare(b.title))
}

// Load all documents for the campaign
const loadDocuments = async () => {
  loading.value = true
  error.value = null

  try {
    documents.value = await DocumentService.list(undefined, props.campaignId)
    // Also load user documents
    await loadUserDocuments()
  } catch (e) {
    error.value = 'Failed to load documents'
  } finally {
    loading.value = false
  }
}

// Load user-created documents
const loadUserDocuments = async () => {
  try {
    const response = await invoke<{ success: boolean; data: Document[] }>('get_user_documents', {
      campaignId: props.campaignId,
      moduleId: null
    })
    if (response.success && response.data) {
      userDocuments.value = response.data
    }
  } catch (e) {
    console.error('Failed to load user documents:', e)
  }
}

// Check if document is an image
const isImageDocument = (doc: Document): boolean => {
  const fileType = (doc as any).file_type || 'markdown'
  return fileType !== 'markdown'
}

// Handle user document click
const handleUserDocumentClick = (doc: Document) => {
  selectDocument(doc)
}

// Handle document created from modal
const handleDocumentCreated = async () => {
  showCreateModal.value = false
  await loadUserDocuments()
  // Select the most recently created document
  if (userDocuments.value.length > 0) {
    const lastDoc = userDocuments.value[userDocuments.value.length - 1]
    selectDocument(lastDoc)
  }
}

// Handle document click - all documents shown now exist in database
const handleDocumentClick = (doc: any) => {
  debugDocument('click', { doc, stage: getDocumentStage(doc.templateId) })
  // Since we only show documents that exist, just select the instance
  if (doc.instance) {
    selectDocument(doc.instance)
  }
}

// Create a new document from template
const createDocument = async (templateId: string, title: string) => {
  try {
    // Use create_document_from_template which creates both file and DB record
    const response = await invoke<{ success: boolean; data: Document }>('create_document_from_template', {
      campaignId: props.campaignId,
      templateId: templateId
    })
    if (response.success && response.data) {
      // Add the new document to our list
      documents.value.push(response.data)
      // Select it immediately
      selectDocument(response.data)
    } else {
    }
  } catch (e) {
  }
}

// Select a document
const selectDocument = (doc: Document) => {
  selectedDocument.value = doc
  emit('selectDocument', doc)
}

// Toggle document completion status
const toggleDocumentCompletion = async (doc: any) => {
  if (!doc.instance) return
  
  try {
    const newCompletedAt = doc.instance.completed_at ? null : new Date().toISOString()
    
    // All documents should be in the database now, so always update via backend
    const updatedDoc = doc.instance.completed_at 
      ? await DocumentService.uncomplete(doc.instance.id)
      : await DocumentService.complete(doc.instance.id)
    
    // Update the document in our local list
    const index = documents.value.findIndex(d => d.id === doc.instance.id)
    if (index !== -1) {
      documents.value[index] = updatedDoc
    }
    
    // Also update the instance reference
    doc.instance = updatedDoc
      
    // Force reactivity update
    documents.value = [...documents.value]
    
    // Emit completion status change
    emit('documentCompletionChanged', updatedDoc)
  } catch (e) {
    console.error('Failed to toggle document completion:', e)
  }
}

// Open export dialog
const openExportDialog = () => {
  showExportDialog.value = true
}

// Confirm delete document
const confirmDeleteDocument = (doc: Document, event: Event) => {
  event.stopPropagation()
  documentToDelete.value = doc
  showDeleteModal.value = true
}

// Delete document
const deleteDocument = async () => {
  if (!documentToDelete.value) return

  try {
    await invoke('delete_document', {
      documentId: documentToDelete.value.id
    })

    // Remove from user documents list
    userDocuments.value = userDocuments.value.filter(d => d.id !== documentToDelete.value!.id)

    // Remove from template documents list
    documents.value = documents.value.filter(d => d.id !== documentToDelete.value!.id)

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

// Watch for campaign or stage changes
watch([() => props.campaignId, () => props.campaignStage], () => {
  loadDocuments()
})

onMounted(() => {
  loadDocuments()
})
</script>

<style scoped>
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

.export-btn {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  font-weight: 500;
  background: var(--color-surface);
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
  border-radius: 0.25rem;
  cursor: pointer;
  transition: all 0.2s;
}

.export-btn:hover:not(:disabled) {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.export-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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
  padding: var(--spacing-sm, 8px);
}

/* Stage groups with dividers */
.stage-groups {
  display: flex;
  flex-direction: column;
}

.stage-group {
  padding-bottom: var(--spacing-sm, 8px);
}

.stage-group:last-of-type {
  padding-bottom: 0;
}

.stage-header {
  margin-bottom: var(--spacing-xs, 4px);
}

.stage-header h4 {
  margin: 0;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--color-text-muted, #666);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Subtle divider between stage groups */
.stage-divider {
  border: none;
  border-top: 1px solid var(--color-border, #333);
  margin: var(--spacing-sm, 8px) 0;
  opacity: 0.5;
}

/* Document items */
.document-items {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs, 4px);
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

.document-item.completed .document-title {
  color: var(--color-text-muted, #888);
}

.document-icon {
  width: 16px;
  height: 16px;
  opacity: 0.7;
  flex-shrink: 0;
}

.document-title {
  flex: 1;
  font-size: 0.875rem;
  color: var(--color-text, #e0e0e0);
}

.document-title.optional {
  font-style: italic;
}

.optional-label {
  font-size: 0.75rem;
  color: var(--color-text-muted, #888);
  margin-left: var(--spacing-xs, 4px);
}

/* Loading state */
.loading-state {
  padding: var(--spacing-lg, 16px);
  text-align: center;
  color: var(--color-text-muted, #888);
  font-size: 0.875rem;
}

/* Active documents (simplified list) */
.active-documents .document-items {
  gap: var(--spacing-xs, 4px);
}

/* User documents section */
.user-documents-section {
  margin-top: var(--spacing-sm, 8px);
}

.document-icon-svg {
  width: 16px;
  height: 16px;
  opacity: 0.7;
  flex-shrink: 0;
  color: var(--color-text-muted, #888);
}

.user-document .document-icon-svg {
  color: var(--color-primary-400, #60a5fa);
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
