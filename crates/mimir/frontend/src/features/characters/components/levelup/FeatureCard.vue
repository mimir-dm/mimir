<template>
  <button
    type="button"
    class="feature-card"
    :class="{
      selected: selected,
      disabled: disabled,
      compact: compact,
      'has-prereq': !!prereqs
    }"
    :disabled="disabled"
    @click="$emit('click')"
  >
    <div class="feature-name">{{ name }}</div>
    <div v-if="source" class="feature-source">{{ source }}</div>
    <div v-if="cost" class="feature-cost">{{ cost }}</div>
    <div v-if="prereqs" class="feature-prereq">{{ prereqs }}</div>
    <div v-if="description && !compact" class="feature-description">
      {{ truncatedDescription }}
    </div>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  name: string
  source?: string
  description?: string
  cost?: string | number
  prereqs?: string
  selected?: boolean
  disabled?: boolean
  compact?: boolean
  maxDescriptionLength?: number
}>(), {
  selected: false,
  disabled: false,
  compact: false,
  maxDescriptionLength: 100
})

defineEmits<{
  (e: 'click'): void
}>()

const truncatedDescription = computed(() => {
  if (!props.description) return ''
  if (props.description.length <= props.maxDescriptionLength) return props.description
  return props.description.slice(0, props.maxDescriptionLength) + '...'
})
</script>

<style scoped>
.feature-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--spacing-xs);
  padding: var(--spacing-md);
  border: 2px solid var(--color-border);
  border-radius: var(--radius-lg);
  background: var(--color-surface);
  cursor: pointer;
  transition: all var(--transition-base);
  text-align: left;
  width: 100%;
}

.feature-card.compact {
  padding: var(--spacing-sm) var(--spacing-md);
}

.feature-card:hover:not(.disabled) {
  border-color: var(--color-primary-300);
  background: var(--color-surface-variant);
}

.feature-card.selected {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50, rgba(var(--color-primary-rgb), 0.1));
}

.feature-card.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.feature-card.has-prereq:not(.selected) {
  border-style: dashed;
}

.feature-name {
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--color-text);
}

.feature-source {
  font-size: 0.7rem;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.feature-cost {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-primary-600);
}

.feature-prereq {
  font-size: 0.7rem;
  color: var(--color-warning, #f59e0b);
  font-style: italic;
}

.feature-description {
  font-size: 0.8rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
}
</style>
