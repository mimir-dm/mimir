<template>
  <CatalogTable 
    :data="objects"
    :config="objectConfig"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @sort="emit('sort', $event)"
    @select="emit('select', $event)"
  />
</template>

<script setup lang="ts">
import type { ObjectSummary } from '../../composables/catalog'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { objectConfig } from '../../../../shared/components/catalog/config/objectConfig'

defineProps<{
  objects: ObjectSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  sort: [column: string]
  select: [obj: ObjectSummary]
}>()
</script>

<style scoped>
/* Custom object type colors are now handled in the formatter */
:deep(.type-siege) {
  background: rgba(231, 76, 60, 0.2);
  color: #e74c3c;
  border: 1px solid rgba(231, 76, 60, 0.4);
}

:deep(.type-generic) {
  background: rgba(149, 165, 166, 0.2);
  color: #95a5a6;
  border: 1px solid rgba(149, 165, 166, 0.4);
  font-style: italic;
}

:deep(.type-default) {
  background: var(--color-surface);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

/* Object-specific styling */
:deep(.catalog-table__cell-center) {
  text-align: center;
}

:deep(.catalog-table__cell-center[data-key="ac"]),
:deep(.catalog-table__cell-center[data-key="hp"]) {
  font-family: monospace;
  font-weight: 500;
}
</style>