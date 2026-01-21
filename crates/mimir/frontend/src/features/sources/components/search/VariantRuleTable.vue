<template>
  <CatalogTable
    :config="variantRuleConfig"
    :data="rules"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @select="$emit('select', $event)"
    @sort="$emit('sort', $event)"
  />
</template>

<script setup lang="ts">
import CatalogTable from '@/shared/components/catalog/CatalogTable.vue'
import { variantRuleConfig } from '@/shared/components/catalog/config/variantRuleConfig'
import type { VariantRuleSummary } from '../../composables/catalog'

// Props
defineProps<{
  rules: VariantRuleSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}>()

// Events
defineEmits<{
  select: [rule: VariantRuleSummary]
  sort: [column: string]
}>()
</script>

