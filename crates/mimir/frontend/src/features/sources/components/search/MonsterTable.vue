<template>
  <CatalogTable 
    :config="monsterConfig" 
    :data="monsters" 
    :search-performed="searchPerformed" 
    :sort-column="sortColumn" 
    :sort-direction="sortDirection" 
    @select="$emit('select', $event)" 
    @sort="$emit('sort', $event)" 
  />
</template>

<script setup lang="ts">
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { monsterConfig } from '../../../../shared/components/catalog/config/monsterConfig'
import type { MonsterSummary } from '../../composables/catalog'

interface Props {
  monsters: MonsterSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
  filters?: {
    sizes?: string[]
    types?: string[]
  }
}

defineProps<Props>()

defineEmits<{
  select: [monster: MonsterSummary]
  sort: [column: string]
  filterUpdate: [filters: { sizes?: string[], types?: string[] }]
}>()
</script>

