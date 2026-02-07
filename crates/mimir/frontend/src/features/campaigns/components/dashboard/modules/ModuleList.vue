<template>
  <div class="sidebar-panel">
    <div class="sidebar-header">
      <h3>Modules</h3>
      <button class="btn-add" @click="$emit('create')" title="Create Module">+</button>
    </div>

    <div v-if="loading" class="modules-loading">Loading...</div>
    <div v-else-if="modules.length === 0" class="modules-empty">No modules yet</div>
    <div v-else class="modules-list">
      <div
        v-for="(mod, index) in modules"
        :key="mod.id"
        class="module-item"
        :class="{ selected: selectedModuleId === mod.id }"
        @click="$emit('select', mod)"
      >
        <span class="module-number">#{{ mod.module_number }}</span>
        <span class="module-name">{{ mod.name }}</span>
        <span class="module-reorder-buttons">
          <button
            class="btn-reorder"
            :disabled="index === 0"
            title="Move up"
            @click.stop="$emit('reorder', mod.id, mod.module_number - 1)"
          >&#9650;</button>
          <button
            class="btn-reorder"
            :disabled="index === modules.length - 1"
            title="Move down"
            @click.stop="$emit('reorder', mod.id, mod.module_number + 1)"
          >&#9660;</button>
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Module } from '@/types'

defineProps<{
  modules: Module[]
  selectedModuleId?: string
  loading?: boolean
}>()

defineEmits<{
  select: [mod: Module]
  create: []
  reorder: [moduleId: string, newPosition: number]
}>()
</script>

<style scoped>
.sidebar-panel {
  width: 280px;
  min-width: 240px;
  max-width: 320px;
  border-right: 1px solid var(--color-border);
  overflow-y: auto;
  background: var(--color-surface);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--color-border);
}

.sidebar-header h3 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text);
}

.btn-add {
  width: 20px;
  height: 20px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}

.btn-add:hover {
  background: var(--color-primary-500);
  color: var(--color-background);
  border-color: var(--color-primary-500);
}

.modules-loading,
.modules-empty {
  padding: var(--spacing-sm) var(--spacing-md);
  font-size: 0.8rem;
  color: var(--color-text-secondary);
}

.modules-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--spacing-xs);
}

.module-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  font-size: 0.875rem;
  color: var(--color-text);
  cursor: pointer;
  border-radius: var(--radius-sm);
}

.module-item:hover {
  background: var(--color-surface-variant);
}

.module-item.selected {
  background: var(--color-primary-100);
}

.module-number {
  font-weight: 600;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
}

.module-reorder-buttons {
  display: flex;
  flex-direction: column;
  gap: 1px;
  margin-left: auto;
  opacity: 0;
  transition: opacity 0.15s;
}

.module-item:hover .module-reorder-buttons {
  opacity: 1;
}

.btn-reorder {
  background: none;
  border: none;
  padding: 0 2px;
  font-size: 0.5rem;
  line-height: 1;
  color: var(--color-text-secondary);
  cursor: pointer;
  border-radius: 2px;
}

.btn-reorder:hover:not(:disabled) {
  color: var(--color-primary);
  background: var(--color-surface-variant);
}

.btn-reorder:disabled {
  opacity: 0.2;
  cursor: default;
}

.module-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
