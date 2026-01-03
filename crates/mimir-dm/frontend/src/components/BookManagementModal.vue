<template>
  <AppModal
    :visible="visible"
    title="Manage Reference Books"
    size="md"
    :closable="!isImporting"
    :close-on-overlay="!isImporting"
    :close-on-escape="!isImporting"
    @close="closeModal"
  >
    <div v-if="isLoadingBooks" class="loading-message">
      Loading books...
    </div>

    <EmptyState
      v-else-if="books.length === 0"
      variant="books"
      title="No books imported yet"
      description="Import book archives to start building your reference library"
    />

    <div v-else class="book-list">
      <div v-for="book in books" :key="book.id" class="book-item">
        <div class="book-info">
          <span class="book-name">{{ book.name }}</span>
          <span v-if="book.image_count" class="book-meta">{{ book.image_count }} images</span>
        </div>
        <button
          @click="handleRemoveBook(book)"
          class="remove-button"
          title="Remove book"
        >
          ×
        </button>
      </div>
    </div>

    <template #footer>
      <div v-if="isImporting" class="import-progress">
        Importing {{ importProgress.current }}/{{ importProgress.total }}: {{ importProgress.currentName }}
      </div>
      <button @click="handleImportBook" class="btn btn-primary" :disabled="isImporting">
        {{ isImporting ? 'Importing...' : 'Import Books' }}
      </button>
      <button @click="closeModal" class="btn btn-secondary" :disabled="isImporting">
        Close
      </button>
    </template>
  </AppModal>

  <!-- Delete Confirmation Modal -->
  <AppModal
    :visible="showDeleteModal"
    title="Remove Book"
    size="sm"
    :stack-index="1"
    @close="cancelDelete"
  >
    <p>Are you sure you want to remove "<strong>{{ bookToDelete?.name }}</strong>" from your library?</p>
    <p class="warning-text">This will remove the book from your reference library.</p>

    <div v-if="deleteError" class="error-message">
      {{ deleteError }}
    </div>

    <template #footer>
      <button @click="cancelDelete" class="btn btn-secondary">
        Cancel
      </button>
      <button @click="confirmDelete" class="btn btn-danger">
        Remove Book
      </button>
    </template>
  </AppModal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import AppModal from '@/components/shared/AppModal.vue'
import EmptyState from '@/shared/components/ui/EmptyState.vue'
import type { BookInfo } from '../types/book'

interface Props {
  visible: boolean
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const books = ref<BookInfo[]>([])
const isLoadingBooks = ref(false)
const isImporting = ref(false)
const importProgress = ref({ current: 0, total: 0, currentName: '' })
const showDeleteModal = ref(false)
const bookToDelete = ref<BookInfo | null>(null)
const deleteError = ref<string | null>(null)

// Load books when modal becomes visible
watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    loadBooks()
  }
})

async function loadBooks() {
  try {
    isLoadingBooks.value = true
    const response = await invoke<{ success: boolean; data: BookInfo[]; message?: string }>('list_library_books')
    
    if (response.success && response.data) {
      books.value = response.data
    } else {
      books.value = []
    }
  } catch (error) {
    console.error('Failed to load books:', error)
    books.value = []
  } finally {
    isLoadingBooks.value = false
  }
}

async function handleImportBook() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Book Archive',
        extensions: ['tar.gz', 'gz']
      }],
      title: 'Select book archives to add to your library'
    })

    if (selected) {
      // Normalize to array
      const filePaths = Array.isArray(selected) ? selected : [selected]

      if (filePaths.length === 0) return

      isImporting.value = true
      importProgress.value = { current: 0, total: filePaths.length, currentName: '' }

      const results: { success: boolean; name: string; error?: string }[] = []

      for (let i = 0; i < filePaths.length; i++) {
        const filePath = filePaths[i]
        const fileName = filePath.split('/').pop() || filePath
        importProgress.value = { current: i + 1, total: filePaths.length, currentName: fileName }

        try {
          const response = await invoke<{ success: boolean; data?: BookInfo; message?: string }>('upload_book_archive', {
            archivePath: filePath
          })

          results.push({
            success: response.success,
            name: fileName,
            error: response.message
          })
        } catch (err) {
          results.push({
            success: false,
            name: fileName,
            error: 'Import failed'
          })
        }
      }

      isImporting.value = false

      // Show results summary
      const succeeded = results.filter(r => r.success).length
      const failed = results.filter(r => !r.success)

      if (failed.length === 0) {
        alert(`Successfully imported ${succeeded} book${succeeded !== 1 ? 's' : ''}!`)
      } else if (succeeded === 0) {
        alert(`Failed to import ${failed.length} book${failed.length !== 1 ? 's' : ''}:\n${failed.map(f => `• ${f.name}: ${f.error}`).join('\n')}`)
      } else {
        alert(`Imported ${succeeded} book${succeeded !== 1 ? 's' : ''}.\n\nFailed to import ${failed.length}:\n${failed.map(f => `• ${f.name}: ${f.error}`).join('\n')}`)
      }

      // Reload the book list
      await loadBooks()
    }
  } catch (error) {
    console.error('Failed to import books:', error)
    isImporting.value = false
  }
}

function handleRemoveBook(book: BookInfo) {
  bookToDelete.value = book
  deleteError.value = null
  showDeleteModal.value = true
}

async function confirmDelete() {
  if (!bookToDelete.value) return

  deleteError.value = null
  try {
    const response = await invoke<{ success: boolean; message?: string }>('remove_book_from_library', {
      bookId: bookToDelete.value.id
    })
    
    if (response.success) {
      showDeleteModal.value = false
      bookToDelete.value = null
      // Reload the book list
      await loadBooks()
    } else {
      deleteError.value = response.message || 'Failed to remove book'
    }
  } catch (error) {
    deleteError.value = 'Failed to remove book. Please try again.'
  }
}

function cancelDelete() {
  showDeleteModal.value = false
  bookToDelete.value = null
  deleteError.value = null
}

function closeModal() {
  emit('close')
}
</script>

<style scoped>
/* Domain-specific styles */
.loading-message {
  text-align: center;
  color: var(--color-text-secondary);
  padding: var(--spacing-xl) 0;
}

.book-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.book-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  transition: background-color var(--transition-fast);
}

.book-item:hover {
  background: var(--color-gray-100);
}

.theme-dark .book-item:hover {
  background: var(--color-gray-800);
}

.book-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.book-name {
  font-weight: 500;
  color: var(--color-text);
}

.book-meta {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.remove-button {
  background: var(--color-error-100);
  color: var(--color-error-600);
  border: 1px solid var(--color-error-200);
  border-radius: var(--radius-sm);
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 1.125rem;
  line-height: 1;
  transition: all var(--transition-fast);
}

.remove-button:hover {
  background: var(--color-error-200);
  color: var(--color-error-700);
}

.theme-dark .remove-button {
  background: var(--color-error-900);
  color: var(--color-error-400);
  border-color: var(--color-error-800);
}

.theme-dark .remove-button:hover {
  background: var(--color-error-800);
  color: var(--color-error-300);
}

.import-progress {
  flex: 1;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
</style>