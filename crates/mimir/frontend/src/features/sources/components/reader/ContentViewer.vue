<template>
  <div class="content-viewer">
    <WelcomeMessage 
      v-if="!selectedBook"
      title="Welcome to the Reference Library"
      :lines="[
        'Add books to your library to start reading.',
        'Click &quot;Add Book&quot; to import book archives (tar.gz files).'
      ]"
    />
    
    <LoadingSpinner 
      v-else-if="isLoading"
      message="Loading book content..."
    />
    
    <div v-else-if="error" class="error-message">
      <h2>Failed to load book</h2>
      <p>{{ error }}</p>
    </div>
    
    <div v-else-if="content" class="book-content">
      <div class="content-wrapper">
        <ContentRenderer :content="content" :selected-book="selectedBook" />
      </div>
    </div>
    
    <div v-else class="error-message">
      <h2>No content available</h2>
      <p>Please select a section from the table of contents.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import WelcomeMessage from '../../../../shared/components/ui/WelcomeMessage.vue'
import LoadingSpinner from '../../../../shared/components/ui/LoadingSpinner.vue'
import ContentRenderer from '../content/ContentRenderer.vue'
import type { BookInfo, BookSection } from '../../../../types/book'

interface Props {
  selectedBook: BookInfo | null
  content: BookSection | null
  isLoading: boolean
  error: string | null
}

const props = defineProps<Props>()
</script>

<style scoped>
.content-viewer {
  height: 100%;
  padding: var(--spacing-lg, 16px);
  overflow-y: auto;
}

.error-message {
  text-align: center;
  padding: var(--spacing-xl, 24px);
}

.error-message h2 {
  color: var(--color-danger, #ff4444);
  margin-bottom: var(--spacing-md, 12px);
}

.error-message p {
  color: var(--color-text-secondary, #999);
}

.book-content {
  height: 100%;
}

.content-wrapper {
  max-width: 900px;
  margin: 0 auto;
}

/* Content typography */
.book-content :deep(h1) {
  font-size: 2rem;
  margin-top: 0;
  margin-bottom: var(--spacing-lg, 16px);
  color: var(--color-text, #e0e0e0);
}

.book-content :deep(h2) {
  font-size: 1.5rem;
  margin-top: var(--spacing-xl, 24px);
  margin-bottom: var(--spacing-md, 12px);
  color: var(--color-text, #e0e0e0);
}

.book-content :deep(h3) {
  font-size: 1.25rem;
  margin-top: var(--spacing-lg, 16px);
  margin-bottom: var(--spacing-sm, 8px);
  color: var(--color-text, #e0e0e0);
}

.book-content :deep(h4) {
  font-size: 1.1rem;
  margin-top: var(--spacing-md, 12px);
  margin-bottom: var(--spacing-sm, 8px);
  color: var(--color-text, #e0e0e0);
}

.book-content :deep(p) {
  margin-bottom: var(--spacing-md, 12px);
  line-height: 1.6;
  color: var(--color-text-secondary, #ccc);
}

.book-content :deep(ul),
.book-content :deep(ol) {
  margin-bottom: var(--spacing-md, 12px);
  padding-left: var(--spacing-xl, 24px);
  color: var(--color-text-secondary, #ccc);
}

.book-content :deep(li) {
  margin-bottom: var(--spacing-xs, 4px);
  line-height: 1.6;
}

.book-content :deep(strong) {
  color: var(--color-text, #e0e0e0);
  font-weight: 600;
}

.book-content :deep(em) {
  font-style: italic;
  color: var(--color-text-emphasis, #f0f0f0);
}

.book-content :deep(code) {
  background: var(--color-surface, #1a1a1a);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 0.9em;
}

.book-content :deep(blockquote) {
  border-left: 4px solid var(--color-primary, #4a9eff);
  padding-left: var(--spacing-md, 12px);
  margin: var(--spacing-md, 12px) 0;
  font-style: italic;
  color: var(--color-text-secondary, #ccc);
}
</style>