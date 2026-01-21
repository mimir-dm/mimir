<template>
  <CatalogTable 
    :data="traps"
    :config="trapConfig"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @sort="emit('sort', $event)"
    @select="emit('select', $event)"
  />
</template>

<script setup lang="ts">
import type { TrapSummary } from '../../composables/catalog'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { trapConfig } from '../../../../shared/components/catalog/config/trapConfig'

defineProps<{
  traps: TrapSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  sort: [column: string]
  select: [trap: TrapSummary]
}>()
</script>

<style scoped>
/* Custom category colors for traps/hazards */
:deep(.trap) {
  background: rgba(231, 76, 60, 0.2);
  color: #e74c3c;
  border: 1px solid rgba(231, 76, 60, 0.4);
}

:deep(.hazard) {
  background: rgba(243, 156, 18, 0.2);
  color: #f39c12;
  border: 1px solid rgba(243, 156, 18, 0.4);
}

:deep(.default) {
  background: var(--color-surface);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
}

/* Type column centering */
:deep(.catalog-table__cell-center) {
  text-align: center;
}
</style>