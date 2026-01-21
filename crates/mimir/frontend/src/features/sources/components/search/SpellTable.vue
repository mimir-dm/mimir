<template>
  <CatalogTable
    :config="spellConfig"
    :data="spells"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="$emit('select', $event)"
    @sort="$emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import type { SpellSummary } from '../../composables/catalog'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { spellConfig } from '../../../../shared/components/catalog/config/spellConfig'

interface Props {
  spells: SpellSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}

defineProps<Props>()

defineEmits<{
  select: [spell: SpellSummary]
  sort: [column: string]
}>()
</script>