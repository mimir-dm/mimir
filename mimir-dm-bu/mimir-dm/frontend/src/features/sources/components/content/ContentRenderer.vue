<template>
  <div class="content-renderer" v-html="renderedContent"></div>
</template>

<script setup lang="ts">
import { computed, watch, nextTick, inject } from 'vue'
import { renderSection } from '../../utils/renderers/contentRenderer'
import { useImageLoader } from '../../composables/useImageLoader'
import type { BookSection } from '../../../../types/book'
import type { BookInfo } from '../../../../types/book'

interface Props {
  content: BookSection
  selectedBook?: BookInfo | null
}

const props = defineProps<Props>()

// Get selected book from parent if not passed as prop
const injectedBook = inject<BookInfo | null>('selectedBook', null)
const currentBook = computed(() => props.selectedBook || injectedBook)

const { loadBookImage } = useImageLoader()

const renderedContent = computed(() => {
  if (!props.content) return ''
  return renderSection(props.content)
})

// Function to load all images in content
function loadImages() {
  if (!currentBook.value) {
    return
  }
  
  // Find all image placeholders and load them
  const imageWrappers = document.querySelectorAll('.image-wrapper[data-image-path]')
  
  imageWrappers.forEach((wrapper) => {
    const imagePath = wrapper.getAttribute('data-image-path')
    const imageId = wrapper.id
    if (imagePath && imageId) {
      loadBookImage(currentBook.value!.id, imagePath, imageId)
    }
  })
}

// Handle image loading after content renders
watch([renderedContent, () => props.selectedBook], () => {
  nextTick(() => {
    loadImages()
  })
}, { immediate: true })
</script>

<style scoped>
.content-renderer {
  /* Content-specific styles are handled by parent */
}

/* Game element styles */
.content-renderer :deep(.dice-roll) {
  color: var(--color-warning, #ffaa00);
  font-weight: 600;
  font-family: 'Courier New', monospace;
}

.content-renderer :deep(.damage-roll) {
  color: var(--color-danger, #ff4444);
  font-weight: 600;
}

.content-renderer :deep(.d20-check) {
  color: var(--color-info, #00aaff);
  font-weight: 600;
}

.content-renderer :deep(.dc-check) {
  color: var(--color-primary, #4a9eff);
  font-weight: 600;
  text-transform: uppercase;
}

.content-renderer :deep(.skill-check) {
  color: var(--color-success, #44ff44);
  font-style: italic;
}

.content-renderer :deep(.action-name) {
  color: var(--color-warning, #ffaa00);
  font-weight: 600;
  font-style: italic;
}

.content-renderer :deep(.condition) {
  color: var(--color-condition, #ff88ff);
  font-style: italic;
  cursor: help;
}

.content-renderer :deep(.status) {
  color: var(--color-status, #88ff88);
  font-style: italic;
  cursor: help;
}

.content-renderer :deep(.note) {
  color: var(--color-note, #ffff88);
  font-style: italic;
}

.content-renderer :deep(.recharge) {
  color: var(--color-recharge, #ff8888);
  font-weight: 600;
}

.content-renderer :deep(.hit-bonus) {
  color: var(--color-success, #44ff44);
  font-weight: 600;
}

/* Inset and special content - removed to use global styles from BookReader.vue */

/* Image containers */
.content-renderer :deep(.image-container) {
  margin: var(--spacing-lg, 16px) 0;
  text-align: center;
}

.content-renderer :deep(.image-wrapper) {
  display: inline-block;
  max-width: 100%;
}

.content-renderer :deep(.image-placeholder) {
  background: var(--color-surface, #1a1a1a);
  border: 1px dashed var(--color-border, #333);
  padding: var(--spacing-lg, 16px);
  color: var(--color-text-secondary, #999);
}

.content-renderer :deep(.image-caption) {
  margin-top: var(--spacing-sm, 8px);
  font-size: 0.9rem;
  color: var(--color-text-secondary, #999);
  font-style: italic;
}
</style>