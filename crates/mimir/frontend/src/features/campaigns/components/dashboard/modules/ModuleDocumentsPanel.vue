<template>
  <section class="dashboard-section documents-section">
    <div class="section-header">
      <h3>Documents</h3>
      <button class="btn-add" @click="$emit('create')" title="Create Document">+</button>
    </div>
    <div v-if="documents.length === 0" class="section-empty">
      No documents yet
    </div>
    <div v-else class="document-cards">
      <div
        v-for="doc in documents"
        :key="doc.id"
        class="document-card"
        @click="$emit('select', doc)"
      >
        <span class="doc-title">{{ formatDocumentTitle(doc.title || 'Untitled') }}</span>
        <button
          class="doc-delete-btn"
          @click="handleDelete(doc, $event)"
          title="Delete document"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
          </svg>
        </button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { Document } from '@/types'

defineProps<{
  documents: Document[]
}>()

const emit = defineEmits<{
  select: [doc: Document]
  create: []
  delete: [doc: Document]
}>()

function formatDocumentTitle(templateId: string): string {
  return templateId
    .replace(/[-_]/g, ' ')
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

function handleDelete(doc: Document, event: Event) {
  event.stopPropagation()
  emit('delete', doc)
}
</script>

<style scoped>
.dashboard-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-md);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-sm);
  padding-bottom: var(--spacing-xs);
  border-bottom: 1px solid var(--color-border);
}

.section-header h3 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
}

.btn-add {
  width: 20px;
  height: 20px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}

.btn-add:hover {
  background: var(--color-primary-500);
  color: var(--color-background);
  border-color: var(--color-primary-500);
}

.section-empty {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  text-align: center;
  padding: var(--spacing-md);
}

.document-cards {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.document-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.document-card:hover {
  border-color: var(--color-primary-500);
}

.doc-title {
  font-size: 0.8rem;
  color: var(--color-text);
}

.doc-delete-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  padding: 0;
  background: transparent;
  color: var(--color-text-muted);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  opacity: 0;
  transition: all var(--transition-fast);
  flex-shrink: 0;
  margin-left: var(--spacing-xs);
}

.document-card:hover .doc-delete-btn {
  opacity: 1;
}

.doc-delete-btn:hover {
  background: var(--color-error-100, rgba(239, 68, 68, 0.1));
  color: var(--color-error);
}

.doc-delete-btn svg {
  width: 14px;
  height: 14px;
}
</style>
