<template>
  <CatalogTable
    :config="languageConfig"
    :data="languages"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="emit('select', $event)"
    @sort="emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import { languageConfig } from '../../../../shared/components/catalog/config/languageConfig'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import type { LanguageSummary } from '../../composables/catalog'

interface Props {
  languages: LanguageSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}

defineProps<Props>()

const emit = defineEmits<{
  sort: [column: string]
  select: [lang: LanguageSummary]
}>()
</script>

<!-- Styles handled by CatalogTable component -->