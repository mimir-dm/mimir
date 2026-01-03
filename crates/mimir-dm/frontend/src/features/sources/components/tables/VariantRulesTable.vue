<template>
  <div class="variant-rules-table">
    <h3>Variant Rules</h3>
    
    <!-- Search and filter controls -->
    <div class="search-controls">
      <input 
        v-model="searchQuery"
        type="text"
        placeholder="Search variant rules..."
        class="search-input"
        @input="debouncedSearch"
      />
      
      <div class="filter-row">
        <select v-model="selectedType" @change="search" class="filter-select">
          <option value="">All Types</option>
          <option v-for="type in types" :key="type" :value="type">{{ type }}</option>
        </select>
        
        <select v-model="selectedSource" @change="search" class="filter-select">
          <option value="">All Sources</option>
          <option v-for="source in sources" :key="source" :value="source">{{ source }}</option>
        </select>
      </div>
    </div>
    
    <!-- Results list -->
    <div class="results-container">
      <div v-if="loading" class="loading-state">
        Loading variant rules...
      </div>
      
      <EmptyState
        v-else-if="rules.length === 0"
        variant="search"
        title="No variant rules found"
        description="No variant rules found matching your criteria."
      />
      
      <div v-else class="rules-list">
        <table class="rules-table">
          <thead>
            <tr>
              <th @click="sortBy('name')" class="sortable">
                Name
                <span class="sort-indicator" v-if="sortColumn === 'name'">
                  {{ sortDirection === 'asc' ? '▲' : '▼' }}
                </span>
              </th>
              <th @click="sortBy('rule_type')" class="sortable">
                Type
                <span class="sort-indicator" v-if="sortColumn === 'rule_type'">
                  {{ sortDirection === 'asc' ? '▲' : '▼' }}
                </span>
              </th>
              <th @click="sortBy('source')" class="sortable">
                Source
                <span class="sort-indicator" v-if="sortColumn === 'source'">
                  {{ sortDirection === 'asc' ? '▲' : '▼' }}
                </span>
              </th>
              <th>Page</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="rule in sortedRules" :key="`${rule.name}-${rule.source}`"
                @click="selectRule(rule)" class="clickable-row">
              <td class="name-cell">{{ rule.name }}</td>
              <td class="type-cell">
                <span :class="['type-badge', getTypeClass(rule.rule_type)]">
                  {{ rule.rule_type || 'General' }}
                </span>
              </td>
              <td class="source-cell">{{ rule.source }}</td>
              <td class="page-cell">{{ rule.page ? `p. ${rule.page}` : '—' }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useVariantRules, type VariantRuleSummary } from '../../composables/catalog'
import EmptyState from '@/shared/components/ui/EmptyState.vue'

const emit = defineEmits<{
  select: [rule: VariantRuleSummary]
}>()

const catalog = useVariantRules()
const rules = ref<VariantRuleSummary[]>([])
const loading = ref(false)
const searchQuery = ref('')
const selectedType = ref('')
const selectedSource = ref('')
const types = ref<string[]>([])
const sources = ref<string[]>([])
const sortColumn = ref('name')
const sortDirection = ref<'asc' | 'desc'>('asc')

const sortedRules = computed(() => {
  const sorted = [...rules.value]
  sorted.sort((a, b) => {
    let aVal = a[sortColumn.value as keyof VariantRuleSummary]
    let bVal = b[sortColumn.value as keyof VariantRuleSummary]
    
    // Handle null/undefined
    if (aVal == null) aVal = ''
    if (bVal == null) bVal = ''
    
    // Convert to strings for comparison
    aVal = String(aVal).toLowerCase()
    bVal = String(bVal).toLowerCase()
    
    if (sortDirection.value === 'asc') {
      return aVal < bVal ? -1 : aVal > bVal ? 1 : 0
    } else {
      return aVal > bVal ? -1 : aVal < bVal ? 1 : 0
    }
  })
  return sorted
})

async function initCatalog() {
  try {
    // No initialization needed for database-backed catalog
    await loadFilters()
    await search()
  } catch (error) {
    console.error('Failed to initialize variant rule catalog:', error)
  }
}

async function loadFilters() {
  try {
    const [loadedTypes, loadedSources] = await Promise.all([
      catalog.getVariantRuleTypes(),
      catalog.getVariantRuleSources()
    ])
    types.value = loadedTypes
    sources.value = loadedSources
  } catch (error) {
    console.error('Failed to load filters:', error)
  }
}

