<template>
  <div class="source-search">
    <!-- Search Header -->
    <div class="search-header">
      <div class="search-controls">
        <!-- Content Type Selector -->
        <div class="content-type-selector">
          <label for="content-type">Type:</label>
          <select 
            id="content-type" 
            v-model="selectedContentType" 
            class="content-type-select"
          >
            <option value="all">All Content</option>
            <option value="spells">Spells</option>
            <option value="items">Items & Equipment</option>
            <option value="monsters">Monsters</option>
            <option value="magic-items">Magic Items</option>
          </select>
        </div>
        
        <!-- Search Input -->
        <div class="search-input-wrapper">
          <input 
            type="text" 
            v-model="searchQuery" 
            @input="handleSearchInput"
            placeholder="Search source content..."
            class="search-input"
          >
        </div>
        
        <!-- Results Count -->
        <div class="results-info" v-if="hasSearched">
          <span class="results-count">{{ totalResults }} results</span>
        </div>
      </div>
    </div>
    
    <!-- Search Results -->
    <div class="search-results">
      <!-- This will eventually be replaced with unified result components -->
      <div class="results-container">
        <!-- Temporary: Use existing table components during migration -->
        <SpellTable
          v-if="selectedContentType === 'spells' && spellResults.length > 0"
          :spells="spellResults"
          :search-performed="hasSearched"
          :sort-column="sortColumn"
          :sort-direction="sortDirection"
          @select="handleContentSelect"
          @sort="handleSort"
        />
        
        <ItemTable
          v-if="(selectedContentType === 'items' || selectedContentType === 'magic-items') && itemResults.length > 0"
          :items="itemResults"
          :search-performed="hasSearched"
          :sort-column="sortColumn"
          :sort-direction="sortDirection"
          @select="handleContentSelect"
          @sort="handleSort"
        />
        
        <MonsterTable
          v-if="selectedContentType === 'monsters' && monsterResults.length > 0"
          :monsters="monsterResults"
          :search-performed="hasSearched"
          :sort-column="sortColumn"
          :sort-direction="sortDirection"
          @select="handleContentSelect"
          @sort="handleSort"
        />
        
        <!-- No Results -->
        <div v-if="hasSearched && totalResults === 0" class="no-results">
          No results found for "{{ searchQuery }}"
        </div>
        
        <!-- Initial State -->
        <div v-if="!hasSearched" class="initial-state">
          <p>Search across all source books for spells, items, monsters, and more.</p>
          <p class="hint">Start typing to search, or select a content type to browse.</p>
        </div>
      </div>
    </div>
    
    <!-- Content Modal (temporary, will be replaced) -->
    <AppModal
      v-for="(modal, index) in modalStack"
      :key="`modal-${index}`"
      :visible="modal.visible"
      :title="modal.title"
      size="md"
      :stack-index="index"
      @close="closeModal(index)"
    >
      <div class="dnd-content" v-html="modal.content"></div>
    </AppModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useSourceSearch } from '../../composables/useSourceSearch'

// Temporarily import existing components during migration
import SpellTable from './SpellTable.vue'
import ItemTable from './ItemTable.vue'
import MonsterTable from './MonsterTable.vue'
import AppModal from '@/components/shared/AppModal.vue'

// Props
interface Props {
  selectedSources?: string[]
}

const props = withDefaults(defineProps<Props>(), {
  selectedSources: () => []
})

// Use the new source search composable
const {
  searchQuery,
  selectedContentType,
  spellResults,
  itemResults,
  monsterResults,
  hasSearched,
  search,
  clearSearch
} = useSourceSearch()

// Modal management (temporary)
const modalStack = ref<any[]>([])

// Sort state
const sortColumn = ref('name')
const sortDirection = ref<'asc' | 'desc'>('asc')

// Computed
const totalResults = computed(() => {
  switch (selectedContentType.value) {
    case 'spells': return spellResults.value.length
    case 'items':
    case 'magic-items': return itemResults.value.length
    case 'monsters': return monsterResults.value.length
    case 'all':
      return spellResults.value.length + 
             itemResults.value.length + 
             monsterResults.value.length
    default: return 0
  }
})

// Search handling
let searchTimeout: ReturnType<typeof setTimeout> | undefined

function handleSearchInput() {
  if (searchTimeout) {
    clearTimeout(searchTimeout)
  }
  searchTimeout = setTimeout(() => {
    search(props.selectedSources)
  }, 300)
}

// Content selection
function handleContentSelect(content: any) {
  // Temporary: just show in modal
  modalStack.value.push({
    visible: true,
    title: content.name,
    content: `<pre>${JSON.stringify(content, null, 2)}</pre>`
  })
}

function closeModal(index?: number) {
  if (index !== undefined) {
    modalStack.value.splice(index, 1)
  } else {
    modalStack.value.pop()
  }
}

// Sort handling
function handleSort(column: string) {
  if (sortColumn.value === column) {
    sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortColumn.value = column
    sortDirection.value = 'asc'
  }
}

// Watch for source changes
watch(() => props.selectedSources, () => {
  if (hasSearched.value) {
    search(props.selectedSources)
  }
}, { deep: true })
</script>

<style scoped>
.source-search {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-background);
}

.search-header {
  padding: 1rem;
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
}

.search-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.content-type-selector {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.content-type-selector label {
  color: var(--color-text-secondary);
  font-size: 0.9rem;
}

.content-type-select {
  padding: 0.5rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  color: var(--color-text);
}

.search-input-wrapper {
  flex: 1;
}

.search-input {
  width: 100%;
  padding: 0.5rem 1rem;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  color: var(--color-text);
  font-size: 1rem;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.results-info {
  color: var(--color-text-secondary);
  font-size: 0.9rem;
}

.search-results {
  flex: 1;
  overflow: auto;
  padding: 1rem;
}

.results-container {
  height: 100%;
}

.no-results,
.initial-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-secondary);
  text-align: center;
}

.initial-state .hint {
  margin-top: 0.5rem;
  font-size: 0.9rem;
  opacity: 0.7;
}
</style>