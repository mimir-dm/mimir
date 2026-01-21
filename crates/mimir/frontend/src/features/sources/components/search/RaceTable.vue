<template>
  <CatalogTable 
    :data="races"
    :config="raceConfig"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @sort="emit('sort', $event)"
    @select="emit('select', $event)"
  />
</template>

<script setup lang="ts">
import type { RaceSummary } from '../../composables/catalog'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { raceConfig } from '../../../../shared/components/catalog/config/raceConfig'

defineProps<{
  races: RaceSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  sort: [column: string]
  select: [race: RaceSummary]
}>()
</script>