async function search() {
  loading.value = true
  try {
    rules.value = await catalog.searchVariantRules({
      query: searchQuery.value || undefined,
      types: selectedType.value ? [selectedType.value] : undefined,
      sources: selectedSource.value ? [selectedSource.value] : undefined
    })
  } catch (error) {
    console.error('Search failed:', error)
    rules.value = []
  } finally {
    loading.value = false
  }
}

let searchTimeout: number | undefined
function debouncedSearch() {
  clearTimeout(searchTimeout)
  searchTimeout = window.setTimeout(() => search(), 300)
}

function sortBy(column: string) {
  if (sortColumn.value === column) {
    sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortColumn.value = column
    sortDirection.value = 'asc'
  }
}

function selectRule(rule: VariantRuleSummary) {
  emit('select', rule)
}

function getTypeClass(type: string | undefined | null): string {
  if (!type) return 'type-general'
  const normalized = type.toLowerCase().replace(/\s+/g, '-')
  return `type-${normalized}`
}

onMounted(() => {
  initCatalog()
})
</script>

<style scoped>
.variant-rules-table {
  padding: var(--spacing-md, 12px);
  background: var(--color-background, #0a0a0a);
  color: var(--color-text, #e0e0e0);
  height: 100%;
  display: flex;
  flex-direction: column;
}

.variant-rules-table h3 {
  margin: 0 0 var(--spacing-md, 12px) 0;
  color: var(--color-primary, #4a9eff);
}

.search-controls {
  margin-bottom: var(--spacing-md, 12px);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm, 8px);
}

.search-input {
  width: 100%;
  padding: var(--spacing-sm, 8px);
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  color: var(--color-text, #e0e0e0);
  font-size: 0.9rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary, #4a9eff);
}

.filter-row {
  display: flex;
  gap: var(--spacing-sm, 8px);
}

.filter-select {
  flex: 1;
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  background: var(--color-surface, #1a1a1a);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  color: var(--color-text, #e0e0e0);
  font-size: 0.85rem;
  cursor: pointer;
}

.results-container {
  flex: 1;
  overflow-y: auto;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--color-text-secondary, #999);
}

.rules-list {
  width: 100%;
  overflow-x: auto;
}

.rules-table {
  width: 100%;
  border-collapse: collapse;
}

.rules-table th {
  text-align: left;
  padding: var(--spacing-sm, 8px);
  border-bottom: 2px solid var(--color-border, #333);
  background: var(--color-surface, #1a1a1a);
  color: var(--color-text-secondary, #999);
  white-space: nowrap;
}

.rules-table th.sortable {
  cursor: pointer;
  user-select: none;
}

.rules-table th.sortable:hover {
  color: var(--color-text, #e0e0e0);
}

.sort-indicator {
  display: inline-block;
  margin-left: 4px;
  font-size: 0.8em;
}

.rules-table tbody tr {
  border-bottom: 1px solid var(--color-border-light, #262626);
  transition: background-color 0.15s ease;
}

.rules-table tbody tr:hover {
  background: var(--color-surface-hover, #262626);
}

.clickable-row {
  cursor: pointer;
}

.rules-table td {
  padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
  color: var(--color-text, #e0e0e0);
}

.name-cell {
  font-weight: 500;
  color: var(--color-primary, #4a9eff);
}

.type-cell {
  white-space: nowrap;
}

.type-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 0.85rem;
  font-weight: 500;
}

.type-general {
  background: var(--color-surface, #1a1a1a);
  color: var(--color-text-secondary, #999);
  border: 1px solid var(--color-border, #333);
}

.type-action-options {
  background: rgba(255, 87, 34, 0.2);
  color: #ff5722;
  border: 1px solid rgba(255, 87, 34, 0.4);
}

.type-v {
  background: rgba(156, 39, 176, 0.2);
  color: #9c27b0;
  border: 1px solid rgba(156, 39, 176, 0.4);
}

.type-o {
  background: rgba(33, 150, 243, 0.2);
  color: #2196f3;
  border: 1px solid rgba(33, 150, 243, 0.4);
}

.source-cell {
  color: var(--color-text-secondary, #999);
  font-size: 0.85rem;
  white-space: nowrap;
}

.page-cell {
  color: var(--color-text-secondary, #999);
  font-size: 0.85rem;
  text-align: center;
}
</style>