<template>
  <CatalogTable 
    :data="feats"
    :config="featConfig"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @sort="emit('sort', $event)"
    @select="emit('select', $event)"
  />
</template>

<script setup lang="ts">
import type { FeatSummary } from '../../composables/catalog'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { featConfig } from '../../../../shared/components/catalog/config/featConfig'

defineProps<{
  feats: FeatSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  sort: [column: string]
  select: [feat: FeatSummary]
}>()
</script>