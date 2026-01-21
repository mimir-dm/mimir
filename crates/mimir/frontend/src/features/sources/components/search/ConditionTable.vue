<template>
  <CatalogTable
    :config="conditionConfig"
    :data="conditions"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="emit('select', $event)"
    @sort="emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import { conditionConfig } from '../../../../shared/components/catalog/config/conditionConfig'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import type { ConditionSummary } from '../../services/SearchService'

interface Props {
  conditions: ConditionSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}

defineProps<Props>()

const emit = defineEmits<{
  select: [condition: ConditionSummary]
  sort: [column: string]
}>()
</script>

<!-- Styles handled by CatalogTable component -->