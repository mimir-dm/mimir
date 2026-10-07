<template>
  <Panel title="Library" variant="surface">

    <div class="library-content">
      <div v-if="isLoadingLibrary" class="loading-message">
        Loading library...
      </div>

      <div v-else-if="libraryBooks.length === 0" class="empty-message">
        <p>No books in library</p>
        <p v-if="isDevelopment" class="dev-note">
          Running in development mode
        </p>
      </div>

      <div v-else class="book-list">
        <!-- Select All / Deselect All controls (catalog mode only) -->
        <div v-if="mode === 'catalog'" class="select-all-controls">
          <button
            class="select-all-btn"
            @click="selectAll"
            :disabled="allSelected"
          >
            Select All
          </button>
          <button
            class="select-all-btn"
            @click="deselectAll"
            :disabled="noneSelected"
          >
            Deselect All
          </button>
          <span class="selection-count">{{ internalSelectedSources.length }}/{{ libraryBooks.length }}</span>
        </div>
        <!-- Reading Mode: Clickable books -->
        <div 
          v-if="mode === 'reading'"
          v-for="book in libraryBooks" 
          :key="book.id"
          :class="['book-item', { active: selectedBook?.id === book.id }]"
          @click="$emit('select', book)"
        >
          <div class="book-info">
            <span class="book-name">{{ book.name }}</span>
            <span class="book-meta">
              <span v-if="book.id === 'test-book'" class="dev-badge">DEV</span>
              <span v-if="book.image_count">{{ book.image_count }} images</span>
            </span>
          </div>
          <button 
            @click.stop="handleRemoveBook(book)"
            class="remove-btn"
            title="Remove from library"
          >
            ×
          </button>
        </div>
        
        <!-- Catalog Mode: Books with checkboxes -->
        <div 
          v-else
          v-for="book in libraryBooks" 
          :key="`catalog-${book.id}`"
          class="book-item-checkbox"
        >
          <label class="book-checkbox-label">
            <input 
              type="checkbox" 
              :value="book.id"
              :checked="internalSelectedSources.includes(book.id)"
              @change="toggleSource(book.id)"
            />
            <div class="book-info">
              <span class="book-name">{{ book.name }}</span>
              <span class="book-meta">
                <span v-if="book.id === 'test-book'" class="dev-badge">DEV</span>
              </span>
            </div>
          </label>
        </div>
      </div>
    </div>
  </Panel>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import Panel from '@/shared/components/layout/Panel.vue'
import type { BookInfo } from '@/types/book'

interface Props {
  libraryBooks: BookInfo[]
  selectedBook: BookInfo | null
  isLoadingLibrary: boolean
  isDevelopment: boolean
  mode?: 'reading' | 'catalog'
}

interface Emits {
  (e: 'select', book: BookInfo): void
  (e: 'updateSources', sources: string[]): void
  (e: 'remove', book: BookInfo): void
}

const props = withDefaults(defineProps<Props>(), {
  mode: 'reading'
})
const emit = defineEmits<Emits>()

// Internal tracking of selected sources
const internalSelectedSources = ref<string[]>([])

// Computed properties for select all / deselect all
const allSelected = computed(() =>
  props.libraryBooks.length > 0 &&
  internalSelectedSources.value.length === props.libraryBooks.length
)

const noneSelected = computed(() =>
  internalSelectedSources.value.length === 0
)

// Initialize sources when books are loaded - select all by default
watch(() => props.libraryBooks, (books) => {
  if (books.length > 0 && internalSelectedSources.value.length === 0) {
    internalSelectedSources.value = books.map(b => b.id)
    emit('updateSources', internalSelectedSources.value)
  }
}, { immediate: true })

function handleRemoveBook(book: BookInfo) {
  emit('remove', book)
}

function toggleSource(bookId: string) {
  const index = internalSelectedSources.value.indexOf(bookId)
  if (index > -1) {
    internalSelectedSources.value.splice(index, 1)
  } else {
    internalSelectedSources.value.push(bookId)
  }
  emit('updateSources', [...internalSelectedSources.value])
}

function selectAll() {
  internalSelectedSources.value = props.libraryBooks.map(b => b.id)
  emit('updateSources', [...internalSelectedSources.value])
}

function deselectAll() {
  internalSelectedSources.value = []
  emit('updateSources', [])
}
</script>

<style scoped>
.library-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.select-all-controls {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs, 4px);
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.select-all-btn {
  padding: 2px 8px;
  font-size: 0.75rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 3px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.select-all-btn:hover:not(:disabled) {
  background: var(--color-surface-hover);
  color: var(--color-text);
}

.select-all-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.selection-count {
  margin-left: auto;
  font-size: 0.75rem;
  color: var(--legacy-text-tertiary);
}

.loading-message,
.empty-message {
  padding: var(--spacing-lg, 16px);
  text-align: center;
  color: var(--color-text-secondary);
}

.dev-note {
  margin-top: var(--spacing-sm, 8px);
  font-size: 0.75rem;
  color: var(--legacy-text-tertiary);
}

.book-list {
  flex: 1;
  overflow-y: auto;
}

.book-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-sm, 8px) var(--spacing-md, 12px);
  cursor: pointer;
  transition: background-color 0.2s;
}

.book-item:hover {
  background: var(--color-surface-hover);
}

.book-item.active {
  background: color-mix(in srgb, var(--legacy-primary) 10%, transparent);
  border-left: 3px solid var(--legacy-primary);
}

.book-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.book-name {
  color: var(--color-text);
  font-size: 0.9rem;
}

.book-meta {
  font-size: 0.75rem;
  color: var(--legacy-text-tertiary);
  display: flex;
  align-items: center;
  gap: var(--spacing-xs, 4px);
}

.dev-badge {
  background: var(--color-warning);
  color: var(--color-background);
  padding: 1px 4px;
  border-radius: 3px;
  font-weight: 600;
  text-transform: uppercase;
}

.remove-btn {
  width: 20px;
  height: 20px;
  padding: 0;
  background: transparent;
  color: var(--color-text-secondary);
  border: none;
  border-radius: 3px;
  cursor: pointer;
  font-size: 1.2rem;
  line-height: 1;
  opacity: 0;
  transition: opacity 0.2s, color 0.2s;
}

.book-item:hover .remove-btn,
.book-item:focus-within .remove-btn {
  opacity: 1;
}

.remove-btn:hover {
  color: var(--legacy-danger);
  background: color-mix(in srgb, var(--legacy-danger) 10%, transparent);
}

/* Catalog checkbox mode styles */
.book-item-checkbox {
  padding: var(--spacing-xs, 4px) var(--spacing-md, 12px);
  transition: background-color 0.2s;
}

.book-item-checkbox:hover {
  background: var(--color-surface-hover);
}

.book-checkbox-label {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm, 8px);
  cursor: pointer;
}

.book-checkbox-label input[type="checkbox"] {
  cursor: pointer;
  margin: 0;
}

.book-checkbox-label .book-info {
  flex: 1;
}
</style>