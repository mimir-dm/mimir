<template>
  <CatalogTable
    :config="itemConfig"
    :data="items"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="$emit('select', $event)"
    @sort="$emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import CatalogTable from '@/shared/components/catalog/CatalogTable.vue'
import { itemConfig } from '@/shared/components/catalog/config/itemConfig'
import type { ItemSummary } from '../../composables/catalog'

interface Props {
  items: ItemSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
  showRarity?: boolean  // For magic items (kept for compatibility)
}

withDefaults(defineProps<Props>(), {
  showRarity: false
})

defineEmits<{
  select: [item: ItemSummary]
  sort: [column: string]
}>()
</script>