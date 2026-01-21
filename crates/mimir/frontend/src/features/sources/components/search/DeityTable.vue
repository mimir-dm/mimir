<template>
  <CatalogTable
    :config="deityConfig"
    :data="deities"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="$emit('select', $event)"
    @sort="$emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import CatalogTable from '@/shared/components/catalog/CatalogTable.vue'
import { deityConfig } from '@/shared/components/catalog/config/deityConfig'
import type { DeitySummary } from '../../composables/catalog'

interface Props {
  deities: DeitySummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}

defineProps<Props>()

defineEmits<{
  sort: [column: string]
  select: [deity: DeitySummary]
}>()
</script>

