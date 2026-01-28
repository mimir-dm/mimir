<template>
  <div class="feature-section">
    <h4 class="section-title">
      {{ title }}
      <span v-if="showSlotCount" class="slot-count">({{ selectedCount }}/{{ maxSlots }})</span>
    </h4>
    <p v-if="description" class="section-note">{{ description }}</p>

    <!-- Search box (optional) -->
    <div v-if="searchable" class="search-box">
      <input
        :value="searchQuery"
        type="text"
        class="search-input"
        :placeholder="searchPlaceholder"
        @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- Feature grid -->
    <div v-if="items.length > 0" class="feature-grid" :class="{ compact: compact }">
      <FeatureCard
        v-for="item in items"
        :key="getItemKey(item)"
        :name="item.name"
        :source="item.source"
        :description="item.description"
        :cost="item.cost"
        :prereqs="item.prereqs"
        :selected="isItemSelected(item)"
        :disabled="isItemDisabled(item)"
        :compact="compact"
        @click="handleItemClick(item)"
      />
    </div>

    <!-- Empty state -->
    <div v-else class="empty-state">
      <slot name="empty">
        <p>No options available</p>
      </slot>
    </div>

    <!-- Selected items list -->
    <div v-if="selectedItems.length > 0 && showSelectedList" class="selected-list">
      <span class="selected-label">Selected:</span>
      <span
        v-for="item in selectedItems"
        :key="getSelectedKey(item)"
        class="selected-tag"
      >
        {{ getSelectedName(item) }}
        <button
          v-if="allowRemove"
          type="button"
          class="remove-tag"
          @click="$emit('remove', item)"
        >&times;</button>
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import FeatureCard from './FeatureCard.vue'
import type { FeatureItem } from '@/features/characters/composables/useFeatureSelection'

const props = withDefaults(defineProps<{
  title: string
  description?: string
  items: FeatureItem[]
  selectedItems: Array<FeatureItem | string>
  maxSlots?: number
  compact?: boolean
  searchable?: boolean
  searchQuery?: string
  searchPlaceholder?: string
  showSelectedList?: boolean
  showSlotCount?: boolean
  allowRemove?: boolean
  /** Custom function to get item key */
  getItemKey?: (item: FeatureItem) => string
  /** Custom function to check if item is selected */
  isItemSelected?: (item: FeatureItem) => boolean
  /** Custom function to check if item is disabled */
  isItemDisabled?: (item: FeatureItem) => boolean
}>(), {
  compact: false,
  searchable: false,
  searchQuery: '',
  searchPlaceholder: 'Search...',
  showSelectedList: true,
  showSlotCount: true,
  allowRemove: false,
  maxSlots: Infinity,
  getItemKey: (item: FeatureItem) => `${item.name}-${item.source}`,
  isItemSelected: () => false,
  isItemDisabled: () => false
})

const emit = defineEmits<{
  (e: 'select', item: FeatureItem): void
  (e: 'remove', item: FeatureItem | string): void
  (e: 'update:searchQuery', value: string): void
}>()

const selectedCount = computed(() => props.selectedItems.length)

function handleItemClick(item: FeatureItem) {
  if (props.isItemDisabled(item)) return
  emit('select', item)
}

function getSelectedKey(item: FeatureItem | string): string {
  if (typeof item === 'string') return item
  return `${item.name}-${item.source}`
}

function getSelectedName(item: FeatureItem | string): string {
  if (typeof item === 'string') return item
  return item.name
}

import { computed } from 'vue'
</script>

<style scoped>
.feature-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border-radius: var(--radius-md);
}

.section-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text);
  display: flex;
  align-items: baseline;
  gap: var(--spacing-sm);
}

.slot-count {
  font-size: 0.85rem;
  font-weight: normal;
  color: var(--color-text-secondary);
}

.section-note {
  margin: 0;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.search-box {
  margin-bottom: var(--spacing-sm);
}

.search-input {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 0.875rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: var(--spacing-md);
}

.feature-grid.compact {
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: var(--spacing-sm);
}

.empty-state {
  padding: var(--spacing-md);
  text-align: center;
  color: var(--color-text-secondary);
}

.empty-state p {
  margin: 0;
}

.selected-list {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--spacing-sm);
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--color-border);
}

.selected-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.selected-tag {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-primary-100);
  color: var(--color-primary-700);
  border-radius: var(--radius-sm);
  font-size: 0.8rem;
  font-weight: 500;
}

.remove-tag {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: 50%;
  color: var(--color-primary-600);
  font-size: 1rem;
  line-height: 1;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.remove-tag:hover {
  background: var(--color-primary-200);
  color: var(--color-primary-800);
}
</style>
