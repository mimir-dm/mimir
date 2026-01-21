<template>
  <CatalogTable
    :config="actionConfig"
    :data="actions"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="$emit('select', $event)"
    @sort="$emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { actionConfig } from '../../../../shared/components/catalog/config/actionConfig'
import type { ActionSummary } from '../../composables/catalog'

interface Props {
  actions: ActionSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}

defineProps<Props>()

defineEmits<{
  select: [action: ActionSummary]
  sort: [column: string]
}>()
</script>