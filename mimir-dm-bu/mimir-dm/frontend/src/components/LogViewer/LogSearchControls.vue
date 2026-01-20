<template>
  <div class="log-controls">
    <div class="search-bar">
      <input
        :value="searchQuery"
        @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
        type="text"
        placeholder="Search logs..."
        class="search-input"
      />
      <button v-if="searchQuery" @click="$emit('clearSearch')" class="clear-search">
        ✕
      </button>
    </div>

    <div v-if="!isChatLog" class="log-level-filters">
      <button
        v-for="level in logLevels"
        :key="level"
        @click="$emit('toggleLevel', level)"
        :class="['level-button', level.toLowerCase(), { active: activeLevels.has(level) }]"
      >
        {{ level }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  searchQuery: string
  logLevels: string[]
  activeLevels: Set<string>
  isChatLog: boolean
}>()

defineEmits<{
  'update:searchQuery': [value: string]
  clearSearch: []
  toggleLevel: [level: string]
}>()
</script>

<style scoped>
.log-controls {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-lg);
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.search-bar {
  position: relative;
  flex: 1;
  max-width: 300px;
}

.search-input {
  width: 100%;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: 0.875rem;
  transition: border-color var(--transition-fast);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
}

.clear-search {
  position: absolute;
  right: var(--spacing-xs);
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  padding: 0;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.log-level-filters {
  display: flex;
  gap: var(--spacing-xs);
}

.level-button {
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.level-button.error {
  border-color: #dc2626;
  color: #dc2626;
}

.level-button.warn {
  border-color: #f59e0b;
  color: #f59e0b;
}

.level-button.info {
  border-color: #3b82f6;
  color: #3b82f6;
}

.level-button.debug {
  border-color: #10b981;
  color: #10b981;
}

.level-button.trace {
  border-color: #8b5cf6;
  color: #8b5cf6;
}

.level-button.active {
  font-weight: 700;
  border-width: 2px;
}
</style>
