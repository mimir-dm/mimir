<template>
  <div
    class="document-item"
    :class="{
      selected: isSelected,
      completed: doc.instance?.completed_at,
      locked: isLocked
    }"
  >
    <!-- Icon on the left -->
    <img
      v-if="isLocked"
      :src="lockedIcon"
      alt="Locked"
      class="document-icon locked"
      title="Stage not yet accessible"
    />
    <img
      v-else
      :src="editIcon"
      alt="Edit"
      class="document-icon"
      @click="$emit('click', doc)"
      title="Edit document"
    />

    <!-- Document title -->
    <span
      class="document-title"
      :class="{ optional: !doc.required }"
      @click="$emit('click', doc)"
    >
      {{ doc.title }}
      <span v-if="!doc.required && showOptionalLabel" class="optional-label">(Optional)</span>
    </span>

    <!-- Completion checkbox -->
    <button
      v-if="doc.instance && !isLocked"
      class="completion-checkbox"
      :class="{ checked: doc.instance?.completed_at }"
      @click.stop="$emit('toggleCompletion', doc)"
      :title="doc.instance?.completed_at ? 'Mark as incomplete' : 'Mark as complete'"
    >
      <span v-if="doc.instance?.completed_at">✓</span>
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  doc: any
  isSelected: boolean
  isLocked: boolean
  editIcon: string
  lockedIcon: string
  showOptionalLabel?: boolean
}>()

defineEmits<{
  click: [doc: any]
  toggleCompletion: [doc: any]
}>()
</script>

<style scoped>
.document-item {
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: all var(--transition-base);
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  border-radius: var(--radius-sm);
  border: 2px solid transparent;
}

.document-item:hover:not(.locked) {
  border-color: var(--color-border);
}

.document-item.selected {
  border-color: var(--color-primary-400);
}

.document-item.completed {
  opacity: 0.8;
}

.document-item.locked {
  opacity: 0.5;
  cursor: not-allowed;
}

.document-icon {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  object-fit: contain;
}

.document-icon.locked {
  cursor: not-allowed;
}

.document-title {
  font-size: 0.875rem;
  color: var(--color-text);
  flex: 1;
}

.document-title.optional {
  font-style: italic;
}

.optional-label {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  font-style: normal;
  margin-left: var(--spacing-xs);
}

.completion-checkbox {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  border: 2px solid var(--color-border);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-base);
  margin-left: auto;
}

.completion-checkbox:hover {
  border-color: var(--color-primary-400);
  background-color: var(--color-primary-50);
}

.completion-checkbox.checked {
  background-color: var(--color-success);
  border-color: var(--color-success);
  color: white;
}

.completion-checkbox.checked:hover {
  background-color: var(--color-success-dark);
  border-color: var(--color-success-dark);
}
</style>
