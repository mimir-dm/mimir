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
          <Trash2 :stroke-width="1.5" aria-hidden="true" />
        </button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { Document } from '@/types'
import { Trash2 } from '@lucide/vue'

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
  color: var(--color-text-secondary);
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
  background: var(--color-error-100);
  color: var(--color-error);
}

.doc-delete-btn svg {
  width: 14px;
  height: 14px;
}
</style>
