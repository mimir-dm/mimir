<template>
  <CatalogTable 
    :data="backgrounds"
    :config="backgroundConfig"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @sort="emit('sort', $event)"
    @select="emit('select', $event)"
  />
</template>

<script setup lang="ts">
import type { BackgroundSummary } from '../../composables/catalog'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { backgroundConfig } from '../../../../shared/components/catalog/config/backgroundConfig'

defineProps<{
  backgrounds: BackgroundSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  sort: [column: string]
  select: [background: BackgroundSummary]
}>()
</script>