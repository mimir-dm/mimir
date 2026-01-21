<template>
  <CatalogTable
    :config="psionicConfig"
    :data="psionics"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="$emit('select', $event)"
    @sort="$emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import CatalogTable from '@/shared/components/catalog/CatalogTable.vue'
import { psionicConfig } from '@/shared/components/catalog/config/psionicConfig'
import type { PsionicSummary } from '../../composables/catalog'

// Props
defineProps<{
  psionics: PsionicSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}>()

// Events
defineEmits<{
  select: [psionic: PsionicSummary]
  sort: [column: string]
}>()
</script>