<template>
  <div class="module-user-documents">
    <div class="documents-header">
      <h3>My Documents</h3>
      <div class="header-controls">
        <span class="document-count" v-if="userDocuments.length > 0">
          {{ userDocuments.length }}
        </span>
        <button
          class="add-btn"
          @click="showCreateModal = true"
          title="Create or upload document"
        >
          <Plus :size="2" aria-hidden="true" />
        </button>
      </div>
    </div>

    <!-- Document List -->
    <div v-if="loading" class="loading-state">
      Loading documents...
    </div>

    <div v-else-if="userDocuments.length > 0" class="document-list">
      <div
        v-for="doc in userDocuments"
        :key="doc.id"
        class="document-item"
        @click="openDocument(doc)"
      >
        <div class="document-icon">
          <ImageIcon v-if="isImageDocument(doc)" :stroke-width="1.5" aria-hidden="true" />
          <FileText v-else :stroke-width="1.5" aria-hidden="true" />
        </div>
        <div class="document-info">
          <span class="document-title">{{ doc.title }}</span>
          <span class="document-type">{{ doc.file_type }}</span>
        </div>
        <button
          class="delete-btn"
          @click.stop="confirmDelete(doc)"
          title="Delete document"
        >
          <Trash2 :stroke-width="1.5" aria-hidden="true" />
        </button>
      </div>
    </div>

    <!-- Empty State -->
    <EmptyState
      v-else
      variant="generic"
      title="No custom documents yet"
      description="Create markdown documents or upload images for this module."
    />

    <!-- Create Document Modal -->
    <CreateDocumentModal
      :visible="showCreateModal"
      :campaign-id="campaignId"
      :module-id="moduleId"
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
      <p>Are you sure you want to delete "{{ documentToDelete?.title }}"? This cannot be undone.</p>
      <template #footer>
        <button class="btn btn-secondary" @click="showDeleteModal = false">Cancel</button>
        <button class="btn btn-danger" @click="deleteDocument">Delete</button>
      </template>
    </AppModal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { FileText, Image as ImageIcon, Plus, Trash2 } from '@lucide/vue'
import { invoke } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import AppModal from '@/components/shared/AppModal.vue'
import CreateDocumentModal from '@/components/dialogs/CreateDocumentModal.vue'

interface UserDocument {
  id: string
  campaign_id: string
  module_id: string | null
  title: string
  file_path: string
  file_type: string
  is_user_created: boolean
  created_at: string
  updated_at: string
}

interface Props {
  moduleId: string
  campaignId: string
}

const props = defineProps<Props>()
const router = useRouter()

const userDocuments = ref<UserDocument[]>([])
const loading = ref(false)
const showCreateModal = ref(false)
const showDeleteModal = ref(false)
const documentToDelete = ref<UserDocument | null>(null)

const imageExtensions = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg']

function isImageDocument(doc: UserDocument): boolean {
  return doc.file_type !== 'markdown'
}

async function loadUserDocuments() {
  loading.value = true
  try {
    const response = await invoke<{ data: UserDocument[] }>('get_user_documents', {
      campaignId: props.campaignId,
      moduleId: props.moduleId
    })
    userDocuments.value = response.data || []
  } catch (error) {
    console.error('Failed to load user documents:', error)
  } finally {
    loading.value = false
  }
}

function openDocument(doc: UserDocument) {
  // Navigate to the module with the document selected
  router.push({
    name: 'module-detail',
    params: { id: props.moduleId },
    query: { documentId: doc.id.toString() }
  })
}

function confirmDelete(doc: UserDocument) {
  documentToDelete.value = doc
  showDeleteModal.value = true
}

async function deleteDocument() {
  if (!documentToDelete.value) return

  try {
    await invoke('delete_document', {
      documentId: documentToDelete.value.id
    })
    userDocuments.value = userDocuments.value.filter(d => d.id !== documentToDelete.value!.id)
    showDeleteModal.value = false
    documentToDelete.value = null
  } catch (error) {
    console.error('Failed to delete document:', error)
  }
}

function handleDocumentCreated() {
  showCreateModal.value = false
  loadUserDocuments()
}

onMounted(() => {
  loadUserDocuments()
})

// Expose reload method
defineExpose({
  loadUserDocuments
})
</script>

<style scoped>
.module-user-documents {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  padding: 1rem;
}

.documents-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.documents-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.header-controls {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.document-count {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  background: var(--color-surface);
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
}

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  background: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: 0.375rem;
  cursor: pointer;
  transition: all 0.2s;
}

.add-btn svg {
  width: 16px;
  height: 16px;
}

.add-btn:hover {
  background: var(--color-primary-600);
  transform: translateY(-1px);
}

.loading-state {
  padding: 1rem;
  text-align: center;
  color: var(--color-text-secondary);
  font-style: italic;
  font-size: 0.875rem;
}

.document-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.document-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 0.75rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  cursor: pointer;
  transition: all 0.15s;
}

.document-item:hover {
  background: var(--color-surface);
  border-color: var(--color-primary-300);
}

.document-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  background: var(--color-primary-100);
  border-radius: 0.25rem;
  color: var(--color-primary-600);
}

.document-icon svg {
  width: 18px;
  height: 18px;
}

.document-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.document-title {
  font-weight: 500;
  font-size: 0.875rem;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.document-type {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-transform: capitalize;
}

.delete-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  background: transparent;
  color: var(--color-text-secondary);
  border: none;
  border-radius: 0.25rem;
  cursor: pointer;
  opacity: 0;
  transition: all 0.15s;
}

.document-item:hover .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  background: var(--color-error-100);
  color: var(--color-error);
}

.delete-btn svg {
  width: 16px;
  height: 16px;
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
