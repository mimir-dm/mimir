<template>
  <div :class="['log-line', levelClass]">
    <span class="line-number">{{ lineNumber }}</span>
    <span class="line-content" v-html="highlightedContent"></span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  content: string
  lineNumber: number
  searchQuery: string
}>()

const levelClass = computed((): string => {
  if (props.content.includes('[ERROR]') || props.content.includes('ERROR')) return 'log-error'
  if (props.content.includes('[WARN]') || props.content.includes('WARN')) return 'log-warn'
  if (props.content.includes('[INFO]') || props.content.includes('INFO')) return 'log-info'
  if (props.content.includes('[DEBUG]') || props.content.includes('DEBUG')) return 'log-debug'
  if (props.content.includes('[TRACE]') || props.content.includes('TRACE')) return 'log-trace'
  return 'log-default'
})

const highlightedContent = computed((): string => {
  if (!props.searchQuery.trim()) return props.content

  const query = props.searchQuery.trim()
  const regex = new RegExp(`(${query})`, 'gi')
  return props.content.replace(regex, '<strong>$1</strong>')
})
</script>

<style scoped>
.log-line {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 0.875rem;
  line-height: 1.4;
  border-radius: var(--radius-sm);
  margin-bottom: 1px;
}

.log-line:hover {
  background: var(--color-surface-variant);
}

.line-number {
  flex-shrink: 0;
  width: 50px;
  text-align: right;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  padding-top: 1px;
}

.line-content {
  flex: 1;
  word-break: break-word;
  white-space: pre-wrap;
}

.log-error {
  border-left: 3px solid #dc2626;
  background: rgba(220, 38, 38, 0.05);
}

.log-warn {
  border-left: 3px solid #f59e0b;
  background: rgba(245, 158, 11, 0.05);
}

.log-info {
  border-left: 3px solid #3b82f6;
  background: rgba(59, 130, 246, 0.05);
}

.log-debug {
  border-left: 3px solid #10b981;
  background: rgba(16, 185, 129, 0.05);
}

.log-trace {
  border-left: 3px solid #8b5cf6;
  background: rgba(139, 92, 246, 0.05);
}

.line-content :deep(strong) {
  font-weight: 700;
  color: var(--color-primary-600);
}
</style>
