<template>
  <div class="catalog-table">
    <div class="catalog-table__header">
      <h2 class="catalog-table__title">{{ config.title }}</h2>
      <div class="catalog-table__filters" v-if="config.filters.length > 0">
        <template v-for="filter in config.filters" :key="filter.key">
          <MultiSelectFilter
            v-if="filter.type === 'multiselect'"
            :label="filter.label"
            :options="getFilterOptions(filter)"
            v-model="filterValues[filter.key]"
          />
          <SelectFilter
            v-else-if="filter.type === 'select'"
            :label="filter.label"
            :placeholder="filter.placeholder"
            :options="filter.options || []"
            :grouped="!!filter.groupedOptions"
            :grouped-options="filter.groupedOptions"
            v-model="filterValues[filter.key]"
          />
          <CheckboxFilter
            v-else-if="filter.type === 'checkbox'"
            :label="filter.label"
            :tooltip="filter.tooltip"
            v-model="filterValues[filter.key]"
          />
          <RangeFilter
            v-else-if="filter.type === 'range'"
            :label="filter.label"
            :min="filter.min"
            :max="filter.max"
            :step="filter.step"
            v-model="filterValues[filter.key]"
          />
        </template>
      </div>
    </div>
    
    <div class="catalog-table__content">
      <div class="catalog-table__results-info">
        <span class="catalog-table__result-count">{{ filteredData.length }} {{ config.name }}</span>
      </div>
      
      <div class="catalog-table__scroll-container">
        <table class="catalog-table__table">
          <thead>
            <tr>
              <th 
                v-for="column in config.columns" 
                :key="column.key"
                :style="column.width ? { width: column.width } : undefined"
              >
                <div 
                  v-if="column.sortable"
                  class="catalog-table__sort-header" 
                  @click="$emit('sort', column.key)"
                >
                  {{ column.label }}
                  <span class="catalog-table__sort-icon">{{ getSortIndicator(column.key) }}</span>
                </div>
                <span v-else>{{ column.label }}</span>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="filteredData.length === 0" class="catalog-table__empty-row">
              <td :colspan="config.columns.length">
                <div class="catalog-table__empty">
                  <h3 v-if="searchPerformed">{{ config.emptyMessage?.title || 'No results found' }}</h3>
                  <h3 v-else>{{ config.emptyMessage?.subtitle || `Search for ${config.name}` }}</h3>
                  <p v-if="searchPerformed">{{ config.emptyMessage?.noResults || `No ${config.name} found matching your criteria` }}</p>
                  <p v-else>{{ config.emptyMessage?.subtitle || `Search for ${config.name} to see results` }}</p>
                </div>
              </td>
            </tr>
            <tr
              v-for="item in sortedData"
              :key="getItemKey(item)"
              class="catalog-table__row"
              @click="$emit('select', item)"
            >
              <td 
                v-for="column in config.columns" 
                :key="column.key"
                :class="column.className"
              >
                <template v-if="column.type === 'badges'">
                  <span 
                    v-for="badge in getCellValue(item, column)"
                    :key="badge"
                    class="catalog-table__cell-badge catalog-table__cell-badge--primary"
                  >
                    {{ badge }}
                  </span>
                </template>
                <template v-else-if="column.type === 'badge'">
                  <span :class="['catalog-table__badge', getCellValue(item, column).variant]">
                    {{ getCellValue(item, column).text }}
                  </span>
                </template>
                <template v-else-if="column.type === 'source'">
                  {{ getCellValue(item, column).source }}
                </template>
                <template v-else-if="column.type === 'name-with-srd'">
                  {{ getCellValue(item, column).name }}
                </template>
                <template v-else-if="column.type === 'prerequisites'">
                  <span v-if="getCellValue(item, column).hasPrerequisites" class="prereq-icon" title="Has prerequisites">
                    ✓
                  </span>
                  <span v-else class="catalog-table__empty">—</span>
                </template>
                <template v-else>
                  {{ getCellValue(item, column) }}
                </template>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import type { CatalogConfig, FilterValues } from './config/types'
import MultiSelectFilter from '../ui/MultiSelectFilter.vue'
import SelectFilter from './filters/SelectFilter.vue'
import CheckboxFilter from './filters/CheckboxFilter.vue'
import RangeFilter from './filters/RangeFilter.vue'

