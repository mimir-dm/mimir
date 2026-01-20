<template>
  <div class="log-viewer-header">
    <div class="header-info">
      <h1 class="window-title">
        Log Viewer
        <span v-if="fileName" class="file-name">- {{ fileName }}</span>
      </h1>
      <div class="file-info">
        <span v-if="totalLines" class="line-count">
          {{ totalLines }} lines
        </span>
        <span v-if="fileName && lastUpdated" class="last-updated">
          Updated: {{ formatTime(lastUpdated) }}
        </span>
      </div>
    </div>

    <div class="header-controls">
      <button
        @click="$emit('toggleAutoScroll')"
        :class="['control-button', { active: autoScroll }]"
        title="Auto-scroll to bottom"
      >
        Auto-scroll
      </button>
      <button
        @click="$emit('toggleLiveMode')"
        :class="['control-button', { active: liveMode }]"
        title="Live updates"
      >
        {{ liveMode ? 'Pause' : 'Play' }} Live
      </button>
      <button
        @click="$emit('refresh')"
        class="control-button"
        title="Refresh"
      >
        Refresh
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  fileName: string
  totalLines: number
  lastUpdated: Date | null
  autoScroll: boolean
  liveMode: boolean
}>()

defineEmits<{
  toggleAutoScroll: []
  toggleLiveMode: []
  refresh: []
}>()

const formatTime = (date: Date): string => {
  return date.toLocaleTimeString()
}
</script>

<style scoped>
.log-viewer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md) var(--spacing-lg);
  background: var(--color-surface-variant);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.header-info {
  flex: 1;
}

.window-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin: 0 0 var(--spacing-xs) 0;
}

.file-name {
  font-weight: 400;
  color: var(--color-primary-600);
}

.file-info {
  display: flex;
  gap: var(--spacing-md);
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.header-controls {
  display: flex;
  gap: var(--spacing-sm);
}

.control-button {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.control-button:hover {
  background: var(--color-gray-100);
  border-color: var(--color-border-hover);
}

.control-button.active {
  background: var(--color-primary-100);
  border-color: var(--color-primary-300);
  color: var(--color-primary-700);
}
</style>
