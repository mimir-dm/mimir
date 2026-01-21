<template>
  <CatalogTable
    :config="classConfig"
    :data="classes"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="$emit('select', $event)"
    @sort="$emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import CatalogTable from '@/shared/components/catalog/CatalogTable.vue'
import { classConfig } from '@/shared/components/catalog/config/classConfig'
import type { ClassSummary } from '../../composables/catalog'

interface Props {
  classes: ClassSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
  availableSources?: string[]
}

defineProps<Props>()

defineEmits<{
  select: [classItem: ClassSummary]
  sort: [column: string]
}>()
</script>
