<template>
  <AppModal
    :visible="visible"
    title="Manage Catalog Sources"
    size="lg"
    :closable="!isImporting && !isDeleting"
    :close-on-overlay="!isImporting && !isDeleting"
    :close-on-escape="!isImporting && !isDeleting"
    @close="closeModal"
  >
    <div v-if="isLoadingBooks" class="loading-message">
      Loading sources...
    </div>

    <EmptyState
      v-else-if="books.length === 0"
      variant="books"
      title="No sources imported yet"
      description="Import a 5etools zip archive to populate your reference library"
    />

    <div v-else class="source-table-container">
      <table class="source-table">
        <thead>
          <tr>
            <th class="col-checkbox">
              <input
                type="checkbox"
                :checked="isAllSelected"
                :indeterminate="isIndeterminate"
                @change="toggleSelectAll"
                title="Select all"
              />
            </th>
            <th class="col-code">Code</th>
            <th class="col-name">Name</th>
            <th class="col-date">Imported</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="book in books"
            :key="book.id"
            :class="{ selected: selectedIds.has(book.id) }"
            @click="toggleSelect(book.id)"
          >
            <td class="col-checkbox" @click.stop>
              <input
                type="checkbox"
                :checked="selectedIds.has(book.id)"
                @change="toggleSelect(book.id)"
              />
            </td>
            <td class="col-code">{{ book.id }}</td>
            <td class="col-name">{{ book.name }}</td>
            <td class="col-date">{{ formatDate(book.imported_at) }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <template #footer>
      <div class="footer-left">
        <span v-if="selectedIds.size > 0" class="selection-count">
          {{ selectedIds.size }} selected
        </span>
        <button
          v-if="selectedIds.size > 0"
          @click="handleDeleteSelected"
          class="btn btn-danger"
          :disabled="isDeleting"
        >
          {{ isDeleting ? 'Deleting...' : `Delete Selected (${selectedIds.size})` }}
        </button>
      </div>
      <div class="footer-right">
        <div v-if="isImporting" class="import-progress">
          Importing: {{ importProgress.currentName }}
        </div>
        <button @click="handleImportBook" class="btn btn-primary" :disabled="isImporting || isDeleting">
          {{ isImporting ? 'Importing...' : 'Import 5etools Data' }}
        </button>
        <button @click="closeModal" class="btn btn-secondary" :disabled="isImporting || isDeleting">
          Close
        </button>
      </div>
    </template>
  </AppModal>

  <!-- Delete Confirmation Modal -->
  <AppModal
    :visible="showDeleteModal"
    title="Remove Sources"
    size="sm"
    :stack-index="1"
    @close="cancelDelete"
  >
    <p v-if="sourcesToDelete.length === 1">
      Are you sure you want to remove "<strong>{{ sourcesToDelete[0]?.name }}</strong>" from the catalog?
    </p>
    <p v-else>
      Are you sure you want to remove <strong>{{ sourcesToDelete.length }} sources</strong> from the catalog?
    </p>
    <p class="warning-text">This will remove all entities from {{ sourcesToDelete.length === 1 ? 'this source' : 'these sources' }}.</p>

    <div v-if="sourcesToDelete.length > 1 && sourcesToDelete.length <= 10" class="sources-list">
      <div v-for="source in sourcesToDelete" :key="source.id" class="source-item">
        {{ source.name }} ({{ source.id }})
      </div>
    </div>

    <div v-if="deleteError" class="error-message">
      {{ deleteError }}
    </div>

    <template #footer>
      <button @click="cancelDelete" class="btn btn-secondary">
        Cancel
      </button>
      <button @click="confirmDelete" class="btn btn-danger" :disabled="isDeleting">
        {{ isDeleting ? 'Deleting...' : `Remove ${sourcesToDelete.length === 1 ? 'Source' : 'Sources'}` }}
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import type { BookInfo, ImportResponse } from '../types/book'

interface Props {
  visible: boolean
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const books = ref<BookInfo[]>([])
const selectedIds = ref<Set<string>>(new Set())
const isLoadingBooks = ref(false)
const isImporting = ref(false)
const isDeleting = ref(false)
const importProgress = ref({ current: 0, total: 0, currentName: '' })
const showDeleteModal = ref(false)
const sourcesToDelete = ref<BookInfo[]>([])
const deleteError = ref<string | null>(null)

// Computed properties for select all
const isAllSelected = computed(() => {
  return books.value.length > 0 && selectedIds.value.size === books.value.length
})

const isIndeterminate = computed(() => {
  return selectedIds.value.size > 0 && selectedIds.value.size < books.value.length
})

// Load books when modal becomes visible
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    loadBooks()
    selectedIds.value.clear()
  }
})

function formatDate(isoDate: string): string {
  try {
    const date = new Date(isoDate)
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  } catch {
    return isoDate
  }
}

function toggleSelect(id: string) {
  const newSet = new Set(selectedIds.value)
  if (newSet.has(id)) {
    newSet.delete(id)
  } else {
    newSet.add(id)
  }
  selectedIds.value = newSet
}

function toggleSelectAll() {
  if (isAllSelected.value) {
    selectedIds.value = new Set()
  } else {
    selectedIds.value = new Set(books.value.map(b => b.id))
  }
}

