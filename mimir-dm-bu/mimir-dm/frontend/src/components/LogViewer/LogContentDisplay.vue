<template>
  <div v-if="loading" class="loading-container">
    <div class="spinner"></div>
    <p>Loading log content...</p>
  </div>

  <div v-else-if="error" class="error-container">
    <p class="error-message">{{ error }}</p>
    <button @click="$emit('retry')" class="retry-button">Try Again</button>
  </div>

  <div v-else class="log-content-container">
    <div
      ref="logContentRef"
      class="log-content"
      @scroll="$emit('scroll', $event)"
    >
      <div v-if="filteredLines.length === 0" class="no-content">
        <p v-if="searchQuery">No lines match your search criteria</p>
        <p v-else-if="fileName">No content to display</p>
        <p v-else>Select a log file to view</p>
      </div>

      <div v-else class="log-lines">
        <LogLine
          v-for="(line, index) in filteredLines"
          :key="`${line.lineNumber}-${index}`"
          :content="line.content"
          :line-number="line.lineNumber"
          :search-query="searchQuery"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import LogLine from './LogLine.vue'

interface LogLine {
  content: string
  lineNumber: number
}

defineProps<{
  loading: boolean
  error: string | null
  filteredLines: LogLine[]
  searchQuery: string
  fileName: string
}>()

defineEmits<{
  scroll: [event: Event]
  retry: []
}>()

const logContentRef = ref<HTMLElement | null>(null)

// Expose ref for parent to access (for scrolling)
defineExpose({
  logContentRef
})
</script>

<style scoped>
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: var(--spacing-md);
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--color-border);
  border-top: 3px solid var(--color-primary-500);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: var(--spacing-md);
}

.error-message {
  color: var(--color-error, #dc2626);
}

.retry-button {
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
}

.log-content-container {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.log-content {
  height: 100%;
  overflow-y: auto;
  padding: var(--spacing-sm);
  background: var(--color-surface);
}

.no-content {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-secondary);
  font-style: italic;
}

.log-lines {
  display: flex;
  flex-direction: column;
}
</style>
