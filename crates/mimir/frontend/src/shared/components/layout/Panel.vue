<template>
  <div class="panel" :class="panelClass">
    <div v-if="title" class="panel-header">
      <h3 class="panel-title">{{ title }}</h3>
      <div v-if="$slots.actions" class="panel-actions">
        <slot name="actions"></slot>
      </div>
    </div>
    <div class="panel-content">
      <slot></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  title?: string
  variant?: 'default' | 'surface' | 'elevated'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default'
})

const panelClass = computed(() => `panel-${props.variant}`)
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md, 12px) var(--spacing-lg, 16px);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.panel-title {
  font-size: var(--font-size-lg, 1.125rem);
  font-weight: 600;
  color: var(--color-text);
  margin: 0;
}

.panel-actions {
  display: flex;
  gap: var(--spacing-sm, 8px);
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-md, 12px);
}

/* Variants */
.panel-default {
  background: var(--color-background);
}

.panel-surface {
  background: var(--color-surface);
}

.panel-elevated {
  background: var(--color-surface);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}
</style>