async function loadBooks() {
  try {
    isLoadingBooks.value = true
    const response = await invoke<{ success: boolean; data: BookInfo[]; error?: string }>('list_catalog_sources')

    if (response.success && response.data) {
      books.value = response.data
    } else {
      books.value = []
    }
  } catch (error) {
    console.error('Failed to load sources:', error)
    books.value = []
  } finally {
    isLoadingBooks.value = false
  }
}

async function handleImportBook() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: '5etools Archive',
        extensions: ['zip']
      }],
      title: 'Select a 5etools zip archive to import'
    })

    if (selected && typeof selected === 'string') {
      const fileName = selected.split('/').pop() || selected
      isImporting.value = true
      importProgress.value = { current: 1, total: 1, currentName: fileName }

      try {
        const response = await invoke<{ success: boolean; data?: ImportResponse; error?: string }>('import_catalog_from_zip', {
          archivePath: selected
        })

        isImporting.value = false

        if (response.success && response.data) {
          alert(response.data.message)
        } else {
          alert(`Import failed: ${response.error || 'Unknown error'}`)
        }
      } catch (err) {
        isImporting.value = false
        const errorMsg = err instanceof Error ? err.message : String(err)
        alert(`Import failed: ${errorMsg}`)
      }

      // Reload the source list
      await loadBooks()
      selectedIds.value.clear()
    }
  } catch (error) {
    console.error('Failed to import sources:', error)
    isImporting.value = false
  }
}

function handleDeleteSelected() {
  sourcesToDelete.value = books.value.filter(b => selectedIds.value.has(b.id))
  deleteError.value = null
  showDeleteModal.value = true
}

async function confirmDelete() {
  if (sourcesToDelete.value.length === 0) return

  deleteError.value = null
  isDeleting.value = true

  try {
    let failedCount = 0
    for (const source of sourcesToDelete.value) {
      try {
        const response = await invoke<{ success: boolean; error?: string }>('delete_catalog_source', {
          sourceCode: source.id
        })
        if (!response.success) {
          failedCount++
        }
      } catch {
        failedCount++
      }
    }

    if (failedCount > 0) {
      deleteError.value = `Failed to delete ${failedCount} source(s)`
    } else {
      showDeleteModal.value = false
      sourcesToDelete.value = []
      selectedIds.value.clear()
    }

    // Reload the source list
    await loadBooks()
  } catch (error) {
    deleteError.value = 'Failed to remove sources. Please try again.'
  } finally {
    isDeleting.value = false
  }
}

function cancelDelete() {
  showDeleteModal.value = false
  sourcesToDelete.value = []
  deleteError.value = null
}

function closeModal() {
  emit('close')
}
</script>

<style scoped>
.loading-message {
  text-align: center;
  color: var(--color-text-secondary);
  padding: var(--spacing-xl) 0;
}

.source-table-container {
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.source-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.source-table thead {
  position: sticky;
  top: 0;
  background: var(--color-surface);
  z-index: 1;
}

.source-table th {
  text-align: left;
  padding: var(--spacing-sm) var(--spacing-md);
  font-weight: 600;
  color: var(--color-text-secondary);
  border-bottom: 2px solid var(--color-border);
  white-space: nowrap;
}

.source-table td {
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
  color: var(--color-text);
}

.source-table tbody tr {
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.source-table tbody tr:hover {
  background: var(--color-surface-variant);
}

.source-table tbody tr.selected {
  background: var(--color-primary-100);
}

.theme-dark .source-table tbody tr.selected {
  background: var(--color-primary-900);
}

.source-table tbody tr:last-child td {
  border-bottom: none;
}

.col-checkbox {
  width: 40px;
  text-align: center;
}

.col-checkbox input[type="checkbox"] {
  cursor: pointer;
  width: 16px;
  height: 16px;
}

.col-code {
  width: 80px;
  font-family: monospace;
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.col-name {
  min-width: 200px;
}

.col-date {
  width: 120px;
  color: var(--color-text-secondary);
  font-size: 0.8125rem;
}

/* Footer layout */
:deep(.modal-footer) {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--spacing-md);
}

.footer-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.footer-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.selection-count {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.import-progress {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 200px;
}

/* Delete modal */
.sources-list {
  margin-top: var(--spacing-md);
  padding: var(--spacing-sm);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
  max-height: 150px;
  overflow-y: auto;
  font-size: 0.875rem;
}

.source-item {
  padding: var(--spacing-xs) 0;
  color: var(--color-text-secondary);
}

.warning-text {
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  margin-top: var(--spacing-sm);
}

.error-message {
  background: var(--color-error-100);
  color: var(--color-error-700);
  border: 1px solid var(--color-error-200);
  border-radius: var(--radius-md);
  padding: var(--spacing-sm) var(--spacing-md);
  margin: var(--spacing-md) 0;
  font-size: 0.875rem;
}

.theme-dark .error-message {
  background: var(--color-error-900);
  color: var(--color-error-300);
  border-color: var(--color-error-800);
}

/* Button styles */
.btn {
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  font-weight: 500;
  font-size: 0.875rem;
  cursor: pointer;
  transition: all var(--transition-fast);
  border: none;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-primary-600);
}

.btn-secondary {
  background: var(--color-surface-variant);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--color-gray-200);
}

.theme-dark .btn-secondary:hover:not(:disabled) {
  background: var(--color-gray-700);
}

.btn-danger {
  background: var(--color-error);
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: var(--color-error-600);
}
</style>
