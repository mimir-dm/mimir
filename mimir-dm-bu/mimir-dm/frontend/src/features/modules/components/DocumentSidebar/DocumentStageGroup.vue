<template>
  <div
    class="stage-group"
    v-show="isAccessible"
  >
    <div class="stage-header">
      <h4>{{ stageName }} ({{ completed }}/{{ total }})</h4>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: percentage + '%' }"
        ></div>
      </div>
    </div>
    <div class="document-items">
      <DocumentItem
        v-for="doc in documents"
        :key="doc.templateId"
        :doc="doc"
        :is-selected="selectedDocumentId === doc.instance?.id"
        :is-locked="!isAccessible"
        :edit-icon="editIcon"
        :locked-icon="lockedIcon"
        :show-optional-label="true"
        @click="$emit('documentClick', doc)"
        @toggle-completion="$emit('toggleCompletion', doc)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import DocumentItem from './DocumentItem.vue'

defineProps<{
  stageName: string
  documents: any[]
  completed: number
  total: number
  percentage: number
  isAccessible: boolean
  selectedDocumentId?: number | string
  editIcon: string
  lockedIcon: string
}>()

defineEmits<{
  documentClick: [doc: any]
  toggleCompletion: [doc: any]
}>()
</script>

<style scoped>
.stage-group {
  padding: var(--spacing-sm) 0;
  border-bottom: 1px solid var(--color-border);
}

.stage-group:last-child {
  border-bottom: none;
}

.stage-header {
  padding: 0 var(--spacing-sm);
  margin-bottom: var(--spacing-sm);
}

.stage-header h4 {
  margin: 0 0 var(--spacing-xs) 0;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
}

.progress-bar {
  height: 16px;
  background-color: var(--color-surface-variant);
  border-radius: 8px;
  overflow: hidden;
  position: relative;
}

.progress-fill {
  height: 100%;
  background-color: var(--color-primary-400);
  border-radius: 8px;
  transition: width var(--transition-base);
  position: relative;
}

.progress-bar::after {
  content: '';
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    45deg,
    transparent,
    transparent 10px,
    var(--color-overlay-light) 10px,
    var(--color-overlay-light) 20px
  );
  border-radius: 8px;
}

.document-items {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}
</style>