interface Props {
  config: CatalogConfig
  data: any[]
  searchPerformed: boolean
  sortColumn: string
  sortDirection: 'asc' | 'desc'
}

const props = defineProps<Props>()

const emit = defineEmits<{
  select: [item: any]
  sort: [column: string]
}>()

// Helper function for filter options
function getFilterOptions(filter: any): string[] {
  if (!filter.options) return []
  return filter.options.map((option: any) => {
    if (typeof option === 'string') return option
    return option.value || option.label || String(option)
  })
}

// Initialize filter values immediately
function initializeFilterValues(): FilterValues {
  const values: FilterValues = {}
  props.config.filters.forEach(filter => {
    switch (filter.type) {
      case 'multiselect':
        values[filter.key] = []
        break
      case 'select':
        values[filter.key] = ''
        break
      case 'checkbox':
        values[filter.key] = false
        break
      case 'range':
        values[filter.key] = { min: undefined, max: undefined }
        break
    }
  })
  return values
}

// Initialize filter values based on config
const filterValues = ref<FilterValues>(initializeFilterValues())

onMounted(() => {
  // Filter values already initialized
})

// Apply filters to data
const filteredData = computed(() => {
  let filtered = [...props.data]
  
  props.config.filters.forEach(filter => {
    const filterValue = filterValues.value[filter.key]
    
    switch (filter.type) {
      case 'multiselect':
        if (Array.isArray(filterValue) && filterValue.length > 0) {
          filtered = filtered.filter(item => {
            const itemValue = item[filter.key]
            if (Array.isArray(itemValue)) {
              return itemValue.some(v => filterValue.includes(String(v)))
            }
            return filterValue.includes(String(itemValue))
          })
        }
        break
        
      case 'select':
        if (filterValue && filterValue !== '') {
          filtered = filtered.filter(item => String(item[filter.key]) === filterValue)
        }
        break
        
      case 'checkbox':
        if (filterValue === true) {
          filtered = filtered.filter(item => Boolean(item[filter.key]))
        }
        break
        
      case 'range':
        if (filterValue && (filterValue.min !== undefined || filterValue.max !== undefined)) {
          filtered = filtered.filter(item => {
            const itemValue = Number(item[filter.key])
            if (filterValue.min !== undefined && itemValue < filterValue.min) return false
            if (filterValue.max !== undefined && itemValue > filterValue.max) return false
            return true
          })
        }
        break
    }
  })
  
  return filtered
})

// Apply sorting to filtered data
const sortedData = computed(() => {
  if (!props.sortColumn || !props.config.columns.some(col => col.key === props.sortColumn && col.sortable)) {
    return filteredData.value
  }
  
  const sorted = [...filteredData.value]
  sorted.sort((a, b) => {
    const aVal = a[props.sortColumn]
    const bVal = b[props.sortColumn]
    
    // Handle number sorting
    if (typeof aVal === 'number' && typeof bVal === 'number') {
      const comparison = aVal - bVal
      return props.sortDirection === 'asc' ? comparison : -comparison
    }
    
    // Handle string sorting
    const aStr = String(aVal || '')
    const bStr = String(bVal || '')
    const comparison = aStr.localeCompare(bStr)
    return props.sortDirection === 'asc' ? comparison : -comparison
  })
  
  return sorted
})

function getCellValue(item: any, column: any): any {
  if (column.formatter) {
    if (column.type === 'badges' || column.type === 'badge' || column.type === 'source' || column.type === 'name-with-srd' || column.type === 'prerequisites') {
      // For badges, badge, source, name-with-srd, and prerequisites types, pass the full item
      return column.formatter(item)
    } else {
      // For regular cells, format the specific field value
      return column.formatter(item[column.key])
    }
  }
  return item[column.key] || '—'
}

function getItemKey(item: any): string {
  // Try common key combinations
  if (item.name && item.source) return `${item.name}-${item.source}`
  if (item.id) return String(item.id)
  if (item.name) return item.name
  return JSON.stringify(item)
}

function getSortIndicator(columnKey: string): string {
  if (props.sortColumn !== columnKey) return ''
  return props.sortDirection === 'asc' ? '▲' : '▼'
}

// Watch for filter changes and emit events if needed
watch(filterValues, () => {
  // Filters are applied automatically via computed properties
}, { deep: true })
</script>

<style scoped>
/* Component uses existing catalog-table styles from catalog-tables.css */
</style>