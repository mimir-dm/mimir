<template>
  <CatalogTable
    :config="optionalFeatureConfig"
    :data="options"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="$emit('select', $event)"
    @sort="$emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import type { OptionalFeatureSummary } from '../../services/SearchService'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { optionalFeatureConfig } from '../../../../shared/components/catalog/config/optionalFeatureConfig'

interface Props {
  options: OptionalFeatureSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}

defineProps<Props>()

defineEmits<{
  select: [option: OptionalFeatureSummary]
  sort: [column: string]
}>()
</script>