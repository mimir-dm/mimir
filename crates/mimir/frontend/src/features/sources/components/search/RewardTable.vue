<template>
  <CatalogTable 
    :data="rewards"
    :config="rewardConfig"
    :search-performed="searchPerformed"
    :sort-column="sortColumn"
    :sort-direction="sortDirection"
    @sort="emit('sort', $event)"
    @select="emit('select', $event)"
  />
</template>

<script setup lang="ts">
import type { RewardSummary } from '../../composables/catalog'
import CatalogTable from '../../../../shared/components/catalog/CatalogTable.vue'
import { rewardConfig } from '../../../../shared/components/catalog/config/rewardConfig'

defineProps<{
  rewards: RewardSummary[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}>()

const emit = defineEmits<{
  sort: [column: string]
  select: [reward: RewardSummary]
}>()
</script>