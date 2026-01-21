<template>
  <CatalogTable
    :config="vehicleConfig"
    :data="vehicles"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="$emit('select', $event)"
    @sort="$emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import CatalogTable from '@/shared/components/catalog/CatalogTable.vue'
import { vehicleConfig } from '@/shared/components/catalog/config/vehicleConfig'
import type { VehicleSummary } from '../../composables/catalog'

interface Props {
  vehicles: VehicleSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}

defineProps<Props>()

defineEmits<{
  sort: [column: string]
  select: [vehicle: VehicleSummary]
}>()
</script>

